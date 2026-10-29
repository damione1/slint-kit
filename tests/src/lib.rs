// SPDX-License-Identifier: GPL-3.0-or-later
#![cfg(test)]
mod ui {
    slint::include_modules!();
}
pub use ui::*;
mod wrapped_field;

#[cfg(test)]
mod tests {
    use super::*;
    use i_slint_backend_testing::{ElementHandle, ElementQuery};
    use slint::platform::{Key, PointerEventButton, WindowEvent};
    use slint::{ComponentHandle, LogicalPosition};
    use std::{cell::Cell, rc::Rc};

    fn fixture() -> Fixture {
        i_slint_backend_testing::init_no_event_loop();
        let window = Fixture::new().unwrap();
        window.show().unwrap();
        window
    }
    fn element(window: &Fixture, label: &str) -> ElementHandle {
        let requested_label = label.to_owned();
        ElementQuery::from_root(window)
            .match_predicate(move |e| {
                e.accessible_label().as_deref() == Some(requested_label.as_str())
            })
            .find_first()
            .unwrap_or_else(|| panic!("missing element {label}"))
    }
    fn key_down(window: &Fixture, key: Key) {
        window
            .window()
            .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    }
    fn key_up(window: &Fixture, key: Key) {
        window
            .window()
            .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    }
    fn key(window: &Fixture, key: Key) {
        key_down(window, key);
        key_up(window, key);
    }
    fn count(window: &Fixture, custom: bool) -> Rc<Cell<u32>> {
        let count = Rc::new(Cell::new(0));
        let target = count.clone();
        if custom {
            window.on_custom_activated(move || target.set(target.get() + 1));
        } else {
            window.on_standard_activated(move || target.set(target.get() + 1));
        }
        count
    }

    #[test]
    fn standard_and_composed_button_share_all_activation_paths() {
        let window = fixture();
        for custom in [false, true] {
            let label = if custom {
                "Custom action"
            } else {
                "Standard action"
            };
            let count = count(&window, custom);
            let button = element(&window, label);
            button.mock_single_click(PointerEventButton::Left);
            assert_eq!(count.get(), 1, "{label}: pointer produces one activation");
            button.invoke_accessible_default_action();
            assert_eq!(
                count.get(),
                2,
                "{label}: assistive action uses same primitive"
            );
            if custom {
                window.invoke_focus_custom();
            } else {
                window.invoke_focus_standard();
            }
            key_down(&window, Key::Return);
            for _ in 0..8 {
                window
                    .window()
                    .dispatch_event(WindowEvent::KeyPressRepeated {
                        text: Key::Return.into(),
                    });
            }
            assert_eq!(
                count.get(),
                3,
                "{label}: held Enter cannot repeat ordinary action"
            );
            key_up(&window, Key::Return);
            key_down(&window, Key::Space);
            assert_eq!(count.get(), 3);
            assert!(if custom {
                window.get_custom_pressed()
            } else {
                window.get_standard_pressed()
            });
            for _ in 0..8 {
                window
                    .window()
                    .dispatch_event(WindowEvent::KeyPressRepeated {
                        text: Key::Space.into(),
                    });
            }
            key_up(&window, Key::Space);
            assert_eq!(count.get(), 4, "{label}: Space activates once at release");
            window.set_disabled(true);
            button.mock_single_click(PointerEventButton::Left);
            button.invoke_accessible_default_action();
            key(&window, Key::Return);
            key(&window, Key::Space);
            assert_eq!(count.get(), 4, "{label}: disabled blocks all routes");
            assert_eq!(button.accessible_enabled(), Some(false));
            window.set_disabled(false);
            window.set_busy(true);
            button.invoke_accessible_default_action();
            assert_eq!(count.get(), 4, "{label}: busy blocks reactivation");
            window.set_busy(false);
        }
    }

