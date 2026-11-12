// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{QaChipLayoutFixture, QaStaticLayoutFixture};
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle, LogicalPosition, LogicalSize, ModelRc, VecModel,
    platform::{Key, PointerEventButton, WindowEvent},
};
use slint_kit::ui::{ChipRow, FileViewEntry, OptionRow, SelectionRef, StepEntry, StepState};
fn platform() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
}
fn settle(app: &impl ComponentHandle) {
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
}
fn model<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(rows))
}
fn options() -> Vec<OptionRow> {
    [(1, "Email"), (2, "Push"), (3, "SMS"), (4, "Calendar")]
        .into_iter()
        .map(|(token, label)| OptionRow {
            token,
            generation: 1,
            label: label.into(),
            description: "".into(),
            disabled: false,
        })
        .collect()
}
fn click(app: &impl ComponentHandle, e: &ElementHandle) {
    let p = e.absolute_position();
    let s = e.size();
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
    settle(app);
}
fn key(app: &impl ComponentHandle, value: Key) {
    let text: slint::SharedString = value.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    settle(app);
}
fn component(app: &impl ComponentHandle, name: &str) -> ElementHandle {
    ElementQuery::from_root(app)
        .match_inherits(name)
        .find_first()
        .unwrap()
}
fn labelled(app: &impl ComponentHandle, label: &str) -> ElementHandle {
    ElementHandle::find_by_accessible_label(app, label)
        .next()
        .unwrap()
}
fn bottom(e: &ElementHandle) -> f32 {
    e.absolute_position().y + e.size().height
}
fn centre(e: &ElementHandle) -> f32 {
    e.absolute_position().y + e.size().height / 2.
}
fn near(a: f32, b: f32) {
    assert!((a - b).abs() < 1.1, "{a} differs from {b}");
}
fn capture(app: &impl ComponentHandle, name: &str) {
    use std::io::Write;
    let Some(dir) = std::env::var_os("KIT_QA_CAPTURE_DIR") else {
        return;
    };
    std::fs::create_dir_all(&dir).unwrap();
    let image = app.window().take_snapshot().unwrap();
    let mut file =
        std::fs::File::create(std::path::PathBuf::from(dir).join(format!("{name}.ppm"))).unwrap();
    write!(file, "P6\n{} {}\n255\n", image.width(), image.height()).unwrap();
    for pixel in image.as_slice() {
        file.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
    }
}
#[test]
fn chip_composites_recover_intrinsic_height_after_repeated_add_remove_and_resize() {
    platform();
    let app = QaChipLayoutFixture::new().unwrap();
    slint_kit::install(&app);
    app.set_options(model(options()));
    app.show().unwrap();
    settle(&app);
    let tags = component(&app, "KitTagInput");
    let editor = tags
        .query_descendants()
        .match_inherits("KitTextField")
        .find_first()
        .unwrap();
    click(&app, &editor);
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: "native draft".into(),
    });
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: "native draft".into(),
    });
    for width in [560., 320., 560.] {
        app.window().set_size(LogicalSize::new(width, 1200.));
        for count in [1, 4, 2, 0, 4, 1] {
            app.set_chips(model(
                options()
                    .into_iter()
                    .take(count)
                    .map(|r| ChipRow {
                        token: r.token,
                        label: r.label,
                        removable: true,
                    })
                    .collect(),
            ));
            app.set_selected(model(
                options()
                    .into_iter()
                    .take(count)
                    .map(|r| SelectionRef {
                        present: true,
                        token: r.token,
                    })
                    .collect(),
            ));
            settle(&app);
            let tags = component(&app, "KitTagInput");
            let editor = tags
                .query_descendants()
                .match_inherits("KitTextField")
                .find_first()
                .unwrap();
            near(
                tags.absolute_position().y + tags.size().height,
                bottom(&editor),
            );
            near(app.get_tag_count_top() - app.get_tag_bottom(), 16.);
            assert_eq!(app.get_draft(), "native draft");
            let tag_chips = tags
                .query_descendants()
                .match_inherits("KitChip")
                .find_all();
            if !tag_chips.is_empty() {
                let last = tag_chips.iter().map(bottom).fold(0., f32::max);
                near(editor.absolute_position().y - last, app.get_label_gap());
            }
            let multi = component(&app, "KitMultiSelect");
            let select = multi
                .query_descendants()
                .match_inherits("KitSelect")
                .find_first()
                .unwrap();
            let chips = multi
                .query_descendants()
                .match_inherits("KitChip")
                .find_all();
            assert_eq!(chips.len(), count);
            if !chips.is_empty() {
                let last = chips.iter().map(bottom).fold(0., f32::max);
                near(select.absolute_position().y - last, app.get_label_gap());
            }
            near(app.get_multi_count_top() - app.get_multi_bottom(), 16.);
        }
    }
    capture(&app, "chips-after-cycles");
}
fn static_fixture() -> QaStaticLayoutFixture {
    platform();
    let app = QaStaticLayoutFixture::new().unwrap();
    slint_kit::install(&app);
    app.set_options(model(options()));
    app.set_files(model(vec![
        FileViewEntry {
            token: 1,
            generation: 1,
            name: "Brief.pdf".into(),
            detail: "PDF, 20 KB".into(),
            removable: true,
            byte_length: "20000".into(),
        },
        FileViewEntry {
            token: 2,
            generation: 1,
            name: "Reference.png".into(),
            detail: "".into(),
            removable: true,
            byte_length: "".into(),
        },
    ]));
    app.set_steps(model(vec![
        StepEntry {
            token: 1,
            generation: 1,
            label: "Account".into(),
            description: "Credentials and security".into(),
            state: StepState::Complete,
            enabled: true,
        },
        StepEntry {
            token: 2,
            generation: 1,
            label: "Workspace".into(),
            description: "Project details".into(),
            state: StepState::Current,
            enabled: true,
        },
        StepEntry {
            token: 3,
            generation: 1,
            label: "Invite team".into(),
            description: "Available after setup".into(),
            state: StepState::Pending,
            enabled: false,
        },
    ]));
    app.show().unwrap();
    settle(&app);
    app
}
#[test]
fn file_names_and_details_are_centred_with_the_trailing_remove_action() {
    let app = static_fixture();
    for (name, base) in [("light", app.get_theme()), ("dark", app.get_dark_theme())] {
        for scale in [1, 2] {
            let mut theme = base.clone();
            theme.text_scale = if scale == 1 { 1. } else { 2. };
            app.set_theme(theme);
            settle(&app);
            let list = component(&app, "KitFileList");
            for (filename, detail) in [("Reference.png", None), ("Brief.pdf", Some("PDF, 20 KB"))] {
                let row = list
                    .query_descendants()
                    .match_predicate({
                        let filename = filename.to_owned();
                        move |e| e.accessible_label().as_deref() == Some(filename.as_str())
                    })
                    .find_all()
                    .into_iter()
                    .max_by(|a, b| a.size().height.total_cmp(&b.size().height))
                    .unwrap();
                let texts = row.query_descendants().match_type_name("Text").find_all();
                let title = texts
                    .iter()
                    .find(|t| t.accessible_label().as_deref() == Some(filename))
                    .unwrap();
                let content_centre = if let Some(detail) = detail {
                    let detail = texts
                        .iter()
                        .find(|t| t.accessible_label().as_deref() == Some(detail))
                        .unwrap();
                    (title.absolute_position().y + bottom(detail)) / 2.
                } else {
                    centre(title)
                };
                near(content_centre, centre(&row));
                let remove = row
                    .query_descendants()
                    .match_inherits("FileRemoveButton")
                    .find_first()
                    .unwrap();
                near(centre(&remove), centre(&row));
            }
            capture(&app, &format!("static-{name}-{scale}"));
        }
    }
}
#[test]
fn disabled_steps_keep_marker_label_alignment_without_a_filled_quiet_strip() {
    let app = static_fixture();
    let steps = component(&app, "KitSteps");
    let rows = steps
        .query_descendants()
        .match_inherits("StepItem")
        .find_all();
    assert_eq!(rows.len(), 3);
    for row in rows {
        let texts = row.query_descendants().match_type_name("Text").find_all();
        let label = texts
            .iter()
            .filter(|e| e.size().width > 50.)
            .min_by(|a, b| a.absolute_position().y.total_cmp(&b.absolute_position().y))
            .unwrap();
        near(label.absolute_position().x - row.absolute_position().x, 40.);
        near(label.absolute_position().y, row.absolute_position().y);
    }
    let disabled = steps
        .query_descendants()
        .match_inherits("KitPressable")
        .find_all()
        .into_iter()
        .find(|e| {
            e.accessible_label()
                .as_deref()
                .is_some_and(|s| s.starts_with("Invite team"))
        })
        .unwrap();
    assert_eq!(disabled.accessible_enabled(), Some(false));
    let snapshot = app.window().take_snapshot().unwrap();
    let point = LogicalPosition::new(
        disabled.absolute_position().x + disabled.size().width - 8.,
        centre(&disabled),
    )
    .to_physical(app.window().scale_factor());
    let x = usize::try_from(point.x).unwrap();
    let y = usize::try_from(point.y).unwrap();
    let width = usize::try_from(snapshot.width()).unwrap();
    assert!(x < width && y < usize::try_from(snapshot.height()).unwrap());
    let pixel = snapshot.as_slice()[y * width + x];
    let canvas = app.get_theme().canvas;
    assert_eq!(
        [pixel.r, pixel.g, pixel.b],
        [canvas.red(), canvas.green(), canvas.blue()]
    );
}
#[test]
fn menu_and_breadcrumb_rows_fit_the_scroll_viewport_and_align_separators() {
    let app = static_fixture();
    for scale in [1, 2] {
        let mut theme = app.get_theme();
        theme.text_scale = if scale == 1 { 1. } else { 2. };
        app.set_theme(theme);
        settle(&app);
        for name in ["KitMenuBar", "KitBreadcrumbs"] {
            let root = component(&app, name);
            let viewport = root
                .query_descendants()
                .match_type_name("Flickable")
                .find_first()
                .unwrap();
            let actions = root
                .query_descendants()
                .match_inherits("KitNavigationAction")
                .find_all();
            assert!(!actions.is_empty());
            for action in &actions {
                near(action.size().height, app.get_row_height());
                assert!(bottom(action) <= bottom(&viewport) + 0.5);
            }
            let first = &actions[0];
            for icon in root
                .query_descendants()
                .match_inherits("KitIcon")
                .find_all()
            {
                near(centre(&icon), centre(first));
            }
        }
    }
}
fn verify_option_icons(app: &impl ComponentHandle) {
    settle(app);
    let rows = ElementQuery::from_root(app)
        .match_inherits("KitOption")
        .find_all();
    assert_eq!(rows.len(), 3);
    for row in &rows {
        let icon = row
            .query_descendants()
            .match_inherits("KitIcon")
            .find_first()
            .unwrap();
        near(centre(&icon), centre(row));
    }
    for pair in rows.windows(2) {
        assert!(pair[1].absolute_position().y >= bottom(&pair[0]) - 0.5);
    }
}
#[test]
fn actual_option_and_keyboard_starters_centre_icons_in_each_composed_row() {
    platform();
    let app = crate::QaOptionDemo::new().unwrap();
    slint_kit::install(&app);
    app.set_test_options(model(options().into_iter().take(3).collect()));
    app.show().unwrap();
    verify_option_icons(&app);
    capture(&app, "option-alignment");
    app.hide().unwrap();
    let app = crate::QaKeyboardDemo::new().unwrap();
    slint_kit::install(&app);
    app.set_test_options(model(options().into_iter().take(3).collect()));
    app.show().unwrap();
    verify_option_icons(&app);
    capture(&app, "keyboard-alignment");
}
#[test]
fn actual_surface_and_trigger_starters_keep_distinct_centred_option_rows() {
    platform();
    let app = crate::QaSurfaceDemo::new().unwrap();
    slint_kit::install(&app);
    app.set_test_options(model(options().into_iter().take(3).collect()));
    app.show().unwrap();
    verify_option_icons(&app);
    capture(&app, "surface-alignment");
    app.hide().unwrap();
    let app = crate::QaTriggerDemo::new().unwrap();
    slint_kit::install(&app);
    app.set_test_options(model(options().into_iter().take(3).collect()));
    app.show().unwrap();
    settle(&app);
    click(&app, &labelled(&app, "Choose destination"));
    verify_option_icons(&app);
    capture(&app, "trigger-alignment");
    key(&app, Key::Escape);
}
#[test]
fn actual_document_starter_fills_its_decorated_parent_without_an_orphan_strip() {
    platform();
    let app = crate::QaDocumentDemo::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    settle(&app);
    for height in [620., 800., 620.] {
        app.window().set_size(LogicalSize::new(640., height));
        settle(&app);
        let document = component(&app, "KitVirtualTextView");
        let parent = ElementQuery::from_root(&app)
            .match_id("App::document-frame")
            .find_first()
            .unwrap();
        near(document.size().height, parent.size().height);
        near(document.absolute_position().y, parent.absolute_position().y);
    }
    capture(&app, "document-fill");
}
