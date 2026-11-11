// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::rc::Rc;
fn entry(name: &str, bytes: &str) -> FileViewEntry {
    FileViewEntry {
        name: name.into(),
        byte_length: bytes.into(),
        detail: format!("{bytes} B").into(),
        removable: true,
        ..Default::default()
    }
}
#[test]
fn file_constraints_use_shared_required_type_count_and_exact_size_rules() {
    i_slint_backend_testing::init_no_event_loop();
    let app = FileConstraintsFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    assert!(app.get_validity().value_missing);
    let entries = Rc::new(VecModel::from(vec![entry(
        "report.pdf",
        "9007199254740993",
    )]));
    app.set_entries(entries.clone().into());
    assert!(app.get_validity().valid);
    entries.set_row_data(0, entry("report.pdf", "9007199254740994"));
    assert!(app.get_validity().range_overflow);
    app.set_entries(ModelRc::new(VecModel::from(vec![entry("report.pdf", "")])));
    assert!(app.get_validity().bad_input);
    assert!(app.get_message().contains("unavailable"));
    app.set_entries(ModelRc::new(VecModel::from(vec![entry("report.exe", "1")])));
    assert!(app.get_validity().type_mismatch);
    app.set_entries(ModelRc::new(VecModel::from(vec![
        entry("one.pdf", "1"),
        entry("two.pdf", "1"),
        entry("three.pdf", "1"),
    ])));
    assert!(app.get_validity().range_overflow);
    assert!(app.get_message().contains("2"));
    app.set_access(FieldAccess::Disabled);
    assert!(!app.get_will_validate());
    assert!(app.invoke_report());
}
#[test]
fn localized_file_constraints_and_metadata_render_without_a_native_dialog() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = FileConstraintsFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    let mut messages = app.get_messages();
    messages.too_large = "{name} : maximum {bytes} octets".into();
    app.set_messages(messages);
    for (name, base) in [("light", app.get_theme()), ("dark", app.get_dark())] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            app.set_theme(theme);
            app.set_entries(ModelRc::new(VecModel::from(vec![entry(
                "brief.pdf",
                "1200",
            )])));
            assert!(app.invoke_report());
            slint::platform::update_timers_and_animations();
            crate::choice_presentation::save_capture(
                &app.window().take_snapshot().unwrap(),
                &format!("file-constraints-valid-{name}-{scale}"),
            );
            app.set_entries(ModelRc::new(VecModel::from(vec![entry(
                "brief.pdf",
                "9007199254740994",
            )])));
            assert!(!app.invoke_report());
            assert!(app.get_message().contains("9007199254740993 octets"));
            slint::platform::update_timers_and_animations();
            crate::choice_presentation::save_capture(
                &app.window().take_snapshot().unwrap(),
                &format!("file-constraints-invalid-{name}-{scale}"),
            );
        }
    }
}