    #[test]
    fn canceled_press_blur_and_disable_cannot_leak_activation() {
        let window = fixture();
        for custom in [false, true] {
            let count = count(&window, custom);
            if custom {
                window.invoke_focus_custom();
            } else {
                window.invoke_focus_standard();
            }
            key_down(&window, Key::Space);
            if custom {
                window.invoke_focus_standard();
            } else {
                window.invoke_focus_custom();
            }
            key_up(&window, Key::Space);
            assert_eq!(count.get(), 0, "blur cancels Space");
            if custom {
                window.invoke_focus_custom();
            } else {
                window.invoke_focus_standard();
            }
            key_down(&window, Key::Space);
            window.set_disabled(true);
            key_up(&window, Key::Space);
            assert_eq!(count.get(), 0, "disabled during gesture cancels it");
            window.set_disabled(false);
            let inside = LogicalPosition::new(if custom { 300. } else { 80. }, 40.);
            let outside = LogicalPosition::new(600., 550.);
            window
                .window()
                .dispatch_event(WindowEvent::PointerMoved { position: inside });
            window.window().dispatch_event(WindowEvent::PointerPressed {
                position: inside,
                button: PointerEventButton::Left,
            });
            window
                .window()
                .dispatch_event(WindowEvent::PointerMoved { position: outside });
            window
                .window()
                .dispatch_event(WindowEvent::PointerReleased {
                    position: outside,
                    button: PointerEventButton::Left,
                });
            assert_eq!(count.get(), 0, "release outside cancels pointer gesture");
            window
                .window()
                .dispatch_event(WindowEvent::PointerMoved { position: inside });
            window.window().dispatch_event(WindowEvent::PointerPressed {
                position: inside,
                button: PointerEventButton::Left,
            });
            if custom {
                window.invoke_cancel_custom();
            } else {
                window.invoke_cancel_standard();
            }
            window
                .window()
                .dispatch_event(WindowEvent::PointerReleased {
                    position: inside,
                    button: PointerEventButton::Left,
                });
            assert_eq!(
                count.get(),
                0,
                "host cancellation (e.g. scrolling) cancels gesture"
            );
        }
    }

    #[test]
    fn ten_thousand_options_are_virtualized_and_keyboard_moves_active_into_view() {
        let window = fixture();
        let options = (0..10_000)
            .map(|i| kit_core::SelectOption::new(i, format!("Option{i:05}")))
            .collect();
        let binding =
            kit_slint::bind_select!(ui, window, options, None, kit_core::Ownership::Local);
        window.invoke_focus_select();
        key(&window, Key::DownArrow);
        assert!(window.get_select_state().open);
        let rows = ElementQuery::from_root(&window)
            .match_inherits("KitOption")
            .find_all();
        assert!(!rows.is_empty(), "ListView must mount visible options");
        assert!(
            rows.len() < 40,
            "10k model instantiated {} visual rows",
            rows.len()
        );
        // Native scrolling writes the Flickable position; subsequent keyboard
        // movement must still bring an offscreen active option into view.
        rows.first().unwrap().scroll(0., -144.);
        key(&window, Key::End);
        assert_eq!(window.get_select_state().active_index, 9999);
        let active = element(&window, "Option09999");
        assert!(active.is_valid());
        assert!(active.size().height > 0.);
        let surface = ElementQuery::from_root(&window)
            .match_inherits("KitSelectSurface")
            .find_first()
            .expect("select surface mounted");
        let active_position = active.absolute_position();
        let surface_position = surface.absolute_position();
        assert!(
            active_position.y >= surface_position.y
                && active_position.y + active.size().height
                    <= surface_position.y + surface.size().height + 1.,
            "active row lies outside popup: active={active_position:?}, surface={surface_position:?}"
        );
        let rows = ElementQuery::from_root(&window)
            .match_inherits("KitOption")
            .find_all();
        assert!(
            rows.len() < 40,
            "End must preserve bounded row instantiation"
        );
        key(&window, Key::PageUp);
        assert_eq!(window.get_select_state().active_index, 9992);
        key(&window, Key::Home);
        assert_eq!(window.get_select_state().active_index, 0);
        key(&window, Key::PageDown);
        assert_eq!(window.get_select_state().active_index, 7);
        for character in "Option05000".chars() {
            let text = character.to_string();
            window.window().dispatch_event(WindowEvent::KeyPressed {
                text: text.as_str().into(),
            });
            window.window().dispatch_event(WindowEvent::KeyReleased {
                text: text.as_str().into(),
            });
        }
        assert_eq!(window.get_select_state().active_index, 5000);
        assert!(element(&window, "Option05000").is_valid());
        key(&window, Key::End);
        window.set_dark(true);
        assert!(window.get_select_state().open);
        assert_eq!(window.get_select_state().active_index, 9999);
        key(&window, Key::Return);
        assert_eq!(binding.selected(), Some(9999));
        assert!(!window.get_select_state().open);
    }

