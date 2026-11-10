// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{FieldDirectionFixture, ui};
use i_slint_backend_testing::{
    AccessibleRole, ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions,
};
use slint::{
    Color, ComponentHandle, Rgba8Pixel, SharedPixelBuffer,
    platform::{Key, PointerEventButton, WindowEvent},
};

fn fixture() -> FieldDirectionFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = FieldDirectionFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    app
}
fn by_id(app: &FieldDirectionFixture, id: &str) -> ElementHandle {
    ElementQuery::from_root(app)
        .match_id(format!("FieldDirectionFixture::{id}"))
        .find_first()
        .unwrap_or_else(|| panic!("missing fixture element {id}"))
}
fn named(scope: &ElementHandle, role: AccessibleRole, label: &str) -> ElementHandle {
    let label = label.to_owned();
    scope
        .query_descendants()
        .match_predicate(move |element| {
            element.accessible_role() == Some(role)
                && element.accessible_label().as_deref() == Some(label.as_str())
        })
        .find_first()
        .unwrap_or_else(|| panic!("missing named descendant with role {role:?}"))
}
fn button(app: &FieldDirectionFixture, label: &str) -> ElementHandle {
    let label = label.to_owned();
    ElementQuery::from_root(app)
        .match_predicate(move |element| {
            element.accessible_role() == Some(AccessibleRole::Button)
                && element.accessible_label().as_deref() == Some(label.as_str())
        })
        .find_first()
        .unwrap()
}
fn key(app: &FieldDirectionFixture, value: impl Into<slint::SharedString>) {
    let text = value.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    slint::platform::update_timers_and_animations();
}
fn ordered_parts(app: &FieldDirectionFixture) -> Vec<ElementHandle> {
    let field = by_id(app, "adorned");
    let icons = field
        .query_descendants()
        .match_inherits("KitIcon")
        .find_all();
    assert_eq!(
        icons.len(),
        4,
        "leading/trailing glyphs and two action glyphs"
    );
    vec![
        button(app, "Leading accessory"),
        icons[0].clone(),
        named(&field, AccessibleRole::Text, "PRE"),
        named(&field, AccessibleRole::TextInput, "Decorated edit"),
        icons[1].clone(),
        named(&field, AccessibleRole::Text, "POST"),
        field
            .query_descendants()
            .match_inherits("KitSpinner")
            .find_first()
            .unwrap(),
        button(app, "Clear text"),
        button(app, "Show password"),
        button(app, "Trailing accessory"),
    ]
}
#[test]
fn theme_and_instance_flow_mirror_every_adornment_without_replacing_the_editor() {
    let app = fixture();
    let mut size = None;
    for flow in [ui::ChoiceFlow::LeftToRight, ui::ChoiceFlow::RightToLeft] {
        let mut theme = app.get_theme();
        theme.direction = flow;
        app.set_theme(theme);
        app.invoke_focus_decorated();
        app.invoke_select_decorated(1, 4);
        app.window().take_snapshot().unwrap();
        assert_eq!(app.get_inherited_flow(), flow);
        assert_eq!(app.get_summary_flow(), flow);
        let parts = ordered_parts(&app);
        for pair in parts.windows(2) {
            let (left, right) = match flow {
                ui::ChoiceFlow::LeftToRight => (&pair[0], &pair[1]),
                ui::ChoiceFlow::RightToLeft => (&pair[1], &pair[0]),
            };
            assert!(
                left.absolute_position().x + left.size().width <= right.absolute_position().x + 1.,
                "adornments overlap or fail to mirror"
            );
        }
        let field_size = by_id(&app, "adorned").size();
        if let Some(previous) = size {
            assert_eq!(
                field_size, previous,
                "direction preserves intrinsic geometry"
            );
        }
        size = Some(field_size);
        button(&app, "Leading accessory").mock_single_click(PointerEventButton::Left);
        button(&app, "Trailing accessory").mock_single_click(PointerEventButton::Left);
        slint::platform::update_timers_and_animations();
        assert!(app.get_decorated_focused());
        assert_eq!(
            (app.get_decorated_anchor(), app.get_decorated_cursor()),
            (1, 4)
        );
    }
    assert_eq!(app.get_leading_count(), 2);
    assert_eq!(app.get_trailing_count(), 2);
    app.set_field_flow(ui::ChoiceFlow::LeftToRight);
    app.window().take_snapshot().unwrap();
    assert_eq!(app.get_theme().direction, ui::ChoiceFlow::RightToLeft);
    assert_eq!((app.get_leading_order(), app.get_trailing_order()), (-1, 1));
    let parts = ordered_parts(&app);
    assert!(
        parts.first().unwrap().absolute_position().x < parts.last().unwrap().absolute_position().x
    );
    app.invoke_select_decorated(5, 5);
    key(&app, "Z");
    assert_eq!(app.get_decorated_text(), "12345Z");
    app.set_field_flow(ui::ChoiceFlow::RightToLeft);
    app.invoke_undo_decorated();
    assert_eq!(app.get_decorated_text(), "12345");
}
#[test]
fn rtl_visual_order_keeps_declared_tab_order_and_native_clear_reveal_actions() {
    let app = fixture();
    let mut theme = app.get_theme();
    theme.direction = ui::ChoiceFlow::RightToLeft;
    app.set_theme(theme);
    // Compare the real native focus transition first. Slint collapses an
    // editor's range on ordinary FocusOut; the kit must retain that behavior.
    app.invoke_focus_native_reference();
    app.invoke_select_native_reference(1, 4);
    app.window().take_snapshot().unwrap();
    key(&app, Key::Tab);
    assert!(!app.get_native_reference_focused());
    let native_after_tab = (app.get_native_anchor(), app.get_native_cursor());
    key(&app, Key::Tab);
    key(&app, " ");
    assert_eq!(app.get_native_revealed_count(), 1);
    assert_eq!(app.get_native_secret(), ui::SecretVisibility::Revealed);
    assert!(app.get_native_reference_focused());
    let native_after_reveal = (app.get_native_anchor(), app.get_native_cursor());
    assert_eq!(native_after_tab, (4, 4));
    assert_eq!(native_after_reveal, native_after_tab);
    key(&app, "X");
    assert_eq!(app.get_native_reference_text(), "1234X5");
    app.invoke_undo_native_reference();
    assert_eq!(app.get_native_reference_text(), "12345");
    assert_eq!(
        (app.get_native_anchor(), app.get_native_cursor()),
        native_after_reveal
    );
    app.invoke_focus_decorated();
    app.window().take_snapshot().unwrap();
    key(&app, Key::Tab);
    key(&app, " ");
    assert_eq!(app.get_decorated_text(), "");
    assert_eq!(app.get_cleared_count(), 1);
    assert!(app.get_decorated_focused());
    app.invoke_undo_decorated();
    assert_eq!(app.get_decorated_text(), "12345");
    app.invoke_select_decorated(1, 4);
    key(&app, Key::Tab);
    assert_eq!(
        (app.get_decorated_anchor(), app.get_decorated_cursor()),
        native_after_tab
    );
    key(&app, Key::Tab);
    key(&app, " ");
    assert_eq!(app.get_secret(), ui::SecretVisibility::Revealed);
    assert_eq!(app.get_revealed_count(), 1);
    assert_eq!(
        (app.get_decorated_anchor(), app.get_decorated_cursor()),
        native_after_reveal
    );
    key(&app, "X");
    assert_eq!(app.get_decorated_text(), "1234X5");
    app.invoke_undo_decorated();
    assert_eq!(app.get_decorated_text(), "12345");
    assert_eq!(
        (app.get_decorated_anchor(), app.get_decorated_cursor()),
        native_after_reveal
    );
    key(&app, Key::Tab);
    key(&app, Key::Tab);
    key(&app, Key::Tab);
    assert!(app.get_leading_focused());
    key(&app, Key::Tab);
    assert!(app.get_trailing_focused());
}
#[test]
fn summary_mirrors_its_heading_without_reordering_error_targets() {
    let app = fixture();
    for flow in [ui::ChoiceFlow::LeftToRight, ui::ChoiceFlow::RightToLeft] {
        let mut theme = app.get_theme();
        theme.direction = flow;
        app.set_theme(theme);
        app.window().take_snapshot().unwrap();
        let summary = by_id(&app, "summary");
        let icon = summary
            .query_descendants()
            .match_inherits("KitIcon")
            .find_first()
            .unwrap();
        let title = named(
            &summary,
            AccessibleRole::Text,
            &format!("{} (2)", app.get_help()),
        );
        match flow {
            ui::ChoiceFlow::LeftToRight => {
                assert!(icon.absolute_position().x < title.absolute_position().x)
            }
            ui::ChoiceFlow::RightToLeft => {
                assert!(icon.absolute_position().x > title.absolute_position().x)
            }
        }
        let first = button(&app, app.get_error().as_str());
        let second = button(&app, app.get_second_error().as_str());
        assert!(first.absolute_position().y < second.absolute_position().y);
        first.mock_single_click(PointerEventButton::Left);
        assert_eq!(app.get_summary_token(), 11);
        second.mock_single_click(PointerEventButton::Left);
        assert_eq!(app.get_summary_token(), 29);
    }
}
#[derive(Debug)]
struct Ink {
    left: f64,
    right: f64,
    width: f64,
}
fn ink(
    frame: &SharedPixelBuffer<Rgba8Pixel>,
    element: &ElementHandle,
    color: Color,
) -> Option<Ink> {
    let origin = element.absolute_position();
    let size = element.size();
    // Clamp and round logical geometry explicitly into this 1x software image.
    let left = origin.x.max(0.).floor() as u32;
    let top = origin.y.max(0.).floor() as u32;
    let right = (origin.x + size.width)
        .max(0.)
        .ceil()
        .min(frame.width() as f32) as u32;
    let bottom = (origin.y + size.height)
        .max(0.)
        .ceil()
        .min(frame.height() as f32) as u32;
    let mut found = None::<(u32, u32)>;
    for y in top..bottom {
        for x in left..right {
            let pixel = frame.as_slice()[usize::try_from(y * frame.width() + x).unwrap()];
            if pixel.r.abs_diff(color.red()) <= 24
                && pixel.g.abs_diff(color.green()) <= 24
                && pixel.b.abs_diff(color.blue()) <= 24
            {
                found = Some(match found {
                    None => (x, x),
                    Some((low, high)) => (low.min(x), high.max(x)),
                });
            }
        }
    }
    found.map(|(low, high)| Ink {
        left: f64::from(low) - f64::from(origin.x),
        right: f64::from(high) - f64::from(origin.x),
        width: f64::from(size.width),
    })
}
#[test]
fn rtl_text_defaults_and_native_alignment_references_render_in_both_scripts_and_scales() {
    let app = fixture();
    let mut failures = Vec::new();
    for (theme_name, base) in [("light", app.get_theme()), ("dark", app.get_dark())] {
        for scale in [1., 2.] {
            for (script, text_script) in [
                ("arabic", crate::FieldTextScript::Arabic),
                ("hebrew", crate::FieldTextScript::Hebrew),
            ] {
                let mut theme = base.clone();
                theme.text_scale = scale;
                theme.direction = ui::ChoiceFlow::RightToLeft;
                app.set_theme(theme.clone());
                app.set_script(text_script);
                app.invoke_blur_fields();
                slint::platform::update_timers_and_animations();
                let frame = app.window().take_snapshot().unwrap();
                crate::choice_presentation::save_capture(
                    &frame,
                    &format!("field-rtl-{script}-{theme_name}-{scale}"),
                );
                assert!(app.get_default_start());
                assert!(app.get_otp_left());
                let mut native_right_gap = None;
                for (kind, prefix) in [("Text", "native"), ("TextInput", "native-input")] {
                    let start = ink(
                        &frame,
                        &by_id(&app, &format!("{prefix}-start")),
                        theme.text_primary,
                    );
                    let left = ink(
                        &frame,
                        &by_id(&app, &format!("{prefix}-left")),
                        theme.text_primary,
                    );
                    let right = ink(
                        &frame,
                        &by_id(&app, &format!("{prefix}-right")),
                        theme.text_primary,
                    );
                    println!(
                        "native RTL reference {kind}/{script}/{theme_name}/{scale}: Start={start:?} Left={left:?} Right={right:?}"
                    );
                    if start.is_none() {
                        failures.push(format!(
                            "{kind}/{script}/{theme_name}/{scale} native Start did not paint"
                        ));
                    }
                    if kind == "Text" {
                        native_right_gap = right.as_ref().map(|bounds| bounds.width - bounds.right);
                    }
                    match (left, right) {
                    (Some(left), Some(right)) if right.left > left.left + right.width * 0.3 => {},
                    observed => failures.push(format!("{kind}/{script}/{theme_name}/{scale} explicit left/right reference failed: {observed:?}")),
                }
                }
                let adorned = by_id(&app, "adorned");
                let checks = [
                    (
                        "label",
                        named(
                            &adorned,
                            AccessibleRole::Text,
                            app.get_field_label().as_str(),
                        ),
                        theme.text_primary,
                    ),
                    (
                        "error",
                        named(&adorned, AccessibleRole::Text, app.get_error().as_str()),
                        theme.danger,
                    ),
                    (
                        "help",
                        named(&adorned, AccessibleRole::Text, app.get_help().as_str()),
                        theme.text_secondary,
                    ),
                    (
                        "editor",
                        named(
                            &by_id(&app, "value-field"),
                            AccessibleRole::TextInput,
                            "RTL value",
                        ),
                        theme.text_primary,
                    ),
                    (
                        "placeholder",
                        named(
                            &by_id(&app, "placeholder-field"),
                            AccessibleRole::Text,
                            app.get_word().as_str(),
                        ),
                        theme.placeholder,
                    ),
                    (
                        "formatted",
                        named(
                            &by_id(&app, "formatted-field"),
                            AccessibleRole::Text,
                            app.get_word().as_str(),
                        ),
                        theme.text_primary,
                    ),
                    (
                        "summary",
                        named(
                            &by_id(&app, "summary"),
                            AccessibleRole::Text,
                            app.get_error().as_str(),
                        ),
                        theme.danger,
                    ),
                ];
                // Glyph side bearings vary with both text and weight. Calibrate the
                // medium-weight label against the identical native Right label.
                let native_label_right_gap = ink(
                    &frame,
                    &by_id(&app, "native-label-right"),
                    theme.text_primary,
                )
                .map(|bounds| bounds.width - bounds.right);
                for (name, element, color) in checks {
                    let reference_gap = if name == "label" {
                        native_label_right_gap
                    } else {
                        native_right_gap
                    };
                    match (ink(&frame, &element, color), reference_gap) {
                    (Some(bounds), Some(reference_gap)) if bounds.left > bounds.width * 0.5 && bounds.width - bounds.right <= reference_gap + 3. * f64::from(scale) => {},
                    observed => failures.push(format!("RTL {name}/{script}/{theme_name}/{scale} must paint at the native physical Right edge: {observed:?}")),
                }
                }
                let otp = named(&by_id(&app, "otp"), AccessibleRole::TextInput, "OTP");
                match ink(&frame, &otp, theme.text_primary) {
                    Some(bounds) if bounds.right < bounds.width * 0.6 => {}
                    observed => failures.push(format!(
                        "OTP must retain its explicit left alignment: {observed:?}"
                    )),
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
