// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{MultiPopupFixture, OptionRow, SelectionRef};
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle, Model, ModelRc, VecModel,
    platform::{Key, PointerEventButton, WindowEvent},
};
use slint_kit::ui::{FieldAccess, Ownership};
use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
};
thread_local! { static BACKEND: OnceCell<()> = const { OnceCell::new() }; }
fn rows(generation: i32) -> ModelRc<OptionRow> {
    ModelRc::new(VecModel::from(
        [
            (11, "Amber", false),
            (22, "Apricot", false),
            (33, "Blocked", true),
            (44, "Banana", false),
        ]
        .map(|(token, label, disabled)| OptionRow {
            token,
            generation,
            label: label.into(),
            disabled,
            ..Default::default()
        })
        .to_vec(),
    ))
}
fn refs(tokens: &[i32]) -> ModelRc<SelectionRef> {
    ModelRc::new(VecModel::from(
        tokens
            .iter()
            .map(|token| SelectionRef {
                present: true,
                token: *token,
            })
            .collect::<Vec<_>>(),
    ))
}
fn fixture() -> MultiPopupFixture {
    BACKEND.with(|backend| {
        backend.get_or_init(|| {
            slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
                mock_time: true,
                renderer_name: Some("software".into()),
                ..Default::default()
            })))
            .unwrap()
        });
    });
    let app = MultiPopupFixture::new().unwrap();
    slint_kit::install(&app);
    app.set_options(rows(1));
    app
}
fn settle(app: &MultiPopupFixture) {
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    slint::platform::update_timers_and_animations();
}
fn key(app: &MultiPopupFixture, key: Key) {
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    settle(app);
}
fn open(app: &MultiPopupFixture) {
    app.invoke_focus_multi();
    key(app, Key::DownArrow);
    assert!(app.get_state().open);
}
fn option(app: &MultiPopupFixture, label: &str) -> ElementHandle {
    let label = label.to_owned();
    ElementQuery::from_root(app)
        .match_inherits("KitOption")
        .match_predicate(move |item| item.accessible_label().as_deref() == Some(label.as_str()))
        .find_first()
        .unwrap()
}
fn capture(app: &MultiPopupFixture, mode: &str, scale: f32, state: &str) {
    let image = app.window().take_snapshot().unwrap();
    let bounds = app.window().size().to_logical(app.window().scale_factor());
    for label in ["Amber", "Apricot"] {
        let row = option(app, label);
        let origin = row.absolute_position();
        let size = row.size();
        assert!(size.width > 0. && size.height > 0.);
        assert!(origin.x >= 0. && origin.y >= 0.);
        assert!(
            origin.x + size.width <= bounds.width + 1.
                && origin.y + size.height <= bounds.height + 1.,
            "{label}"
        );
    }
    if let Some(dir) = std::env::var_os("KIT_MULTI_POPUP_RENDER_DIR") {
        use std::io::Write;
        std::fs::create_dir_all(&dir).unwrap();
        let path =
            std::path::PathBuf::from(dir).join(format!("multi-popup-{mode}-{scale}-{state}.ppm"));
        let mut file = std::fs::File::create(path).unwrap();
        write!(file, "P6\n{} {}\n255\n", image.width(), image.height()).unwrap();
        for pixel in image.as_slice() {
            file.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
        }
    }
}
#[test]
fn multiple_markers_and_keyboard_choices_keep_the_session_query_and_focus_until_escape() {
    for (mode, dark) in [("light", false), ("dark", true)] {
        for scale in [1., 2.] {
            let app = fixture();
            let themes = app.global::<slint_kit::ui::KitThemes>();
            let mut theme = if dark {
                themes.get_dark()
            } else {
                themes.get_light()
            };
            theme.text_scale = scale;
            theme.reduced_motion = true;
            app.set_theme(theme);
            app.set_selected(refs(&[11, 22]));
            app.show().unwrap();
            open(&app);
            app.set_query("a".into());
            settle(&app);
            assert_eq!(option(&app, "Amber").accessible_item_selected(), Some(true));
            assert_eq!(
                option(&app, "Apricot").accessible_item_selected(),
                Some(true)
            );
            assert_eq!(
                option(&app, "Banana").accessible_item_selected(),
                Some(false)
            );
            capture(&app, mode, scale, "two-selected");
            let resets = Rc::new(RefCell::new(Vec::new()));
            let recorded = resets.clone();
            app.on_query_edited(move |query| recorded.borrow_mut().push(query));
            key(&app, Key::Return);
            assert!(app.get_state().open && app.get_focused());
            assert_eq!(app.get_state().active.token, 11);
            assert_eq!(app.get_query(), "a");
            assert_eq!(
                app.get_selected()
                    .iter()
                    .map(|r| r.token)
                    .collect::<Vec<_>>(),
                [22]
            );
            assert_eq!(
                option(&app, "Amber").accessible_item_selected(),
                Some(false)
            );
            assert_eq!(
                option(&app, "Apricot").accessible_item_selected(),
                Some(true)
            );
            assert!(resets.borrow().is_empty());
            capture(&app, mode, scale, "one-selected");
            key(&app, Key::Escape);
            assert!(!app.get_state().open);
            assert_eq!(app.get_query(), "");
            assert_eq!(
                resets
                    .borrow()
                    .iter()
                    .map(|q| q.as_str())
                    .collect::<Vec<_>>(),
                [""]
            );
            open(&app);
            assert_eq!(app.get_rows().row_count(), 4);
            assert_eq!(
                option(&app, "Apricot").accessible_item_selected(),
                Some(true)
            );
            key(&app, Key::Escape);
        }
    }
}
#[test]
fn controlled_markers_wait_for_ack_and_limit_rejection_keeps_the_popup_open() {
    let app = fixture();
    app.set_ownership(Ownership::Controlled);
    app.set_limit(1);
    let events = Rc::new(RefCell::new(Vec::new()));
    let recorded = events.clone();
    app.on_selection_changed(move |row, added| recorded.borrow_mut().push((row.token, added)));
    app.show().unwrap();
    open(&app);
    key(&app, Key::Return);
    assert!(app.get_state().open);
    assert_eq!(events.borrow().as_slice(), [(11, true)]);
    assert_eq!(app.get_selected().row_count(), 0);
    assert_eq!(
        option(&app, "Amber").accessible_item_selected(),
        Some(false)
    );
    app.set_selected(refs(&[11]));
    settle(&app);
    assert_eq!(option(&app, "Amber").accessible_item_selected(), Some(true));
    app.set_close_on_select(true);
    option(&app, "Apricot").mock_single_click(PointerEventButton::Left);
    settle(&app);
    assert_eq!(
        events.borrow().len(),
        1,
        "the limit rejects an addition before closing"
    );
    assert!(app.get_state().open);
    option(&app, "Amber").mock_single_click(PointerEventButton::Left);
    settle(&app);
    assert!(!app.get_state().open);
    assert_eq!(events.borrow().as_slice(), [(11, true), (11, false)]);
    assert_eq!(
        app.get_selected().row_count(),
        1,
        "removal is also controlled until acknowledged"
    );
}
#[test]
fn pointer_toggles_stay_open_and_stale_or_blocked_actions_do_not_change_values() {
    let app = fixture();
    app.show().unwrap();
    open(&app);
    option(&app, "Apricot").mock_single_click(PointerEventButton::Left);
    settle(&app);
    assert!(app.get_state().open);
    assert_eq!(app.get_selected().row_count(), 1);
    assert_eq!(
        option(&app, "Apricot").accessible_item_selected(),
        Some(true)
    );
    let blocked = option(&app, "Blocked");
    blocked.invoke_accessible_default_action();
    settle(&app);
    assert_eq!(app.get_selected().row_count(), 1);
    let row = option(&app, "Amber");
    let origin = row.absolute_position();
    let size = row.size();
    let position =
        slint::LogicalPosition::new(origin.x + size.width / 2., origin.y + size.height / 2.);
    app.window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    app.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    app.set_options(rows(2));
    settle(&app);
    app.window().dispatch_event(WindowEvent::PointerReleased {
        position,
        button: PointerEventButton::Left,
    });
    settle(&app);
    assert_eq!(
        app.get_selected()
            .iter()
            .map(|r| r.token)
            .collect::<Vec<_>>(),
        [22]
    );
    app.set_access(FieldAccess::ReadOnly);
    settle(&app);
    key(&app, Key::Return);
    assert_eq!(app.get_selected().row_count(), 1);
    app.set_access(FieldAccess::Disabled);
    key(&app, Key::Space);
    assert_eq!(app.get_selected().row_count(), 1);
}
#[test]
fn markers_follow_stable_tokens_after_source_reorder_and_true_closing_policy_resets_query() {
    let app = fixture();
    app.set_close_on_select(true);
    app.set_selected(refs(&[22]));
    let source = rows(2);
    let mut reordered = source.iter().collect::<Vec<_>>();
    reordered.swap(0, 1);
    app.set_options(ModelRc::new(VecModel::from(reordered)));
    app.show().unwrap();
    open(&app);
    app.set_query("a".into());
    settle(&app);
    assert_eq!(
        option(&app, "Apricot").accessible_item_selected(),
        Some(true)
    );
    assert_eq!(
        option(&app, "Amber").accessible_item_selected(),
        Some(false)
    );
    option(&app, "Amber").invoke_accessible_default_action();
    settle(&app);
    assert!(!app.get_state().open);
    assert_eq!(app.get_query(), "");
    let mut selected = app
        .get_selected()
        .iter()
        .map(|r| r.token)
        .collect::<Vec<_>>();
    selected.sort();
    assert_eq!(selected, [11, 22]);
}

