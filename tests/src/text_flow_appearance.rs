// SPDX-License-Identifier: GPL-3.0-or-later
use crate::TextFlowAppearanceFixture;
use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle, LogicalPosition, Rgba8Pixel, SharedPixelBuffer, platform::WindowEvent,
};

fn fixture() -> TextFlowAppearanceFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = TextFlowAppearanceFixture::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    for _ in 0..3 {
        slint::platform::update_timers_and_animations();
        window.window().take_snapshot().unwrap();
    }
    window
}

fn distance(pixel: Rgba8Pixel, color: slint::Color) -> i32 {
    (pixel.r as i32 - color.red() as i32).abs()
        + (pixel.g as i32 - color.green() as i32).abs()
        + (pixel.b as i32 - color.blue() as i32).abs()
}

fn nearest(buffer: &SharedPixelBuffer<Rgba8Pixel>, row: usize, color: slint::Color) -> i32 {
    let top = 8 + row * 50;
    (top..top + 30)
        .flat_map(|y| (8..200).map(move |x| (x, y)))
        .map(|(x, y)| distance(buffer.as_slice()[y * buffer.width() as usize + x], color))
        .min()
        .unwrap()
}

fn paints_as(
    window: &TextFlowAppearanceFixture,
    row: usize,
    expected: slint::Color,
    other: slint::Color,
) {
    let buffer = window.window().take_snapshot().unwrap();
    let near = nearest(&buffer, row, expected);
    assert!(near < nearest(&buffer, row, other), "row {row}: {near}");
    assert!(near < 40, "row {row}: {near}");
}

#[test]
fn activatable_words_are_links_unless_the_flow_is_plain() {
    let window = fixture();
    paints_as(&window, 0, window.get_accent(), window.get_primary());
    paints_as(&window, 1, window.get_primary(), window.get_accent());
}

#[test]
fn a_plain_word_shows_the_accent_while_hovered() {
    let window = fixture();
    window.window().dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(20., 70.),
    });
    for _ in 0..3 {
        slint::platform::update_timers_and_animations();
    }
    paints_as(&window, 1, window.get_accent_hover(), window.get_primary());
}

#[test]
fn muted_fragments_use_the_secondary_ink_and_literals_the_primary() {
    let window = fixture();
    paints_as(&window, 2, window.get_secondary(), window.get_primary());
    paints_as(&window, 3, window.get_primary(), window.get_secondary());
}

#[test]
fn role_sets_the_paragraph_typography() {
    let window = fixture();
    assert!(window.get_title_height() > window.get_body_height());
}
