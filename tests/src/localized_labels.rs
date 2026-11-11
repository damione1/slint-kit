// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{LocalizedFeedbackFixture, LocalizedNavigationFixture, OptionRow};
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle, Model, ModelRc, VecModel,
    platform::{Key, PointerEventButton, WindowEvent},
};
use slint_kit::ui::{KitStrings, KitThemes, Ownership, ToastEntry};
use std::{
    cell::{Cell, OnceCell, RefCell},
    rc::Rc,
};
thread_local! { static BACKEND: OnceCell<()> = const { OnceCell::new() }; }
fn backend() {
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
}
fn rows() -> ModelRc<OptionRow> {
    ModelRc::new(VecModel::from(
        [(11, "Accueil"), (22, "Rapports"), (33, "Détails")]
            .map(|(token, label)| OptionRow {
                token,
                generation: 3,
                label: label.into(),
                ..Default::default()
            })
            .to_vec(),
    ))
}
fn navigation() -> LocalizedNavigationFixture {
    backend();
    let app = LocalizedNavigationFixture::new().unwrap();
    slint_kit::install(&app);
    app.set_options(rows());
    app
}
fn feedback() -> LocalizedFeedbackFixture {
    backend();
    let app = LocalizedFeedbackFixture::new().unwrap();
    slint_kit::install(&app);
    app.set_entries(ModelRc::new(VecModel::from(vec![ToastEntry {
        token: 71,
        title: "Rapport".into(),
        dismissible: true,
        ..Default::default()
    }])));
    app
}
fn settle<W: ComponentHandle>(app: &W) {
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    slint::platform::update_timers_and_animations();
}
fn french<W: ComponentHandle>(app: &W)
where
    for<'a> KitStrings<'a>: slint::Global<'a, W>,
{
    let strings = app.global::<KitStrings>();
    strings.set_dismiss_label_template("Fermer {title}".into());
    strings.set_breadcrumbs_label("Fil d’Ariane".into());
    strings.set_pagination_label("Navigation entre les pages".into());
    strings.set_notifications_label("Boîte de réception".into());
    strings.set_carousel_position_template("Diapositive {current} sur {total}".into());
    strings.set_pagination_position_template("Page {current} sur {total}".into());
}
fn component<W: ComponentHandle>(app: &W, name: &str) -> ElementHandle {
    ElementQuery::from_root(app)
        .match_inherits(name)
        .find_first()
        .unwrap()
}
fn named<W: ComponentHandle>(app: &W, name: &str) -> ElementHandle {
    let label = name.to_owned();
    ElementQuery::from_root(app)
        .match_predicate(move |item| item.accessible_label().as_deref() == Some(label.as_str()))
        .find_first()
        .unwrap_or_else(|| panic!("missing {name}"))
}
fn click<W: ComponentHandle>(app: &W, label: &str) {
    named(app, label).mock_single_click(PointerEventButton::Left);
    settle(app);
}
#[test]
fn window_defaults_instance_labels_and_toast_entry_labels_reach_the_actual_accessible_actions() {
    let nav = navigation();
    let feedback = feedback();
    nav.show().unwrap();
    feedback.show().unwrap();
    settle(&nav);
    settle(&feedback);
    assert_eq!(
        component(&nav, "KitBreadcrumbs")
            .accessible_label()
            .as_deref(),
        Some("Breadcrumbs")
    );
    assert_eq!(
        component(&nav, "KitPagination")
            .accessible_label()
            .as_deref(),
        Some("Pagination")
    );
    assert_eq!(
        component(&feedback, "KitNotificationCenter")
            .accessible_label()
            .as_deref(),
        Some("Notifications")
    );
    named(&feedback, "Dismiss Enregistré");
    named(&feedback, "Dismiss Confirmation");
    named(&feedback, "Dismiss Rapport");
    french(&feedback);
    settle(&feedback);
    // Globals belong to each window; changing one must not relabel another.
    assert_eq!(
        component(&nav, "KitBreadcrumbs")
            .accessible_label()
            .as_deref(),
        Some("Breadcrumbs")
    );
    french(&nav);
    settle(&nav);
    assert_eq!(
        component(&nav, "KitBreadcrumbs")
            .accessible_label()
            .as_deref(),
        Some("Fil d’Ariane")
    );
    assert_eq!(
        component(&nav, "KitPagination")
            .accessible_label()
            .as_deref(),
        Some("Navigation entre les pages")
    );
    assert_eq!(
        component(&feedback, "KitNotificationCenter")
            .accessible_label()
            .as_deref(),
        Some("Boîte de réception")
    );
    feedback.set_alert_title("Résumé mis à jour".into());
    settle(&feedback);
    named(&feedback, "Fermer Résumé mis à jour");
    let strings = feedback.global::<KitStrings>();
    strings.set_dismiss_label_template("{title} — retirer".into());
    settle(&feedback);
    named(&feedback, "Résumé mis à jour — retirer");
    feedback.set_entries(ModelRc::new(VecModel::from(vec![ToastEntry {
        token: 71,
        title: "Rapport".into(),
        dismissible: true,
        dismiss_label: "Archiver ce message".into(),
        ..Default::default()
    }])));
    settle(&feedback);
    named(&feedback, "Archiver ce message");
    feedback.invoke_override_labels();
    nav.invoke_override_labels();
    settle(&feedback);
    settle(&nav);
    assert_eq!(
        component(&feedback, "KitNotificationCenter")
            .accessible_label()
            .as_deref(),
        Some("Messages personnalisés")
    );
    assert_eq!(
        component(&nav, "KitBreadcrumbs")
            .accessible_label()
            .as_deref(),
        Some("Chemin personnalisé")
    );
    assert_eq!(
        component(&nav, "KitPagination")
            .accessible_label()
            .as_deref(),
        Some("Navigation personnalisée")
    );
    let alert_count = Rc::new(Cell::new(0));
    let observed = alert_count.clone();
    feedback.on_alert_dismissed(move || observed.set(observed.get() + 1));
    let closed = Rc::new(RefCell::new(Vec::new()));
    let observed = closed.clone();
    feedback.on_toast_dismissed(move |token| observed.borrow_mut().push(token));
    click(&feedback, "Fermer ce rapport");
    click(&feedback, "Retirer cette confirmation");
    click(&feedback, "Archiver ce message");
    assert_eq!(alert_count.get(), 1);
    assert_eq!(&*closed.borrow(), &[17, 71]);
}
#[test]
fn position_formats_follow_real_navigation_empty_bounds_and_controlled_acknowledgment() {
    let app = navigation();
    app.show().unwrap();
    settle(&app);
    assert_eq!(app.get_page_label(), "1 / 3");
    assert_eq!(app.get_slide_label(), "1 / 3");
    french(&app);
    settle(&app);
    assert_eq!(app.get_page_label(), "Page 1 sur 3");
    assert_eq!(app.get_slide_label(), "Diapositive 1 sur 3");
    let chosen = Rc::new(Cell::new(0));
    let observed = chosen.clone();
    let expected_generation = app.get_breadcrumb_rows().row_data(0).unwrap().generation;
    app.on_breadcrumb_activated(move |token, generation| {
        assert_eq!(generation, expected_generation);
        observed.set(token);
    });
    click(&app, "Accueil");
    assert_eq!(chosen.get(), 11);
    click(&app, "Page suivante");
    assert_eq!(app.get_page_label(), "Page 2 sur 3");
    app.invoke_focus_carousel();
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::RightArrow.into(),
    });
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::RightArrow.into(),
    });
    settle(&app);
    assert_eq!(app.get_slide_label(), "Diapositive 2 sur 3");
    app.set_ownership(Ownership::Controlled);
    let proposed = Rc::new(Cell::new(-1));
    let observed = proposed.clone();
    app.on_page_changed(move |page| observed.set(page));
    click(&app, "Page suivante");
    assert_eq!(proposed.get(), 2);
    assert_eq!(app.get_page_label(), "Page 2 sur 3");
    app.set_page_index(99);
    settle(&app);
    assert_eq!(app.get_page_label(), "Page 3 sur 3");
    app.set_page_index(-2);
    settle(&app);
    assert_eq!(app.get_page_label(), "Page 1 sur 3");
    app.set_page_count(-1);
    app.set_options(ModelRc::default());
    settle(&app);
    assert_eq!(app.get_page_label(), "Page 0 sur 0");
    assert_eq!(app.get_slide_label(), "Diapositive 0 sur 0");
    app.invoke_override_labels();
    settle(&app);
    assert_eq!(app.get_page_label(), "Position personnalisée");
    assert_eq!(app.get_slide_label(), "Album personnalisé");
}
fn capture<W: ComponentHandle>(app: &W, name: &str) {
    let image = app.window().take_snapshot().unwrap();
    let bounds = app.window().size().to_logical(app.window().scale_factor());
    for kind in [
        "KitAlert",
        "KitToast",
        "KitNotificationCenter",
        "KitBreadcrumbs",
        "KitPagination",
        "KitCarousel",
    ] {
        for item in ElementQuery::from_root(app).match_inherits(kind).find_all() {
            let origin = item.absolute_position();
            let size = item.size();
            assert!(origin.x >= 0. && origin.y >= 0. && size.width > 0. && size.height > 0.);
            assert!(
                origin.x + size.width <= bounds.width + 1.
                    && origin.y + size.height <= bounds.height + 1.,
                "{kind} fits intrinsic window"
            );
        }
    }
    if let Some(dir) = std::env::var_os("KIT_LOCALIZED_LABELS_RENDER_DIR") {
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
fn localized_labels_and_long_position_formats_render_intrinsically_in_both_themes_and_text_scales()
{
    for (mode, dark) in [("light", false), ("dark", true)] {
        for scale in [1., 2.] {
            let nav = navigation();
            let feedback = feedback();
            let themes = nav.global::<KitThemes>();
            let mut theme = if dark {
                themes.get_dark()
            } else {
                themes.get_light()
            };
            theme.text_scale = scale;
            theme.reduced_motion = true;
            nav.set_theme(theme.clone());
            feedback.set_theme(theme);
            french(&nav);
            french(&feedback);
            nav.show().unwrap();
            feedback.show().unwrap();
            settle(&nav);
            settle(&feedback);
            capture(&nav, &format!("navigation-{mode}-{scale}"));
            capture(&feedback, &format!("feedback-{mode}-{scale}"));
        }
    }
}
