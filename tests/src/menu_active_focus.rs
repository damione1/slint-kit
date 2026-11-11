// SPDX-License-Identifier: GPL-3.0-or-later
//! Prepared runtime regressions for actual row-scope focus, not an AT verdict.
use crate::{MenuActiveFocusFixture, OptionRow};
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle, Model, ModelRc, VecModel,
    platform::{Key, PointerEventButton, WindowEvent},
};
use slint_kit::{Choices, SelectOption};
use std::{cell::RefCell, rc::Rc};

fn fixture() -> (MenuActiveFocusFixture, Choices<u32>) {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = MenuActiveFocusFixture::new().unwrap();
    slint_kit::install(&app);
    let choices = Choices::from_options(
        (0..2000u32)
            .map(|key| {
                let mut option = SelectOption::new(key, format!("Row {key}"));
                option.disabled = key == 1998;
                option
            })
            .collect::<Vec<_>>(),
    )
    .unwrap();
    app.set_options(choices.model());
    app.set_selected(choices.selection(&0).unwrap());
    app.show().unwrap();
    app.window().take_snapshot().unwrap();
    (app, choices)
}
fn key_without_flush(app: &MenuActiveFocusFixture, text: impl Into<slint::SharedString>) {
    let text = text.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
}
fn settle(app: &MenuActiveFocusFixture) {
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    slint::platform::update_timers_and_animations();
}
fn key(app: &MenuActiveFocusFixture, value: impl Into<slint::SharedString>) {
    key_without_flush(app, value);
    settle(app);
}
fn rows(app: &MenuActiveFocusFixture) -> Vec<ElementHandle> {
    ElementQuery::from_root(app)
        .match_inherits("KitMenu")
        .find_first()
        .unwrap()
        .query_descendants()
        .match_inherits("KitNavigationAction")
        .find_all()
}
fn chosen(app: &MenuActiveFocusFixture) -> Rc<RefCell<Vec<OptionRow>>> {
    let events = Rc::new(RefCell::new(Vec::new()));
    let target = events.clone();
    app.on_chosen(move |row| target.borrow_mut().push(row));
    events
}

fn capture_menu(app: &MenuActiveFocusFixture) {
    let Some(directory) = std::env::var_os("KIT_CONTROL_CAPTURE_DIR") else {
        return;
    };
    use std::io::Write;
    std::fs::create_dir_all(&directory).unwrap();
    for (name, base) in [
        ("light", app.get_light_theme()),
        ("dark", app.get_dark_theme()),
    ] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            theme.reduced_motion = true;
            app.set_theme(theme);
            key(app, Key::End);
            assert!(app.get_active_row_focused());
            let image = app.window().take_snapshot().unwrap();
            let mut output = std::fs::File::create(
                std::path::PathBuf::from(&directory)
                    .join(format!("menu-native-row-focus-{name}-{scale}.ppm")),
            )
            .unwrap();
            write!(output, "P6\n{} {}\n255\n", image.width(), image.height()).unwrap();
            for pixel in image.as_slice() {
                output.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
            }
        }
    }
}

#[test]
fn explicit_menu_navigation_focuses_the_materialized_row_and_preserves_controlled_selection() {
    let (app, choices) = fixture();
    let events = chosen(&app);
    app.invoke_focus_menu();
    assert!(app.get_menu_focused());
    assert!(
        !app.get_active_row_focused(),
        "initial entry keeps the composite Tab stop"
    );
    key(&app, Key::End);
    assert_eq!(app.get_active_index(), 1999);
    assert!(
        app.get_active_row_focused(),
        "the actual native row scope reported FocusIn"
    );
    let materialized = rows(&app);
    assert!(
        materialized.len() < 100,
        "the model must remain virtualized"
    );
    let last = materialized
        .iter()
        .find(|row| row.accessible_label().as_deref() == Some("Row 1999"))
        .unwrap();
    assert_eq!(last.accessible_item_index(), Some(1999));
    assert_eq!(last.accessible_item_count(), Some(2000));
    assert!(
        last.query_descendants()
            .match_inherits("KitFocusTarget")
            .find_first()
            .is_some()
    );
    assert_eq!(app.get_selected(), choices.selection(&0).unwrap());
    key(&app, Key::UpArrow);
    assert_eq!(
        app.get_active_index(),
        1997,
        "keys bubble from the row scope and skip disabled"
    );
    assert!(app.get_active_row_focused());
    assert!(
        events.borrow().is_empty(),
        "focus/navigation do not accept a controlled value"
    );
    // Held Return goes through the existing translator exactly once.
    for _ in 0..2 {
        app.window().dispatch_event(WindowEvent::KeyPressed {
            text: Key::Return.into(),
        });
    }
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Return.into(),
    });
    settle(&app);
    assert_eq!(events.borrow().len(), 1);
    assert_eq!(events.borrow()[0].token, app.get_active_token());
    assert_eq!(app.get_selected(), choices.selection(&0).unwrap());
    capture_menu(&app);
}

