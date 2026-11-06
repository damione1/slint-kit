// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use crate::ui;
use kit_core::{
    DismissReason, NavigationStep, Ownership, SelectEvent, SelectOption, SelectSession,
    SelectState, SelectionIntent, reduce_selection,
};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ChoiceToken(i32);
impl ChoiceToken {
    fn new(value: i32) -> Option<Self> {
        (value >= 0).then_some(Self(value))
    }
}

impl From<ui::NavigationStep> for NavigationStep {
    fn from(value: ui::NavigationStep) -> Self {
        match value {
            ui::NavigationStep::Next => Self::Next,
            ui::NavigationStep::Previous => Self::Previous,
            ui::NavigationStep::First => Self::First,
            ui::NavigationStep::Last => Self::Last,
            ui::NavigationStep::PageNext => Self::PageNext,
            ui::NavigationStep::PagePrevious => Self::PagePrevious,
        }
    }
}
impl From<ui::CloseReason> for DismissReason {
    fn from(value: ui::CloseReason) -> Self {
        match value {
            ui::CloseReason::Escape => Self::Escape,
            ui::CloseReason::OutsidePress => Self::OutsidePress,
            ui::CloseReason::SystemBack => Self::SystemBack,
            ui::CloseReason::TriggerToggle => Self::TriggerToggle,
            ui::CloseReason::Programmatic => Self::Programmatic,
            ui::CloseReason::HostClosed => Self::HostClosed,
        }
    }
}
impl From<ui::Ownership> for Ownership {
    fn from(value: ui::Ownership) -> Self {
        match value {
            ui::Ownership::Local => Self::Local,
            ui::Ownership::Controlled => Self::Controlled,
        }
    }
}

fn tracked<T: Clone + 'static>(model: &ModelRc<T>) -> Vec<T> {
    model.model_tracker().track_row_count_changes();
    (0..model.row_count())
        .filter_map(|index| {
            model.model_tracker().track_row_data_changes(index);
            model.row_data(index)
        })
        .collect()
}

fn options(rows: &[ui::OptionRow]) -> Option<Vec<SelectOption<ChoiceToken>>> {
    let mut keys = HashSet::with_capacity(rows.len());
    let mut generation = None;
    rows.iter()
        .map(|row| {
            let key = ChoiceToken::new(row.token)?;
            if row.generation < 0
                || !keys.insert(key)
                || generation.is_some_and(|value| value != row.generation)
            {
                return None;
            }
            generation = Some(row.generation);
            Some(SelectOption {
                key,
                label: row.label.to_string(),
                description: row.description.to_string(),
                disabled: row.disabled,
            })
        })
        .collect()
}

fn selection(value: ui::SelectionRef) -> Option<ChoiceToken> {
    if value.present {
        ChoiceToken::new(value.token)
    } else {
        None
    }
}
fn reference(value: Option<ChoiceToken>) -> ui::SelectionRef {
    match value {
        Some(token) => ui::SelectionRef {
            present: true,
            token: token.0,
        },
        None => ui::SelectionRef::default(),
    }
}
fn session(value: ui::SelectionSession) -> SelectSession<ChoiceToken> {
    SelectSession {
        selection: SelectState {
            selected: selection(value.state.selected),
            active: selection(value.state.active),
            open: value.state.open,
        },
        typeahead: value.typeahead.to_string(),
        last_typeahead_ms: value
            .has_timestamp
            .then(|| u64::try_from(value.timestamp).ok())
            .flatten(),
    }
}

