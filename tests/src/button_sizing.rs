// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::ElementQuery;
use slint::ComponentHandle;

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
        assert!(button.size().height >= 48.);
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
