// SPDX-License-Identifier: GPL-3.0-or-later
//! The focus ring follows the latest input: a pointer never leaves one behind,
//! and focus moved from code shows one only after keyboard input.
use crate::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition};

fn fixture() -> FocusVisibilityFixture {
    i_slint_backend_testing::init_no_event_loop();
    let app = FocusVisibilityFixture::new().unwrap();
    kit_slint::install(&app);
    app.show().unwrap();
    slint::platform::update_timers_and_animations();
    app
}
fn click_at(app: &FocusVisibilityFixture, position: LogicalPosition) {
    for event in [
        WindowEvent::PointerMoved { position },
        WindowEvent::PointerPressed {
            position,
            button: PointerEventButton::Left,
        },
        WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        },
    ] {
        app.window().dispatch_event(event);
    }
    slint::platform::update_timers_and_animations();
}
fn click(app: &FocusVisibilityFixture, label: &str) {
    let element = ElementHandle::find_by_accessible_label(app, label)
        .find(|element| element.accessible_role().is_some())
        .unwrap_or_else(|| panic!("{label}"));
    let origin = element.absolute_position();
    let size = element.size();
    click_at(
        app,
        LogicalPosition::new(origin.x + size.width / 2., origin.y + size.height / 2.),
    );
}
fn press(app: &FocusVisibilityFixture, key: Key) {
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    slint::platform::update_timers_and_animations();
}
fn rings(app: &FocusVisibilityFixture) -> usize {
    ElementQuery::from_root(app)
        .match_inherits("FocusOutline")
        .find_all()
        .len()
}

#[test]
fn a_clicked_tab_keeps_focus_without_a_ring_until_a_key_is_pressed() {
    let app = fixture();
    click(&app, "Audio");
    assert_eq!(app.get_selected_token(), 2);
    assert!(app.get_tabs_focused());
    assert_eq!(rings(&app), 0, "pointer");
    press(&app, Key::RightArrow);
    assert_eq!(app.get_selected_token(), 3);
    assert_eq!(rings(&app), 1, "arrow navigation");
    click(&app, "General");
    assert_eq!(rings(&app), 0, "pointer after keys");
}

#[test]
fn focus_restored_after_a_pointer_close_shows_no_ring() {
    let app = fixture();
    click(&app, "Open panel");
    click(&app, "Close panel");
    assert!(app.get_trigger_focused());
    assert!(!app.get_trigger_ring());
    assert_eq!(rings(&app), 0);
}

#[test]
fn focus_restored_after_a_keyboard_close_shows_the_ring() {
    let app = fixture();
    click(&app, "Open panel");
    click(&app, "Audio");
    press(&app, Key::Escape);
    assert!(!app.get_panel_open());
    assert!(app.get_trigger_focused());
    assert!(app.get_trigger_ring());
    assert_eq!(rings(&app), 1);
}

#[test]
fn programmatic_focus_before_any_pointer_input_shows_the_ring() {
    let app = fixture();
    app.invoke_focus_trigger();
    slint::platform::update_timers_and_animations();
    assert!(app.get_trigger_ring());
}

#[test]
fn tab_traversal_shows_the_ring_after_pointer_input() {
    let app = fixture();
    click(&app, "Open panel");
    press(&app, Key::Tab);
    assert!(rings(&app) >= 1);
}

#[test]
fn a_dragged_slider_shows_no_ring_until_it_is_stepped_by_keyboard() {
    let app = fixture();
    app.invoke_focus_trigger();
    let slider = ElementHandle::find_by_accessible_label(&app, "Volume")
        .next()
        .unwrap();
    let origin = slider.absolute_position();
    let size = slider.size();
    click_at(
        &app,
        LogicalPosition::new(origin.x + size.width / 2., origin.y + size.height / 2.),
    );
    assert_eq!(rings(&app), 0, "pointer");
    press(&app, Key::RightArrow);
    assert_eq!(rings(&app), 1, "keyboard");
}

#[test]
fn a_clicked_field_label_focuses_its_control_without_a_ring() {
    let app = fixture();
    let label = ElementQuery::from_root(&app)
        .match_inherits("KitFieldLabel")
        .find_first()
        .unwrap();
    let origin = label.absolute_position();
    let size = label.size();
    click_at(
        &app,
        LogicalPosition::new(origin.x + size.width / 2., origin.y + size.height / 2.),
    );
    assert!(app.get_trigger_focused());
    assert!(!app.get_trigger_ring());
}

#[test]
fn tab_from_a_text_field_rings_the_next_control_after_pointer_input() {
    let app = fixture();
    click(&app, "Open panel");
    let field = ElementQuery::from_root(&app)
        .match_inherits("KitTextField")
        .find_first()
        .unwrap();
    let editor = crate::native_editor(&field);
    let origin = editor.absolute_position();
    let size = editor.size();
    click_at(
        &app,
        LogicalPosition::new(origin.x + size.width / 2., origin.y + size.height / 2.),
    );
    assert_eq!(rings(&app), 1, "the editor keeps its focus edge");
    press(&app, Key::Tab);
    assert_eq!(rings(&app), 1, "the slider reached by Tab");
}
