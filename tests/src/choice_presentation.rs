// SPDX-License-Identifier: GPL-3.0-or-later
//! Vertical and filled sliders, vertical segmented controls, chip buttons and
//! the information tone.
use super::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{Brush, Color, ComponentHandle, LogicalPosition};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

fn fixture() -> ChoicePresentationFixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = ChoicePresentationFixture::new().unwrap();
    kit_slint::init_form_validation!(ui, window);
    window.show().unwrap();
    window
}
fn with_text_scale(window: &ChoicePresentationFixture, scale: f32) {
    let mut theme = window.get_light();
    theme.text_scale = scale;
    window.set_theme(theme);
}
fn element(window: &ChoicePresentationFixture, label: &str) -> ElementHandle {
    let requested = label.to_owned();
    ElementQuery::from_root(window)
        .match_predicate(move |element| {
            element.accessible_label().as_deref() == Some(requested.as_str())
        })
        .find_first()
        .unwrap_or_else(|| panic!("missing element {label}"))
}
fn key(window: &ChoicePresentationFixture, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}
fn click(window: &ChoicePresentationFixture, x: f32, y: f32) {
    let position = LogicalPosition::new(x, y);
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
}
type Requests = Rc<RefCell<Vec<(String, f32)>>>;
fn record_positions(window: &ChoicePresentationFixture) -> Requests {
    let requests: Requests = Rc::default();
    let observed = requests.clone();
    window.on_position_requested(move |slider, value| {
        observed.borrow_mut().push((slider.to_string(), value));
    });
    requests
}
fn last(requests: &Requests) -> (String, f32) {
    requests
        .borrow()
        .last()
        .cloned()
        .expect("a position request")
}
fn solid(color: Color) -> Brush {
    Brush::SolidColor(color)
}

#[test]
fn vertical_sliders_map_the_bottom_to_zero_at_normal_and_doubled_text() {
    let window = fixture();
    let requests = record_positions(&window);
    for scale in [1., 2.] {
        with_text_scale(&window, scale);
        for (name, id) in [
            ("Vertical level", "line-vertical"),
            ("Filled level", "filled-vertical"),
        ] {
            let slider = element(&window, name);
            let origin = slider.absolute_position();
            let size = slider.size();
            let middle_x = origin.x + size.width / 2.;
            click(&window, middle_x, origin.y + size.height - 1.);
            assert_eq!(last(&requests).0, id, "{name} at {scale}");
            assert!(last(&requests).1 < 0.01, "{name} at {scale}: bottom");
            click(&window, middle_x, origin.y + 1.);
            assert!(last(&requests).1 > 0.99, "{name} at {scale}: top");
            click(&window, middle_x, origin.y + size.height / 2.);
            assert!(
                (last(&requests).1 - 0.5).abs() < 0.001,
                "{name} at {scale}: middle"
            );
        }
        // A line slider travels between its thumb centre's limits: the band
        // above the top limit is the maximum, and a quarter of the travel
        // below that limit is three quarters of the value.
        let edge = window.get_line_edge();
        let line = element(&window, "Vertical level");
        let origin = line.absolute_position();
        let size = line.size();
        let travel = size.height - 2. * edge;
        click(&window, origin.x + size.width / 2., origin.y + edge / 2.);
        assert_eq!(last(&requests).0, "line-vertical");
        assert!(last(&requests).1 > 0.999, "line edge band at {scale}");
        click(
            &window,
            origin.x + size.width / 2.,
            origin.y + edge + travel / 4.,
        );
        assert!(
            (last(&requests).1 - 0.75).abs() < 0.001,
            "line quarter at {scale}"
        );
        // A filled slider has no edge band: the same offset is below the top.
        let filled = element(&window, "Filled level");
        let origin = filled.absolute_position();
        let size = filled.size();
        click(&window, origin.x + size.width / 2., origin.y + edge / 2.);
        assert_eq!(last(&requests).0, "filled-vertical");
        assert!(
            (last(&requests).1 - (1. - edge / 2. / size.height)).abs() < 0.001,
            "filled edge band at {scale}"
        );
        // A filled slider travels its whole length: a quarter from the top
        // is three quarters of the value.
        let filled = element(&window, "Filled level");
        let origin = filled.absolute_position();
        let size = filled.size();
        click(
            &window,
            origin.x + size.width / 2.,
            origin.y + size.height / 4.,
        );
        assert_eq!(last(&requests).0, "filled-vertical");
        assert!((last(&requests).1 - 0.75).abs() < 0.001);
        let horizontal = element(&window, "Filled width");
        let origin = horizontal.absolute_position();
        let size = horizontal.size();
        click(
            &window,
            origin.x + size.width / 4.,
            origin.y + size.height / 2.,
        );
        assert_eq!(last(&requests).0, "filled-horizontal");
        assert!((last(&requests).1 - 0.25).abs() < 0.001);
    }
}

