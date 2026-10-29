// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::ui;
use crate::variants::{alert_severities, control_sizes};
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{ComponentHandle, Model};

fn show<T: ComponentHandle>(window: T) -> T {
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    window
}
fn labelled(
    root: &impl i_slint_backend_testing::ElementRoot,
    label: &str,
) -> Option<ElementHandle> {
    ElementHandle::find_by_accessible_label(root, label).next()
}
fn contains(outer: &ElementHandle, inner: &ElementHandle) -> bool {
    let (o, i) = (outer.absolute_position(), inner.absolute_position());
    let (os, is) = (outer.size(), inner.size());
    let tolerance = 0.01;
    i.x >= o.x - tolerance
        && i.y >= o.y - tolerance
        && i.x + is.width <= o.x + os.width + tolerance
        && i.y + is.height <= o.y + os.height + tolerance
}

#[test]
fn field_action_icons_fit_their_buttons_at_scaled_text() {
    i_slint_backend_testing::init_no_event_loop();
    let window = show(FieldActionFixture::new().unwrap());
    for base in [window.get_theme(), window.get_tailwind()] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            window.set_theme(theme.clone());
            slint::platform::update_timers_and_animations();
            for size in control_sizes() {
                let labels: &[&str] = match size {
                    ControlSize::Compact => &["Clear compact", "Reveal compact"],
                    ControlSize::Medium => &["Clear medium", "Reveal medium"],
                    ControlSize::Comfortable => &["Clear comfortable"],
                };
                let side = window.invoke_icon_size(theme.clone(), size);
                for label in labels {
                    let action = labelled(&window, label)
                        .unwrap_or_else(|| panic!("missing action {label}"));
                    let icon = action
                        .query_descendants()
                        .match_inherits("KitIcon")
                        .find_first()
                        .expect("the action draws an icon");
                    // The icon follows the field's size and the text scale...
                    assert_eq!(icon.size().width, side, "{label} at {scale}");
                    // ...and its pressable grows with it, so nothing spills out.
                    assert!(
                        contains(&action, &icon),
                        "{label} at {scale}: icon {:?} {:?} outside action {:?} {:?}",
                        icon.absolute_position(),
                        icon.size(),
                        action.absolute_position(),
                        action.size()
                    );
                }
            }
        }
    }
}

/// The child that paints a pressable's elevation: a rectangle over the whole host.
fn shadow_layer(button: &ElementHandle) -> Option<ElementHandle> {
    let (origin, size) = (button.absolute_position(), button.size());
    button
        .query_descendants()
        .match_type_name("Rectangle")
        .match_predicate(move |e| e.absolute_position() == origin && e.size() == size)
        .find_first()
}

#[test]
fn control_elevation_lifts_filled_buttons_without_moving_the_layout() {
    i_slint_backend_testing::init_no_event_loop();
    let window = show(ControlElevationFixture::new().unwrap());
    let labels = [
        ("Primary lift", true),
        ("Secondary lift", true),
        ("Destructive lift", true),
        ("Outline flat", false),
        ("Quiet flat", false),
    ];
    let measure = |theme: KitTheme| {
        window.set_theme(theme);
        slint::platform::update_timers_and_animations();
        let heights: Vec<f32> = labels
            .iter()
            .map(|(label, _)| labelled(&window, label).unwrap().size().height)
            .collect();
        (heights, window.get_stack_height())
    };
    let tailwind = window.get_tailwind();
    assert_eq!(tailwind.elevation.control, ElevationLevel::Xs);
    let mut flat = tailwind.clone();
    flat.elevation.control = ElevationLevel::None;
    let (flat_heights, flat_stack) = measure(flat);
    let (heights, stack) = measure(tailwind);
    // The shadow paints outside the buttons; it never enters their measure.
    assert_eq!(heights, flat_heights);
    assert_eq!(stack, flat_stack);
    let spacing = window.get_stack_spacing();
    let sum: f32 = heights.iter().sum();
    assert!((stack - (sum + spacing * (heights.len() as f32 - 1.))).abs() < 0.01);
    for (label, lifted) in labels {
        let button = labelled(&window, label).unwrap();
        assert_eq!(shadow_layer(&button).is_some(), lifted, "{label}");
    }
    // The default themes are flat: no button paints a shadow layer.
    window.set_theme(window.get_light());
    slint::platform::update_timers_and_animations();
    for (label, _) in labels {
        assert!(
            shadow_layer(&labelled(&window, label).unwrap()).is_none(),
            "{label}"
        );
    }
}

fn state_fixture() -> ControlStateFixture {
    i_slint_backend_testing::init_no_event_loop();
    show(ControlStateFixture::new().unwrap())
}

#[test]
fn every_alert_severity_draws_its_own_mark() {
    let window = state_fixture();
    for severity in alert_severities() {
        let expected = match severity {
            AlertSeverity::Information => window.get_information_glyph(),
            AlertSeverity::Success => window.get_check_glyph(),
            AlertSeverity::Warning | AlertSeverity::Error => window.get_warning_glyph(),
        };
        assert_eq!(window.invoke_alert_mark(severity), expected, "{severity:?}");
    }
    assert_ne!(window.get_information_glyph(), window.get_check_glyph());
}

#[test]
fn a_checking_field_shows_a_silent_spinner_and_its_description() {
    let window = state_fixture();
    let spinners = || {
        ElementQuery::from_root(&window)
            .match_inherits("KitSpinner")
            .find_all()
    };
    assert_eq!(spinners().len(), 1);
    // The text description announces the check; the mark itself is silent.
    assert!(labelled(&window, "Checking availability").is_some());
    assert_eq!(
        spinners()[0].accessible_role(),
        Some(i_slint_backend_testing::AccessibleRole::None)
    );
    window.set_checking_validation(ValidationPresentation::Pristine);
    slint::platform::update_timers_and_animations();
    assert!(spinners().is_empty());
    assert!(labelled(&window, "Checking availability").is_none());
}

