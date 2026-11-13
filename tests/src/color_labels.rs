// SPDX-License-Identifier: GPL-3.0-or-later
use crate::ColorLabelsFixture;
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::ComponentHandle;
use std::cell::OnceCell;
thread_local! { static BACKEND: OnceCell<()> = const { OnceCell::new() }; }
fn fixture(localized: bool) -> ColorLabelsFixture {
    BACKEND.with(|backend| {
        backend.get_or_init(|| {
            slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
                mock_time: true,
                renderer_name: Some("software".into()),
                ..Default::default()
            })))
            .unwrap();
        });
    });
    let app = ColorLabelsFixture::new().unwrap();
    slint_kit::install(&app);
    app.set_localized(localized);
    app
}
fn settle(app: &ColorLabelsFixture) {
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
}
fn labels(app: &ColorLabelsFixture) -> Vec<String> {
    ElementQuery::from_root(app)
        .match_predicate(|_| true)
        .find_all()
        .iter()
        .filter_map(ElementHandle::accessible_label)
        .map(|label| label.to_string())
        .collect()
}
fn has(app: &ColorLabelsFixture, wanted: &str) -> bool {
    labels(app).iter().any(|label| label == wanted)
}
#[test]
fn host_labels_replace_every_built_in_string_of_the_field_popup_and_picker() {
    let app = fixture(true);
    settle(&app);
    app.invoke_open_picker();
    settle(&app);
    for wanted in [
        "Ouvrir le sélecteur de couleur pour Accent",
        "Aperçu du brouillon",
        "Rouge vif",
        "Choisir RVB 34, 197, 94",
        "Rouge (brouillon)",
        "Vert (brouillon)",
        "Bleu (brouillon)",
        "Opacité (brouillon)",
        "Annuler",
        "Appliquer la couleur",
        "Rouge",
        "Vert",
        "Bleu",
        "Opacité",
    ] {
        assert!(
            has(&app, wanted),
            "missing {wanted:?} in {:?}",
            labels(&app)
        );
    }
    for english in [
        "Cancel",
        "Apply color",
        "Red draft",
        "Draft color preview",
        "Open color picker for Accent",
        "Choose RGB 239, 68, 68",
    ] {
        assert!(!has(&app, english), "leaked {english:?}");
    }
}
#[test]
fn unlocalised_components_keep_the_english_defaults() {
    let app = fixture(false);
    settle(&app);
    app.invoke_open_picker();
    settle(&app);
    for wanted in [
        "Open color picker for Accent",
        "Draft color preview",
        "Choose RGB 239, 68, 68",
        "Choose RGB 34, 197, 94",
        "Red draft",
        "Cancel",
        "Apply color",
        "Red",
        "Alpha",
    ] {
        assert!(
            has(&app, wanted),
            "missing {wanted:?} in {:?}",
            labels(&app)
        );
    }
}