#[test]
fn vertical_handles_sit_at_the_value_measured_from_the_bottom() {
    let window = fixture();
    let unit = window.get_theme().spacing.unit;
    let edge = window.get_line_edge();
    for level in [0.25, 0.8] {
        window.set_level(level);
        // The round thumb's centre sits at the value along the travel.
        let line = element(&window, "Vertical level");
        let origin = line.absolute_position();
        let size = line.size();
        let thumb = line
            .query_descendants()
            .match_type_name("Rectangle")
            .match_predicate(|rectangle| {
                rectangle.size().width > 0. && rectangle.size().width == rectangle.size().height
            })
            .find_all()
            .into_iter()
            .max_by(|left, right| left.size().width.total_cmp(&right.size().width))
            .expect("a round thumb");
        let centre = thumb.absolute_position().y + thumb.size().height / 2.;
        let expected = origin.y + edge + (1. - level) * (size.height - 2. * edge);
        assert!(
            (centre - expected).abs() < 0.01,
            "line thumb at {level}: {centre} instead of {expected}"
        );
        // The filled handle bar sits inside the fill, below its top edge.
        let filled = element(&window, "Filled level");
        let origin = filled.absolute_position();
        let size = filled.size();
        let bar = filled
            .query_descendants()
            .match_type_name("Rectangle")
            .match_predicate(move |rectangle| {
                (rectangle.size().height - 1.5 * unit).abs() < 0.01
                    && rectangle.size().width < size.width
            })
            .find_first()
            .expect("a handle bar");
        let fill_top = origin.y + (1. - level) * size.height;
        let bar_top = bar.absolute_position().y;
        assert!(
            bar_top > fill_top,
            "filled handle at {level}: {bar_top} above the fill top {fill_top}"
        );
        assert!(bar_top + bar.size().height < origin.y + size.height);
    }
}

#[test]
fn vertical_slider_keys_step_up_and_right_forward() {
    let window = fixture();
    let steps = Rc::new(RefCell::new(Vec::new()));
    let observed = steps.clone();
    window.on_navigated(move |slider, step| {
        assert_eq!(slider.as_str(), "line-vertical");
        observed.borrow_mut().push(step);
    });
    window.invoke_focus_line_vertical();
    for pressed in [
        Key::UpArrow,
        Key::DownArrow,
        Key::RightArrow,
        Key::LeftArrow,
        Key::Home,
        Key::End,
    ] {
        key(&window, pressed);
    }
    assert_eq!(
        *steps.borrow(),
        [
            NavigationStep::Next,
            NavigationStep::Previous,
            NavigationStep::Next,
            NavigationStep::Previous,
            NavigationStep::First,
            NavigationStep::Last,
        ]
    );
    let slider = element(&window, "Vertical level");
    assert_eq!(
        slider.accessible_orientation(),
        Some(i_slint_backend_testing::Orientation::Vertical)
    );
    assert_eq!(slider.accessible_value().as_deref(), Some("70 percent"));
    // Read-only and disabled sliders ignore keys and pointer gestures.
    let requests = record_positions(&window);
    for access in [FieldAccess::ReadOnly, FieldAccess::Disabled] {
        window.set_access(access);
        key(&window, Key::UpArrow);
        let origin = slider.absolute_position();
        click(&window, origin.x + 4., origin.y + 4.);
    }
    assert_eq!(steps.borrow().len(), 6);
    assert!(requests.borrow().is_empty());
}

