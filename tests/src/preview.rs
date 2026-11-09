// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{PreviewFamily, PreviewFixture};
use i_slint_backend_testing::ElementQuery;
use slint::{ComponentHandle, Model};
use std::{cell::Cell, rc::Rc};

fn fixture(family: PreviewFamily) -> PreviewFixture {
    let window = PreviewFixture::new().unwrap();
    // Deliberately omit install: this is the editor/viewer's service state.
    window.set_family(family);
    window.show().unwrap();
    window
}

fn has_label(window: &PreviewFixture, label: &str) -> bool {
    let label = label.to_owned();
    ElementQuery::from_root(window)
        .match_predicate(move |element| element.accessible_label().as_deref() == Some(&label))
        .find_first()
        .is_some()
}

#[test]
fn declarative_options_render_without_install_across_selection_families() {
    i_slint_backend_testing::init_no_event_loop();
    for family in [
        PreviewFamily::Tabs,
        PreviewFamily::Menu,
        PreviewFamily::MenuBar,
        PreviewFamily::Breadcrumbs,
        PreviewFamily::Accordion,
        PreviewFamily::Radio,
        PreviewFamily::RadioHorizontal,
        PreviewFamily::Segments,
        PreviewFamily::Toggles,
        PreviewFamily::Surface,
    ] {
        let window = fixture(family);
        for label in ["Alpha", "Beta", "Blocked"] {
            assert!(has_label(&window, label), "{family:?}: missing {label}");
        }
        // Replacing declarative data must update the preview, not cache it.
        let mut replacement = window.get_options().row_data(0).unwrap();
        replacement.label = "Replacement".into();
        window.set_options(slint::ModelRc::new(slint::VecModel::from(vec![
            replacement,
        ])));
        assert!(
            has_label(&window, "Replacement"),
            "{family:?}: stale preview"
        );
        assert!(
            !has_label(&window, "Beta"),
            "{family:?}: removed row remains"
        );
    }
}

#[test]
fn preview_rendering_does_not_admit_uninitialized_domain_choices() {
    i_slint_backend_testing::init_no_event_loop();
    let window = fixture(PreviewFamily::Menu);
    let chosen = Rc::new(Cell::new(0));
    let count = chosen.clone();
    window.on_chosen(move |_| count.set(count.get() + 1));
    let alpha = ElementQuery::from_root(&window)
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Alpha"))
        .find_first()
        .unwrap();
    alpha.mock_single_click(slint::platform::PointerEventButton::Left);
    assert_eq!(chosen.get(), 0);
    slint_kit::install(&window);
    slint::platform::update_timers_and_animations();
    let alpha = ElementQuery::from_root(&window)
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Alpha"))
        .find_first()
        .unwrap();
    alpha.mock_single_click(slint::platform::PointerEventButton::Left);
    assert_eq!(
        chosen.get(),
        1,
        "installation enables the existing controller"
    );
}

#[test]
fn rating_preview_uses_the_configured_scale() {
    i_slint_backend_testing::init_no_event_loop();
    let window = fixture(PreviewFamily::Rating);
    for label in ["1", "4", "7"] {
        assert!(has_label(&window, label), "missing {label}");
    }
    window.set_rating_scale(3);
    assert!(has_label(&window, "3"));
    assert!(!has_label(&window, "4"));
    window.set_rating_scale(0);
    assert!(!has_label(&window, "1"));
}

#[test]
fn carousel_preview_shows_a_static_slide_without_an_active_projection() {
    i_slint_backend_testing::init_no_event_loop();
    let window = fixture(PreviewFamily::Carousel);
    assert!(has_label(&window, "Alpha"));
    assert!(has_label(&window, "1 / 3"));
}

#[test]
fn explicit_calendar_and_field_presentations_survive_without_rust_services() {
    i_slint_backend_testing::init_no_event_loop();
    for (family, value) in [
        (PreviewFamily::Text, "Hello"),
        (PreviewFamily::Time, "12:30"),
        (PreviewFamily::Color, "#123456"),
    ] {
        let window = fixture(family);
        assert!(
            ElementQuery::from_root(&window)
                .match_predicate(move |element| element.accessible_value().as_deref() == Some(value))
                .find_first()
                .is_some(),
            "{family:?}: authored text must remain visible"
        );
    }
    let calendar = fixture(PreviewFamily::Calendar);
    assert!(has_label(&calendar, "November 8, 2026"));
    let select = fixture(PreviewFamily::Select);
    assert!(has_label(&select, "Choose an option"));
}
