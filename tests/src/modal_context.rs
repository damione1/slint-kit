// SPDX-License-Identifier: GPL-3.0-or-later
//! The window-level modal context makes kit controls of other layers inert.
use crate::{DialogState, ModalContextFixture};
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};
use std::{cell::Cell, rc::Rc};

fn fixture() -> ModalContextFixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = ModalContextFixture::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    window
}
fn element(window: &ModalContextFixture, label: &str) -> ElementHandle {
    let requested = label.to_owned();
    ElementQuery::from_root(window)
        .match_predicate(move |e| e.accessible_label().as_deref() == Some(requested.as_str()))
        .find_first()
        .unwrap_or_else(|| panic!("missing element {label}"))
}
fn enabled(window: &ModalContextFixture, label: &str) -> bool {
    element(window, label).accessible_enabled().unwrap_or(true)
}
fn state(open: bool, topmost: bool) -> DialogState {
    DialogState {
        open,
        topmost,
        busy: false,
    }
}
fn counter() -> Rc<Cell<u32>> {
    Rc::new(Cell::new(0))
}
fn tab(window: &ModalContextFixture) {
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Tab.into(),
    });
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Tab.into(),
    });
    slint::platform::update_timers_and_animations();
}

#[test]
fn open_modal_makes_window_controls_inert_and_keeps_its_own_active() {
    let window = fixture();
    let background = counter();
    let lower = counter();
    let (b, l) = (background.clone(), lower.clone());
    window.on_background_activated(move || b.set(b.get() + 1));
    window.on_lower_activated(move || l.set(l.get() + 1));
    assert!(!window.get_modal_open());
    element(&window, "Background action").invoke_accessible_default_action();
    assert_eq!(background.get(), 1);

    window.set_lower(state(true, true));
    slint::platform::update_timers_and_animations();
    assert!(window.get_modal_open());
    assert_eq!(window.get_modal_layer(), 1);
    for label in [
        "Background action",
        "Background field",
        "Background check",
        "Background select",
        "Background slider",
        "Background range — lower",
        "Background range — upper",
    ] {
        assert!(!enabled(&window, label), "{label} must be inert");
    }
    assert!(enabled(&window, "Lower action"));
    element(&window, "Background action").invoke_accessible_default_action();
    assert_eq!(background.get(), 1, "an inert control must not activate");
    element(&window, "Lower action").invoke_accessible_default_action();
    assert_eq!(lower.get(), 1);

    window.set_lower(state(false, false));
    slint::platform::update_timers_and_animations();
    assert!(!window.get_modal_open());
    for label in [
        "Background action",
        "Background field",
        "Background check",
        "Background select",
        "Background slider",
        "Background range — lower",
        "Background range — upper",
    ] {
        assert!(enabled(&window, label), "{label} must recover");
    }
    element(&window, "Background action").invoke_accessible_default_action();
    assert_eq!(background.get(), 2);
}

#[test]
fn nested_modal_hides_the_lower_layer_and_restores_it_on_close() {
    let window = fixture();
    window.set_lower(state(true, false));
    window.set_upper(state(true, true));
    slint::platform::update_timers_and_animations();
    assert_eq!(window.get_modal_layer(), 2);
    assert!(!enabled(&window, "Lower action"));
    assert!(!enabled(&window, "Background action"));
    assert!(enabled(&window, "Upper action"));

    window.set_upper(state(false, false));
    window.set_lower(state(true, true));
    slint::platform::update_timers_and_animations();
    assert_eq!(window.get_modal_layer(), 1);
    assert!(enabled(&window, "Lower action"));
    assert!(!enabled(&window, "Background action"));

    window.set_lower(state(false, false));
    slint::platform::update_timers_and_animations();
    assert_eq!(window.get_modal_layer(), 0);
    assert!(enabled(&window, "Background action"));
}