#[test]
fn sliders_keep_their_cross_extent_and_filled_sliders_are_touch_sized() {
    let window = fixture();
    for scale in [1., 2.] {
        with_text_scale(&window, scale);
        let theme = window.get_theme();
        let unit = theme.spacing.unit;
        assert!(window.get_line_cross() >= 10. * unit, "line at {scale}");
        assert_eq!(window.get_line_travel_minimum(), 120.);
        let filled = window.get_filled_cross();
        assert!(filled >= 44., "filled slider {filled} at {scale}");
        assert!(filled >= 12. * unit);
        assert!(element(&window, "Filled width").size().height >= 44.);
        assert!(element(&window, "Filled level").size().width >= 44.);
        // The thickness is a minimum, so a short layout cannot squeeze a
        // horizontal filled slider below it.
        let minimum = window.get_filled_min_cross();
        assert!(
            minimum >= 12. * unit && minimum >= 44.,
            "minimum at {scale}"
        );
        let squeezed = window.get_squeezed_height();
        assert!(squeezed >= 12. * unit, "squeezed {squeezed} at {scale}");
    }
    // The fill of a vertical filled slider grows from the bottom edge.
    with_text_scale(&window, 1.);
    window.set_level(0.25);
    let slider = element(&window, "Filled level");
    let origin = slider.absolute_position();
    let size = slider.size();
    let fill = slider
        .query_descendants()
        .match_type_name("Rectangle")
        .match_predicate(move |rectangle| {
            (rectangle.size().width - size.width).abs() < 0.01
                && (rectangle.size().height - size.height / 4.).abs() < 0.01
        })
        .find_first()
        .expect("a fill a quarter of the track high");
    assert!(
        (fill.absolute_position().y - (origin.y + size.height * 0.75)).abs() < 0.01,
        "the fill ends at the bottom edge"
    );
}

#[test]
fn vertical_segmented_control_stacks_segments_and_navigates_with_up_and_down() {
    let window = fixture();
    let low = element(&window, "Low");
    let medium = element(&window, "Medium");
    let high = element(&window, "High");
    assert_eq!(low.absolute_position().x, medium.absolute_position().x);
    assert_eq!(medium.absolute_position().x, high.absolute_position().x);
    assert!(low.absolute_position().y + low.size().height <= medium.absolute_position().y);
    assert!(medium.absolute_position().y + medium.size().height <= high.absolute_position().y);
    assert_eq!(
        element(&window, "Speed").accessible_orientation(),
        Some(i_slint_backend_testing::Orientation::Vertical)
    );
    assert_eq!(low.accessible_checked(), Some(true));
    // Native screen readers read the state only from checkable elements.
    for segment in [&low, &medium, &high] {
        assert_eq!(segment.accessible_checkable(), Some(true));
    }
    let steps = Rc::new(RefCell::new(Vec::new()));
    let observed = steps.clone();
    window.on_speed_navigate(move |step| observed.borrow_mut().push(step));
    window.invoke_focus_speeds();
    key(&window, Key::DownArrow);
    key(&window, Key::UpArrow);
    // Horizontal arrows do not move a vertical group.
    key(&window, Key::RightArrow);
    key(&window, Key::LeftArrow);
    assert_eq!(
        *steps.borrow(),
        [NavigationStep::Next, NavigationStep::Previous]
    );
    let activated = Rc::new(Cell::new((0, 0)));
    let observed = activated.clone();
    window.on_speed_activate(move |token, generation| observed.set((token, generation)));
    let origin = high.absolute_position();
    click(
        &window,
        origin.x + high.size().width / 2.,
        origin.y + high.size().height / 2.,
    );
    assert_eq!(activated.get(), (33, 3));
}

