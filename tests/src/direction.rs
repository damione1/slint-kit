// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{ComponentHandle, platform::PointerEventButton};

#[test]
fn logical_chrome_mirrors_without_changing_intrinsic_sizes_or_activation() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = DirectionFixture::new().unwrap();
    window.show().unwrap();
    for (index, base) in [
        window.get_theme(),
        window.global::<ui::KitThemes>().get_dark(),
    ]
    .into_iter()
    .enumerate()
    {
        for scale in [1., 2.] {
            let mut reference = None;
            let mut start_bubble_x = None;
            for (name, direction) in [
                ("ltr", ui::ChoiceFlow::LeftToRight),
                ("rtl", ui::ChoiceFlow::RightToLeft),
            ] {
                let mut theme = base.clone();
                theme.text_scale = scale;
                theme.direction = direction;
                window.set_theme(theme);
                slint::platform::update_timers_and_animations();
                let button = ElementQuery::from_root(&window)
                    .match_id("DirectionFixture::button")
                    .find_first()
                    .unwrap();
                let icons = button
                    .query_descendants()
                    .match_inherits("KitIcon")
                    .find_all();
                assert_eq!(icons.len(), 2);
                assert_eq!(
                    icons[0].absolute_position().x < icons[1].absolute_position().x,
                    direction == ui::ChoiceFlow::LeftToRight,
                    "leading and trailing remain logical"
                );
                let badge = ElementQuery::from_root(&window)
                    .match_id("DirectionFixture::badge")
                    .find_first()
                    .unwrap();
                let badge_icon = badge
                    .query_descendants()
                    .match_inherits("KitIcon")
                    .find_first()
                    .unwrap();
                let badge_label = badge
                    .query_descendants()
                    .match_id("KitBadge::label")
                    .find_first()
                    .unwrap();
                assert_eq!(
                    badge_icon.absolute_position().x < badge_label.absolute_position().x,
                    direction == ui::ChoiceFlow::LeftToRight
                );
                let bar = ElementQuery::from_root(&window)
                    .match_id("ScrollFrame::vertical-bar")
                    .find_first()
                    .unwrap();
                let pan = ElementQuery::from_root(&window)
                    .match_id("ScrollFrame::pan")
                    .find_first()
                    .unwrap();
                if direction == ui::ChoiceFlow::LeftToRight {
                    assert!(
                        bar.absolute_position().x
                            >= pan.absolute_position().x + pan.size().width - 1.
                    );
                    reference = Some((button.size(), badge.size(), pan.size()));
                } else {
                    assert!(
                        bar.absolute_position().x + bar.size().width
                            <= pan.absolute_position().x + 1.
                    );
                    assert_eq!(
                        reference,
                        Some((button.size(), badge.size(), pan.size())),
                        "direction preserves intrinsic geometry and stable gutter"
                    );
                }
                let bubble = ElementQuery::from_root(&window)
                    .match_id("KitChatBubble::bubble")
                    .find_first()
                    .unwrap();
                assert!(
                    button.absolute_position().y + button.size().height
                        <= badge.absolute_position().y + 1.,
                    "the intrinsic button stays above the badge at enlarged text"
                );
                assert!(
                    badge.absolute_position().y + badge.size().height
                        <= bubble.absolute_position().y + 1.,
                    "the intrinsic badge stays above the message"
                );
                let scroll = ElementQuery::from_root(&window)
                    .match_id("DirectionFixture::scroll")
                    .find_first()
                    .unwrap();
                assert!(
                    bubble.absolute_position().y + bubble.size().height
                        <= scroll.absolute_position().y + 1.,
                    "the content-height message stays above the scroll viewport"
                );
                if direction == ui::ChoiceFlow::LeftToRight {
                    start_bubble_x = Some(bubble.absolute_position().x);
                } else {
                    assert!(
                        bubble.absolute_position().x > start_bubble_x.unwrap(),
                        "ChatSide.start moves to the logical right"
                    );
                }
                let back = i_slint_backend_testing::ElementHandle::find_by_accessible_label(
                    &window, "Back",
                )
                .next()
                .unwrap();
                let continue_action =
                    i_slint_backend_testing::ElementHandle::find_by_accessible_label(
                        &window,
                        "Continue action",
                    )
                    .next()
                    .unwrap();
                assert_eq!(
                    back.absolute_position().x < continue_action.absolute_position().x,
                    direction == ui::ChoiceFlow::LeftToRight,
                    "dialog footer actions follow logical direction"
                );
                let before = window.get_actions();
                button.mock_single_click(PointerEventButton::Left);
                assert_eq!(window.get_actions(), before + 1);
                if let Some(directory) = std::env::var_os("KIT_FOUNDATION_RENDER_DIR") {
                    use std::io::Write;
                    let snapshot = window.window().take_snapshot().unwrap();
                    std::fs::create_dir_all(&directory).unwrap();
                    let path = std::path::PathBuf::from(directory)
                        .join(format!("direction-{name}-{index}-{scale}.ppm"));
                    let mut file = std::fs::File::create(path).unwrap();
                    write!(
                        file,
                        "P6\n{} {}\n255\n",
                        snapshot.width(),
                        snapshot.height()
                    )
                    .unwrap();
                    for pixel in snapshot.as_slice() {
                        file.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
                    }
                }
            }
        }
    }
}
