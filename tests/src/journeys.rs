// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle, LogicalPosition, Model,
    platform::{Key, PointerEventButton, WindowEvent},
};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct StepId(u16);
fn row(label: &str, state: ui::StepState, enabled: bool) -> ui::StepEntry {
    ui::StepEntry {
        label: label.into(),
        description: format!("Details for {label}").into(),
        state,
        enabled,
        ..Default::default()
    }
}
fn named(window: &JourneyFixture, label: &str) -> ElementHandle {
    ElementHandle::find_by_accessible_label(window, label)
        .next()
        .unwrap()
}
fn key(window: &JourneyFixture, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}
#[test]
fn interactive_steps_resolve_keys_and_guard_disabled_and_replaced_gestures() {
    i_slint_backend_testing::init_no_event_loop();
    let window = JourneyFixture::new().unwrap();
    let steps = slint_kit::KeyedModel::new([
        (StepId(101), row("Account", ui::StepState::Complete, true)),
        (StepId(203), row("Workspace", ui::StepState::Current, true)),
        (StepId(307), row("Invite", ui::StepState::Pending, false)),
    ])
    .unwrap();
    window.set_entries(steps.model());
    let accepted = Rc::new(RefCell::new(Vec::new()));
    window.on_step_requested({
        let steps = steps.clone();
        let accepted = accepted.clone();
        move |token, generation| {
            if let Some(id) = steps.accept(token, generation) {
                accepted.borrow_mut().push(id);
            }
        }
    });
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    key(&window, Key::Tab);
    key(&window, Key::Space);
    key(&window, Key::Return);
    assert_eq!(*accepted.borrow(), [StepId(101), StepId(101)]);
    named(&window, "Workspace, current step").mock_single_click(PointerEventButton::Left);
    named(&window, "Invite, pending").invoke_accessible_default_action();
    assert_eq!(*accepted.borrow(), [StepId(101), StepId(101), StepId(203)]);
    let account = named(&window, "Account, complete");
    let point = LogicalPosition::new(
        account.absolute_position().x + account.size().width / 2.,
        account.absolute_position().y + account.size().height / 2.,
    );
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position: point });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position: point,
        button: PointerEventButton::Left,
    });
    let stale = steps.model().row_data(0).unwrap();
    steps
        .replace([
            (StepId(203), row("Workspace", ui::StepState::Current, true)),
            (StepId(101), row("Account", ui::StepState::Complete, true)),
        ])
        .unwrap();
    slint::platform::update_timers_and_animations();
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position: point,
            button: PointerEventButton::Left,
        });
    assert_eq!(
        accepted.borrow().len(),
        3,
        "replacement cancels the held gesture"
    );
    assert!(steps.accept(stale.token, stale.generation).is_none());
    window.set_disabled(true);
    slint::platform::update_timers_and_animations();
    named(&window, "Account, complete").invoke_accessible_default_action();
    key(&window, Key::Return);
    assert_eq!(accepted.borrow().len(), 3);
}

