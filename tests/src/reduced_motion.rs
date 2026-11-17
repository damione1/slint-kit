// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{ProgressMode, ReducedMotionFixture};
use i_slint_backend_testing::{
    ElementHandle, ElementQuery, MotionPreference, TestingBackend, TestingBackendOptions,
};
use slint::{ComponentHandle, LogicalPosition, platform::WindowEvent};
use std::time::Duration;

fn fixture() -> ReducedMotionFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = ReducedMotionFixture::new().unwrap();
    window.show().unwrap();
    window
}

fn advance(ms: u64) {
    i_slint_backend_testing::mock_elapsed_time(Duration::from_millis(ms));
    slint::platform::update_timers_and_animations();
}

fn component(window: &ReducedMotionFixture, name: &str) -> ElementHandle {
    ElementQuery::from_root(window)
        .match_inherits(name)
        .find_first()
        .unwrap()
}

fn rectangles(element: &ElementHandle) -> Vec<ElementHandle> {
    element
        .query_descendants()
        .match_type_name("Rectangle")
        .find_all()
}

fn near(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.1, "{actual} != {expected}");
}

#[test]
fn transitions_obey_application_and_system_preferences_and_custom_durations() {
    let window = fixture();
    for base in [window.get_light(), window.get_dark()] {
        for scale in [1., 2.] {
            for application in [false, true] {
                for system in [false, true] {
                    let mut theme = base.clone();
                    theme.text_scale = scale;
                    theme.reduced_motion = application;
                    window.set_theme(theme.clone());
                    i_slint_backend_testing::set_motion_preference(if system {
                        MotionPreference::Reduced
                    } else {
                        MotionPreference::NoPreference
                    });
                    window.set_progress(0.2);
                    window.window().dispatch_event(WindowEvent::PointerExited);
                    window.window().take_snapshot().unwrap();
                    advance(1000);
                    window.window().take_snapshot().unwrap();
                    let progress = rectangles(&component(&window, "KitProgress"));
                    let (track, fill) = (&progress[0], &progress[1]);
                    near(fill.size().width, track.size().width * 0.2);
                    window.set_progress(0.8);
                    if application || system {
                        near(fill.size().width, track.size().width * 0.8);
                    } else {
                        near(fill.size().width, track.size().width * 0.2);
                        advance(100);
                        assert!(fill.size().width > track.size().width * 0.2);
                        assert!(fill.size().width < track.size().width * 0.8);
                    }
                    advance(1000);
                    near(fill.size().width, track.size().width * 0.8);

                    let bar = component(&window, "KitScrollBar");
                    let thumb = rectangles(&bar).remove(0);
                    let rest = thumb.size().width;
                    let point = bar.absolute_position();
                    window.window().dispatch_event(WindowEvent::PointerMoved {
                        position: LogicalPosition::new(
                            point.x + bar.size().width / 2.,
                            point.y + 10.,
                        ),
                    });
                    let target = theme.geometry.scroll_active_thumb;
                    assert!(target > rest);
                    if application || system {
                        near(thumb.size().width, target);
                    } else {
                        near(thumb.size().width, rest);
                        advance(450);
                        assert!(
                            thumb.size().width > rest && thumb.size().width < target,
                            "the 900ms consumer duration survives the shorter theme duration"
                        );
                    }
                    advance(1000);
                    near(thumb.size().width, target);
                }
            }
        }
    }
}

#[test]
fn visual_timers_and_ticks_need_the_application_flag_even_with_system_reduction() {
    let window = fixture();
    window.set_mode(ProgressMode::Indeterminate);
    for (name, base) in [("light", window.get_light()), ("dark", window.get_dark())] {
        for scale in [1., 2.] {
            for application in [false, true] {
                for system in [false, true] {
                    let mut theme = base.clone();
                    theme.text_scale = scale;
                    theme.reduced_motion = application;
                    window.set_theme(theme);
                    i_slint_backend_testing::set_motion_preference(if system {
                        MotionPreference::Reduced
                    } else {
                        MotionPreference::NoPreference
                    });
                    advance(2000);
                    let before = window.window().take_snapshot().unwrap();
                    let dots = rectangles(&component(&window, "KitSpinner"));
                    let opacity: Vec<_> =
                        dots.iter().map(ElementHandle::computed_opacity).collect();
                    let skeleton = component(&window, "KitSkeleton");
                    let pulse = skeleton.computed_opacity();
                    let progress = rectangles(&component(&window, "KitProgress"));
                    let x = progress[1].absolute_position().x;
                    advance(700);
                    let after = window.window().take_snapshot().unwrap();
                    let next: Vec<_> = dots.iter().map(ElementHandle::computed_opacity).collect();
                    if application {
                        assert_eq!(opacity, vec![1.; 8]);
                        assert_eq!(next, opacity);
                        assert_eq!(pulse, 1.);
                        assert_eq!(skeleton.computed_opacity(), 1.);
                        near(progress[1].absolute_position().x, x);
                        assert_eq!(before.as_slice(), after.as_slice());
                    } else {
                        assert_ne!(next, opacity);
                        assert!((progress[1].absolute_position().x - x).abs() > 1.);
                        assert_ne!(before.as_slice(), after.as_slice());
                        // The system skips the opacity interpolation, but not the pulse timer.
                        if system {
                            assert_ne!(skeleton.computed_opacity(), pulse);
                        }
                    }
                    if let Some(directory) = std::env::var_os("KIT_MOTION_RENDER_DIR") {
                        use std::io::Write;
                        std::fs::create_dir_all(&directory).unwrap();
                        let path = std::path::PathBuf::from(directory).join(format!(
                            "motion-{name}-{scale}-app-{application}-system-{system}.ppm"
                        ));
                        let mut file = std::fs::File::create(path).unwrap();
                        write!(file, "P6\n{} {}\n255\n", after.width(), after.height()).unwrap();
                        for pixel in after.as_slice() {
                            file.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn skeleton_pulses_normally_and_application_reduction_restores_a_solid_shape() {
    let window = fixture();
    let skeleton = component(&window, "KitSkeleton");
    assert_eq!(skeleton.computed_opacity(), 1.);
    advance(650);
    let _ = skeleton.computed_opacity();
    advance(400);
    near(skeleton.computed_opacity(), 0.5);
    let mut theme = window.get_theme();
    theme.reduced_motion = true;
    window.set_theme(theme);
    // The new opacity target is immediate, without waiting for a timer.
    near(skeleton.computed_opacity(), 1.);
    advance(1300);
    near(skeleton.computed_opacity(), 1.);
}
