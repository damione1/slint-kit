// SPDX-License-Identifier: GPL-3.0-or-later
//! Prototype P4 of issue 446: a hand copy of `KitAlert` owned by a consumer
//! crate that imports `@kit` only.
#![cfg(test)]
mod ui {
    pub use slint_kit::ui::*;
    slint::include_modules!();
}
use slint::ComponentHandle;
use ui::*;

fn frame(app: &App) -> Vec<u8> {
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap().as_bytes().to_vec()
}

fn differing(a: &[u8], b: &[u8]) -> usize {
    a.chunks(4).zip(b.chunks(4)).filter(|(a, b)| a != b).count()
}

// One test owns the platform: the testing backend is set once per process.
#[test]
fn the_copy_draws_the_same_frames_as_the_kit_alert() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = App::new().unwrap();
    app.show().unwrap();
    let blank = {
        app.set_heading("".into());
        app.set_message("".into());
        let blank = frame(&app);
        app.set_heading("Storage is almost full".into());
        app.set_message("Remove old recordings to keep saving new ones.".into());
        blank
    };
    for severity in [
        AlertSeverity::Information,
        AlertSeverity::Success,
        AlertSeverity::Warning,
        AlertSeverity::Error,
    ] {
        for placement in [
            AlertActions::Below,
            AlertActions::Trailing,
            AlertActions::Inline,
        ] {
            app.set_severity(severity);
            app.set_placement(placement);
            app.set_local(false);
            let kit = frame(&app);
            app.set_local(true);
            let local = frame(&app);
            // Guards against comparing two frames that draw no copy at all.
            assert!(differing(&kit, &blank) > 500, "{severity:?} {placement:?}");
            assert_eq!(differing(&kit, &local), 0, "{severity:?} {placement:?}");
        }
    }
}
