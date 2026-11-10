// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
use slint::ComponentHandle;

fn save_snapshot(window: &impl ComponentHandle, name: &str) {
    let snapshot = window.window().take_snapshot().unwrap();
    if let Some(directory) = std::env::var_os("KIT_FOUNDATION_RENDER_DIR") {
        use std::io::Write;
        std::fs::create_dir_all(&directory).unwrap();
        let mut file =
            std::fs::File::create(std::path::PathBuf::from(directory).join(format!("{name}.ppm")))
                .unwrap();
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
#[test]
fn browse_drop_zone_states_render_in_both_themes_and_enlarged_text() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = DropZoneFixture::new().unwrap();
    window.show().unwrap();
    for (index, base) in [
        window.get_theme(),
        window.global::<ui::KitThemes>().get_dark(),
    ]
    .into_iter()
    .enumerate()
    {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            window.set_theme(theme);
            for (state, access) in [
                ("editable", FieldAccess::Editable),
                ("read-only", FieldAccess::ReadOnly),
                ("disabled", FieldAccess::Disabled),
            ] {
                window.set_access(access);
                slint::platform::update_timers_and_animations();
                if access != FieldAccess::Editable {
                    assert!(
                        !window.get_focused(),
                        "inactive drop zone has no focus halo"
                    );
                }
                save_snapshot(&window, &format!("drop-zone-{state}-{index}-{scale}"));
                if access == FieldAccess::Editable {
                    use slint::{
                        LogicalPosition,
                        platform::{Key, PointerEventButton, WindowEvent},
                    };
                    let zone = i_slint_backend_testing::ElementHandle::find_by_accessible_label(
                        &window,
                        "Choose or drop documents",
                    )
                    .next()
                    .unwrap();
                    let origin = zone.absolute_position();
                    let size = zone.size();
                    let inside = LogicalPosition::new(
                        origin.x + size.width / 2.,
                        origin.y + size.height / 2.,
                    );
                    let before = window.window().take_snapshot().unwrap();
                    window
                        .window()
                        .dispatch_event(WindowEvent::PointerMoved { position: inside });
                    slint::platform::update_timers_and_animations();
                    let hover = window.window().take_snapshot().unwrap();
                    assert_ne!(
                        before.as_slice(),
                        hover.as_slice(),
                        "browse hover has visible feedback"
                    );
                    save_snapshot(&window, &format!("drop-zone-hover-{index}-{scale}"));
                    window.window().dispatch_event(WindowEvent::PointerPressed {
                        position: inside,
                        button: PointerEventButton::Left,
                    });
                    slint::platform::update_timers_and_animations();
                    let pressed = window.window().take_snapshot().unwrap();
                    assert_ne!(
                        hover.as_slice(),
                        pressed.as_slice(),
                        "browse press has visible feedback"
                    );
                    save_snapshot(&window, &format!("drop-zone-pressed-{index}-{scale}"));
                    window
                        .window()
                        .dispatch_event(WindowEvent::PointerReleased {
                            position: inside,
                            button: PointerEventButton::Left,
                        });
                    window.window().dispatch_event(WindowEvent::PointerMoved {
                        position: LogicalPosition::new(410., 230.),
                    });
                    window.invoke_focus_zone();
                    slint::platform::update_timers_and_animations();
                    window.window().dispatch_event(WindowEvent::KeyPressed {
                        text: Key::Return.into(),
                    });
                    window.window().dispatch_event(WindowEvent::KeyReleased {
                        text: Key::Return.into(),
                    });
                    slint::platform::update_timers_and_animations();
                    assert!(window.get_focused());
                    save_snapshot(&window, &format!("drop-zone-focus-{index}-{scale}"));
                }
            }
        }
    }
}