fn project(
    rows: &[ui::OptionRow],
    selected: Option<ChoiceToken>,
    active: Option<ChoiceToken>,
    open: bool,
    generation: i32,
) -> ui::SelectState {
    let available = |key: ChoiceToken| rows.iter().any(|row| row.token == key.0);
    let eligible = |key: ChoiceToken| rows.iter().any(|row| row.token == key.0 && !row.disabled);
    let active = if open {
        active
            .filter(|key| eligible(*key))
            .or_else(|| selected.filter(|key| eligible(*key)))
            .or_else(|| {
                rows.iter()
                    .find(|row| !row.disabled)
                    .and_then(|row| ChoiceToken::new(row.token))
            })
    } else {
        None
    };
    ui::SelectState {
        selected: reference(selected),
        active: reference(active),
        open,
        active_index: active
            .and_then(|key| rows.iter().position(|row| row.token == key.0))
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(-1),
        generation,
        selection_availability: match selected {
            Some(key) if available(key) => ui::SelectionAvailability::Available,
            Some(_) => ui::SelectionAvailability::Unavailable,
            None => ui::SelectionAvailability::Absent,
        },
    }
}

fn reduce(
    model: &ModelRc<ui::OptionRow>,
    current: ui::SelectionSession,
    ownership: ui::Ownership,
    event: SelectEvent<ChoiceToken>,
) -> ui::SelectionTransition {
    let rows = tracked(model);
    let Some(options) = options(&rows) else {
        return ui::SelectionTransition {
            session: current,
            chosen: ui::SelectionRef::default(),
        };
    };
    let generation = current.state.generation;
    let result = reduce_selection(options, session(current), ownership.into(), event)
        .expect("projection keys validated before reduction");
    let chosen = match result.intent {
        Some(SelectionIntent::Commit(token)) => reference(Some(token)),
        Some(SelectionIntent::Clear) | None => ui::SelectionRef::default(),
    };
    let timestamp = result.session.last_typeahead_ms;
    ui::SelectionTransition {
        chosen,
        session: ui::SelectionSession {
            state: project(
                &rows,
                result.session.selection.selected,
                result.session.selection.active,
                result.session.selection.open,
                generation,
            ),
            typeahead: result.session.typeahead.into(),
            has_timestamp: timestamp.is_some(),
            timestamp: timestamp
                .map(|value| i32::try_from(value).expect("timestamp entered as i32"))
                .unwrap_or(0),
        },
    }
}

