// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    CivilDateView, CommitPolicy, LocalDateTimeView, TemporalBlurCommitFixture, ZonedDateTimeCommit,
    ZonedTimePresentation,
};
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};
use std::{cell::RefCell, rc::Rc};

fn fixture() -> TemporalBlurCommitFixture {
    i_slint_backend_testing::init_no_event_loop();
    let app = TemporalBlurCommitFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    app
}
fn key(app: &TemporalBlurCommitFixture, text: impl Into<slint::SharedString>) {
    let text = text.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    slint::platform::update_timers_and_animations();
}
fn type_text(app: &TemporalBlurCommitFixture, text: &str) {
    for character in text.chars() {
        key(app, character.to_string());
    }
}
fn date(day: i32) -> CivilDateView {
    CivilDateView {
        year: 2026,
        month: 10,
        day,
    }
}
fn picker_choices(app: &TemporalBlurCommitFixture) -> Rc<RefCell<Vec<CivilDateView>>> {
    let chosen = Rc::new(RefCell::new(Vec::new()));
    let output = chosen.clone();
    app.on_picker_chosen(move |value| output.borrow_mut().push(value));
    chosen
}
fn local_choices(app: &TemporalBlurCommitFixture) -> Rc<RefCell<Vec<LocalDateTimeView>>> {
    let chosen = Rc::new(RefCell::new(Vec::new()));
    let output = chosen.clone();
    app.on_local_chosen(move |value| output.borrow_mut().push(value));
    chosen
}
fn zoned_choices(app: &TemporalBlurCommitFixture) -> Rc<RefCell<Vec<ZonedDateTimeCommit>>> {
    let chosen = Rc::new(RefCell::new(Vec::new()));
    let output = chosen.clone();
    app.on_zoned_chosen(move |value| output.borrow_mut().push(value));
    chosen
}

#[test]
fn a_typed_date_is_accepted_once_when_tab_leaves_the_picker() {
    let app = fixture();
    let chosen = picker_choices(&app);
    app.invoke_focus_picker();
    app.set_picker_text("".into());
    type_text(&app, "2026-10-05");
    assert_eq!(app.get_picked(), date(19), "typing alone does not accept");
    assert!(chosen.borrow().is_empty());
    key(&app, Key::Tab);
    assert_eq!(app.get_picked(), date(5));
    assert_eq!(app.get_picker_text(), "2026-10-05");
    assert_eq!(chosen.borrow().as_slice(), &[date(5)]);
    app.invoke_focus_picker();
    key(&app, Key::Return);
    key(&app, Key::Tab);
    assert_eq!(
        chosen.borrow().len(),
        1,
        "Enter and blur on the unchanged date do not repeat chosen"
    );
}

#[test]
fn an_invalid_typed_date_on_blur_keeps_the_accepted_date_and_the_draft() {
    let app = fixture();
    let chosen = picker_choices(&app);
    app.invoke_focus_picker();
    app.set_picker_text("".into());
    type_text(&app, "2026-13-45");
    key(&app, Key::Tab);
    assert_eq!(app.get_picked(), date(19));
    assert!(chosen.borrow().is_empty());
    assert_eq!(
        app.get_picker_text(),
        "2026-13-45",
        "the draft stays editable"
    );
    assert!(!app.get_picker_validity().valid);
    assert_eq!(
        app.get_invalid_count(),
        0,
        "a blur never reports constraints; only report-validity() and Enter do"
    );
}

#[test]
fn the_submit_policy_defers_a_typed_date_to_enter() {
    let app = fixture();
    app.set_commit_policy(CommitPolicy::OnSubmit);
    let chosen = picker_choices(&app);
    app.invoke_focus_picker();
    app.set_picker_text("".into());
    type_text(&app, "2026-10-05");
    key(&app, Key::Tab);
    assert_eq!(app.get_picked(), date(19), "on-submit ignores blur");
    assert!(chosen.borrow().is_empty());
    app.invoke_focus_picker();
    key(&app, Key::Return);
    assert_eq!(app.get_picked(), date(5));
    assert_eq!(chosen.borrow().as_slice(), &[date(5)]);
}

#[test]
fn the_date_time_pair_commits_only_when_focus_leaves_the_whole_group() {
    let app = fixture();
    let chosen = local_choices(&app);
    let initial = app.get_local_value();
    app.invoke_focus_local();
    app.set_local_date("".into());
    type_text(&app, "2026-10-20");
    key(&app, Key::Tab);
    assert!(
        app.get_local_focused(),
        "Tab from the date editor lands on the time editor of the same group"
    );
    assert_eq!(app.get_local_value(), initial);
    assert!(
        chosen.borrow().is_empty(),
        "moving inside the group is not a blur"
    );
    key(&app, Key::Tab);
    assert!(!app.get_local_focused());
    let expected = LocalDateTimeView {
        date: date(20),
        time: crate::LocalTimeView {
            hour: 9,
            minute: 30,
            second: 0,
        },
    };
    assert_eq!(app.get_local_value(), expected);
    assert_eq!(chosen.borrow().as_slice(), std::slice::from_ref(&expected));
}

#[test]
fn an_invalid_time_on_group_blur_keeps_the_accepted_pair() {
    let app = fixture();
    let chosen = local_choices(&app);
    let initial = app.get_local_value();
    app.invoke_focus_local();
    key(&app, Key::Tab);
    assert!(app.get_local_focused());
    app.set_local_time("".into());
    type_text(&app, "25:99");
    key(&app, Key::Tab);
    assert!(!app.get_local_focused());
    assert_eq!(app.get_local_value(), initial);
    assert!(chosen.borrow().is_empty());
    assert_eq!(app.get_local_time(), "25:99");
    assert_eq!(
        app.get_invalid_count(),
        0,
        "leaving the group with an invalid part does not fire invalid()"
    );
}

#[test]
fn the_zoned_group_resolves_when_focus_leaves_its_last_editor() {
    let app = fixture();
    let chosen = zoned_choices(&app);
    app.invoke_focus_zoned();
    app.set_zoned_date("".into());
    type_text(&app, "2026-11-03");
    key(&app, Key::Tab);
    assert!(
        app.get_zoned_focused(),
        "date to time stays inside the group"
    );
    assert_eq!(app.get_resolution(), ZonedTimePresentation::Unresolved);
    key(&app, Key::Tab);
    assert!(
        app.get_zoned_focused(),
        "time to zone stays inside the group"
    );
    assert_eq!(app.get_resolution(), ZonedTimePresentation::Unresolved);
    assert!(chosen.borrow().is_empty());
    key(&app, Key::Tab);
    assert!(!app.get_zoned_focused());
    assert!(app.get_next_focused());
    assert_eq!(app.get_resolution(), ZonedTimePresentation::Unique);
    let accepted = chosen.borrow();
    assert_eq!(accepted.len(), 1);
    assert_eq!(accepted[0].local.date.day, 3);
    assert_eq!(accepted[0].local.date.month, 11);
    assert_eq!(accepted[0].zone, "America/Toronto");
    assert_eq!(accepted[0].utc_offset_seconds, -5 * 3600);
    assert!(slint_kit::ResolvedDateTime::try_from(accepted[0].clone()).is_ok());
}