#[test]
fn chip_button_activates_by_pointer_and_keyboard_unless_disabled() {
    let window = fixture();
    let activations = Rc::new(Cell::new(0));
    let observed = activations.clone();
    window.on_chip_activated(move || observed.set(observed.get() + 1));
    let chip = element(&window, "Open orders");
    assert_eq!(chip.accessible_checkable(), Some(true));
    assert_eq!(chip.accessible_checked(), Some(false));
    let origin = chip.absolute_position();
    let center = (
        origin.x + chip.size().width / 2.,
        origin.y + chip.size().height / 2.,
    );
    click(&window, center.0, center.1);
    assert_eq!(activations.get(), 1);
    window.invoke_focus_chip();
    key(&window, Key::Space);
    key(&window, Key::Return);
    assert_eq!(activations.get(), 3);
    window.set_chip_selected(true);
    assert_eq!(chip.accessible_checked(), Some(true));
    // A shortcut chip is a plain button: no toggle state to announce.
    window.set_chip_checkable(false);
    assert_eq!(chip.accessible_checkable(), Some(false));
    assert_eq!(chip.accessible_checked(), Some(false));
    window.set_chip_checkable(true);
    window.set_chip_disabled(true);
    click(&window, center.0, center.1);
    key(&window, Key::Space);
    chip.invoke_accessible_default_action();
    assert_eq!(activations.get(), 3);
    assert_eq!(chip.accessible_enabled(), Some(false));
}

/// The selected fill and edge of each tone; every variant is listed.
fn tone_paint(theme: &KitTheme, tone: Tone) -> (Color, Color) {
    match tone {
        Tone::Neutral => (theme.pressed, theme.text_primary),
        Tone::Accent => (theme.accent_subtle, theme.accent),
        Tone::Success => (theme.success_subtle, theme.success),
        Tone::Warning => (theme.warning_subtle, theme.warning),
        Tone::Danger => (theme.danger_subtle, theme.danger),
        Tone::Information => (theme.information_subtle, theme.information),
    }
}

#[test]
fn chip_button_takes_its_tone_when_selected_in_both_themes() {
    let window = fixture();
    for theme in [window.get_light(), window.get_dark()] {
        window.set_theme(theme.clone());
        for tone in [
            Tone::Neutral,
            Tone::Accent,
            Tone::Success,
            Tone::Warning,
            Tone::Danger,
            Tone::Information,
        ] {
            window.set_chip_tone(tone);
            window.set_chip_selected(false);
            assert_eq!(window.get_chip_background(), solid(theme.surface));
            assert_eq!(window.get_chip_border(), solid(theme.border_subtle));
            assert_eq!(window.get_chip_border_width(), theme.shape.border_width);
            window.set_chip_selected(true);
            let (fill, edge) = tone_paint(&theme, tone);
            assert_eq!(window.get_chip_background(), solid(fill), "{tone:?}");
            assert_eq!(window.get_chip_border(), solid(edge), "{tone:?}");
            // Selection is also a strong edge, never colour alone.
            assert_eq!(
                window.get_chip_border_width(),
                theme.shape.border_width_strong,
                "{tone:?}"
            );
            // An unselected chip under the pointer takes the pressed fill
            // and the border-strong edge; a selected chip's edge differs.
            assert_ne!(edge, theme.border_strong, "{tone:?}");
        }
    }
}

/// Presses the pointer at `position` without releasing it.
fn press(window: &ChoicePresentationFixture, x: f32, y: f32) {
    let position = LogicalPosition::new(x, y);
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
}
fn release(window: &ChoicePresentationFixture, x: f32, y: f32) {
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position: LogicalPosition::new(x, y),
            button: PointerEventButton::Left,
        });
}

