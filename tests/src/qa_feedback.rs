// SPDX-License-Identifier: GPL-3.0-or-later
use crate::QaRatingFixture;
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle, LogicalPosition, Model,
    platform::{Key, PointerEventButton, WindowEvent},
};
fn fixture() -> QaRatingFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = QaRatingFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    settle();
    app
}
fn settle() {
    slint::platform::update_timers_and_animations();
}
fn score(app: &QaRatingFixture, label: &str) -> ElementHandle {
    let label = label.to_owned();
    ElementQuery::from_root(app)
        .match_inherits("SegmentedOption")
        .match_predicate(move |e| e.accessible_label().as_deref() == Some(label.as_str()))
        .find_first()
        .unwrap()
}
fn click(app: &impl ComponentHandle, element: &ElementHandle) {
    let p = element.absolute_position();
    let s = element.size();
    let position = LogicalPosition::new(p.x + s.width / 2., p.y + s.height / 2.);
    app.window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    app.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    app.window().dispatch_event(WindowEvent::PointerReleased {
        position,
        button: PointerEventButton::Left,
    });
    settle();
}
fn key(app: &impl ComponentHandle, value: Key) {
    let text: slint::SharedString = value.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    settle();
}
#[test]
fn local_rating_retains_the_accepted_pointer_value() {
    let app = fixture();
    click(&app, &score(&app, "3"));
    assert!(
        app.get_rating_value().present,
        "ready={} generation={} activations={} proposals={} rows={:?}",
        app.get_service_ready(),
        app.get_generation(),
        app.get_activations(),
        app.get_proposals(),
        app.get_scores().iter().collect::<Vec<_>>()
    );
    assert_eq!(app.get_rating_value().step, 3);
    assert_eq!(app.get_proposals(), 1);
    assert_eq!(score(&app, "3").accessible_checked(), Some(true));
    let outside = ElementHandle::find_by_accessible_label(&app, "Outside")
        .next()
        .unwrap();
    click(&app, &outside);
    assert_eq!(app.get_rating_value().step, 3);
    click(&app, &score(&app, "5"));
    assert_eq!(app.get_rating_value().step, 5);
    key(&app, Key::LeftArrow);
    assert_eq!(app.get_rating_value().step, 4);
}

