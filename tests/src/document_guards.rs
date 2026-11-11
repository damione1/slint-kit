// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{DocumentGuardsFixture, TextLine};
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle, LogicalPosition, Model, ModelRc, SharedString, VecModel,
    platform::{Key, PointerEventButton, WindowEvent},
};
use slint_kit::{KeyedModel, ui::KitThemes};
use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
};
thread_local! { static BACKEND: OnceCell<()> = const { OnceCell::new() }; }
fn fixture() -> DocumentGuardsFixture {
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
    let app = DocumentGuardsFixture::new().unwrap();
    slint_kit::install(&app);
    app
}
fn raw_lines() -> Vec<TextLine> {
    [(101, 1, "Alpha"), (502, 2, "Beta"), (907, 3, "Gamma")]
        .map(|(token, number, text)| TextLine {
            token,
            number,
            text: text.into(),
            selected: false,
        })
        .to_vec()
}
fn settle(app: &DocumentGuardsFixture) {
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    slint::platform::update_timers_and_animations();
}
fn key(app: &DocumentGuardsFixture, text: impl Into<SharedString>) {
    let text = text.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    settle(app);
}
fn shortcut(app: &DocumentGuardsFixture, text: &str) {
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    key(app, text);
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
    settle(app);
}
fn element(app: &DocumentGuardsFixture, label: &str) -> ElementHandle {
    let label = label.to_owned();
    ElementQuery::from_root(app)
        .match_predicate(move |item| item.accessible_label().as_deref() == Some(label.as_str()))
        .find_first()
        .unwrap()
}
fn click(app: &DocumentGuardsFixture, label: &str) {
    element(app, label).mock_single_click(PointerEventButton::Left);
    settle(app);
}
fn down(app: &DocumentGuardsFixture, label: &str) -> LogicalPosition {
    let item = element(app, label);
    let pos = item.absolute_position();
    let size = item.size();
    let point = LogicalPosition::new(pos.x + size.width / 2., pos.y + size.height / 2.);
    app.window().dispatch_event(WindowEvent::PointerPressed {
        position: point,
        button: PointerEventButton::Left,
    });
    point
}
fn up(app: &DocumentGuardsFixture, point: LogicalPosition) {
    app.window().dispatch_event(WindowEvent::PointerReleased {
        position: point,
        button: PointerEventButton::Left,
    });
    settle(app);
}
#[test]
fn same_model_shrink_clear_and_invalid_indices_admit_no_editor_commit_save_or_copy() {
    for clear in [false, true] {
        let app = fixture();
        let lines = Rc::new(VecModel::from(raw_lines()));
        app.set_lines(ModelRc::from(lines.clone()));
        app.set_active_index(2);
        app.set_draft("Gamma draft".into());
        app.show().unwrap();
        settle(&app);
        let commits = Rc::new(RefCell::new(Vec::new()));
        let observed = commits.clone();
        app.on_committed(move |token, generation, text| {
            observed.borrow_mut().push((token, generation, text))
        });
        let copied = Rc::new(RefCell::new(Vec::new()));
        let observed = copied.clone();
        app.on_copied(move |token, generation| observed.borrow_mut().push((token, generation)));
        let selected = Rc::new(RefCell::new(Vec::new()));
        let observed = selected.clone();
        app.on_selected(move |token, generation| observed.borrow_mut().push((token, generation)));
        app.invoke_seed_clipboard();
        click(&app, "Edit selected line");
        if clear {
            lines.set_vec(Vec::new());
        } else {
            lines.remove(2);
        }
        // Dispatch before an explicit flush: admission must read current row count.
        key(&app, Key::Return);
        for active in if clear {
            vec![0, -1, 99]
        } else {
            vec![2, -1, 99]
        } {
            app.set_active_index(active);
            settle(&app);
            for label in ["Edit selected line", "Save line", "Copy line"] {
                assert_eq!(
                    element(&app, label).accessible_enabled(),
                    Some(false),
                    "{label}, index={active}"
                );
            }
            element(&app, "Save line").invoke_accessible_default_action();
            element(&app, "Copy line").invoke_accessible_default_action();
            app.invoke_focus_document();
            key(&app, Key::Return);
            assert!(commits.borrow().is_empty());
            assert!(copied.borrow().is_empty());
            assert!(selected.borrow().is_empty());
            app.invoke_paste_clipboard();
            assert_eq!(app.get_clipboard_text(), "Clipboard sentinel");
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct LineKey(&'static str);
fn keyed_lines(reverse: bool) -> Vec<(LineKey, TextLine)> {
    let mut data = vec![
        (
            LineKey("alpha"),
            TextLine {
                number: 1,
                text: "Alpha".into(),
                ..Default::default()
            },
        ),
        (
            LineKey("beta"),
            TextLine {
                number: 2,
                text: "Beta".into(),
                ..Default::default()
            },
        ),
    ];
    if reverse {
        data.reverse();
    }
    data
}
#[test]
fn pending_actions_reject_new_generations_and_valid_native_operations_keep_typed_identity() {
    let app = fixture();
    let rows = KeyedModel::new(keyed_lines(false)).unwrap();
    rows.bind(
        &app,
        DocumentGuardsFixture::set_lines,
        DocumentGuardsFixture::set_generation,
        DocumentGuardsFixture::set_active_index,
    );
    rows.bind_active(&app, |app, row| {
        app.set_draft(row.map_or_else(Default::default, |row| row.text))
    });
    let first = rows.model().row_data(0).unwrap();
    let key = rows.select(first.token, rows.generation()).unwrap();
    assert_eq!(key, LineKey("alpha"));
    let raw_commits = Rc::new(RefCell::new(Vec::new()));
    let raw_observed = raw_commits.clone();
    let commits = Rc::new(RefCell::new(Vec::new()));
    let observed = commits.clone();
    let model = rows.clone();
    app.on_committed(move |token, generation, text| {
        raw_observed.borrow_mut().push((token, generation));
        if let Some(key) = model.update_active(token, generation, |row| row.text = text.clone()) {
            observed.borrow_mut().push((key, text));
        }
    });
    let raw_copies = Rc::new(RefCell::new(Vec::new()));
    let raw_observed = raw_copies.clone();
    let copies = Rc::new(RefCell::new(Vec::new()));
    let observed = copies.clone();
    let model = rows.clone();
    app.on_copied(move |token, generation| {
        raw_observed.borrow_mut().push((token, generation));
        if let Some(key) = model.accept(token, generation) {
            observed.borrow_mut().push(key);
        }
    });
    app.show().unwrap();
    settle(&app);
    app.invoke_seed_clipboard();
    for (label, reverse) in [("Save line", true), ("Copy line", false)] {
        let point = down(&app, label);
        let old_generation = rows.generation();
        rows.replace(keyed_lines(reverse)).unwrap();
        assert_ne!(rows.generation(), old_generation);
        // No timer flush between re-projection and release: handler guards are required too.
        up(&app, point);
        assert!(commits.borrow().is_empty());
        assert!(copies.borrow().is_empty());
        assert!(raw_commits.borrow().is_empty());
        assert!(raw_copies.borrow().is_empty());
    }
    click(&app, "Edit selected line");
    shortcut(&app, "a");
    key(&app, "Edited é漢");
    key(&app, Key::Return);
    assert_eq!(commits.borrow().len(), 1);
    assert_eq!(commits.borrow()[0], (LineKey("alpha"), "Edited é漢".into()));
    assert_eq!(rows.get(&LineKey("beta")).unwrap().text, "Beta");
    element(&app, "Save line").invoke_accessible_default_action();
    settle(&app);
    assert_eq!(commits.borrow().len(), 2);
    element(&app, "Copy line").invoke_accessible_default_action();
    settle(&app);
    assert_eq!(&*copies.borrow(), &[LineKey("alpha")]);
    app.invoke_paste_clipboard();
    assert_eq!(app.get_clipboard_text(), "Edited é漢");
    rows.remove_by_key(&LineKey("alpha"));
    settle(&app);
    assert_eq!(app.get_active_index(), -1);
    assert_eq!(element(&app, "Save line").accessible_enabled(), Some(false));
}
fn capture(app: &DocumentGuardsFixture, name: &str) {
    let doc = ElementQuery::from_root(app)
        .match_inherits("KitVirtualTextView")
        .find_first()
        .unwrap();
    let origin = doc.absolute_position();
    let size = doc.size();
    for label in ["Edit selected line", "Save line", "Copy line"] {
        let item = element(app, label);
        let pos = item.absolute_position();
        let bounds = item.size();
        assert!(bounds.width > 0. && bounds.height > 0.);
        assert!(
            pos.x >= origin.x
                && pos.y >= origin.y
                && pos.x + bounds.width <= origin.x + size.width + 1.
                && pos.y + bounds.height <= origin.y + size.height + 1.,
            "{label}"
        );
    }
    let image = app.window().take_snapshot().unwrap();
    if let Some(dir) = std::env::var_os("KIT_DOCUMENT_GUARDS_RENDER_DIR") {
        use std::io::Write;
        std::fs::create_dir_all(&dir).unwrap();
        let mut file =
            std::fs::File::create(std::path::PathBuf::from(dir).join(format!("{name}.ppm")))
                .unwrap();
        write!(file, "P6\n{} {}\n255\n", image.width(), image.height()).unwrap();
        for pixel in image.as_slice() {
            file.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
        }
    }
}
#[test]
fn invalid_and_valid_footer_states_render_intrinsically_in_both_themes_and_text_scales() {
    for (mode, dark) in [("light", false), ("dark", true)] {
        for scale in [1., 2.] {
            for invalid in [false, true] {
                let app = fixture();
                let themes = app.global::<KitThemes>();
                let mut theme = if dark {
                    themes.get_dark()
                } else {
                    themes.get_light()
                };
                theme.text_scale = scale;
                theme.reduced_motion = true;
                app.set_theme(theme);
                let lines = Rc::new(VecModel::from(raw_lines()));
                app.set_lines(ModelRc::from(lines.clone()));
                app.set_active_index(2);
                app.set_draft("Gamma draft".into());
                app.show().unwrap();
                settle(&app);
                if invalid {
                    lines.remove(2);
                    settle(&app);
                }
                for label in ["Edit selected line", "Save line", "Copy line"] {
                    assert_eq!(element(&app, label).accessible_enabled(), Some(!invalid));
                }
                capture(
                    &app,
                    &format!(
                        "document-{mode}-{scale}-{}",
                        if invalid { "invalid" } else { "valid" }
                    ),
                );
            }
        }
    }
}
