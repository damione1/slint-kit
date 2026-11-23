// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
use slint::{ComponentHandle, Rgba8Pixel, SharedPixelBuffer};

fn software_window() -> DisplayTonesFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = DisplayTonesFixture::new().unwrap();
    window.show().unwrap();
    window
}

fn pixel(buffer: &SharedPixelBuffer<Rgba8Pixel>, x: usize, y: usize) -> Rgba8Pixel {
    buffer.as_slice()[y * buffer.width() as usize + x]
}

fn channels(color: slint::Color) -> [u8; 3] {
    [color.red(), color.green(), color.blue()]
}

fn leftmost_ink(window: &DisplayTonesFixture) -> usize {
    let buffer = window.window().take_snapshot().unwrap();
    let background = channels(window.get_canvas());
    let top = window.get_link_y().ceil() as usize;
    let bottom = (window.get_link_y() + window.get_link_height()).floor() as usize;
    (0..buffer.width() as usize)
        .find(|&x| {
            (top..bottom).any(|y| {
                let p = pixel(&buffer, x, y);
                [p.r, p.g, p.b] != background
            })
        })
        .expect("link text is painted")
}

#[test]
fn badge_shrinks_to_its_parent_and_keeps_its_intrinsic_width_otherwise() {
    i_slint_backend_testing::init_no_event_loop();
    let window = DisplayTonesFixture::new().unwrap();
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    assert!((window.get_short_width() - window.get_short_natural()).abs() < 0.5);
    assert!(
        window.get_long_width() <= 224.5,
        "{}",
        window.get_long_width()
    );
    assert!(window.get_long_width() > window.get_short_width());
}

#[test]
fn badge_dot_takes_a_leading_slot() {
    i_slint_backend_testing::init_no_event_loop();
    let window = DisplayTonesFixture::new().unwrap();
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    let plain = window.get_plain_width();
    assert!((window.get_short_width() - plain).abs() < 0.5);
    window.set_dot(true);
    slint::platform::update_timers_and_animations();
    assert!(window.get_short_width() > plain + 4.);
}

#[test]
fn subtle_badge_is_tinted_without_a_border() {
    let window = software_window();
    let edge = |window: &DisplayTonesFixture| {
        let buffer = window.window().take_snapshot().unwrap();
        let y = (window.get_short_y() + window.get_short_height() / 2.) as usize;
        let p = pixel(&buffer, 8, y);
        [p.r, p.g, p.b]
    };
    let subtle = channels(window.get_accent_subtle());
    assert_ne!(
        edge(&window),
        subtle,
        "solid accent badge keeps its accent edge"
    );
    window.set_subtle(true);
    slint::platform::update_timers_and_animations();
    assert_eq!(edge(&window), subtle);
}

fn image_nodes(window: &DisplayTonesFixture) -> usize {
    i_slint_backend_testing::ElementHandle::find_by_element_type_name(window, "KitAvatar")
        .flat_map(|avatar| {
            avatar
                .query_descendants()
                .match_accessible_role(i_slint_backend_testing::AccessibleRole::Image)
                .find_all()
        })
        .count()
}

#[test]
fn flat_muted_avatar_is_an_unbordered_tile_and_can_be_decorative() {
    let window = software_window();
    slint::platform::update_timers_and_animations();
    assert_eq!(image_nodes(&window), 1);
    let buffer = window.window().take_snapshot().unwrap();
    let y = (window.get_tile_y() + 16.) as usize;
    let p = pixel(&buffer, 8, y);
    assert_eq!([p.r, p.g, p.b], channels(window.get_inset()));
    let decorated = DisplayTonesFixture::new().unwrap();
    decorated.set_decorative(true);
    decorated.show().unwrap();
    slint::platform::update_timers_and_animations();
    assert_eq!(image_nodes(&decorated), 0);
}

fn bar_height(caption: &str) -> f32 {
    let window = DisplayTonesFixture::new().unwrap();
    window.set_caption(caption.into());
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    window.get_bar_height()
}

#[test]
fn progress_caption_adds_a_line_below_the_bar() {
    i_slint_backend_testing::init_no_event_loop();
    let bare = bar_height("");
    let captioned = bar_height("Downloading 12 of 40 MB");
    assert!(captioned > bare + 8., "{bare} -> {captioned}");
}

#[test]
fn link_button_text_aligns_to_the_start_when_asked() {
    let window = software_window();
    slint::platform::update_timers_and_animations();
    let centered = leftmost_ink(&window);
    window.set_start_aligned(true);
    slint::platform::update_timers_and_animations();
    let start = leftmost_ink(&window);
    assert!(
        start <= 10,
        "start-aligned text sits at the padding edge: {start}"
    );
    assert!(
        centered > start + 20,
        "centered text is inset: {centered} vs {start}"
    );
}
