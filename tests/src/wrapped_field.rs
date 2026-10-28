use super::*;
use i_slint_backend_testing::ElementQuery;
use slint::{ComponentHandle, platform::PointerEventButton};

#[test]
fn wrapped_field_constraints_are_finite_and_the_label_still_focuses_the_editor() {
    i_slint_backend_testing::init_no_event_loop();
    let window = WrappedFieldFixture::new().unwrap();
    for scale in [1., 2.] {
        let mut theme = window.get_theme();
        theme.text_scale = scale;
        window.set_theme(theme);
        for width in [360., 760.] {
            window.set_viewport_width(width);
            window.show().unwrap();
            slint::platform::update_timers_and_animations();
            let label = ElementQuery::from_root(&window)
                .match_inherits("KitFieldLabel")
                .find_first()
                .expect("field label");
            let size = label.size();
            assert!(size.width.is_finite() && size.height.is_finite());
            assert!(size.width > 0. && size.width <= width);
            assert!(size.height > 0.);
            ElementQuery::from_root(&window)
                .match_inherits("KitButton")
                .find_first()
                .expect("create button")
                .mock_single_click(PointerEventButton::Left);
            assert!(!window.get_field_focused());
            label.mock_single_click(PointerEventButton::Left);
            assert!(window.get_field_focused());
            window.set_draft("Exact opaque draft · é".into());
            assert_eq!(window.get_draft(), "Exact opaque draft · é");
        }
    }
}
