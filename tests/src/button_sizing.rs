// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::ElementQuery;
use slint::ComponentHandle;

#[test]
fn native_tab_reveals_the_entire_button_focus_outline_at_either_scroll_edge() {
    use slint::platform::{Key, WindowEvent};
    i_slint_backend_testing::init_no_event_loop();
    let window = ButtonScrollFixture::new().unwrap();
    window.show().unwrap();
    for scale in [1., 2.] {
        let mut theme = window.get_theme();
        theme.text_scale = scale;
        window.set_theme(theme);
        for from_below in [false, true] {
            window.invoke_focus_outside();
            if from_below {
                window.invoke_reset_below();
            } else {
                window.invoke_reset_above();
            }
            window.window().dispatch_event(WindowEvent::KeyPressed {
                text: Key::Tab.into(),
            });
            window.window().dispatch_event(WindowEvent::KeyReleased {
                text: Key::Tab.into(),
            });
            assert!(window.get_target_focused());
            assert!(
                window.get_outline_top() >= window.get_viewport_top() - 0.1,
                "top outline clipped at scale {scale}, from_below={from_below}"
            );
            assert!(
                window.get_outline_bottom() <= window.get_viewport_bottom() + 0.1,
                "bottom outline clipped at scale {scale}, from_below={from_below}"
            );
        }
    }
}

#[test]
fn filled_button_accepts_its_bounded_viewport_with_a_long_label_and_double_text() {
    i_slint_backend_testing::init_no_event_loop();
    let window = ButtonSizingFixture::new().unwrap();
    window.show().unwrap();
    for scale in [1., 2.] {
        let mut theme = window.get_theme();
        theme.text_scale = scale;
        window.set_theme(theme);
        let button = ElementQuery::from_root(&window)
            .match_inherits("KitButton")
            .find_first()
            .unwrap();
        assert!(
            (button.size().width - 160.).abs() < 1.,
            "fill width must follow its 160px viewport, scale {scale}: {:?}",
            button.size()
        );
        assert!(button.size().height >= window.get_theme().spacing.comfortable.height);
        let label = button
            .query_descendants()
            .match_type_name("Text")
            .find_first()
            .expect("rendered button label");
        assert!(label.absolute_position().x >= button.absolute_position().x);
        assert!(
            label.absolute_position().x + label.size().width
                <= button.absolute_position().x + button.size().width,
            "the visible label must fit too, scale {scale}"
        );
    }
    window.set_mode(ControlWidth::Intrinsic);
    let button = ElementQuery::from_root(&window)
        .match_inherits("KitButton")
        .find_first()
        .unwrap();
    assert!(
        button.size().width > 160.,
        "intrinsic mode retains the natural label width"
    );
}