#[test]
fn a_pending_virtual_row_request_cannot_steal_editor_focus_selection_or_undo() {
    let (app, _) = fixture();
    app.invoke_focus_menu();
    key_without_flush(&app, Key::End);
    app.invoke_focus_reference();
    app.invoke_select_reference();
    settle(&app);
    assert!(app.get_reference_focused());
    assert!(!app.get_active_row_focused());
    assert_eq!(
        (app.get_reference_anchor(), app.get_reference_cursor()),
        (1, 4)
    );
    key(&app, "X");
    assert_eq!(app.get_reference_text(), "nXve");
    app.invoke_undo_reference();
    assert_eq!(
        app.get_reference_text(),
        "nve",
        "native undo first removes the inserted text"
    );
    app.invoke_undo_reference();
    assert_eq!(
        app.get_reference_text(),
        "native",
        "native undo then restores the replaced selection"
    );
    // Disable/re-enable and a fresh generation cannot revive the old nonce.
    app.set_interactive(false);
    settle(&app);
    app.set_interactive(true);
    app.set_options(ModelRc::new(VecModel::from(vec![OptionRow {
        token: 77,
        generation: 500,
        label: "Replacement".into(),
        ..Default::default()
    }])));
    app.set_query("Replace".into());
    settle(&app);
    assert!(app.get_reference_focused());
    key(&app, "Z");
    assert!(app.get_reference_text().contains('Z'));
}

#[test]
fn host_redirection_and_consumed_requests_survive_projection_recreation_without_refocusing() {
    let (app, choices) = fixture();
    app.set_redirect_navigation(true);
    app.invoke_focus_menu();
    key(&app, Key::End);
    assert!(
        app.get_reference_focused(),
        "host callback wins before a request can be emitted"
    );
    assert!(!app.get_active_row_focused());
    app.set_redirect_navigation(false);
    app.invoke_focus_menu();
    key(&app, Key::Home);
    assert!(app.get_active_row_focused());
    key(&app, Key::End);
    assert!(app.get_active_row_focused());
    app.invoke_focus_reference();
    let mut reversed = choices.model().iter().collect::<Vec<_>>();
    reversed.reverse();
    app.set_options(ModelRc::new(VecModel::from(reversed)));
    settle(&app);
    assert!(
        app.get_reference_focused(),
        "a recreated target cannot replay an applied request"
    );
    assert!(!app.get_active_row_focused());
    key(&app, "Y");
    assert!(app.get_reference_text().contains('Y'));
}

#[test]
fn pointer_down_keeps_its_row_press_while_navigation_focus_settles() {
    let (app, choices) = fixture();
    let events = chosen(&app);
    app.invoke_focus_menu();
    key_without_flush(&app, Key::End);
    let destination = rows(&app)
        .into_iter()
        .find(|row| row.accessible_label().as_deref() == Some("Row 1999"))
        .unwrap();
    let origin = destination.absolute_position();
    let position = slint::LogicalPosition::new(
        origin.x + destination.size().width / 2.,
        origin.y + destination.size().height / 2.,
    );
    app.window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    app.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    settle(&app);
    assert!(app.get_menu_focused());
    assert!(events.borrow().is_empty());
    app.window().dispatch_event(WindowEvent::PointerReleased {
        position,
        button: PointerEventButton::Left,
    });
    settle(&app);
    assert_eq!(
        events.borrow().len(),
        1,
        "row transfer must not cancel the pointer press"
    );
    assert_eq!(
        events.borrow()[0].token,
        choices.selection(&1999).unwrap().token
    );
    assert_eq!(app.get_selected(), choices.selection(&0).unwrap());
}

#[test]
fn dismissal_cancels_the_unmaterialized_request_and_next_navigation_still_works() {
    let (app, _) = fixture();
    app.invoke_focus_menu();
    key_without_flush(&app, Key::End);
    key_without_flush(&app, Key::Escape);
    settle(&app);
    assert!(app.get_menu_focused());
    assert!(
        !app.get_active_row_focused(),
        "dismissed request cannot apply during later layout"
    );
    key(&app, Key::Home);
    assert!(
        app.get_active_row_focused(),
        "a fresh explicit action can request a new nonce"
    );
}

#[test]
fn native_tab_and_backtab_keep_one_menu_stop_after_row_navigation() {
    let (app, _) = fixture();
    app.invoke_focus_menu();
    key(&app, Key::End);
    assert!(app.get_active_row_focused());
    key(&app, Key::Tab);
    assert!(app.get_reference_focused());
    key(&app, Key::Backtab);
    assert!(app.get_menu_focused());
    assert!(
        !app.get_active_row_focused(),
        "Tab entry starts on the composite scope"
    );
    key(&app, Key::End);
    assert!(app.get_active_row_focused());
    key(&app, Key::Backtab);
    assert!(
        app.get_reference_focused(),
        "Backtab leaves the row without a second ancestor stop"
    );
}