#[test]
fn failed_host_requests_without_an_error_show_the_failed_label() {
    let window = state_fixture();
    for label in ["Files could not be read", "Folder could not be read"] {
        assert!(labelled(&window, label).is_some(), "{label}");
    }
    window.set_file_state(HostRequestPresentation::Idle);
    slint::platform::update_timers_and_animations();
    for label in ["Files could not be read", "Folder could not be read"] {
        assert!(labelled(&window, label).is_none(), "{label}");
    }
}

#[test]
fn the_palette_announces_the_draft_swatch_as_selected() {
    let window = state_fixture();
    window.invoke_open_palette();
    slint::platform::update_timers_and_animations();
    let chosen = labelled(&window, "Choose RGB 79, 70, 229").expect("indigo swatch");
    assert_eq!(chosen.accessible_item_selectable(), Some(true));
    assert_eq!(chosen.accessible_item_selected(), Some(true));
    let other = labelled(&window, "Choose RGB 239, 68, 68").expect("red swatch");
    assert_eq!(other.accessible_item_selected(), Some(false));
}

#[test]
fn the_active_text_line_paints_the_inset_ring_while_the_view_has_focus() {
    let window = state_fixture();
    let line = labelled(&window, "2: second line").expect("active line");
    let ring_on_line = || {
        let (origin, size) = (line.absolute_position(), line.size());
        ElementQuery::from_root(&window)
            .match_inherits("FocusOutline")
            .match_predicate(move |e| e.absolute_position() == origin && e.size() == size)
            .find_first()
    };
    assert!(ring_on_line().is_none());
    window.invoke_focus_lines();
    slint::platform::update_timers_and_animations();
    // Inside the line's own edge, so neighbouring lines never cover it.
    assert!(ring_on_line().is_some());
}

#[test]
fn an_open_select_dismisses_programmatically_when_its_access_changes() {
    let window = state_fixture();
    assert_eq!(window.get_select_dismissals().row_count(), 0);
    for access in [FieldAccess::ReadOnly, FieldAccess::Disabled] {
        window.set_select_dismissals(slint::ModelRc::new(slint::VecModel::default()));
        window.set_select_access(access);
        slint::platform::update_timers_and_animations();
        let dismissals: Vec<CloseReason> = window.get_select_dismissals().iter().collect();
        assert_eq!(dismissals, [CloseReason::Programmatic], "{access:?}");
    }
}

/// An sRGB pixel of a snapshot, compared with a theme color.
fn assert_pixel(
    snapshot: &slint::SharedPixelBuffer<slint::Rgba8Pixel>,
    x: f32,
    y: f32,
    expected: slint::Color,
    what: &str,
) {
    let (column, row) = (x.floor() as usize, y.floor() as usize);
    let pixel = snapshot.as_slice()[row * snapshot.width() as usize + column];
    let near = |a: u8, b: u8| a.abs_diff(b) <= 3;
    assert!(
        near(pixel.r, expected.red())
            && near(pixel.g, expected.green())
            && near(pixel.b, expected.blue()),
        "{what}: {pixel:?} at ({column}, {row}), expected {expected:?}"
    );
}
/// The middle of an element's top edge, one half pixel inside it.
fn top_edge(element: &ElementHandle) -> (f32, f32) {
    let origin = element.absolute_position();
    (origin.x + element.size().width / 2., origin.y + 0.5)
}

#[test]
fn invalid_choice_groups_and_composites_draw_a_danger_edge() {
    // This test thread rasterizes headlessly with the software renderer.
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = ControlStateRenderFixture::new().unwrap();
    kit_slint::init_form_validation!(ui, window);
    let window = show(window);
    let theme = window.get_theme();
    let find = |kind: &str| {
        ElementQuery::from_root(&window)
            .match_inherits(kind)
            .find_first()
            .unwrap_or_else(|| panic!("missing {kind}"))
    };
    let trigger = find("KitMultiSelect")
        .query_descendants()
        .match_inherits("KitSelectTrigger")
        .find_first()
        .expect("multi-select trigger");
    let segmented = find("KitSegmentedControl");
    let toggle = find("KitToggleGroup")
        .query_descendants()
        .match_inherits("KitPressable")
        .find_first()
        .expect("toggle option");
    let zone = find("KitFileDropZone");

    let snapshot = window.window().take_snapshot().unwrap();
    for (element, what) in [
        (&trigger, "trigger"),
        (&segmented, "segmented"),
        (&toggle, "toggle"),
    ] {
        let (x, y) = top_edge(element);
        assert_pixel(
            &snapshot,
            x,
            y,
            theme.border_subtle,
            &format!("pristine {what}"),
        );
    }
    // A disabled drop zone fills with the disabled tone, inside its padding.
    let origin = zone.absolute_position();
    assert_pixel(
        &snapshot,
        origin.x + zone.size().width / 2.,
        origin.y + theme.spacing.unit,
        theme.disabled_fill,
        "disabled drop zone",
    );

    window.invoke_report();
    slint::platform::update_timers_and_animations();
    let snapshot = window.window().take_snapshot().unwrap();
    // The multi-select owns the constraint and forwards it to its trigger.
    for (element, what) in [
        (&trigger, "trigger"),
        (&segmented, "segmented"),
        (&toggle, "toggle"),
    ] {
        let (x, y) = top_edge(element);
        assert_pixel(&snapshot, x, y, theme.danger, &format!("invalid {what}"));
    }
}
