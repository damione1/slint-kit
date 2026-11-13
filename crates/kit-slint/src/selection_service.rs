// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use crate::ui;
use kit_core::{
    DismissReason, NavigationStep, Ownership, SelectEvent, SelectOption, SelectSession,
    SelectState, SelectionIntent, ValidatedOptions, reduce_validated_selection,
};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use std::cell::{OnceCell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

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
            ui::CloseReason::CloseButton => Self::CloseButton,
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
impl From<ui::SelectionRetention> for kit_core::SelectionRetention {
    fn from(value: ui::SelectionRetention) -> Self {
        match value {
            ui::SelectionRetention::BoundToOptions => Self::BoundToOptions,
            ui::SelectionRetention::Retain => Self::Retain,
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

fn options(rows: &[ui::OptionRow]) -> Option<ValidatedOptions<ChoiceToken>> {
    let mut generation = None;
    let options = rows
        .iter()
        .map(|row| {
            let key = ChoiceToken::new(row.token)?;
            if row.generation < 0 || generation.is_some_and(|value| value != row.generation) {
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
        .collect::<Option<Vec<_>>>()?;
    ValidatedOptions::new(options).ok()
}

/// What one option model contained the last time an event read it. Events
/// reuse it while the model still holds the same rows, so a key press does not
/// rebuild and re-validate the whole option list.
struct Snapshot {
    rows: Vec<ui::OptionRow>,
    /// `None` when the rows are not a valid option model.
    options: RefCell<Option<ValidatedOptions<ChoiceToken>>>,
    valid: bool,
    /// Lowercased label and description of each row, built on the first query.
    folded: OnceCell<Vec<(String, String)>>,
}

impl Snapshot {
    fn new(rows: Vec<ui::OptionRow>) -> Self {
        let options = options(&rows);
        Self {
            valid: options.is_some(),
            options: RefCell::new(options),
            rows,
            folded: OnceCell::new(),
        }
    }

    fn folded(&self) -> &[(String, String)] {
        self.folded.get_or_init(|| {
            self.rows
                .iter()
                .map(|row| (row.label.to_lowercase(), row.description.to_lowercase()))
                .collect()
        })
    }
}

/// Models beyond this many are rebuilt from scratch on every event.
const SNAPSHOT_SLOTS: usize = 8;

thread_local! {
    static SNAPSHOTS: RefCell<Vec<Rc<Snapshot>>> = const { RefCell::new(Vec::new()) };
}

/// The rows of `model`, registering the model as a dependency of the running
/// binding. The cache is compared by content, so a changed model is always
/// read again.
fn snapshot(model: &ModelRc<ui::OptionRow>) -> Rc<Snapshot> {
    let tracker = model.model_tracker();
    tracker.track_row_count_changes();
    let count = model.row_count();
    for index in 0..count {
        tracker.track_row_data_changes(index);
    }
    let same = |snapshot: &Snapshot| {
        snapshot.rows.len() == count
            && snapshot
                .rows
                .iter()
                .enumerate()
                .all(|(index, row)| model.row_data(index).as_ref() == Some(row))
    };
    let cached = SNAPSHOTS.with(|cache| {
        let mut cache = cache.try_borrow_mut().ok()?;
        let position = cache.iter().position(|entry| same(entry))?;
        let entry = cache.remove(position);
        cache.push(entry.clone());
        Some(entry)
    });
    if let Some(entry) = cached {
        return entry;
    }
    let entry = Rc::new(Snapshot::new(
        (0..count)
            .filter_map(|index| model.row_data(index))
            .collect(),
    ));
    SNAPSHOTS.with(|cache| {
        if let Ok(mut cache) = cache.try_borrow_mut() {
            if cache.len() == SNAPSHOT_SLOTS {
                cache.remove(0);
            }
            cache.push(entry.clone());
        }
    });
    entry
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
    let timeout_ms = u64::try_from(value.typeahead_timeout_ms)
        .ok()
        .filter(|&ms| ms > 0)
        .unwrap_or(700);
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
        typeahead_timeout_ms: timeout_ms,
        wrap: value.wrap,
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
    retention: ui::SelectionRetention,
    event: SelectEvent<ChoiceToken>,
) -> ui::SelectionTransition {
    let snapshot = snapshot(model);
    let Some(options) = snapshot.options.take() else {
        return ui::SelectionTransition {
            session: current,
            chosen: ui::SelectionRef::default(),
        };
    };
    let generation = current.state.generation;
    let (result, options) = reduce_validated_selection(
        options,
        session(current),
        ownership.into(),
        kit_core::SelectionRetention::from(retention).into(),
        event,
    );
    snapshot.options.replace(Some(options));
    let chosen = match result.intent {
        Some(SelectionIntent::Commit(token)) => reference(Some(token)),
        Some(SelectionIntent::Clear) | None => ui::SelectionRef::default(),
    };
    let timestamp = result.session.last_typeahead_ms;
    let mut state = project(
        &snapshot.rows,
        result.session.selection.selected,
        result.session.selection.active,
        result.session.selection.open,
        generation,
    );
    if retention == ui::SelectionRetention::Retain && state.selected.present {
        state.selection_availability = ui::SelectionAvailability::Available;
    }
    ui::SelectionTransition {
        chosen,
        session: ui::SelectionSession {
            state,
            typeahead: result.session.typeahead.into(),
            has_timestamp: timestamp.is_some(),
            timestamp: timestamp
                .map(|value| i32::try_from(value).expect("timestamp entered as i32"))
                .unwrap_or(0),
            typeahead_timeout_ms: i32::try_from(result.session.typeahead_timeout_ms).unwrap_or(700),
            wrap: result.session.wrap,
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
    service.on_chip_labels(|model, separator| {
        let rows = tracked(&model);
        rows.iter()
            .map(|row| row.label.as_str())
            .collect::<Vec<_>>()
            .join(separator.as_str())
            .into()
    });
    service.on_project(
        |source, model, selected, active, open, generation, retention| {
            let source = snapshot(&source);
            let rows = snapshot(&model);
            if !source.valid || !rows.valid {
                return ui::SelectState {
                    active_index: -1,
                    generation: -1,
                    ..Default::default()
                };
            }
            let mut state = project(
                &rows.rows,
                selection(selected.clone()),
                selection(active),
                open,
                generation,
            );
            state.selection_availability = match (selection(selected), retention) {
                (None, ui::SelectionRetention::BoundToOptions | ui::SelectionRetention::Retain) => {
                    ui::SelectionAvailability::Absent
                }
                (
                    Some(key),
                    ui::SelectionRetention::BoundToOptions | ui::SelectionRetention::Retain,
                ) if source.rows.iter().any(|row| row.token == key.0) => {
                    ui::SelectionAvailability::Available
                }
                (Some(_), ui::SelectionRetention::Retain) => ui::SelectionAvailability::Available,
                (Some(_), ui::SelectionRetention::BoundToOptions) => {
                    ui::SelectionAvailability::Unavailable
                }
            };
            state
        },
    );
    service.on_reconcile(|rows, current, ownership, retention| {
        reduce(&rows, current, ownership, retention, SelectEvent::Reconcile).session
    });
    service.on_open(|rows, current, ownership| {
        reduce(
            &rows,
            current,
            ownership,
            ui::SelectionRetention::BoundToOptions,
            SelectEvent::Open,
        )
        .session
    });
    service.on_navigate(|rows, current, step, page_rows, ownership| {
        let Ok(page_size) = usize::try_from(page_rows) else {
            return current;
        };
        reduce(
            &rows,
            current,
            ownership,
            ui::SelectionRetention::BoundToOptions,
            SelectEvent::Navigate {
                step: step.into(),
                page_size,
            },
        )
        .session
    });
    service.on_previous_row(|rows, current, columns, ownership| {
        let Ok(columns) = usize::try_from(columns) else {
            return current;
        };
        reduce(
            &rows,
            current,
            ownership,
            ui::SelectionRetention::BoundToOptions,
            SelectEvent::PreviousRow { columns },
        )
        .session
    });
    service.on_next_row(|rows, current, columns, ownership| {
        let Ok(columns) = usize::try_from(columns) else {
            return current;
        };
        reduce(
            &rows,
            current,
            ownership,
            ui::SelectionRetention::BoundToOptions,
            SelectEvent::NextRow { columns },
        )
        .session
    });
    service.on_typeahead_active(|current, timestamp| {
        let Ok(now_ms) = u64::try_from(timestamp) else {
            return false;
        };
        session(current).is_typeahead_active(now_ms)
    });
    service.on_dismiss(|rows, current, reason, ownership| {
        reduce(
            &rows,
            current,
            ownership,
            ui::SelectionRetention::BoundToOptions,
            SelectEvent::Dismiss(reason.into()),
        )
        .session
    });
    service.on_activate(
        |rows, current, token, generation, ownership, close_on_select| {
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
            reduce(
                &rows,
                current,
                ownership,
                ui::SelectionRetention::BoundToOptions,
                if close_on_select {
                    SelectEvent::Activate(token)
                } else {
                    SelectEvent::ActivateKeepingOpen(token)
                },
            )
        },
    );
    service.on_typeahead(|rows, current, text, timestamp, ownership| {
        let Ok(now_ms) = u64::try_from(timestamp) else {
            return current;
        };
        reduce(
            &rows,
            current,
            ownership,
            ui::SelectionRetention::BoundToOptions,
            SelectEvent::Typeahead {
                text: text.to_string(),
                now_ms,
            },
        )
        .session
    });
    service.on_rows(|model, query, generation| {
        let snapshot = snapshot(&model);
        if !snapshot.valid || generation < 0 {
            return ModelRc::default();
        }
        let query = query.to_lowercase();
        let matches = |folded: &(String, String)| {
            query.is_empty() || folded.0.contains(&query) || folded.1.contains(&query)
        };
        ModelRc::new(VecModel::from(
            snapshot
                .rows
                .iter()
                .zip(snapshot.folded())
                .filter(|(_, folded)| matches(folded))
                .map(|(row, _)| ui::OptionRow {
                    generation,
                    ..row.clone()
                })
                .collect::<Vec<_>>(),
        ))
    });
    service.on_choice_index(|model, token| {
        let rows = tracked(&model);
        let mut tokens = HashSet::new();
        let mut active = None;
        for (index, row) in rows.iter().enumerate() {
            let Some(key) = ChoiceToken::new(row.token) else {
                return -1;
            };
            if !tokens.insert(key) {
                return -1;
            }
            if row.token == token && !row.disabled {
                active = i32::try_from(index).ok();
            }
        }
        active.unwrap_or(-1)
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
        let snapshot = snapshot(&model);
        let rows = &snapshot.rows;
        if generation < 0
            || generation != current
            || !snapshot.valid
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
    service.on_option_item(|items, token| {
        tracked(&items)
            .into_iter()
            .find(|item| item.token == token)
            .unwrap_or_else(|| ui::OptionItem {
                token,
                ..Default::default()
            })
    });
    service.on_generation(|model| {
        let snapshot = snapshot(&model);
        if !snapshot.valid {
            return -1;
        }
        snapshot.rows.first().map_or(0, |row| row.generation)
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
            limit_reached: false,
        };
        let snapshot = snapshot(&rows);
        let rows = &snapshot.rows;
        let Some(options) = snapshot
            .options
            .borrow()
            .as_ref()
            .map(|options| options.as_slice().to_vec())
        else {
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
        // A token whose option was replaced is no longer a selection: it must
        // neither count against the limit nor survive a new transition.
        let selected_keys: Vec<_> = selected_keys
            .into_iter()
            .filter(|key| rows.iter().any(|row| row.token == key.0))
            .collect();
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
        let intent = match controller.toggle(&ChoiceToken(token)) {
            Ok(intent) => intent,
            Err(kit_core::ChoiceFailure::LimitReached) => {
                return ui::MultipleTransition {
                    limit_reached: true,
                    ..reject()
                };
            }
            Err(kit_core::ChoiceFailure::Missing | kit_core::ChoiceFailure::Disabled) => {
                return reject();
            }
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
            limit_reached: false,
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
