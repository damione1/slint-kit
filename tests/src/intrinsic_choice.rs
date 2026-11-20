// SPDX-License-Identifier: GPL-3.0-or-later
use crate::IntrinsicChoiceFixture;
use slint::ComponentHandle;
use slint_kit::ui::ChoiceLabelPosition;

#[test]
fn intrinsic_switch_and_checkbox_fit_their_caption_at_every_text_scale() {
    i_slint_backend_testing::init_no_event_loop();
    let window = IntrinsicChoiceFixture::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    for caption in ["Hide", "Hide this microphone"] {
        window.set_caption(caption.into());
        for scale in [1., 2.] {
            let mut theme = window.get_theme();
            theme.text_scale = scale;
            window.set_theme(theme);
            for side in [ChoiceLabelPosition::End, ChoiceLabelPosition::Start] {
                window.set_side(side);
                slint::platform::update_timers_and_animations();
                let padding = 2. * window.get_switch_padding();
                let label = window.get_label_natural();
                let gap = window.get_label_gap();
                let switch = padding + window.get_track_width() + gap + label;
                let checkbox = padding + window.get_box_size() + gap + label;
                assert!(
                    window.get_switch_width() + 0.5 >= switch,
                    "switch keeps `{caption}` on one line at scale {scale}: {} < {switch}",
                    window.get_switch_width()
                );
                assert!(
                    window.get_checkbox_width() + 0.5 >= checkbox,
                    "checkbox keeps `{caption}` on one line at scale {scale}: {} < {checkbox}",
                    window.get_checkbox_width()
                );
            }
        }
    }
}

#[test]
fn intrinsic_switch_without_a_label_reserves_only_its_track() {
    i_slint_backend_testing::init_no_event_loop();
    let window = IntrinsicChoiceFixture::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    for scale in [1., 2.] {
        let mut theme = window.get_theme();
        theme.text_scale = scale;
        window.set_theme(theme);
        slint::platform::update_timers_and_animations();
        let track = 2. * window.get_switch_padding() + window.get_track_width();
        assert!(
            (window.get_empty_switch_width() - track).abs() < 0.5,
            "an unlabeled switch is its track plus padding at scale {scale}: {} vs {track}",
            window.get_empty_switch_width()
        );
    }
}