pub(crate) fn install_selection<W>(window: &W)
where
    W: ComponentHandle,
    for<'a> ui::KitSelectionService<'a>: slint::Global<'a, W>,
{
    let service = window.global::<ui::KitSelectionService>();
    if service.get_ready() {
        return;
    }
    service.on_project(|source, model, selected, active, open, generation| {
        let source = tracked(&source);
        let rows = tracked(&model);
        if options(&source).is_none() || options(&rows).is_none() {
            return ui::SelectState {
                active_index: -1,
                generation: -1,
                ..Default::default()
            };
        }
        let mut state = project(
            &rows,
            selection(selected.clone()),
            selection(active),
            open,
            generation,
        );
        state.selection_availability = match selection(selected) {
            Some(key) if source.iter().any(|row| row.token == key.0) => {
                ui::SelectionAvailability::Available
            }
            Some(_) => ui::SelectionAvailability::Unavailable,
            None => ui::SelectionAvailability::Absent,
        };
        state
    });
    service.on_reconcile(|rows, current, ownership| {
        reduce(&rows, current, ownership, SelectEvent::Reconcile).session
    });
    service.on_open(|rows, current, ownership| {
        reduce(&rows, current, ownership, SelectEvent::Open).session
    });
    service.on_navigate(|rows, current, step, ownership| {
        reduce(
            &rows,
            current,
            ownership,
            SelectEvent::Navigate {
                step: step.into(),
                page_size: 7,
            },
        )
        .session
    });
    service.on_dismiss(|rows, current, reason, ownership| {
        reduce(
            &rows,
            current,
            ownership,
            SelectEvent::Dismiss(reason.into()),
        )
        .session
    });
    service.on_activate(|rows, current, token, generation, ownership| {
        let Some(token) = ChoiceToken::new(token) else {
            return ui::SelectionTransition {
                session: current,
                chosen: ui::SelectionRef::default(),
            };
        };
        if generation < 0 || generation != current.state.generation {
            return ui::SelectionTransition {
                session: current,
                chosen: ui::SelectionRef::default(),
            };
        }
        reduce(&rows, current, ownership, SelectEvent::Activate(token))
    });
    service.on_typeahead(|rows, current, text, timestamp, ownership| {
        let Ok(now_ms) = u64::try_from(timestamp) else {
            return current;
        };
        reduce(
            &rows,
            current,
            ownership,
            SelectEvent::Typeahead {
                text: text.to_string(),
                now_ms,
            },
        )
        .session
    });
    service.on_rows(|model, query, generation| {
        let rows = tracked(&model);
        if options(&rows).is_none() || generation < 0 {
            return ModelRc::default();
        }
        let query = query.to_lowercase();
        ModelRc::new(VecModel::from(
            rows.into_iter()
                .filter(|row| {
                    query.is_empty()
                        || row.label.to_lowercase().contains(&query)
                        || row.description.to_lowercase().contains(&query)
                })
                .map(|mut row| {
                    row.generation = generation;
                    row
                })
                .collect::<Vec<_>>(),
        ))
    });
    service.on_choices(|model, selected| {
        ModelRc::new(VecModel::from(
            tracked(&model)
                .into_iter()
                .map(|row| ui::ChoiceRow {
                    token: row.token,
                    label: row.label,
                    disabled: row.disabled,
                    selected: selected.present && selected.token == row.token,
                })
                .collect::<Vec<_>>(),
        ))
    });
    service.on_disclosures(|model, expanded| {
        let expanded: Vec<_> = tracked(&expanded)
            .into_iter()
            .filter(|value| value.present)
            .map(|value| value.token)
            .collect();
        ModelRc::new(VecModel::from(
            tracked(&model)
                .into_iter()
                .map(|item| {
                    let open = expanded.contains(&item.token);
                    ui::DisclosureRow {
                        item,
                        expanded: open,
                    }
                })
                .collect::<Vec<_>>(),
        ))
    });
    service.on_disclosure(|item, expanded| {
        let open = tracked(&expanded)
            .iter()
            .any(|value| value.present && value.token == item.token);
        ui::DisclosureRow {
            item,
            expanded: open,
        }
    });
    service.on_disclosure_options(|rows| {
        ModelRc::new(VecModel::from(
            tracked(&rows)
                .into_iter()
                .map(|row| row.item)
                .collect::<Vec<_>>(),
        ))
    });
    service.on_disclosure_source(|rows, token| {
        tracked(&rows)
            .into_iter()
            .find(|row| row.item.token == token)
            .unwrap_or_default()
    });
    service.on_disclose(|model, expanded, token, generation, current, open, mode| {
        let rows = tracked(&model);
        if generation < 0
            || generation != current
            || options(&rows).is_none()
            || !rows.iter().any(|row| row.token == token && !row.disabled)
        {
            return expanded;
        }
        let mut next: Vec<_> = tracked(&expanded)
            .into_iter()
            .filter(|value| {
                value.present
                    && value.token != token
                    && rows.iter().any(|row| row.token == value.token)
            })
            .collect();
        if open {
            match mode {
                ui::ExpansionMode::Single => next.clear(),
                ui::ExpansionMode::Multiple => {}
            }
            next.push(ui::SelectionRef {
                present: true,
                token,
            });
        }
        ModelRc::new(VecModel::from(next))
    });
    service.on_label(|model, selected| {
        tracked(&model)
            .into_iter()
            .find(|row| selected.present && row.token == selected.token)
            .map(|row| row.label)
            .unwrap_or_else(SharedString::default)
    });
    service.on_source(|model, token| {
        tracked(&model)
            .into_iter()
            .find(|row| row.token == token)
            .unwrap_or_default()
    });
    service.on_generation(|model| {
        let rows = tracked(&model);
        if options(&rows).is_none() {
            return -1;
        }
        rows.first().map_or(0, |row| row.generation)
    });
    service.on_next_generation(|current| {
        if current < 0 {
            return -1;
        }
        // Exhaustion invalidates gestures instead of reusing an earlier identity.
        current.checked_add(1).unwrap_or(-1)
    });
    let epoch = web_time::Instant::now();
    service.on_timestamp(move || {
        // A wrap starts a fresh typeahead session (the core also handles a
        // backwards clock). It never changes option or gesture generations.
        let elapsed = epoch.elapsed().as_millis()
            % (u128::try_from(i32::MAX).expect("positive clock bound") + 1);
        i32::try_from(elapsed).expect("clock is bounded before projection")
    });
    service.on_multiple_choices(|rows, selected| {
        let selected = tracked(&selected);
        ModelRc::new(VecModel::from(
            tracked(&rows)
                .into_iter()
                .map(|row| ui::ChoiceRow {
                    selected: selected
                        .iter()
                        .any(|key| key.present && key.token == row.token),
                    token: row.token,
                    label: row.label,
                    disabled: row.disabled,
                })
                .collect::<Vec<_>>(),
        ))
    });
    service.on_chips(|rows, selected| {
        let rows = tracked(&rows);
        ModelRc::new(VecModel::from(
            tracked(&selected)
                .into_iter()
                .filter(|key| key.present)
                .filter_map(|key| {
                    rows.iter()
                        .find(|row| row.token == key.token)
                        .map(|row| ui::ChipRow {
                            token: row.token,
                            label: row.label.clone(),
                            removable: !row.disabled,
                        })
                })
                .collect::<Vec<_>>(),
        ))
    });
    service.on_toggle(|rows, selected, token, generation, limit| {
        let reject = || ui::MultipleTransition {
            selected: selected.clone(),
            accepted: false,
            added: false,
        };
        let rows = tracked(&rows);
        let Some(options) = options(&rows) else {
            return reject();
        };
        if !rows
            .iter()
            .any(|row| row.token == token && row.generation == generation && !row.disabled)
            || limit < 0
        {
            return reject();
        }
        let selected_keys = tracked(&selected)
            .into_iter()
            .map(selection)
            .collect::<Option<Vec<_>>>();
        let Some(selected_keys) = selected_keys else {
            return reject();
        };
        let limit = if limit == 0 {
            None
        } else {
            usize::try_from(limit).ok()
        };
        let Ok(mut controller) =
            kit_core::MultiSelectController::new(options, selected_keys, Ownership::Local, limit)
        else {
            return reject();
        };
        let Ok(intent) = controller.toggle(&ChoiceToken(token)) else {
            return reject();
        };
        let added = match intent {
            kit_core::MultiSelectionIntent::Add(_) => true,
            kit_core::MultiSelectionIntent::Remove(_) => false,
            kit_core::MultiSelectionIntent::Clear => false,
        };
        ui::MultipleTransition {
            selected: ModelRc::new(VecModel::from(
                controller
                    .selected()
                    .iter()
                    .map(|key| reference(Some(*key)))
                    .collect::<Vec<_>>(),
            )),
            accepted: true,
            added,
        }
    });
    service.on_rating_options(|steps| {
        let Some(scale) = u16::try_from(steps)
            .ok()
            .and_then(|steps| kit_core::RatingScale::new(steps).ok())
        else {
            return ModelRc::default();
        };
        ModelRc::new(VecModel::from(
            (1..=scale.steps())
                .map(|step| ui::OptionRow {
                    token: i32::from(step),
                    generation: steps,
                    label: step.to_string().into(),
                    ..Default::default()
                })
                .collect::<Vec<_>>(),
        ))
    });
    service.set_ready(true);
}

impl TryFrom<ui::RatingView> for Option<kit_core::Rating> {
    type Error = kit_core::ParseIssue;
    fn try_from(value: ui::RatingView) -> Result<Self, Self::Error> {
        let steps = u16::try_from(value.scale).map_err(|_| kit_core::ParseIssue::InvalidRange)?;
        let scale = kit_core::RatingScale::new(steps)?;
        if !value.present {
            return Ok(None);
        }
        let step = u16::try_from(value.step).map_err(|_| kit_core::ParseIssue::InvalidRange)?;
        scale.score(step).map(Some)
    }
}