#[test]
fn tab_never_reaches_an_inert_background_control() {
    let window = fixture();
    window.set_lower(state(true, true));
    slint::platform::update_timers_and_animations();
    let background = counter();
    let b = background.clone();
    window.on_background_activated(move || b.set(b.get() + 1));
    for _ in 0..6 {
        tab(&window);
        window.window().dispatch_event(WindowEvent::KeyPressed {
            text: Key::Return.into(),
        });
        window.window().dispatch_event(WindowEvent::KeyReleased {
            text: Key::Return.into(),
        });
    }
    assert_eq!(background.get(), 0);
}

#[test]
fn navigation_composites_become_inert_behind_a_modal_and_recover() {
    let window = fixture();
    let labels = [
        "Tab A",
        "Tab B",
        "Menu A",
        "Menu B",
        "Bar A",
        "Bar B",
        "Section A",
        "Section B",
        "Background collapse",
        "Slide back",
        "Slide forward",
        "Crumb A",
        "Page back",
        "Page forward",
    ];
    for label in labels {
        assert!(enabled(&window, label), "{label} must start enabled");
    }
    window.set_lower(state(true, true));
    slint::platform::update_timers_and_animations();
    for label in labels {
        assert!(!enabled(&window, label), "{label} must be inert");
    }
    assert!(
        enabled(&window, "LowerTab A"),
        "a tab in the modal stays active"
    );
    window.set_lower(state(false, false));
    slint::platform::update_timers_and_animations();
    for label in labels {
        assert!(enabled(&window, label), "{label} must recover");
    }
}

mod composites {
    use super::state;
    use crate::ModalContextCompositesFixture;
    use i_slint_backend_testing::ElementQuery;
    use slint::ComponentHandle;

    /// Controls of composites that live in the window, by accessible label.
    const BACKGROUND: &[&str] = &[
        "BG cal back",
        "BG cal forward",
        "BG calendar",
        "BG date",
        "BG date open",
        "BG range cancel",
        "BG time",
        "BG down BG hours",
        "BG up BG hours",
        "BG dt — date",
        "BG dt — time",
        "BG color",
        "BG open BG color",
        "BG red",
        "BG wheel",
        "BG browse BG files",
        "BG folder browse",
        "BG drop",
        "BG keypad",
        "BG delete",
        "BG scroll",
        "BG form",
        "BG notice open",
        "BG notice mark",
        "BG notice dismiss",
        "BG column",
        "BG row",
        "BG article",
        "1: BG line",
        "BG save line",
        "BG copy line",
        "BG line editor",
    ];

    fn fixture() -> ModalContextCompositesFixture {
        i_slint_backend_testing::init_no_event_loop();
        let window = ModalContextCompositesFixture::new().unwrap();
        slint_kit::install(&window);
        window.show().unwrap();
        slint::platform::update_timers_and_animations();
        window
    }
    /// Every element carrying the label that reports an enabled state; a plain
    /// text label next to a field reports none and is skipped.
    fn enabled(window: &ModalContextCompositesFixture, label: &str) -> bool {
        let requested = label.to_owned();
        let states: Vec<bool> = ElementQuery::from_root(window)
            .match_predicate(move |e| e.accessible_label().as_deref() == Some(requested.as_str()))
            .find_all()
            .iter()
            .filter_map(|e| e.accessible_enabled())
            .collect();
        assert!(!states.is_empty(), "missing element {label}");
        states.into_iter().all(|enabled| enabled)
    }

    #[test]
    fn remaining_composites_become_inert_behind_a_modal_and_recover() {
        let window = fixture();
        for label in BACKGROUND {
            assert!(enabled(&window, label), "{label} must start enabled");
        }
        window.set_lower(state(true, true));
        slint::platform::update_timers_and_animations();
        assert!(window.get_modal_open());
        for label in BACKGROUND {
            assert!(!enabled(&window, label), "{label} must be inert");
        }
        for label in [
            "Lower action",
            "Lower keypad",
            "Lower calendar",
            "Lower column",
        ] {
            assert!(enabled(&window, label), "{label} in the modal stays active");
        }
        window.set_lower(state(false, false));
        slint::platform::update_timers_and_animations();
        for label in BACKGROUND {
            assert!(enabled(&window, label), "{label} must recover");
        }
    }
}