    #[test]
    fn runtime_theme_preserves_real_text_input_and_selection_state() {
        let window = fixture();
        window.set_dialog_state(DialogState {
            open: true,
            topmost: true,
            busy: false,
        });
        window.invoke_focus_editor();
        for text in ["é", "漢", "🦊"] {
            window
                .window()
                .dispatch_event(WindowEvent::KeyPressed { text: text.into() });
            window
                .window()
                .dispatch_event(WindowEvent::KeyReleased { text: text.into() });
        }
        assert_eq!(window.get_edit_text().as_str(), "é漢🦊");
        window.set_dark(true);
        window
            .window()
            .dispatch_event(WindowEvent::KeyPressed { text: "!".into() });
        window
            .window()
            .dispatch_event(WindowEvent::KeyReleased { text: "!".into() });
        assert_eq!(window.get_edit_text().as_str(), "é漢🦊!");
    }
    #[test]
    fn android_back_dismisses_only_the_topmost_idle_dialog_and_is_consumed_while_busy() {
        let window = fixture();
        let reasons = Rc::new(std::cell::RefCell::new(Vec::new()));
        let observed = reasons.clone();
        window.on_dismiss_dialog(move |reason| observed.borrow_mut().push(reason));
        for (topmost, busy) in [(true, false), (true, true), (false, false)] {
            window.set_dialog_state(DialogState {
                open: true,
                topmost,
                busy,
            });
            window.invoke_focus_editor();
            let result = window
                .window()
                .dispatch_event_with_result(WindowEvent::KeyPressed {
                    text: Key::Back.into(),
                })
                .unwrap();
            assert_eq!(
                matches!(result, slint::platform::WindowEventDispatchResult::Accepted),
                topmost
            );
            key_up(&window, Key::Back);
        }
        assert_eq!(*reasons.borrow(), [CloseReason::SystemBack]);
        assert_eq!(window.get_edit_text().as_str(), "");
    }

    #[test]
    fn dialog_cycles_registered_custom_controls_and_restores_real_focus() {
        use kit_core::{FocusDirection, FocusId, FocusRegistry, FocusScopeId};
        let window = fixture();
        let mut registry = FocusRegistry::new();
        let root = FocusScopeId(0);
        let dialog = FocusScopeId(1);
        registry.register_scope(dialog, root).unwrap();
        let editor = registry.register(FocusId(1), dialog, 0, true);
        let action = registry.register(FocusId(2), dialog, 1, true);
        let trigger = registry.register(FocusId(3), root, 0, true);
        window.invoke_focus_standard();
        assert!(window.get_standard_focused());
        registry.push_trap(dialog).unwrap();
        window.set_dialog_state(DialogState {
            open: true,
            topmost: true,
            busy: false,
        });
        let request = registry.request(editor).unwrap();
        window.set_focus_request(FocusRequest {
            present: true,
            token: 1,
            generation: 1,
            request: request.serial as i32,
        });
        slint::platform::update_timers_and_animations();
        assert!(window.get_editor_focused());
        let registry = Rc::new(std::cell::RefCell::new(registry));
        let weak = window.as_weak();
        let registry_for_tab = registry.clone();
        window.on_focus_next(move |reverse| {
            let window = weak.upgrade().unwrap();
            let current = if window.get_editor_focused() {
                Some(editor)
            } else if window.get_dialog_action_focused() {
                Some(action)
            } else {
                None
            };
            let direction = if reverse {
                FocusDirection::Backward
            } else {
                FocusDirection::Forward
            };
            if let Some(request) = registry_for_tab
                .borrow_mut()
                .cycle(dialog, current, direction)
            {
                window.set_focus_request(FocusRequest {
                    present: true,
                    token: request.target.id.0 as i32,
                    generation: request.target.generation as i32,
                    request: request.serial as i32,
                });
            }
        });
        key(&window, Key::Tab);
        assert!(window.get_dialog_action_focused());
        assert!(!window.get_standard_focused());
        key(&window, Key::Tab);
        assert!(window.get_editor_focused());
        key_down(&window, Key::Shift);
        key(&window, Key::Tab);
        key_up(&window, Key::Shift);
        assert!(
            window.get_dialog_action_focused(),
            "Shift+Tab wraps backwards"
        );
        registry.borrow_mut().set_eligible(action, false);
        window.set_dialog_action_eligible(false);
        key(&window, Key::Tab);
        assert!(window.get_editor_focused());
        key(&window, Key::Tab);
        assert!(
            window.get_editor_focused(),
            "disabled custom target is skipped"
        );
        let weak = window.as_weak();
        let registry_for_close = registry.clone();
        window.on_dismiss_dialog(move |_| {
            let window = weak.upgrade().unwrap();
            window.set_dialog_state(DialogState {
                open: false,
                topmost: false,
                busy: false,
            });
            let mut registry = registry_for_close.borrow_mut();
            registry.pop_trap(dialog);
            let restored = registry.restore(&[trigger], root).unwrap();
            assert_eq!(restored.target, trigger);
            window.invoke_focus_standard();
        });
        key(&window, Key::Escape);
        assert!(!window.get_dialog_state().open);
        assert!(window.get_standard_focused());
    }