#[test]
fn chip_button_reports_the_row_its_press_started_on() {
    let window = fixture();
    let requests = Rc::new(RefCell::new(Vec::new()));
    let observed = requests.clone();
    window
        .on_chip_activate(move |token, generation| observed.borrow_mut().push((token, generation)));
    let chip = element(&window, "Open orders");
    let origin = chip.absolute_position();
    let (x, y) = (
        origin.x + chip.size().width / 2.,
        origin.y + chip.size().height / 2.,
    );
    click(&window, x, y);
    assert_eq!(*requests.borrow(), [(7, 1)]);
    // The chip is rebound to another row during a press: nothing fires.
    press(&window, x, y);
    window.set_chip_token(8);
    release(&window, x, y);
    // A new generation of the same row during a press cancels it too.
    press(&window, x, y);
    window.set_chip_generation(2);
    release(&window, x, y);
    assert_eq!(*requests.borrow(), [(7, 1)]);
    click(&window, x, y);
    window.invoke_focus_chip();
    key(&window, Key::Return);
    assert_eq!(*requests.borrow(), [(7, 1), (8, 2), (8, 2)]);
}

#[test]
fn chip_button_grows_with_its_size_and_text() {
    let window = fixture();
    for scale in [1., 2.] {
        with_text_scale(&window, scale);
        let theme = window.get_theme();
        for (size, control) in [
            (ControlSize::Compact, theme.spacing.compact.height),
            (ControlSize::Comfortable, theme.spacing.comfortable.height),
        ] {
            window.set_chip_size(size);
            assert!(window.get_chip_height() >= control, "{size:?} at {scale}");
            assert_eq!(window.get_chip_height(), window.get_chip_min_height());
        }
    }
    // Doubled text makes the chip taller and wider at every size.
    for size in [ControlSize::Compact, ControlSize::Comfortable] {
        window.set_chip_size(size);
        with_text_scale(&window, 1.);
        let (height, width) = (window.get_chip_height(), window.get_chip_preferred_width());
        with_text_scale(&window, 2.);
        assert!(window.get_chip_height() > height, "{size:?} height");
        assert!(window.get_chip_preferred_width() > width, "{size:?} width");
    }
}

/// The WCAG relative luminance of a colour.
fn luminance(color: Color) -> f32 {
    let channel = |value: u8| {
        let value = f32::from(value) / 255.;
        if value <= 0.039_28 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.red()) + 0.7152 * channel(color.green()) + 0.0722 * channel(color.blue())
}
/// The WCAG contrast ratio of two colours.
fn contrast(first: Color, second: Color) -> f32 {
    let (first, second) = (luminance(first), luminance(second));
    (first.max(second) + 0.05) / (first.min(second) + 0.05)
}

#[test]
fn filled_slider_default_paints_contrast_in_both_themes() {
    let window = fixture();
    for theme in [window.get_light(), window.get_dark()] {
        window.set_theme(theme.clone());
        let track = window.get_filled_track().color();
        let fill = window.get_filled_fill().color();
        let handle = window.get_filled_handle().color();
        assert_ne!(track, theme.canvas);
        assert_ne!(track, fill);
        // WCAG 1.4.11: the value fill stands out from its track and from
        // the pages the slider sits on, and the handle from the fill.
        for (what, ratio) in [
            ("fill on track", contrast(fill, track)),
            ("fill on canvas", contrast(fill, theme.canvas)),
            ("fill on surface", contrast(fill, theme.surface)),
            ("handle on fill", contrast(handle, fill)),
            (
                "edge on canvas",
                contrast(theme.border_strong, theme.canvas),
            ),
            (
                "edge on surface",
                contrast(theme.border_strong, theme.surface),
            ),
        ] {
            assert!(ratio >= 3., "{what}: {ratio}");
        }
    }
}

#[test]
fn information_badge_uses_the_information_tokens() {
    let window = fixture();
    for theme in [window.get_light(), window.get_dark()] {
        window.set_theme(theme.clone());
        window.set_badge_tone(Tone::Information);
        assert_eq!(
            window.get_badge_background(),
            solid(theme.information_subtle)
        );
        assert_eq!(window.get_badge_border(), solid(theme.information));
    }
}

