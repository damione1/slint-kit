// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
use slint::ComponentHandle;

#[test]
fn stat_renders_every_tone_in_both_themes_and_text_scales() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = StatFixture::new().unwrap();
    window.show().unwrap();
    for (mode, base) in [
        ("light", window.get_theme()),
        ("dark", window.get_dark_theme()),
    ] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            window.set_theme(theme.clone());
            for (tone, expected) in [
                (Tone::Neutral, theme.text_primary),
                (Tone::Accent, theme.accent),
                (Tone::Success, theme.success),
                (Tone::Warning, theme.warning),
                (Tone::Danger, theme.danger),
                (Tone::Information, theme.information),
            ] {
                window.set_tone(tone);
                slint::platform::update_timers_and_animations();
                let snapshot = window.window().take_snapshot().unwrap();
                assert!(
                    snapshot.as_slice().iter().any(|p| p.r == expected.red()
                        && p.g == expected.green()
                        && p.b == expected.blue()),
                    "{mode} {scale} {tone:?}"
                );
                if let Some(directory) = std::env::var_os("KIT_FOUNDATION_RENDER_DIR") {
                    use std::io::Write;
                    std::fs::create_dir_all(&directory).unwrap();
                    let path = std::path::PathBuf::from(directory)
                        .join(format!("stat-{mode}-{scale}-{tone:?}.ppm"));
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
