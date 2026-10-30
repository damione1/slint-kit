// SPDX-License-Identifier: GPL-3.0-or-later
//! Window-local, typed application models for the choice examples.
use super::ui;
use kit::{
    CheckState, DecimalStep, DecimalValue, DismissReason, MultiSelectController, NavigationStep,
    NumericBounds, OrderedRange, Ownership, ParseIssue, RangeController, Rating, RatingScale,
    SearchController, SearchState, SelectBinding, SelectController, SelectOption, SelectProjection,
    SelectionAvailability, SelectionIntent, StepDirection,
};
use slint::ComponentHandle;
use std::{cell::RefCell, convert::Infallible, hash::Hash, rc::Rc, time::Duration};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Plan {
    Personal,
    Team,
    Enterprise,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Layout {
    List,
    Board,
    Calendar,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ExportFormat {
    Pdf,
    Csv,
    Json,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ScoreStep(u16);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct MemberId(u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TagId(i32);
struct Tag {
    id: TagId,
    label: String,
    comparison: String,
}

const CHOICE_GENERATION: i32 = 1;

struct Choices {
    consent: CheckState,
    notifications: bool,
    radio: SelectController<Plan>,
    segmented: SelectController<Layout>,
    toggles: MultiSelectController<ExportFormat>,
    rating_choices: SelectController<ScoreStep>,
    rating: Option<Rating>,
    tags: Vec<Tag>,
    next_tag: i32,
    slider: ExactSlider,
    range: RangeController,
}

fn single<K: Clone + Eq + Hash>(
    options: Vec<SelectOption<K>>,
    selected: Option<K>,
) -> SelectController<K> {
    let mut value = SelectController::new(options, selected, Ownership::Local);
    value.open();
    value
}
fn decimal(value: i64) -> DecimalValue {
    DecimalValue::from_integer(value)
}
fn bounds() -> NumericBounds<DecimalValue> {
    NumericBounds::new(decimal(0), decimal(100)).expect("constant ordered bounds")
}
impl Choices {
    fn new() -> Self {
        let mut unavailable = SelectOption::new(Plan::Enterprise, "Enterprise — unavailable");
        unavailable.disabled = true;
        let mut toggles = MultiSelectController::new(
            vec![
                SelectOption::new(ExportFormat::Pdf, "PDF"),
                SelectOption::new(ExportFormat::Csv, "CSV"),
                SelectOption::new(ExportFormat::Json, "JSON"),
            ],
            vec![ExportFormat::Pdf],
            Ownership::Local,
            None,
        )
        .expect("unique enum keys");
        toggles.navigate(NavigationStep::First, 3);
        Self {
            consent: CheckState::Mixed,
            notifications: true,
            radio: single(
                vec![
                    SelectOption::new(Plan::Personal, "Personal"),
                    SelectOption::new(Plan::Team, "Team"),
                    unavailable,
                ],
                Some(Plan::Team),
            ),
            segmented: single(
                vec![
                    SelectOption::new(Layout::List, "List"),
                    SelectOption::new(Layout::Board, "Board"),
                    SelectOption::new(Layout::Calendar, "Calendar"),
                ],
                Some(Layout::Board),
            ),
            toggles,
            rating_choices: single(
                (1..=5)
                    .map(|step| SelectOption::new(ScoreStep(step), format!("{step} / 5")))
                    .collect(),
                None,
            ),
            rating: None,
            tags: vec![
                Tag {
                    id: TagId(1),
                    label: "native".into(),
                    comparison: "native".into(),
                },
                Tag {
                    id: TagId(2),
                    label: "design".into(),
                    comparison: "design".into(),
                },
            ],
            next_tag: 3,
            slider: ExactSlider {
                value: decimal(35),
                snapshot: None,
            },
            range: RangeController::new(
                bounds(),
                OrderedRange::new(decimal(20), decimal(80)).expect("ordered range"),
                DecimalStep::new(decimal(1)).expect("positive step"),
            )
            .expect("bounded range"),
        }
    }
}

struct ExactSlider {
    value: DecimalValue,
    snapshot: Option<DecimalValue>,
}
impl ExactSlider {
    fn set(&mut self, value: DecimalValue) -> Result<(), ParseIssue> {
        bounds().validate(value)?;
        self.value = value;
        Ok(())
    }
    fn navigate(&mut self, step: NavigationStep) -> Result<(), ParseIssue> {
        self.set(navigation_value(
            self.value,
            step,
            decimal(0),
            decimal(100),
        )?)
    }
    fn gesture(&mut self, gesture: ui::SliderGesture) {
        match gesture {
            ui::SliderGesture::Started => self.snapshot = Some(self.value),
            ui::SliderGesture::Committed => self.snapshot = None,
            ui::SliderGesture::Cancelled => {
                if let Some(value) = self.snapshot.take() {
                    self.value = value;
                }
            }
        }
    }
}

fn navigation(step: ui::NavigationStep) -> NavigationStep {
    match step {
        ui::NavigationStep::Next => NavigationStep::Next,
        ui::NavigationStep::Previous => NavigationStep::Previous,
        ui::NavigationStep::First => NavigationStep::First,
        ui::NavigationStep::Last => NavigationStep::Last,
        ui::NavigationStep::PageNext => NavigationStep::PageNext,
        ui::NavigationStep::PagePrevious => NavigationStep::PagePrevious,
    }
}
fn thumb(value: ui::RangeThumb) -> kit::RangeThumb {
    match value {
        ui::RangeThumb::Lower => kit::RangeThumb::Start,
        ui::RangeThumb::Upper => kit::RangeThumb::End,
    }
}
fn navigation_value(
    value: DecimalValue,
    step: NavigationStep,
    minimum: DecimalValue,
    maximum: DecimalValue,
) -> Result<DecimalValue, ParseIssue> {
    match step {
        NavigationStep::First => Ok(minimum),
        NavigationStep::Last => Ok(maximum),
        NavigationStep::Next => DecimalStep::new(decimal(1))?.apply(
            value,
            StepDirection::Increment,
            NumericBounds::new(minimum, maximum)?,
        ),
        NavigationStep::Previous => DecimalStep::new(decimal(1))?.apply(
            value,
            StepDirection::Decrement,
            NumericBounds::new(minimum, maximum)?,
        ),
        NavigationStep::PageNext => DecimalStep::new(decimal(10))?.apply(
            value,
            StepDirection::Increment,
            NumericBounds::new(minimum, maximum)?,
        ),
        NavigationStep::PagePrevious => DecimalStep::new(decimal(10))?.apply(
            value,
            StepDirection::Decrement,
            NumericBounds::new(minimum, maximum)?,
        ),
    }
}
// The pointer supplies a finite normalized renderer position. This demo's domain
// is exact integral percentage points, quantized once at that explicit boundary.
fn pointer_value(position: f32) -> Result<DecimalValue, ParseIssue> {
    if !position.is_finite() || !(0.0..=1.0).contains(&position) {
        return Err(ParseIssue::InvalidRange);
    }
    let points = format!("{:.0}", (position * 100.0).round())
        .parse::<i64>()
        .map_err(|_| ParseIssue::Overflow)?;
    Ok(decimal(points))
}
fn position(value: DecimalValue) -> f32 {
    value
        .to_string()
        .parse::<f32>()
        .expect("bounded renderer projection")
        / 100.0
}
fn model<T: Clone + 'static>(values: Vec<T>) -> slint::ModelRc<T> {
    slint::ModelRc::new(slint::VecModel::from(values))
}
fn active<K: Clone + Eq + Hash>(controller: &SelectController<K>) -> i32 {
    controller
        .state()
        .active
        .as_ref()
        .and_then(|key| controller.options().iter().position(|row| &row.key == key))
        .and_then(|index| i32::try_from(index).ok())
        .unwrap_or(-1)
}
fn single_rows<K: Clone + Eq + Hash>(controller: &SelectController<K>) -> Vec<ui::ChoiceRow> {
    controller
        .options()
        .iter()
        .enumerate()
        .map(|(index, row)| ui::ChoiceRow {
            token: i32::try_from(index).expect("bounded catalogue"),
            label: row.label.as_str().into(),
            disabled: row.disabled,
            selected: controller.state().selected.as_ref() == Some(&row.key),
        })
        .collect()
}
fn single_navigate<K: Clone + Eq + Hash>(
    controller: &mut SelectController<K>,
    step: NavigationStep,
) {
    controller.navigate(step, 3);
    controller.set_selected(controller.state().active.clone());
}
fn single_activate<K: Clone + Eq + Hash>(
    controller: &mut SelectController<K>,
    token: i32,
    generation: i32,
) {
    if generation != CHOICE_GENERATION {
        return;
    }
    let Some(key) = usize::try_from(token)
        .ok()
        .and_then(|index| controller.options().get(index))
        .map(|row| row.key.clone())
    else {
        return;
    };
    controller.activate(&key);
    controller.open();
}
fn publish(window: &ui::Showcase, state: &Choices) {
    let view = window.global::<ui::FormExamples>();
    view.set_choice_generation(CHOICE_GENERATION);
    view.set_choices_consent(match state.consent {
        CheckState::Unchecked => ui::CheckState::Unchecked,
        CheckState::Checked => ui::CheckState::Checked,
        CheckState::Mixed => ui::CheckState::Mixed,
    });
    view.set_choices_notifications(state.notifications);
    view.set_radio_options(model(single_rows(&state.radio)));
    view.set_radio_active(active(&state.radio));
    view.set_segmented_options(model(single_rows(&state.segmented)));
    view.set_segmented_active(active(&state.segmented));
    view.set_toggle_options(model(
        state
            .toggles
            .options()
            .iter()
            .enumerate()
            .map(|(index, row)| ui::ChoiceRow {
                token: i32::try_from(index).expect("bounded catalogue"),
                label: row.label.as_str().into(),
                disabled: row.disabled,
                selected: state.toggles.selected().contains(&row.key),
            })
            .collect(),
    ));
    view.set_toggle_active(
        state
            .toggles
            .active()
            .and_then(|key| {
                state
                    .toggles
                    .options()
                    .iter()
                    .position(|row| &row.key == key)
            })
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(-1),
    );
    view.set_rating_options(model(single_rows(&state.rating_choices)));
    view.set_rating_active(active(&state.rating_choices));
    view.set_rating_label(
        state
            .rating
            .as_ref()
            .map_or_else(
                || "Not rated".to_owned(),
                |_| {
                    format!(
                        "Rating: {} of 5",
                        state
                            .rating_choices
                            .state()
                            .selected
                            .as_ref()
                            .expect("chosen score")
                            .0
                    )
                },
            )
            .into(),
    );
    view.set_tags(model(
        state
            .tags
            .iter()
            .map(|tag| ui::ChipRow {
                token: tag.id.0,
                label: tag.label.as_str().into(),
                removable: true,
            })
            .collect(),
    ));
    view.set_slider_position(position(state.slider.value));
    view.set_slider_label(format!("{} %", state.slider.value).into());
    let range = state.range.value();
    view.set_range_lower(position(range.start()));
    view.set_range_upper(position(range.end()));
    view.set_range_lower_label(range.start().to_string().into());
    view.set_range_upper_label(range.end().to_string().into());
}
fn update_rating(state: &mut Choices) {
    state.rating = state.rating_choices.state().selected.as_ref().map(|score| {
        RatingScale::new(5)
            .expect("nonempty scale")
            .score(score.0)
            .expect("score belongs to model")
    });
}
fn status(window: &ui::Showcase, value: impl Into<slint::SharedString>) {
    window
        .global::<ui::FormExamples>()
        .set_choices_status(value.into());
}

pub fn install(window: &ui::Showcase) {
    let state = Rc::new(RefCell::new(Choices::new()));
    publish(window, &state.borrow());
    status(
        window,
        "Stable identities, one Tab stop per choice group. Tags compare trimmed Unicode lowercase text; accents remain distinct.",
    );
    let weak = window.as_weak();
    let target = state.clone();
    window
        .global::<ui::FormExamples>()
        .on_choices_consent_toggled(move |value| {
            let Some(window) = weak.upgrade() else {
                return;
            };
            target.borrow_mut().consent = match value {
                ui::CheckState::Unchecked => CheckState::Unchecked,
                ui::CheckState::Checked => CheckState::Checked,
                ui::CheckState::Mixed => CheckState::Mixed,
            };
            publish(&window, &target.borrow());
        });
    let weak = window.as_weak();
    let target = state.clone();
    window
        .global::<ui::FormExamples>()
        .on_choices_notifications_toggled(move |value| {
            let Some(window) = weak.upgrade() else {
                return;
            };
            target.borrow_mut().notifications = value;
            publish(&window, &target.borrow());
        });
    let weak = window.as_weak();
    let target = state.clone();
    window
        .global::<ui::FormExamples>()
        .on_choice_navigate(move |example, step| {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let mut state = target.borrow_mut();
            let step = navigation(step);
            match example {
                ui::ChoiceExample::Radio => single_navigate(&mut state.radio, step),
                ui::ChoiceExample::Segmented => single_navigate(&mut state.segmented, step),
                ui::ChoiceExample::Toggle => {
                    state.toggles.navigate(step, 3);
                }
                ui::ChoiceExample::Rating => {
                    single_navigate(&mut state.rating_choices, step);
                    update_rating(&mut state);
                }
            }
            publish(&window, &state);
        });
    let weak = window.as_weak();
    let target = state.clone();
    window
        .global::<ui::FormExamples>()
        .on_choice_activate(move |example, token, generation| {
            let Some(window) = weak.upgrade() else {
                return;
            };
            if generation != CHOICE_GENERATION {
                return;
            }
            let mut state = target.borrow_mut();
            match example {
                ui::ChoiceExample::Radio => single_activate(&mut state.radio, token, generation),
                ui::ChoiceExample::Segmented => {
                    single_activate(&mut state.segmented, token, generation)
                }
                ui::ChoiceExample::Toggle => {
                    if let Some(key) = usize::try_from(token)
                        .ok()
                        .and_then(|index| state.toggles.options().get(index))
                        .map(|row| row.key)
                    {
                        let _ = state.toggles.toggle(&key);
                    }
                }
                ui::ChoiceExample::Rating => {
                    single_activate(&mut state.rating_choices, token, generation);
                    update_rating(&mut state);
                }
            }
            publish(&window, &state);
        });
    let weak = window.as_weak();
    let target = state.clone();
    window
        .global::<ui::FormExamples>()
        .on_rating_clear(move || {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let mut state = target.borrow_mut();
            state.rating = None;
            state.rating_choices.set_selected(None);
            publish(&window, &state);
        });
    install_tags(window, state.clone());
    install_sliders(window, state);
    install_suggestions(window);
    install_multi(window);
}

fn install_tags(window: &ui::Showcase, state: Rc<RefCell<Choices>>) {
    let weak = window.as_weak();
    let target = state.clone();
    window
        .global::<ui::FormExamples>()
        .on_tag_create(move |text| {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let label = text.trim();
            let comparison = label.to_lowercase();
            let mut state = target.borrow_mut();
            let failure = if label.is_empty() {
                Some("Enter a tag.")
            } else if label.chars().count() > 32 {
                Some("Tags are limited to 32 Unicode scalar values.")
            } else if state.tags.len() >= 10 {
                Some("A maximum of ten tags is allowed.")
            } else if state.tags.iter().any(|tag| tag.comparison == comparison) {
                Some("This tag already exists (case-insensitive).")
            } else {
                None
            };
            if let Some(error) = failure {
                window
                    .global::<ui::FormExamples>()
                    .set_tag_error(error.into());
                return;
            }
            let Some(next) = state.next_tag.checked_add(1) else {
                window
                    .global::<ui::FormExamples>()
                    .set_tag_error("Tag identity space exhausted.".into());
                return;
            };
            let id = TagId(state.next_tag);
            state.next_tag = next;
            state.tags.push(Tag {
                id,
                label: label.into(),
                comparison,
            });
            window.global::<ui::FormExamples>().set_tag_error("".into());
            window.global::<ui::FormExamples>().set_tag_text("".into());
            publish(&window, &state);
        });
    let weak = window.as_weak();
    window
        .global::<ui::FormExamples>()
        .on_tag_remove(move |token| {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let mut state = state.borrow_mut();
            state.tags.retain(|tag| tag.id != TagId(token));
            publish(&window, &state);
        });
}

fn install_sliders(window: &ui::Showcase, state: Rc<RefCell<Choices>>) {
    // The input drafts are initialized once and then stay native/owned by the
    // editor until commit. Updating unrelated choices never rewrites a caret.
    {
        let state = state.borrow();
        let view = window.global::<ui::FormExamples>();
        view.set_slider_text(state.slider.value.to_string().into());
        view.set_range_lower_text(state.range.value().start().to_string().into());
        view.set_range_upper_text(state.range.value().end().to_string().into());
    }
    let weak = window.as_weak();
    let target = state.clone();
    window
        .global::<ui::FormExamples>()
        .on_slider_text_commit(move |text| {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let mut state = target.borrow_mut();
            match DecimalValue::parse_canonical(text.trim()).and_then(|value| {
                if value.scale() != 0 {
                    return Err(ParseIssue::ScaleExceeded);
                }
                state.slider.set(value)
            }) {
                Ok(()) => {
                    window
                        .global::<ui::FormExamples>()
                        .set_slider_text(state.slider.value.to_string().into());
                    window
                        .global::<ui::FormExamples>()
                        .set_slider_error("".into());
                }
                Err(_) => window
                    .global::<ui::FormExamples>()
                    .set_slider_error("Use a whole number from 0 to 100.".into()),
            }
            publish(&window, &state);
        });
    let weak = window.as_weak();
    let target = state.clone();
    window
        .global::<ui::FormExamples>()
        .on_range_text_commit(move |view_thumb, text| {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let mut state = target.borrow_mut();
            match DecimalValue::parse_canonical(text.trim()).and_then(|value| {
                if value.scale() != 0 {
                    return Err(ParseIssue::ScaleExceeded);
                }
                state.range.set(thumb(view_thumb), value)
            }) {
                Ok(()) => {
                    let range = state.range.value();
                    let view = window.global::<ui::FormExamples>();
                    match view_thumb {
                        ui::RangeThumb::Lower => {
                            view.set_range_lower_text(range.start().to_string().into())
                        }
                        ui::RangeThumb::Upper => {
                            view.set_range_upper_text(range.end().to_string().into())
                        }
                    };
                    view.set_range_error("".into());
                    match view_thumb {
                        ui::RangeThumb::Lower => view.set_range_lower_error("".into()),
                        ui::RangeThumb::Upper => view.set_range_upper_error("".into()),
                    }
                }
                Err(_) => {
                    let view = window.global::<ui::FormExamples>();
                    let message = "Use whole numbers from 0 to 100, with start ≤ end.".into();
                    match view_thumb {
                        ui::RangeThumb::Lower => view.set_range_lower_error(message),
                        ui::RangeThumb::Upper => view.set_range_upper_error(message),
                    }
                }
            }
            publish(&window, &state);
        });
    let weak = window.as_weak();
    let target = state.clone();
    window
        .global::<ui::FormExamples>()
        .on_slider_position_requested(move |value| {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let mut state = target.borrow_mut();
            if let Err(issue) = pointer_value(value).and_then(|value| state.slider.set(value)) {
                window
                    .global::<ui::FormExamples>()
                    .set_slider_error(format!("Choose a value from 0 to 100 ({issue:?}).").into());
            }
            window
                .global::<ui::FormExamples>()
                .set_slider_text(state.slider.value.to_string().into());
            publish(&window, &state);
        });
    let weak = window.as_weak();
    let target = state.clone();
    window
        .global::<ui::FormExamples>()
        .on_slider_navigate(move |step| {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let mut state = target.borrow_mut();
            if let Err(issue) = state.slider.navigate(navigation(step)) {
                window
                    .global::<ui::FormExamples>()
                    .set_slider_error(format!("Choose a value from 0 to 100 ({issue:?}).").into());
            }
            window
                .global::<ui::FormExamples>()
                .set_slider_text(state.slider.value.to_string().into());
            publish(&window, &state);
        });
    let weak = window.as_weak();
    let target = state.clone();
    window
        .global::<ui::FormExamples>()
        .on_slider_gesture(move |gesture| {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let mut state = target.borrow_mut();
            state.slider.gesture(gesture);
            window
                .global::<ui::FormExamples>()
                .set_slider_text(state.slider.value.to_string().into());
            publish(&window, &state);
        });
    let weak = window.as_weak();
    let target = state.clone();
    window
        .global::<ui::FormExamples>()
        .on_range_position_requested(move |view_thumb, value| {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let mut state = target.borrow_mut();
            if let Err(issue) =
                pointer_value(value).and_then(|value| state.range.set(thumb(view_thumb), value))
            {
                window.global::<ui::FormExamples>().set_range_error(
                    format!("Keep start ≤ end within 0 to 100 ({issue:?}).").into(),
                );
            }
            sync_range_text(&window, &state);
            publish(&window, &state);
        });
    let weak = window.as_weak();
    let target = state.clone();
    window
        .global::<ui::FormExamples>()
        .on_range_navigate(move |view_thumb, step| {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let mut state = target.borrow_mut();
            let range = state.range.value();
            let thumb = thumb(view_thumb);
            let (current, minimum, maximum) = match thumb {
                kit::RangeThumb::Start => (range.start(), decimal(0), range.end()),
                kit::RangeThumb::End => (range.end(), range.start(), decimal(100)),
            };
            if let Err(issue) = navigation_value(current, navigation(step), minimum, maximum)
                .and_then(|value| state.range.set(thumb, value))
            {
                window.global::<ui::FormExamples>().set_range_error(
                    format!("Keep start ≤ end within 0 to 100 ({issue:?}).").into(),
                );
            }
            sync_range_text(&window, &state);
            publish(&window, &state);
        });
    let weak = window.as_weak();
    window
        .global::<ui::FormExamples>()
        .on_range_gesture(move |gesture| {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let mut state = state.borrow_mut();
            match gesture {
                ui::SliderGesture::Started => state.range.begin_drag(),
                ui::SliderGesture::Committed => state.range.commit_drag(),
                ui::SliderGesture::Cancelled => state.range.cancel_drag(),
            };
            sync_range_text(&window, &state);
            publish(&window, &state);
        });
}

fn sync_range_text(window: &ui::Showcase, state: &Choices) {
    let view = window.global::<ui::FormExamples>();
    let range = state.range.value();
    view.set_range_lower_text(range.start().to_string().into());
    view.set_range_upper_text(range.end().to_string().into());
}

fn members() -> Vec<SelectOption<MemberId>> {
    (1..=256)
        .map(|id| {
            let mut row = SelectOption::new(
                MemberId(id),
                format!(
                    "{} · {id:04}",
                    match id % 4 {
                        0 => "Elliot Alderson",
                        1 => "Darlene Alderson",
                        2 => "Trenton",
                        3 => "Mobley",
                        _ => unreachable!("modulo four"),
                    }
                ),
            );
            row.description = format!("Collaborator #{id:04}");
            row.disabled = id % 13 == 0;
            row
        })
        .collect()
}
fn projected(snapshot: &SelectProjection) -> (slint::ModelRc<ui::OptionRow>, ui::SelectState) {
    let selection = |token: Option<i32>| ui::SelectionRef {
        present: token.is_some(),
        token: token.unwrap_or(-1),
    };
    (
        model(
            snapshot
                .rows
                .iter()
                .map(|row| ui::OptionRow {
                    token: row.token,
                    generation: row.generation,
                    label: row.label.as_str().into(),
                    description: row.description.as_str().into(),
                    disabled: row.disabled,
                })
                .collect(),
        ),
        ui::SelectState {
            selected: selection(snapshot.selected),
            active: selection(snapshot.active),
            open: snapshot.open,
            active_index: snapshot.active_index,
            generation: snapshot.generation,
            selection_availability: match snapshot.selection_availability {
                SelectionAvailability::Absent => ui::SelectionAvailability::Absent,
                SelectionAvailability::Available => ui::SelectionAvailability::Available,
                SelectionAvailability::Unavailable => ui::SelectionAvailability::Unavailable,
            },
        },
    )
}
fn install_suggestions(window: &ui::Showcase) {
    let catalog = Rc::new(members());
    let selected = Rc::new(RefCell::new(None::<MemberId>));
    let weak = window.as_weak();
    let binding = SelectBinding::new(
        catalog.as_ref().clone(),
        None,
        Ownership::Controlled,
        move |snapshot| {
            if let Some(window) = weak.upgrade() {
                let (rows, state) = projected(&snapshot);
                let view = window.global::<ui::FormExamples>();
                view.set_suggestion_rows(rows);
                view.set_suggestion_state(state);
            }
        },
    );
    let choice = selected.clone();
    binding.on_intent(move |intent| match intent {
        SelectionIntent::Commit(key) => *choice.borrow_mut() = Some(key),
        SelectionIntent::Clear => {}
    });
    let target = binding.clone();
    window
        .global::<ui::FormExamples>()
        .on_suggestion_open(move || target.open());
    let target = binding.clone();
    window
        .global::<ui::FormExamples>()
        .on_suggestion_navigate(move |step| target.navigate(navigation(step)));
    let target = binding.clone();
    window
        .global::<ui::FormExamples>()
        .on_suggestion_dismiss(move || target.dismiss(DismissReason::Programmatic));
    let target = binding.clone();
    window
        .global::<ui::FormExamples>()
        .on_suggestion_typeahead(move |text| target.typeahead(text.as_str()));
    let target = binding.clone();
    let choice = selected.clone();
    let source = catalog.clone();
    let weak = window.as_weak();
    window
        .global::<ui::FormExamples>()
        .on_suggestion_activate(move |token, generation| {
            target.activate(token, generation);
            let selected = *choice.borrow();
            target.set_selected(selected);
            if let Some(window) = weak.upgrade() {
                let label = selected
                    .and_then(|key| source.iter().find(|row| row.key == key))
                    .map_or("", |row| row.label.as_str());
                window
                    .global::<ui::FormExamples>()
                    .set_suggestion_label(label.into());
            }
        });
    let search = Rc::new(RefCell::new(SearchController::<
        SelectOption<MemberId>,
        Infallible,
    >::default()));
    let weak = window.as_weak();
    window
        .global::<ui::FormExamples>()
        .on_suggestion_query(move |query| {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let ticket = search.borrow_mut().search(query.to_string());
            let Some(ticket) = ticket else {
                let _ = binding.reconcile(catalog.as_ref().clone());
                window
                    .global::<ui::FormExamples>()
                    .set_suggestion_search(ui::SearchPresentation::Ready);
                return;
            };
            window
                .global::<ui::FormExamples>()
                .set_suggestion_search(ui::SearchPresentation::Loading);
            let _ = binding.reconcile(vec![]);
            let lower = query.to_lowercase();
            let rows = catalog
                .iter()
                .filter(|row| {
                    row.label.to_lowercase().contains(&lower)
                        || row.description.to_lowercase().contains(&lower)
                })
                .cloned()
                .collect();
            let weak_state = Rc::downgrade(&search);
            let weak_window = window.as_weak();
            let target = binding.clone();
            slint::Timer::single_shot(Duration::from_millis(120), move || {
                let (Some(search), Some(window)) = (weak_state.upgrade(), weak_window.upgrade())
                else {
                    return;
                };
                if !search.borrow_mut().complete(ticket, Ok(rows)) {
                    return;
                }
                let state = search.borrow();
                match state.state() {
                    SearchState::Ready(rows) => {
                        let _ = target.reconcile(rows.clone());
                        window
                            .global::<ui::FormExamples>()
                            .set_suggestion_search(ui::SearchPresentation::Ready);
                    }
                    SearchState::Empty => {
                        window
                            .global::<ui::FormExamples>()
                            .set_suggestion_search(ui::SearchPresentation::Idle);
                    }
                    SearchState::Loading(_) => {
                        window
                            .global::<ui::FormExamples>()
                            .set_suggestion_search(ui::SearchPresentation::Loading);
                    }
                    SearchState::Failed(error) => match *error {},
                }
            });
        });
}

fn publish_multi(window: &ui::Showcase, choices: &MultiSelectController<MemberId>) {
    window.global::<ui::FormExamples>().set_multi_chips(model(
        choices
            .selected()
            .iter()
            .filter_map(|key| choices.options().iter().find(|row| &row.key == key))
            .map(|row| ui::ChipRow {
                token: i32::try_from(row.key.0).expect("bounded identities"),
                label: row.label.as_str().into(),
                removable: true,
            })
            .collect(),
    ));
}
fn available(choices: &MultiSelectController<MemberId>) -> Vec<SelectOption<MemberId>> {
    choices
        .options()
        .iter()
        .filter(|row| !choices.selected().contains(&row.key))
        .cloned()
        .collect()
}
fn install_multi(window: &ui::Showcase) {
    let choices = Rc::new(RefCell::new(
        MultiSelectController::new(
            members(),
            vec![MemberId(1), MemberId(2)],
            Ownership::Local,
            Some(5),
        )
        .expect("unique member keys"),
    ));
    publish_multi(window, &choices.borrow());
    let weak = window.as_weak();
    let binding = SelectBinding::new(
        available(&choices.borrow()),
        None,
        Ownership::Controlled,
        move |snapshot| {
            if let Some(window) = weak.upgrade() {
                let (rows, state) = projected(&snapshot);
                let view = window.global::<ui::FormExamples>();
                view.set_multi_rows(rows);
                view.set_multi_state(state);
            }
        },
    );
    let pending = Rc::new(RefCell::new(None));
    let intent = pending.clone();
    binding.on_intent(move |value| {
        *intent.borrow_mut() = Some(value);
    });
    let target = binding.clone();
    window
        .global::<ui::FormExamples>()
        .on_multi_open(move || target.open());
    let target = binding.clone();
    window
        .global::<ui::FormExamples>()
        .on_multi_navigate(move |step| target.navigate(navigation(step)));
    let target = binding.clone();
    window
        .global::<ui::FormExamples>()
        .on_multi_dismiss(move || target.dismiss(DismissReason::Programmatic));
    let target = binding.clone();
    window
        .global::<ui::FormExamples>()
        .on_multi_typeahead(move |text| target.typeahead(text.as_str()));
    let target = binding.clone();
    window
        .global::<ui::FormExamples>()
        .on_multi_query(move |query| target.query(query.as_str()));
    let target = binding.clone();
    let state = choices.clone();
    let weak = window.as_weak();
    window
        .global::<ui::FormExamples>()
        .on_multi_activate(move |token, generation| {
            pending.borrow_mut().take();
            target.activate(token, generation);
            let Some(SelectionIntent::Commit(key)) = pending.borrow_mut().take() else {
                return;
            };
            let Some(window) = weak.upgrade() else {
                return;
            };
            {
                let mut choices = state.borrow_mut();
                if let Err(issue) = choices.toggle(&key) {
                    window.global::<ui::FormExamples>().set_multi_error(
                        format!("Select up to five reviewers ({issue:?}).").into(),
                    );
                    return;
                }
                publish_multi(&window, &choices);
                let _ = target.reconcile(available(&choices));
            }
            target.set_selected(None);
            target.open();
        });
    let weak = window.as_weak();
    window
        .global::<ui::FormExamples>()
        .on_multi_remove(move |token| {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let Ok(raw) = u32::try_from(token) else {
                return;
            };
            let key = MemberId(raw);
            let mut choices = choices.borrow_mut();
            if choices.selected().contains(&key) {
                let _ = choices.toggle(&key);
                publish_multi(&window, &choices);
                let _ = binding.reconcile(available(&choices));
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalized_pointer_is_quantized_at_the_domain_boundary() {
        assert_eq!(pointer_value(0.355).unwrap(), decimal(36));
        assert_eq!(pointer_value(1.0).unwrap(), decimal(100));
        assert!(pointer_value(f32::NAN).is_err());
        assert!(pointer_value(-0.1).is_err());
    }
    #[test]
    fn range_navigation_uses_the_other_thumb_as_a_bound() {
        assert_eq!(
            navigation_value(
                decimal(80),
                NavigationStep::First,
                decimal(20),
                decimal(100)
            )
            .unwrap(),
            decimal(20)
        );
        assert!(
            navigation_value(decimal(80), NavigationStep::Next, decimal(0), decimal(80)).is_err()
        );
    }
    #[test]
    fn choice_demo_never_changes_profile_consent_or_notifications() {
        i_slint_backend_testing::init_no_event_loop();
        let window = ui::Showcase::new().unwrap();
        super::super::form_fields::install(&window);
        install(&window);
        let form = window.global::<ui::FormExamples>();
        form.set_consent(ui::CheckState::Unchecked);
        form.set_notifications(false);
        form.invoke_choices_consent_toggled(ui::CheckState::Checked);
        form.invoke_choices_notifications_toggled(true);
        assert_eq!(form.get_consent(), ui::CheckState::Unchecked);
        assert!(!form.get_notifications());
        assert_eq!(form.get_choices_consent(), ui::CheckState::Checked);
        assert!(form.get_choices_notifications());
    }
}