#[test]
fn host_opens_and_closes_the_list_and_reads_the_open_state() {
    let app = fixture();
    app.show().unwrap();
    assert!(!app.get_is_open());
    app.invoke_open_multi();
    settle(&app);
    assert!(app.get_is_open() && app.get_state().open);
    app.invoke_close_multi();
    settle(&app);
    assert!(!app.get_is_open() && !app.get_state().open);
    app.set_access(FieldAccess::ReadOnly);
    app.invoke_open_multi();
    settle(&app);
    assert!(!app.get_is_open(), "a read-only control never opens");
}

#[test]
fn rejected_additions_report_the_limit_and_stale_tokens_do_not_count_against_it() {
    let app = fixture();
    app.set_limit(1);
    let reached = Rc::new(RefCell::new(Vec::new()));
    let recorded = reached.clone();
    app.on_limit_reached(move |row| recorded.borrow_mut().push(row.token));
    app.show().unwrap();
    open(&app);
    assert!(!app.get_at_limit());
    option(&app, "Amber").mock_single_click(PointerEventButton::Left);
    settle(&app);
    assert!(app.get_at_limit());
    assert!(reached.borrow().is_empty());
    option(&app, "Apricot").mock_single_click(PointerEventButton::Left);
    settle(&app);
    assert_eq!(reached.borrow().as_slice(), [22]);
    assert_eq!(app.get_selected().row_count(), 1);
    // Removing is never limited; a disabled row is not a limit rejection.
    option(&app, "Blocked").invoke_accessible_default_action();
    settle(&app);
    assert_eq!(reached.borrow().as_slice(), [22]);
    // The option holding token 11 is replaced: the stale token frees the slot.
    app.set_options(ModelRc::new(VecModel::from(vec![OptionRow {
        token: 22,
        generation: 2,
        label: "Apricot".into(),
        ..Default::default()
    }])));
    settle(&app);
    assert!(!app.get_at_limit());
    option(&app, "Apricot").mock_single_click(PointerEventButton::Left);
    settle(&app);
    assert_eq!(reached.borrow().as_slice(), [22]);
    assert_eq!(
        app.get_selected()
            .iter()
            .map(|r| r.token)
            .collect::<Vec<_>>(),
        [22]
    );
}
