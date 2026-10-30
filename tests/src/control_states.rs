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
    for base in [window.get_theme()] {
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
    let tailwind = window.get_light();
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
    // The shipped default follows the same filled/flat intent policy.
    window.set_theme(window.get_light());
    slint::platform::update_timers_and_animations();
    for (label, lifted) in labels {
        assert_eq!(
            shadow_layer(&labelled(&window, label).unwrap()).is_some(),
            lifted,
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

fn software_rendering() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
}

#[test]
fn received_chat_bubbles_keep_a_surface_and_outline_on_canvas_or_panels() {
    software_rendering();
    let window = show(ChatBubbleRenderFixture::new().unwrap());
    for mut theme in [window.get_theme(), window.get_dark()] {
        for scale in [1., 2.] {
            theme.text_scale = scale;
            window.set_theme(theme.clone());
            for on_surface in [false, true] {
                window.set_surface_background(on_surface);
                let snapshot = window.window().take_snapshot().unwrap();
                for (id, fill, edge) in [
                    ("incoming", theme.surface, Some(theme.border_subtle)),
                    ("outgoing", theme.accent, None),
                ] {
                    let message = ElementQuery::from_root(&window)
                        .match_id(format!("ChatBubbleRenderFixture::{id}"))
                        .find_first()
                        .unwrap();
                    let text = message
                        .query_descendants()
                        .match_type_name("Text")
                        .find_all();
                    let body = text
                        .iter()
                        .find(|element| {
                            element.accessible_label().as_deref()
                                == Some(if id == "incoming" {
                                    "Can you review the project?"
                                } else {
                                    "I will review it today."
                                })
                        })
                        .unwrap();
                    let y = body.absolute_position().y + body.size().height / 2.;
                    let x = body.absolute_position().x - theme.spacing.unit * 1.5;
                    assert_pixel(
                        &snapshot,
                        x,
                        y,
                        fill,
                        &format!("{id} fill at {scale}, panel={on_surface}"),
                    );
                    if let Some(edge) = edge {
                        assert_ne!(
                            edge, theme.surface,
                            "the contour remains visible on a surface panel"
                        );
                        let x = body.absolute_position().x - theme.spacing.unit * 3.5;
                        assert_pixel(
                            &snapshot,
                            x,
                            y,
                            edge,
                            &format!("{id} outline at {scale}, panel={on_surface}"),
                        );
                    }
                }
            }
        }
    }
}

fn pixel_at(
    snapshot: &slint::SharedPixelBuffer<slint::Rgba8Pixel>,
    x: f32,
    y: f32,
) -> slint::Color {
    let pixel =
        snapshot.as_slice()[y.floor() as usize * snapshot.width() as usize + x.floor() as usize];
    slint::Color::from_rgb_u8(pixel.r, pixel.g, pixel.b)
}
/// Every pixel inside an element's bounds, `inset` from its edges.
fn pixels_in(
    snapshot: &slint::SharedPixelBuffer<slint::Rgba8Pixel>,
    element: &ElementHandle,
    inset: f32,
) -> Vec<slint::Color> {
    let (origin, size) = (element.absolute_position(), element.size());
    let (left, top) = (
        (origin.x + inset).ceil() as usize,
        (origin.y + inset).ceil() as usize,
    );
    let (right, bottom) = (
        (origin.x + size.width - inset).floor() as usize,
        (origin.y + size.height - inset).floor() as usize,
    );
    (top..bottom)
        .flat_map(|y| (left..right).map(move |x| (x, y)))
        .map(|(x, y)| pixel_at(snapshot, x as f32, y as f32))
        .collect()
}
fn near(a: slint::Color, b: slint::Color) -> bool {
    a.red().abs_diff(b.red()) <= 3
        && a.green().abs_diff(b.green()) <= 3
        && a.blue().abs_diff(b.blue()) <= 3
}
fn selected_choice_themes(window: &SelectedChoiceRenderFixture) -> [(&'static str, KitTheme); 2] {
    [("light", window.get_light()), ("dark", window.get_dark())]
}

#[test]
fn selected_segments_and_toggles_draw_legible_on_accent_labels_in_every_theme() {
    software_rendering();
    let window = show(SelectedChoiceRenderFixture::new().unwrap());
    for (name, theme) in selected_choice_themes(&window) {
        window.set_theme(theme.clone());
        slint::platform::update_timers_and_animations();
        let snapshot = window.window().take_snapshot().unwrap();
        for label in ["Board", "PDF"] {
            // The option is the largest element carrying the label; its text
            // carries it too.
            let option = ElementHandle::find_by_accessible_label(&window, label)
                .max_by(|a, b| a.size().width.total_cmp(&b.size().width))
                .unwrap_or_else(|| panic!("missing {label}"));
            // Inside its rounded corners, so the surrounding canvas is excluded.
            let pixels = pixels_in(&snapshot, &option, theme.shape.radius_control);
            let fill = pixel_at(
                &snapshot,
                option.absolute_position().x + option.size().width / 2.,
                option.absolute_position().y + theme.spacing.unit,
            );
            assert!(near(fill, theme.accent), "{name} {label}: fill {fill:?}");
            // The label's ink is the pixel that differs most from the fill.
            let ink = pixels
                .iter()
                .copied()
                .max_by(|a, b| {
                    crate::theme_tokens::contrast(*a, fill)
                        .total_cmp(&crate::theme_tokens::contrast(*b, fill))
                })
                .expect("option pixels");
            assert!(
                near(ink, theme.on_accent),
                "{name} {label}: label ink {ink:?}, expected on-accent {:?}",
                theme.on_accent
            );
            let ratio = crate::theme_tokens::contrast(ink, fill);
            assert!(ratio >= 4.5, "{name} {label}: label contrast {ratio:.2}");
        }
    }
}

#[test]
fn disabled_choice_marks_stay_visible_on_their_disabled_rows_in_every_theme() {
    software_rendering();
    let window = show(SelectedChoiceRenderFixture::new().unwrap());
    for (name, theme) in selected_choice_themes(&window) {
        window.set_theme(theme.clone());
        slint::platform::update_timers_and_animations();
        let snapshot = window.window().take_snapshot().unwrap();
        // The disabled radio: its ring's top edge, on the disabled row fill.
        let marks: Vec<ElementHandle> = ElementQuery::from_root(&window)
            .match_inherits("RadioMark")
            .find_all();
        let row = ElementHandle::find_by_accessible_label(&window, "Enterprise")
            .max_by(|a, b| a.size().width.total_cmp(&b.size().width))
            .expect("disabled row");
        let disabled = marks
            .iter()
            .find(|mark| contains(&row, mark))
            .expect("the disabled row draws a radio mark");
        assert!(
            pixels_in(&snapshot, disabled, 0.)
                .iter()
                .any(|pixel| near(*pixel, theme.disabled_text)),
            "{name}: the disabled radio ring takes the disabled ink"
        );
        let y = disabled.absolute_position().y + disabled.size().height / 2.;
        let row = pixel_at(&snapshot, disabled.absolute_position().x - 2., y);
        assert!(
            near(row, theme.disabled_fill),
            "{name}: disabled row fill {row:?}"
        );
        assert!(
            crate::theme_tokens::contrast(theme.disabled_text, theme.disabled_fill) >= 2.,
            "{name}: disabled mark against its row"
        );
        // Unlabelled disabled checkbox and switch: only their marks ink.
        for kind in ["KitCheckbox", "KitSwitch"] {
            let control = ElementQuery::from_root(&window)
                .match_inherits(kind)
                .find_first()
                .unwrap_or_else(|| panic!("missing {kind}"));
            assert!(
                pixels_in(&snapshot, &control, 0.)
                    .iter()
                    .any(|pixel| near(*pixel, theme.disabled_text)),
                "{name}: disabled {kind} draws no visible mark"
            );
        }
    }
}

#[test]
fn a_header_that_cannot_sort_keeps_the_header_tone_and_square_cells() {
    software_rendering();
    let window = show(DataTableHeaderRenderFixture::new().unwrap());
    for (name, theme) in [("light", window.get_light()), ("dark", window.get_dark())] {
        window.set_theme(theme.clone());
        slint::platform::update_timers_and_animations();
        let snapshot = window.window().take_snapshot().unwrap();
        for label in ["Project, ascending", "Owner"] {
            let cell = labelled(&window, label).unwrap_or_else(|| panic!("missing {label}"));
            let (origin, size) = (cell.absolute_position(), cell.size());
            // The cell's corners and middle edge all show the header tone:
            // no disabled fill, and no rounded fill inside the table's clip.
            for (x, y) in [
                (origin.x + 2., origin.y + size.height - 2.),
                (origin.x + size.width / 2., origin.y + size.height - 2.),
                (origin.x + size.width - 2., origin.y + size.height - 2.),
                (origin.x + 2., origin.y + size.height / 2.),
            ] {
                assert_pixel(
                    &snapshot,
                    x,
                    y,
                    theme.canvas,
                    &format!("{name} {label} header"),
                );
            }
        }
        // Header labels start at the cell inset, aligned with the row text.
        let header = labelled(&window, "Owner").unwrap();
        let label = header
            .query_descendants()
            .match_predicate(|e| e.accessible_label().as_deref() == Some("Owner"))
            .find_first()
            .expect("header label");
        let inset = label.absolute_position().x - header.absolute_position().x;
        assert!(
            (inset - theme.spacing.unit * 4.).abs() < 0.5,
            "{name}: header label inset {inset}"
        );
    }
}
