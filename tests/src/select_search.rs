// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{OptionRow, SearchControl, SelectSearchFixture};
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle, Model, ModelRc, SharedString, VecModel,
    platform::{Key, WindowEvent},
};
use slint_kit::ui::{FieldAccess, KitThemes, Ownership};
use std::{cell::OnceCell, rc::Rc};
thread_local! { static BACKEND: OnceCell<()> = const { OnceCell::new() }; }
fn fixture(control: SearchControl, searchable: bool, focus_search: bool) -> SelectSearchFixture {
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
    let app = SelectSearchFixture::new().unwrap();
    slint_kit::install(&app);
    app.set_control(control);
    app.set_searchable(searchable);
    app.set_focus_search_on_open(focus_search);
    app.set_options(ModelRc::new(VecModel::from(
        [
            (11, "Amber", false),
            (22, "Apricot", false),
            (33, "Blocked", true),
            (44, "Banana", false),
        ]
        .map(|(token, label, disabled)| OptionRow {
            token,
            generation: 1,
            label: label.into(),
            disabled,
            ..Default::default()
        })
        .to_vec(),
    )));
    app
}
fn settle(app: &SelectSearchFixture) {
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    slint::platform::update_timers_and_animations();
}
fn key(app: &SelectSearchFixture, text: impl Into<SharedString>) {
    let text = text.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    settle(app);
}
fn search(app: &SelectSearchFixture) -> Option<ElementHandle> {
    ElementQuery::from_root(app)
        .match_inherits("TextInput")
        .match_predicate(|item| {
            item.accessible_label().as_deref() == Some("Filtrer les destinations")
        })
        .find_first()
}
fn show(app: &SelectSearchFixture) {
    app.show().unwrap();
    settle(app);
    app.invoke_focus_control();
    settle(app);
}
#[test]
fn non_searchable_selects_keep_prefix_navigation_and_access_guards_without_an_editor() {
    for control in [SearchControl::Single, SearchControl::Multiple] {
        let app = fixture(control, false, true);
        show(&app);
        key(&app, "b");
        assert!(app.get_state().open);
        assert!(search(&app).is_none());
        assert!(app.get_query().is_empty());
        assert_eq!(app.get_state().active.token, 44);
        key(&app, Key::Return);
        match control {
            SearchControl::Single => assert_eq!(app.get_single_selected().token, 44),
            SearchControl::Multiple => {
                assert_eq!(app.get_multi_selected().row_data(0).unwrap().token, 44)
            }
            SearchControl::Surface => unreachable!(),
        }
        key(&app, Key::Escape);
        for access in [FieldAccess::Disabled, FieldAccess::ReadOnly] {
            app.set_access(access);
            settle(&app);
            app.invoke_focus_control();
            key(&app, "a");
            assert!(!app.get_state().open);
            assert!(app.get_query().is_empty());
        }
    }
}
#[test]
fn searchable_controls_route_the_opening_character_and_list_typing_to_native_editing() {
    for control in [
        SearchControl::Single,
        SearchControl::Multiple,
        SearchControl::Surface,
    ] {
        for focus_search in [true, false] {
            let app = fixture(control, true, focus_search);
            app.set_ownership(Ownership::Controlled);
            let events = Rc::new(std::cell::RefCell::new(Vec::new()));
            let captured = events.clone();
            app.on_query_edited(move |query| captured.borrow_mut().push(query.to_string()));
            show(&app);
            // The first printable key must be inserted once, not used as prefix typeahead.
            key(&app, "b");
            assert_eq!(app.get_query(), "b", "{control:?}, focus={focus_search}");
            assert_eq!(
                search(&app).unwrap().accessible_value().as_deref(),
                Some("b")
            );
            assert_eq!(
                app.get_rows()
                    .iter()
                    .map(|row| row.token)
                    .collect::<Vec<_>>(),
                [11, 33, 44], // Amber, Blocked and Banana match the filter.
            );
            key(&app, Key::Backspace);
            assert!(app.get_query().is_empty());
            key(&app, "z");
            assert_eq!(app.get_rows().row_count(), 0);
            assert!(
                ElementQuery::from_root(&app)
                    .match_predicate(
                        |item| item.accessible_label().as_deref() == Some("Aucune destination")
                    )
                    .find_first()
                    .is_some()
            );
            key(&app, Key::Backspace);
            key(&app, "ban");
            assert_eq!(app.get_query(), "ban");
            key(&app, Key::DownArrow);
            key(&app, Key::Return);
            assert!(
                !app.get_single_selected().present,
                "controlled value requires host acknowledgement"
            );
            assert_eq!(app.get_multi_selected().row_count(), 0);
            if control == SearchControl::Single {
                assert!(!app.get_state().open);
                assert!(app.get_query().is_empty());
            } else {
                assert!(app.get_state().open);
                assert_eq!(app.get_query(), "ban");
            }
            assert_eq!(&events.borrow()[..3], &["b", "", "z"]);
        }
    }
    // Explicit list-first focus still offers the editor through keyboard traversal.
    for control in [
        SearchControl::Single,
        SearchControl::Multiple,
        SearchControl::Surface,
    ] {
        let app = fixture(control, true, false);
        show(&app);
        if control != SearchControl::Surface {
            key(&app, Key::DownArrow);
        }
        key(&app, Key::Tab);
        assert!(!app.get_outside_focused());
        key(&app, "a");
        assert_eq!(app.get_query(), "a");
        key(&app, Key::Tab);
        assert!(app.get_outside_focused());
        if control != SearchControl::Surface {
            assert!(!app.get_state().open);
            assert!(app.get_query().is_empty());
        }
    }
}
#[test]
fn searched_popup_geometry_and_localized_labels_render_in_both_themes_and_text_scales() {
    for control in [SearchControl::Single, SearchControl::Multiple] {
        for (mode, dark) in [("light", false), ("dark", true)] {
            for scale in [1., 2.] {
                let app = fixture(control, true, false);
                let themes = app.global::<KitThemes>();
                let mut theme = if dark {
                    themes.get_dark()
                } else {
                    themes.get_light()
                };
                theme.text_scale = scale;
                theme.reduced_motion = true;
                app.set_theme(theme);
                show(&app);
                key(&app, "ban");
                assert_eq!(app.get_query(), "ban");
                let editor = search(&app).unwrap();
                let bounds = app.window().size().to_logical(app.window().scale_factor());
                let size = editor.size();
                assert!(size.width > 0. && size.height > 0.);
                assert!(size.width <= bounds.width && size.height <= bounds.height);
                let row = ElementQuery::from_root(&app)
                    .match_inherits("KitOption")
                    .match_predicate(|item| item.accessible_label().as_deref() == Some("Banana"))
                    .find_first()
                    .unwrap();
                assert!(row.size().height >= 48. && row.size().width > 0.);
                let image = app.window().take_snapshot().unwrap();
                if let Some(dir) = std::env::var_os("KIT_SELECT_SEARCH_RENDER_DIR") {
                    use std::io::Write;
                    std::fs::create_dir_all(&dir).unwrap();
                    let mut file = std::fs::File::create(
                        std::path::PathBuf::from(dir)
                            .join(format!("search-{control:?}-{mode}-{scale}.ppm")),
                    )
                    .unwrap();
                    write!(file, "P6\n{} {}\n255\n", image.width(), image.height()).unwrap();
                    for pixel in image.as_slice() {
                        file.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
                    }
                }
            }
        }
    }
}
