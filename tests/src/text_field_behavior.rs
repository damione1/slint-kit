// SPDX-License-Identifier: GPL-3.0-or-later
use crate::*;
use slint::{ComponentHandle, platform::{Key, WindowEvent}};
use std::{cell::RefCell, rc::Rc};
fn fixture() -> TextFieldBehaviorFixture {
    i_slint_backend_testing::init_no_event_loop();
    let app = TextFieldBehaviorFixture::new().unwrap();
    kit_slint::install(&app);
    app.show().unwrap();
    app.invoke_focus_field();
    app
}
fn key(app: &TextFieldBehaviorFixture, text: impl Into<slint::SharedString>) {
    let text = text.into();
    app.window().dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window().dispatch_event(WindowEvent::KeyReleased { text });
    slint::platform::update_timers_and_animations();
}
#[test]
fn change_policy_commits_the_final_native_edit_and_ignores_navigation() {
    let app = fixture();
    let commits = Rc::new(RefCell::new(Vec::new()));
    let observed = commits.clone();
    app.on_committed(move |commit| observed.borrow_mut().push(commit.value.text.to_string()));
    for text in ["a", "b", "c"] { key(&app, text); }
    assert_eq!(*commits.borrow(), ["a", "ab", "abc"]);
    key(&app, Key::LeftArrow);
    key(&app, Key::RightArrow);
    assert_eq!(commits.borrow().len(), 3);
}