#[test]
fn segmented_keyboard_activation_checks_the_live_row_and_fails_closed_without_initialization() {
    i_slint_backend_testing::init_no_event_loop();
    let window = ChoicePresentationFixture::new().unwrap();
    window.show().unwrap();
    let activations = Rc::new(RefCell::new(Vec::new()));
    let observed = activations.clone();
    window.on_speed_activate(move |token, generation| {
        observed.borrow_mut().push((token, generation))
    });
    window.invoke_focus_speeds();
    key(&window, Key::Return);
    assert!(
        activations.borrow().is_empty(),
        "uninitialized bridge fails closed"
    );
    kit_slint::init_form_validation!(ui, window);
    key(&window, Key::Return);
    assert_eq!(*activations.borrow(), [(31, 3)]);
    for token in [0, 999] {
        window.set_speed_active(token);
        key(&window, Key::Return);
    }
    let rows = Rc::new(slint::VecModel::from(vec![ChoiceRow {
        token: 31,
        label: "Low".into(),
        selected: true,
        disabled: true,
    }]));
    window.set_speeds(rows.clone().into());
    window.set_speed_active(31);
    key(&window, Key::Space);
    assert_eq!(*activations.borrow(), [(31, 3)]);
    use slint::Model;
    rows.set_row_data(
        0,
        ChoiceRow {
            disabled: false,
            ..rows.row_data(0).unwrap()
        },
    );
    key(&window, Key::Return);
    assert_eq!(*activations.borrow(), [(31, 3), (31, 3)]);
    rows.push(rows.row_data(0).unwrap());
    key(&window, Key::Return);
    assert_eq!(
        activations.borrow().len(),
        2,
        "duplicate identities are rejected"
    );
}

#[test]
fn segmented_grid_keeps_two_columns_and_token_icons_after_reordering() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = ChoicePresentationFixture::new().unwrap();
    kit_slint::init_form_validation!(ui, window);
    window.show().unwrap();
    window.set_speed_width(320.);

    window.set_speed_direction(ChoiceDirection::Horizontal);
    window.set_speed_columns(2);
    let rows: Vec<_> = (1..=4)
        .map(|token| ChoiceRow {
            token,
            label: token.to_string().into(),
            selected: token == 1,
            disabled: false,
        })
        .collect();
    window.set_speeds(slint::ModelRc::new(slint::VecModel::from(rows.clone())));
    window.set_speed_icons(slint::ModelRc::new(slint::VecModel::from(vec![
        ChoiceIcon {
            token: 1,
            glyph: window.get_theme().icons.check,
        },
    ])));
    for mut theme in [window.get_light(), window.get_dark()] {
        for scale in [1., 2.] {
            theme.text_scale = scale;
            window.set_theme(theme.clone());
            let options: Vec<_> = (1..=4)
                .map(|value| element(&window, &value.to_string()))
                .collect();
            assert_eq!(
                options[0].absolute_position().y,
                options[1].absolute_position().y
            );
            assert_eq!(
                options[2].absolute_position().y,
                options[3].absolute_position().y
            );
            assert!(
                options[0].absolute_position().y + options[0].size().height
                    <= options[2].absolute_position().y
            );
            assert_eq!(
                options[0].absolute_position().x,
                options[2].absolute_position().x
            );
            assert_eq!(
                options[0]
                    .query_descendants()
                    .match_inherits("KitIcon")
                    .find_all()
                    .len(),
                1
            );
            assert!(
                options[1]
                    .query_descendants()
                    .match_inherits("KitIcon")
                    .find_all()
                    .is_empty()
            );

            let snapshot = window.window().take_snapshot().unwrap();
            save_capture(
                &snapshot,
                &format!(
                    "segments-grid-{}-{scale}",
                    if theme.canvas.red() > 100 {
                        "light"
                    } else {
                        "dark"
                    }
                ),
            );
        }
    }
    window.set_speeds(slint::ModelRc::new(slint::VecModel::from(
        rows.into_iter().rev().collect::<Vec<_>>(),
    )));
    assert_eq!(
        element(&window, "1")
            .query_descendants()
            .match_inherits("KitIcon")
            .find_all()
            .len(),
        1
    );
    assert!(
        element(&window, "1").absolute_position().x > element(&window, "4").absolute_position().x
    );
    let uneven: Vec<_> = (1..=5)
        .map(|token| ChoiceRow {
            token,
            label: token.to_string().into(),
            selected: false,
            disabled: false,
        })
        .collect();
    window.set_speeds(slint::ModelRc::new(slint::VecModel::from(uneven)));
    assert!(
        element(&window, "5").absolute_position().y > element(&window, "3").absolute_position().y
    );
    let activated = Rc::new(RefCell::new(Vec::new()));
    let observed = activated.clone();
    window.on_speed_activate(move |token, generation| {
        observed.borrow_mut().push((token, generation))
    });
    for flow in [ChoiceFlow::LeftToRight, ChoiceFlow::RightToLeft] {
        window.set_speed_flow(flow);
        // Keep a handle while its real row becomes the final empty cell.
        // Invisible grid cells are excluded from fresh accessibility queries.
        let rows = Rc::new(slint::VecModel::from(
            (1..=6)
                .map(|token| ChoiceRow {
                    token,
                    label: token.to_string().into(),
                    disabled: false,
                    selected: false,
                })
                .collect::<Vec<_>>(),
        ));
        window.set_speeds(rows.clone().into());
        let placeholder = element(&window, "6");
        rows.remove(5);
        assert_eq!(placeholder.accessible_enabled(), Some(false));
        placeholder.invoke_accessible_default_action();
        window.set_speed_active(0);
        window.invoke_focus_speeds();
        key(&window, Key::Return);
        assert!(
            activated.borrow().is_empty(),
            "no placeholder token in {flow:?}"
        );
        element(&window, "5").invoke_accessible_default_action();
        assert_eq!(
            *activated.borrow(),
            [(5, 3)],
            "the real cell remains actionable"
        );
        activated.borrow_mut().clear();
    }
}