#[test]
fn journey_layouts_and_semantic_markers_render_in_both_themes_and_text_scales() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = JourneyFixture::new().unwrap();
    window.set_entries(slint::ModelRc::new(slint::VecModel::from(vec![
        row("Account", ui::StepState::Complete, true),
        row("Workspace", ui::StepState::Current, true),
        row("Invite", ui::StepState::Pending, false),
    ])));
    window.show().unwrap();
    for (index, base) in [
        window.get_theme(),
        window.global::<ui::KitThemes>().get_dark(),
    ]
    .into_iter()
    .enumerate()
    {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            window.set_theme(theme.clone());
            window.set_events(slint::ModelRc::new(slint::VecModel::from(vec![
                ui::TimelineEntry {
                    title: "Saved".into(),
                    description: "Success".into(),
                    tone: Tone::Success,
                    ..Default::default()
                },
                ui::TimelineEntry {
                    title: "Needs attention".into(),
                    description: "Review required".into(),
                    tone: Tone::Warning,
                    glyph: theme.icons.warning.clone(),
                    ..Default::default()
                },
            ])));
            for (layout, orientation) in [
                ("vertical", slint::language::Orientation::Vertical),
                ("horizontal", slint::language::Orientation::Horizontal),
            ] {
                window.set_orientation(orientation);
                slint::platform::update_timers_and_animations();
                let first = named(&window, "Account, complete");
                let second = named(&window, "Workspace, current step");
                if orientation == slint::language::Orientation::Vertical {
                    assert!(
                        (first.absolute_position().x - second.absolute_position().x).abs() < 1.
                    );
                    assert!(
                        second.absolute_position().y
                            >= first.absolute_position().y + first.size().height
                    );
                } else {
                    assert!(
                        (first.absolute_position().y - second.absolute_position().y).abs() < 1.
                    );
                    assert!(
                        second.absolute_position().x
                            >= first.absolute_position().x + first.size().width
                    );
                }
                let marker_count = || {
                    ElementQuery::from_root(&window)
                        .match_id("StepItem::marker")
                        .find_all()
                        .len()
                };
                assert_eq!(marker_count(), 3);
                let snapshot = window.window().take_snapshot().unwrap();
                if let Some(directory) = std::env::var_os("KIT_FOUNDATION_RENDER_DIR") {
                    use std::io::Write;
                    std::fs::create_dir_all(&directory).unwrap();
                    let path = std::path::PathBuf::from(directory)
                        .join(format!("journeys-{layout}-{index}-{scale}.ppm"));
                    let mut file = std::fs::File::create(path).unwrap();
                    write!(
                        file,
                        "P6\n{} {}\n255\n",
                        snapshot.width(),
                        snapshot.height()
                    )
                    .unwrap();
                    for pixel in snapshot.as_slice() {
                        file.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
                    }
                }
                let group = named(&window, "Setup journey");
                let content_bottom = [
                    "Account, complete",
                    "Workspace, current step",
                    "Invite, pending",
                ]
                .into_iter()
                .map(|label| {
                    let item = named(&window, label);
                    item.absolute_position().y + item.size().height
                })
                .fold(group.absolute_position().y, f32::max);
                assert!(
                    (group.absolute_position().y + group.size().height - content_bottom).abs() < 1.,
                    "the step group follows its content height in both orientations"
                );
                let markers = ElementQuery::from_root(&window)
                    .match_id("KitTimeline::marker")
                    .find_all();
                assert_eq!(markers.len(), 2);
                for (marker, ink) in markers.iter().zip([theme.success, theme.warning]) {
                    let origin = marker.absolute_position();
                    let extent = marker.size();
                    assert!(origin.y >= 0. && origin.y + extent.height <= 720.);
                    let background = [
                        theme.canvas.red(),
                        theme.canvas.green(),
                        theme.canvas.blue(),
                    ]
                    .map(f64::from);
                    let expected = [ink.red(), ink.green(), ink.blue()].map(f64::from);
                    let delta: [f64; 3] =
                        std::array::from_fn(|channel| expected[channel] - background[channel]);
                    let denominator = delta.iter().map(|value| value * value).sum::<f64>();
                    assert!(denominator > 0., "the status ink differs from the canvas");
                    let width = usize::try_from(snapshot.width()).unwrap();
                    assert!(
                        snapshot
                            .as_slice()
                            .iter()
                            .enumerate()
                            .any(|(index, pixel)| {
                                let x = f64::from(u32::try_from(index % width).unwrap()) + 0.5;
                                let y = f64::from(u32::try_from(index / width).unwrap()) + 0.5;
                                if x < f64::from(origin.x)
                                    || x >= f64::from(origin.x + extent.width)
                                    || y < f64::from(origin.y)
                                    || y >= f64::from(origin.y + extent.height)
                                {
                                    return false;
                                }
                                let actual = [pixel.r, pixel.g, pixel.b].map(f64::from);
                                let coverage = (0..3)
                                    .map(|channel| {
                                        (actual[channel] - background[channel]) * delta[channel]
                                    })
                                    .sum::<f64>()
                                    / denominator;
                                // A thin glyph may contain no fully opaque pixel. Require
                                // visible status ink in this marker's bounds, allowing only
                                // alpha coverage and two levels of byte rounding error.
                                (0.25..=1.01).contains(&coverage)
                                    && (0..3).all(|channel| {
                                        (actual[channel]
                                            - (background[channel] + coverage * delta[channel]))
                                            .abs()
                                            <= 2.
                                    })
                            }),
                        "timeline marker renders its semantic ink within its own bounds"
                    );
                }
                window.set_connectors(false);
                slint::platform::update_timers_and_animations();
                assert_eq!(
                    marker_count(),
                    3,
                    "hiding connectors preserves markers and labels"
                );
                window.set_connectors(true);
            }
        }
    }
}