    #[test]
    fn actual_light_and_dark_theme_tokens_meet_contrast_thresholds() {
        fn luminance(color: slint::Color) -> f64 {
            let linear = |c: u8| {
                let s = f64::from(c) / 255.;
                if s <= 0.04045 {
                    s / 12.92
                } else {
                    ((s + 0.055) / 1.055).powf(2.4)
                }
            };
            linear(color.red()) * 0.2126
                + linear(color.green()) * 0.7152
                + linear(color.blue()) * 0.0722
        }
        fn contrast(a: slint::Color, b: slint::Color) -> f64 {
            let a = luminance(a);
            let b = luminance(b);
            (a.max(b) + 0.05) / (a.min(b) + 0.05)
        }
        let window = fixture();
        for (name, theme) in [
            ("light", window.get_light_theme()),
            ("dark", window.get_dark_theme()),
        ] {
            for (text_name, text) in [
                ("primary", theme.text_primary),
                ("secondary", theme.text_secondary),
            ] {
                for (surface_name, surface) in [
                    ("surface", theme.surface),
                    ("canvas", theme.canvas),
                    ("raised", theme.surface_raised),
                ] {
                    assert!(
                        contrast(text, surface) >= 4.5,
                        "{name} {text_name}/{surface_name}: {}",
                        contrast(text, surface)
                    );
                }
            }
            assert!(
                contrast(theme.on_accent, theme.accent) >= 4.5,
                "{name} accent label"
            );
            assert!(
                contrast(theme.focus.color, theme.surface) >= 3.,
                "{name} focus ring"
            );
            for intent in [
                ButtonIntent::Primary,
                ButtonIntent::Secondary,
                ButtonIntent::Outline,
                ButtonIntent::Quiet,
                ButtonIntent::Destructive,
            ] {
                let recipe = window.invoke_recipe(theme.clone(), intent);
                for (state, fill) in [
                    ("rest", recipe.fill),
                    ("hover", recipe.hover_fill),
                    ("pressed", recipe.pressed_fill),
                ] {
                    // Quiet rest is transparent over the current surface.
                    let fill = if fill.color().alpha() == 0 {
                        theme.surface
                    } else {
                        fill.color()
                    };
                    let ratio = contrast(recipe.foreground, fill);
                    assert!(
                        ratio >= 4.5,
                        "{name} {intent:?} {state} label contrast {ratio:.3}"
                    );
                }
            }
        }
    }
    struct NestedHarness {
        window: NestedFixture,
        binding: kit_slint::SelectBinding<u32>,
        stack: Rc<std::cell::RefCell<kit_core::OverlayStack>>,
        select_dismisses: Rc<Cell<u32>>,
        dialog_dismisses: Rc<Cell<u32>>,
        tab_count: Rc<Cell<u32>>,
    }
    fn nested_key(window: &NestedFixture, key: Key) {
        window
            .window()
            .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
        window
            .window()
            .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    }
    fn close_reason(reason: CloseReason) -> kit_core::DismissReason {
        match reason {
            CloseReason::Escape => kit_core::DismissReason::Escape,
            CloseReason::OutsidePress => kit_core::DismissReason::OutsidePress,
            CloseReason::SystemBack => kit_core::DismissReason::SystemBack,
            CloseReason::TriggerToggle => kit_core::DismissReason::TriggerToggle,
            CloseReason::Programmatic => kit_core::DismissReason::Programmatic,
            CloseReason::HostClosed => kit_core::DismissReason::HostClosed,
        }
    }
    fn nested() -> NestedHarness {
        use kit_core::{
            FocusDirection, FocusId, FocusRegistry, FocusScopeId, OverlayId, OverlaySpec,
            OverlayStack,
        };
        i_slint_backend_testing::init_no_event_loop();
        let window = NestedFixture::new().unwrap();
        window.show().unwrap();
        let binding = kit_slint::bind_select!(
            ui,
            window,
            vec![
                kit_core::SelectOption::new(1u32, "One"),
                kit_core::SelectOption::new(2u32, "Two")
            ],
            None,
            kit_core::Ownership::Local
        );
        let mut registry = FocusRegistry::new();
        let root = FocusScopeId(0);
        let dialog = FocusScopeId(1);
        registry.register_scope(dialog, root).unwrap();
        let background = registry.register(FocusId(1), root, 0, true);
        let editor = registry.register(FocusId(2), dialog, 0, true);
        let select = registry.register(FocusId(3), dialog, 1, true);
        let action = registry.register(FocusId(4), dialog, 2, true);
        registry.push_trap(dialog).unwrap();
        let registry = Rc::new(std::cell::RefCell::new(registry));
        let stack = Rc::new(std::cell::RefCell::new(OverlayStack::new()));
        let mut spec = OverlaySpec::new(OverlayId(1), None, true);
        spec.restore_focus = Some(background);
        stack.borrow_mut().push(spec).unwrap();
        window.set_dialog_state(DialogState {
            open: true,
            topmost: true,
            busy: false,
        });
        window.invoke_focus_select();
        let select_dismisses = Rc::new(Cell::new(0));
        let dialog_dismisses = Rc::new(Cell::new(0));
        let tab_count = Rc::new(Cell::new(0));
        let weak = window.as_weak();
        let target = binding.clone();
        let overlays = stack.clone();
        window.on_select_open(move || {
            let window = weak.upgrade().unwrap();
            let mut spec = OverlaySpec::new(OverlayId(2), Some(OverlayId(1)), false);
            spec.restore_focus = Some(select);
            overlays.borrow_mut().push(spec).unwrap();
            window.set_dialog_state(DialogState {
                open: true,
                topmost: false,
                busy: false,
            });
            target.open();
        });
        let weak = window.as_weak();
        let target = binding.clone();
        let overlays = stack.clone();
        let count = select_dismisses.clone();
        window.on_select_dismiss(move |reason| {
            if !target.snapshot().open {
                return;
            }
            let result = overlays
                .borrow_mut()
                .close(OverlayId(2), close_reason(reason));
            if matches!(result, kit_core::CloseResult::Closed(_)) {
                count.set(count.get() + 1);
                target.dismiss(close_reason(reason));
                weak.upgrade().unwrap().set_dialog_state(DialogState {
                    open: true,
                    topmost: true,
                    busy: false,
                });
            }
        });
        let weak = window.as_weak();
        let focus_registry = registry.clone();
        let count = dialog_dismisses.clone();
        let overlays = stack.clone();
        window.on_dismiss_dialog(move |reason| {
            let result = overlays
                .borrow_mut()
                .close(OverlayId(1), close_reason(reason));
            if matches!(result, kit_core::CloseResult::Closed(_)) {
                count.set(count.get() + 1);
                let window = weak.upgrade().unwrap();
                window.set_dialog_state(DialogState {
                    open: false,
                    topmost: false,
                    busy: false,
                });
                let mut registry = focus_registry.borrow_mut();
                registry.pop_trap(dialog);
                assert_eq!(
                    registry.restore(&[background], root).unwrap().target,
                    background
                );
                window.invoke_focus_background();
            }
        });
        let weak = window.as_weak();
        let focus_registry = registry.clone();
        let count = tab_count.clone();
        let advance = Rc::new(move |reverse: bool, from_select: bool| {
            let window = weak.upgrade().unwrap();
            let current = if from_select || window.get_select_focused() {
                Some(select)
            } else if window.get_editor_focused() {
                Some(editor)
            } else if window.get_action_focused() {
                Some(action)
            } else {
                None
            };
            let direction = if reverse {
                FocusDirection::Backward
            } else {
                FocusDirection::Forward
            };
            let request = focus_registry
                .borrow_mut()
                .cycle(dialog, current, direction)
                .unwrap();
            match request.target.id {
                FocusId(2) => window.invoke_focus_editor(),
                FocusId(3) => window.invoke_focus_select(),
                FocusId(4) => window.invoke_focus_action(),
                other => panic!("unexpected dialog target {other:?}"),
            }
            count.set(count.get() + 1);
        });
        let next = advance.clone();
        window.on_focus_next(move |reverse| next(reverse, false));
        window.on_select_tab(move |reverse| {
            advance(reverse, true);
            TabDisposition::HostHandled
        });
        NestedHarness {
            window,
            binding,
            stack,
            select_dismisses,
            dialog_dismisses,
            tab_count,
        }
    }
    #[test]
    fn nested_escape_closes_select_then_dialog_and_restores_each_real_trigger() {
        let harness = nested();
        let window = &harness.window;
        nested_key(window, Key::DownArrow);
        assert!(window.get_select_state().open);
        assert_eq!(harness.stack.borrow().len(), 2);
        nested_key(window, Key::Escape);
        slint::platform::update_timers_and_animations();
        assert!(!window.get_select_state().open);
        assert!(window.get_dialog_state().open);
        assert!(
            window.get_select_focused(),
            "Escape restores native Select trigger"
        );
        assert_eq!(harness.select_dismisses.get(), 1);
        assert_eq!(harness.dialog_dismisses.get(), 0);
        assert_eq!(harness.binding.selected(), None);
        nested_key(window, Key::Escape);
        slint::platform::update_timers_and_animations();
        assert!(!window.get_dialog_state().open);
        assert!(window.get_background_focused());
        assert_eq!(harness.dialog_dismisses.get(), 1);
        assert!(harness.stack.borrow().is_empty());
    }
    #[test]
    fn nested_android_back_closes_select_then_dialog_and_restores_each_real_trigger() {
        let harness = nested();
        let window = &harness.window;
        nested_key(window, Key::DownArrow);
        nested_key(window, Key::Back);
        slint::platform::update_timers_and_animations();
        assert!(!window.get_select_state().open);
        assert!(window.get_dialog_state().open);
        assert!(window.get_select_focused());
        assert_eq!(harness.select_dismisses.get(), 1);
        assert_eq!(harness.dialog_dismisses.get(), 0);
        assert_eq!(harness.binding.selected(), None);
        nested_key(window, Key::Back);
        slint::platform::update_timers_and_animations();
        assert!(!window.get_dialog_state().open);
        assert!(window.get_background_focused());
        assert_eq!(harness.dialog_dismisses.get(), 1);
        assert!(harness.stack.borrow().is_empty());
    }
    #[test]
    fn nested_select_tab_closes_and_advances_without_delayed_focus_theft() {
        let harness = nested();
        let window = &harness.window;
        nested_key(window, Key::DownArrow);
        assert!(window.get_select_state().open);
        nested_key(window, Key::Tab);
        slint::platform::update_timers_and_animations();
        assert!(!window.get_select_state().open);
        assert!(window.get_dialog_state().open);
        assert!(
            window.get_action_focused(),
            "Tab leaves Select on the next dialog control"
        );
        assert_eq!(harness.tab_count.get(), 1);
        assert_eq!(harness.select_dismisses.get(), 1);
        assert_eq!(harness.dialog_dismisses.get(), 0);
        i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(250));
        slint::platform::update_timers_and_animations();
        assert!(
            window.get_action_focused(),
            "deferred popup close must not steal focus back"
        );
        assert_eq!(
            harness.select_dismisses.get(),
            1,
            "native closure fallback stays idempotent"
        );
    }
}

#[cfg(test)]
mod part2;

#[cfg(test)]
mod form_corrections;

#[cfg(test)]
mod base_data;
mod button_sizing;
mod icons;
mod menu_focus;
mod native_dialog;
mod theme_tokens;

#[cfg(test)]
mod base_feedback;

mod datetime_localization;
mod numeric_input;
mod scroll_bar;
mod select_geometry;

mod html_validation;

mod phone;