#[test]
fn selected_segment_focus_readonly_choices_and_custom_disabled_slider_paint_render() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = ChoicePresentationFixture::new().unwrap();
    kit_slint::init_form_validation!(ui, window);
    window.show().unwrap();
    let switches = Rc::new(Cell::new(0));
    let observed = switches.clone();
    window.on_switch_toggled(move |_| observed.set(observed.get() + 1));
    for mut theme in [window.get_light(), window.get_dark()] {
        for scale in [1., 2.] {
            theme.text_scale = scale;
            theme.reduced_motion = true;
            window.set_theme(theme.clone());
            window.set_access(FieldAccess::Editable);
            window.invoke_focus_speeds();
            let selected = element(&window, "Low");
            let outline = selected
                .query_descendants()
                .match_inherits("FocusOutline")
                .find_first()
                .expect("selected row focus outline");
            let snapshot = window.window().take_snapshot().unwrap();
            let origin = outline.absolute_position();
            let pixel = snapshot.as_slice()[((origin.y + 1.).floor() * snapshot.width() as f32
                + (origin.x + outline.size().width / 2.).floor())
                as usize];
            assert_eq!(
                [pixel.r, pixel.g, pixel.b],
                [
                    theme.on_accent.red(),
                    theme.on_accent.green(),
                    theme.on_accent.blue()
                ]
            );
            assert!(contrast(theme.on_accent, theme.accent) >= 3.);
            save_capture(
                &snapshot,
                &format!(
                    "choices-focused-{}-{scale}",
                    if theme.canvas.red() > 100 {
                        "light"
                    } else {
                        "dark"
                    }
                ),
            );

            // Focus can remain on the group's active token while its row is
            // disabled by a host update. The neutral fill needs the normal ring.
            use slint::Model;
            let previous_rows = window.get_speeds();
            let disabled_rows: Vec<_> = previous_rows
                .iter()
                .map(|row| ChoiceRow {
                    disabled: row.token == 31,
                    ..row
                })
                .collect();
            window.set_speeds(slint::ModelRc::new(slint::VecModel::from(disabled_rows)));
            window.invoke_focus_speeds();
            let disabled_outline = element(&window, "Low")
                .query_descendants()
                .match_inherits("FocusOutline")
                .find_first()
                .unwrap();
            let disabled_snapshot = window.window().take_snapshot().unwrap();
            let origin = disabled_outline.absolute_position();
            let pixel = disabled_snapshot.as_slice()[((origin.y + 1.).floor()
                * disabled_snapshot.width() as f32
                + (origin.x + disabled_outline.size().width / 2.).floor())
                as usize];
            assert_eq!(
                [pixel.r, pixel.g, pixel.b],
                [
                    theme.focus.color.red(),
                    theme.focus.color.green(),
                    theme.focus.color.blue()
                ]
            );
            window.set_speeds(previous_rows);
            let selected = element(&window, "Low");
            window.set_access(FieldAccess::ReadOnly);
            let snapshot = window.window().take_snapshot().unwrap();
            let origin = selected.absolute_position();
            let pixel = snapshot.as_slice()[((origin.y + selected.size().height / 2.).floor()
                * snapshot.width() as f32
                + (origin.x + theme.spacing.unit * 2.).floor())
                as usize];
            assert_eq!(
                [pixel.r, pixel.g, pixel.b],
                [
                    theme.pressed.red(),
                    theme.pressed.green(),
                    theme.pressed.blue()
                ]
            );
            let switch = element(&window, "Power");
            switch.invoke_accessible_default_action();
            assert_eq!(
                switch.accessible_description().as_deref(),
                Some("Read only")
            );
            let track = switch
                .query_descendants()
                .match_type_name("Rectangle")
                .match_predicate(|rectangle| {
                    rectangle.size().width > rectangle.size().height * 1.5
                        && rectangle.size().height > 10.
                        && rectangle.size().width < 100.
                })
                .find_all()
                .into_iter()
                .min_by(|a, b| a.size().height.total_cmp(&b.size().height))
                .unwrap();
            let origin = track.absolute_position();
            save_capture(
                &snapshot,
                &format!(
                    "choices-readonly-{}-{scale}",
                    if theme.canvas.red() > 100 {
                        "light"
                    } else {
                        "dark"
                    }
                ),
            );
            let pixel = snapshot.as_slice()[((origin.y + track.size().height / 2.).floor()
                * snapshot.width() as f32
                + (origin.x + 4.).floor()) as usize];
            assert_eq!(
                [pixel.r, pixel.g, pixel.b],
                [
                    theme.text_secondary.red(),
                    theme.text_secondary.green(),
                    theme.text_secondary.blue()
                ]
            );
            window.set_access(FieldAccess::Disabled);
            window.set_custom_slider_paints(true);
            let snapshot = window.window().take_snapshot().unwrap();
            for (x, expected) in [(30usize, [0x24, 0x68, 0x13]), (330, [0x13, 0x57, 0x24])] {
                let pixel =
                    snapshot.as_slice()[320 * usize::try_from(snapshot.width()).unwrap() + x];
                assert_eq!([pixel.r, pixel.g, pixel.b], expected);
            }

            assert!(
                snapshot
                    .as_slice()
                    .iter()
                    .any(|pixel| [pixel.r, pixel.g, pixel.b] == [0xf5, 0xa9, 0xc8]),
                "custom disabled handle paint"
            );
            save_capture(
                &snapshot,
                &format!(
                    "choices-disabled-{}-{scale}",
                    if theme.canvas.red() > 100 {
                        "light"
                    } else {
                        "dark"
                    }
                ),
            );
            window.set_custom_slider_paints(false);
        }
    }
    assert_eq!(switches.get(), 0);
}

pub(super) fn save_capture(snapshot: &slint::SharedPixelBuffer<slint::Rgba8Pixel>, name: &str) {
    let Ok(directory) = std::env::var("KIT_CONTROL_CAPTURE_DIR") else {
        return;
    };
    std::fs::create_dir_all(&directory).unwrap();
    let mut bytes = format!("P6\n{} {}\n255\n", snapshot.width(), snapshot.height()).into_bytes();
    for pixel in snapshot.as_slice() {
        bytes.extend([pixel.r, pixel.g, pixel.b]);
    }
    std::fs::write(
        std::path::Path::new(&directory).join(format!("{name}.ppm")),
        bytes,
    )
    .unwrap();
}
