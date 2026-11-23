// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
use slint::{ComponentHandle, Rgba8Pixel, SharedPixelBuffer};

fn software_window() -> DisplayVariantsFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = DisplayVariantsFixture::new().unwrap();
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    window
}

fn pixel(buffer: &SharedPixelBuffer<Rgba8Pixel>, x: f32, y: f32) -> [u8; 3] {
    let p = buffer.as_slice()[y as usize * buffer.width() as usize + x as usize];
    [p.r, p.g, p.b]
}

fn channels(color: slint::Color) -> [u8; 3] {
    [color.red(), color.green(), color.blue()]
}

#[test]
fn avatar_takes_a_small_step_and_an_explicit_tint() {
    let window = software_window();
    assert_eq!(window.get_tile_width(), 32.);
    let buffer = window.window().take_snapshot().unwrap();
    let fill = channels(window.get_custom_fill());
    assert_eq!(
        pixel(&buffer, window.get_tile_x() + 4., window.get_tile_y() + 16.),
        fill
    );
    let ink = channels(window.get_custom_ink());
    let inked = (0..32).any(|dx| {
        (0..32).any(|dy| {
            pixel(
                &buffer,
                window.get_tile_x() + dx as f32,
                window.get_tile_y() + dy as f32,
            ) == ink
        })
    });
    assert!(inked, "the glyph is drawn in the explicit ink");
}

#[test]
fn keycap_has_a_thicker_bottom_rim_than_its_top() {
    let window = software_window();
    let buffer = window.window().take_snapshot().unwrap();
    let x = window.get_key_x() + 6.;
    let top = window.get_key_y();
    let bottom = top + window.get_key_height();
    let rim = channels(window.get_border_strong());
    let face = channels(window.get_raised());
    assert_eq!(pixel(&buffer, x, top), rim);
    assert_eq!(pixel(&buffer, x, top + 1.), face);
    assert_eq!(pixel(&buffer, x, bottom - 1.), rim);
    assert_eq!(pixel(&buffer, x, bottom - 2.), rim);
    assert_eq!(pixel(&buffer, x, bottom - 3.), face);
}

#[test]
fn card_and_surface_wear_the_border_of_their_tone() {
    let window = software_window();
    let edge = |window: &DisplayVariantsFixture| {
        let buffer = window.window().take_snapshot().unwrap();
        [window.get_card_y(), window.get_surface_y()].map(|y| pixel(&buffer, 8., y + 20.))
    };
    assert_eq!(edge(&window), [channels(window.get_card_border()); 2]);
    window.set_card_tone(Tone::Warning);
    slint::platform::update_timers_and_animations();
    assert_eq!(edge(&window), [channels(window.get_warning()); 2]);
    window.set_card_tone(Tone::Danger);
    slint::platform::update_timers_and_animations();
    assert_eq!(edge(&window), [channels(window.get_danger()); 2]);
}

#[test]
fn soft_destructive_button_is_a_flat_danger_tint() {
    let window = software_window();
    let buffer = window.window().take_snapshot().unwrap();
    let y = window.get_soft_y() + window.get_soft_height() / 2.;
    assert_eq!(pixel(&buffer, 20., y), channels(window.get_danger_subtle()));
}

fn empty_ink(window: &DisplayVariantsFixture, from: f32) -> Option<usize> {
    let buffer = window.window().take_snapshot().unwrap();
    let canvas = channels(window.get_canvas());
    let top = window.get_empty_y() as usize;
    let bottom = top + window.get_empty_height() as usize;
    (from as usize..buffer.width() as usize)
        .find(|&x| (top..bottom).any(|y| pixel(&buffer, x as f32, y as f32) != canvas))
}

#[test]
fn empty_state_draws_an_icon_tile_and_centres_its_content_on_request() {
    let window = software_window();
    window.window().take_snapshot().unwrap();
    let bare = window.get_empty_height();
    let left = empty_ink(&window, 0.).expect("title is painted");
    window.set_centered(true);
    slint::platform::update_timers_and_animations();
    let centred = empty_ink(&window, 0.).expect("title is painted");
    assert!(centred > left + 20, "{left} -> {centred}");

    window.set_with_icon(true);
    slint::platform::update_timers_and_animations();
    window.window().take_snapshot().unwrap();
    let tall = window.get_empty_height();
    assert!(tall >= bare + 64., "{bare} -> {tall}");
    let buffer = window.window().take_snapshot().unwrap();
    let tile_left = 8. + (304. - 64.) / 2.;
    let y = window.get_empty_y() + 24. + 4.;
    assert_eq!(
        pixel(&buffer, tile_left + 4., y),
        channels(window.get_inset())
    );
    window.set_centered(false);
    slint::platform::update_timers_and_animations();
    let buffer = window.window().take_snapshot().unwrap();
    assert_eq!(
        pixel(&buffer, 8. + 24. + 4., y),
        channels(window.get_inset())
    );
}

#[test]
fn overline_is_uppercase_tracked_and_keeps_its_source_label() {
    let window = software_window();
    assert_eq!(window.get_overline_text(), "CALENDARS");
    assert_eq!(window.get_overline_label(), "Calendars");
    assert_eq!(window.get_overline_font_size(), 12.);
    let extra = window.get_overline_width() - window.get_plain_width();
    assert!(
        (extra - 9. * 12. * 0.08).abs() < 1.,
        "tracking adds {extra}"
    );
}
