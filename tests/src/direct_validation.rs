// SPDX-License-Identifier: GPL-3.0-or-later
use crate::ZonedReportFocusFixture;
use slint::ComponentHandle;

#[test]
fn composed_focus_projection_tracks_native_take_focus_and_typing() {
    i_slint_backend_testing::init_no_event_loop();
    let app = ZonedReportFocusFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    app.invoke_focus_sentinel();
    assert!(app.get_sentinel_focus());
    assert!(!app.get_zoned_focus());
    app.invoke_take_focus_zoned();
    assert!(app.get_zoned_focus());
    assert!(!app.get_sentinel_focus());
    app.window()
        .dispatch_event(slint::platform::WindowEvent::KeyPressed { text: "2".into() });
    app.window()
        .dispatch_event(slint::platform::WindowEvent::KeyReleased { text: "2".into() });
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_date_text(), "2");
}