#[test]
fn controlled_rating_retains_the_host_binding_across_proposals_and_acknowledgments() {
    use slint_kit::ui::{FieldAccess, Ownership, RatingView};
    let app = fixture();
    app.set_ownership(Ownership::Controlled);
    settle();
    click(&app, &score(&app, "3"));
    assert!(!app.get_rating_value().present);
    assert_eq!(app.get_last_proposal().step, 3);
    assert_eq!(score(&app, "3").accessible_checked(), Some(false));
    app.set_rating_value(app.get_last_proposal());
    settle();
    assert_eq!(score(&app, "3").accessible_checked(), Some(true));
    click(&app, &score(&app, "5"));
    assert_eq!(app.get_rating_value().step, 3);
    assert_eq!(score(&app, "3").accessible_checked(), Some(true));
    assert_eq!(score(&app, "5").accessible_checked(), Some(false));
    app.set_rating_value(app.get_last_proposal());
    settle();
    assert_eq!(score(&app, "5").accessible_checked(), Some(true));
    app.set_access(FieldAccess::ReadOnly);
    settle();
    let proposals = app.get_proposals();
    click(&app, &score(&app, "2"));
    assert_eq!(app.get_proposals(), proposals);
    app.set_rating_value(RatingView {
        present: true,
        step: 2,
        scale: 5,
    });
    settle();
    assert_eq!(score(&app, "2").accessible_checked(), Some(true));
}
fn platform() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
}
#[test]
fn actual_slider_starter_keeps_visible_and_accessible_exact_values_in_sync() {
    platform();
    let app = crate::QaSliderDemo::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    settle();
    let slider = ElementQuery::from_root(&app)
        .match_inherits("KitSlider")
        .find_first()
        .unwrap();
    click(&app, &slider);
    assert_eq!(app.get_test_volume(), "50");
    key(&app, Key::RightArrow);
    assert_eq!(app.get_test_volume(), "55");
    assert_eq!(slider.accessible_value().as_deref(), Some("55"));
    assert_eq!(
        slider.accessible_description().as_deref(),
        Some("55 percent")
    );
    assert!(
        ElementHandle::find_by_accessible_label(&app, "Value: 55")
            .next()
            .is_some()
    );
    let value = app
        .get_test_volume()
        .parse::<slint_kit::DecimalValue>()
        .unwrap();
    assert_eq!(value.to_string(), "55");
}
#[test]
fn actual_keypad_starter_opens_without_scope_focus_and_tabs_to_a_native_key() {
    use std::{cell::RefCell, rc::Rc};
    platform();
    let app = crate::QaKeypadDemo::new().unwrap();
    slint_kit::install(&app);
    let digits = Rc::new(RefCell::new(Vec::new()));
    let observed = digits.clone();
    app.on_test_digit(move |digit| observed.borrow_mut().push(digit));
    app.show().unwrap();
    settle();
    app.window().take_snapshot().unwrap();
    assert!(
        ElementQuery::from_root(&app)
            .match_inherits("FocusOutline")
            .find_all()
            .is_empty()
    );
    key(&app, Key::Tab);
    app.window().take_snapshot().unwrap();
    let keypad = ElementQuery::from_root(&app)
        .match_inherits("KitKeypad")
        .find_first()
        .unwrap();
    let rings = ElementQuery::from_root(&app)
        .match_inherits("FocusOutline")
        .find_all();
    assert!(!rings.is_empty());
    assert!(
        rings
            .iter()
            .all(|ring| ring.size().width < keypad.size().width
                && ring.size().height < keypad.size().height)
    );
    key(&app, Key::Space);
    assert_eq!(&*digits.borrow(), &[1]);
}
fn capture(app: &impl ComponentHandle, name: &str) {
    use std::io::Write;
    let Some(directory) = std::env::var_os("KIT_QA_CAPTURE_DIR") else {
        return;
    };
    let image = app.window().take_snapshot().unwrap();
    std::fs::create_dir_all(&directory).unwrap();
    let mut file =
        std::fs::File::create(std::path::PathBuf::from(directory).join(format!("{name}.ppm")))
            .unwrap();
    write!(file, "P6\n{} {}\n255\n", image.width(), image.height()).unwrap();
    for pixel in image.as_slice() {
        file.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
    }
}
#[test]
fn actual_feedback_demos_render_initial_and_keyboard_states_at_both_text_scales() {
    platform();
    let themes = QaRatingFixture::new().unwrap();
    let dark_theme = themes.get_dark_theme();
    let app = crate::QaRatingDemo::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    settle();
    for (name, base) in [
        ("light", app.get_test_theme()),
        ("dark", dark_theme.clone()),
    ] {
        for scale in [1, 2] {
            let mut theme = base.clone();
            theme.text_scale = if scale == 1 { 1. } else { 2. };
            app.set_test_theme(theme);
            settle();
            let score = ElementQuery::from_root(&app)
                .match_inherits("SegmentedOption")
                .match_predicate(|e| e.accessible_label().as_deref() == Some("3"))
                .find_first()
                .unwrap();
            click(&app, &score);
            assert_eq!(app.get_test_value().step, 3);
            assert_eq!(
                ElementQuery::from_root(&app)
                    .match_inherits("SegmentedOption")
                    .match_predicate(|e| e.accessible_label().as_deref() == Some("3"))
                    .find_first()
                    .unwrap()
                    .accessible_checked(),
                Some(true)
            );
            capture(&app, &format!("rating-{name}-{scale}"));
        }
    }
    app.hide().unwrap();
    let app = crate::QaSliderDemo::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    settle();
    for (name, base) in [
        ("light", app.get_test_theme()),
        ("dark", dark_theme.clone()),
    ] {
        for scale in [1, 2] {
            let mut theme = base.clone();
            theme.text_scale = if scale == 1 { 1. } else { 2. };
            app.set_test_theme(theme);
            settle();
            let slider = ElementQuery::from_root(&app)
                .match_inherits("KitSlider")
                .find_first()
                .unwrap();
            click(&app, &slider);
            capture(&app, &format!("slider-{name}-{scale}"));
        }
    }
    app.hide().unwrap();
    let app = crate::QaKeypadDemo::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    settle();
    for (name, base) in [
        ("light", app.get_test_theme()),
        ("dark", dark_theme.clone()),
    ] {
        for scale in [1, 2] {
            let mut theme = base.clone();
            theme.text_scale = if scale == 1 { 1. } else { 2. };
            app.set_test_theme(theme);
            settle();
            capture(&app, &format!("keypad-{name}-{scale}"));
        }
    }
    key(&app, Key::Tab);
    capture(&app, "keypad-keyboard-focus");
}
