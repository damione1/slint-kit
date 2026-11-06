// SPDX-License-Identifier: GPL-3.0-or-later
use crate::SliderPaintFixture;
use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
use slint::{Brush, Color, ComponentHandle};

fn pixel(image: &slint::SharedPixelBuffer<slint::Rgba8Pixel>, x: usize, y: usize) -> [u8; 3] {
    let value = image.as_slice()[y * image.width() as usize + x];
    [value.r, value.g, value.b]
}

fn near(actual: [u8; 3], expected: [u8; 3], context: &str) {
    assert!(
        actual.iter().zip(expected).all(|(a, b)| a.abs_diff(b) <= 3),
        "{context}: {actual:?} != {expected:?}"
    );
}

fn over(foreground: Color, background: Color) -> [u8; 3] {
    let alpha = u32::from(foreground.alpha());
    [foreground.red(), foreground.green(), foreground.blue()]
        .into_iter()
        .zip([background.red(), background.green(), background.blue()])
        .map(|(a, b)| {
            u8::try_from((u32::from(a) * alpha + u32::from(b) * (255 - alpha) + 127) / 255).unwrap()
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

#[test]
fn filled_slider_software_paint_preserves_round_bounds_and_brush_coordinates() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = SliderPaintFixture::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    for (theme_name, base) in [("light", window.get_light()), ("dark", window.get_dark())] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            window.set_theme(theme.clone());
            let opaque = theme.accent;
            let translucent = Color::from_argb_u8(128, opaque.red(), opaque.green(), opaque.blue());
            let transparent = Color::from_argb_u8(0, opaque.red(), opaque.green(), opaque.blue());
            for (name, paint, solid) in [
                ("opaque", Brush::SolidColor(opaque), Some(opaque)),
                (
                    "translucent",
                    Brush::SolidColor(translucent),
                    Some(translucent),
                ),
                (
                    "transparent",
                    Brush::SolidColor(transparent),
                    Some(transparent),
                ),
                ("gradient", window.get_gradient(), None),
                // Equal stops are still a gradient: its box/branch must not
                // silently change because conversion to color looks equal.
                ("flat-gradient", window.get_flat_gradient(), None),
            ] {
                window.set_paint(paint);
                window.set_fraction(0.);
                let empty = window.window().take_snapshot().unwrap();
                for (percent, fraction) in [(0, 0.), (1, 0.01), (50, 0.5), (100, 1.)] {
                    window.set_fraction(fraction);
                    let image = window.window().take_snapshot().unwrap();
                    let context = format!("{theme_name}-{scale}-{name}-{percent}");
                    if let Some(color) = solid {
                        // No value may paint outside the full rounded track,
                        // including a one-percent sliver and alpha-only fills.
                        for (x, y, width, height) in [(40, 40, 100, 240), (200, 40, 240, 100)] {
                            for (dx, dy) in [
                                (1, 1),
                                (width - 2, 1),
                                (1, height - 2),
                                (width - 2, height - 2),
                            ] {
                                near(
                                    pixel(&image, x + dx, y + dy),
                                    pixel(&empty, x + dx, y + dy),
                                    &context,
                                );
                            }
                        }
                        let expected = over(
                            if percent == 0 { transparent } else { color },
                            theme.pressed,
                        );
                        near(pixel(&image, 90, 278), expected, &context);
                        near(pixel(&image, 201, 90), expected, &context);
                    }
                    // Gradient colors stay normalized to the partial box.
                    // The reference excludes rounded corners, whose clipping
                    // is an upstream Software-renderer limitation for brushes.
                    if solid.is_none() && percent >= 50 {
                        for ratio in [0.25, 0.75] {
                            let x = 200 + (240. * fraction * ratio) as usize;
                            near(pixel(&image, x, 90), pixel(&image, x, 230), &context);
                            let y =
                                40 + (240. * (1. - fraction) + 240. * fraction * ratio) as usize;
                            near(pixel(&image, 90, y), pixel(&image, 530, y), &context);
                        }
                    }
                    crate::choice_presentation::save_capture(
                        &image,
                        &format!("slider-paint-{context}"),
                    );
                }
            }
        }
    }
}
