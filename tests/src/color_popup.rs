// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{ColorPopupFixture, ui};
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle, LogicalPosition,
    platform::{Key, PointerEventButton, WindowEvent},
};
use std::cell::OnceCell;
thread_local! { static BACKEND: OnceCell<()> = const { OnceCell::new() }; }
fn fixture() -> ColorPopupFixture {
    BACKEND.with(|backend| {
        backend.get_or_init(|| {
            slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
                mock_time: true,
                renderer_name: Some("software".into()),
                ..Default::default()
            })))
            .unwrap();
        });
    });
    let app = ColorPopupFixture::new().unwrap();
    slint_kit::install(&app);
    app
}
fn settle(app: &ColorPopupFixture) -> slint::SharedPixelBuffer<slint::Rgba8Pixel> {
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap()
}
fn key(app: &ColorPopupFixture, value: Key) {
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: value.into() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text: value.into() });
    settle(app);
}
fn id(app: &ColorPopupFixture, id: &str) -> ElementHandle {
    ElementQuery::from_root(app)
        .match_id(id)
        .find_first()
        .unwrap()
}
fn button(app: &ColorPopupFixture, label: &str) -> ElementHandle {
    let label = label.to_owned();
    ElementQuery::from_root(app)
        .match_inherits("KitButton")
        .match_predicate(move |e| e.accessible_label().as_deref() == Some(label.as_str()))
        .find_first()
        .unwrap()
}
// ElementHandle positions inside ChildWindow omit its offset (SDK1.18.1).
// Map the actual popup-local child geometry into the tested host rectangle.
fn popup_origin(app: &ColorPopupFixture) -> LogicalPosition {
    let field = ElementQuery::from_root(app)
        .match_inherits("KitColorField")
        .find_first()
        .unwrap();
    let anchor = field.absolute_position();
    let size = id(app, "KitColorField::popup-keys").size();
    let viewport = app.window().size().to_logical(app.window().scale_factor());
    let gap = app.get_theme().spacing.unit;
    let below = viewport.height - gap - anchor.y - field.size().height - gap;
    LogicalPosition::new(
        anchor.x.clamp(gap, viewport.width - gap - size.width),
        if below + 0.5 >= size.height {
            anchor.y + field.size().height + gap
        } else {
            anchor.y - gap - size.height
        },
    )
}
fn click(app: &ColorPopupFixture, element: &ElementHandle) {
    let offset = popup_origin(app);
    let p = element.absolute_position();
    let s = element.size();
    let position = LogicalPosition::new(
        offset.x + p.x + s.width / 2.,
        offset.y + p.y + s.height / 2.,
    );
    for event in [
        WindowEvent::PointerMoved { position },
        WindowEvent::PointerPressed {
            position,
            button: PointerEventButton::Left,
        },
        WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        },
    ] {
        app.window().dispatch_event(event);
    }
    settle(app);
}
fn wheel(app: &ColorPopupFixture, amount: f32) {
    let origin = popup_origin(app);
    let viewport = id(app, "ScrollFrame::flickable");
    let p = viewport.absolute_position();
    let s = viewport.size();
    app.window().dispatch_event(WindowEvent::PointerScrolled {
        position: LogicalPosition::new(
            origin.x + p.x + s.width / 2.,
            origin.y + p.y + s.height / 2.,
        ),
        delta_x: 0.,
        delta_y: amount,
    });
    settle(app);
}
fn assert_action_visible(app: &ColorPopupFixture, label: &str) {
    let viewport = id(app, "ScrollFrame::flickable");
    let vp = viewport.absolute_position();
    let vs = viewport.size();
    let item = button(app, label);
    let p = item.absolute_position();
    let s = item.size();
    assert!(s.width > 0. && s.height > 0.);
    assert!(
        p.x >= vp.x - 0.5 && p.x + s.width <= vp.x + vs.width + 0.5,
        "{label} inline bounds: {p:?}/{s:?} in {vp:?}/{vs:?}"
    );
    assert!(
        p.y >= vp.y - 0.5 && p.y + s.height <= vp.y + vs.height + 0.5,
        "{label} vertical bounds: {p:?}/{s:?} in {vp:?}/{vs:?}"
    );
}
#[test]
fn edge_popup_geometry_and_scroll_keep_actions_reachable_in_both_themes_and_text_scales() {
    for (mode, dark) in [("light", false), ("dark", true)] {
        for scale in [1., 2.] {
            for bottom in [false, true] {
                let app = fixture();
                app.set_bottom(bottom);
                app.set_narrow(bottom);
                let mut theme = if dark {
                    app.get_dark_theme()
                } else {
                    app.get_theme()
                };
                theme.text_scale = scale;
                app.set_theme(theme);
                app.show().unwrap();
                settle(&app);
                app.invoke_open_picker();
                let frame = settle(&app);
                assert!(app.get_open());
                let popup = id(&app, "KitColorField::popup-keys").size();
                let origin = popup_origin(&app);
                let viewport = app.window().size().to_logical(app.window().scale_factor());
                assert!(popup.width > 0. && popup.height > 0.);
                assert!(
                    origin.x >= 0.
                        && origin.y >= 0.
                        && origin.x + popup.width <= viewport.width + 0.5
                        && origin.y + popup.height <= viewport.height + 0.5,
                    "{mode}/{scale}/{bottom}: {origin:?}/{popup:?} in {viewport:?}"
                );
                if bottom {
                    let anchor = ElementQuery::from_root(&app)
                        .match_inherits("KitColorField")
                        .find_first()
                        .unwrap()
                        .absolute_position();
                    assert!(origin.y + popup.height < anchor.y);
                }
                // The surface must actually paint at the computed host origin;
                // a popup-local geometry getter alone cannot establish that.
                let width = usize::try_from(frame.width()).unwrap();
                let pixel = frame
                    .as_slice()
                    .iter()
                    .enumerate()
                    .find_map(|(index, pixel)| {
                        let x = f32::from(u16::try_from(index % width).unwrap());
                        let y = f32::from(u16::try_from(index / width).unwrap());
                        ((x - origin.x - popup.width / 2.).abs() <= 0.5
                            && (y - origin.y - 3.).abs() <= 0.5)
                            .then_some(pixel)
                    })
                    .expect("sample within the actual viewport");
                let fill = app.get_theme().surface_raised.to_argb_u8();
                assert_eq!(
                    [pixel.r, pixel.g, pixel.b],
                    [fill.red, fill.green, fill.blue],
                    "surface at the actual popup origin"
                );
                key(&app, Key::Tab);
                let apply = button(&app, "Apply color");
                wheel(&app, 10000.);
                let before = apply.absolute_position().y;
                wheel(&app, -10000.);
                if scale == 2. {
                    assert!(
                        apply.absolute_position().y < before - 1.,
                        "200% requires and performs scrolling"
                    );
                }
                assert_action_visible(&app, "Cancel");
                assert_action_visible(&app, "Apply color");
                let frame = settle(&app);
                if let Some(directory) = std::env::var_os("KIT_COLOR_POPUP_RENDER_DIR") {
                    std::fs::create_dir_all(&directory).unwrap();
                    let mut bytes =
                        format!("P6\n{} {}\n255\n", frame.width(), frame.height()).into_bytes();
                    for pixel in frame.as_slice() {
                        bytes.extend([pixel.r, pixel.g, pixel.b]);
                    }
                    std::fs::write(
                        std::path::PathBuf::from(directory)
                            .join(format!("color-popup-{mode}-{scale}-{bottom}.ppm")),
                        bytes,
                    )
                    .unwrap();
                }
                click(&app, &button(&app, "Apply color"));
                assert!(!app.get_open());
                assert_eq!(app.get_proposals(), 1);
                assert_eq!(app.get_value().color.red, 12);
                app.hide().unwrap();
            }
        }
    }
}
#[test]
fn scrolled_keyboard_cancel_and_apply_preserve_controlled_absence_until_host_acknowledgment() {
    let app = fixture();
    app.set_narrow(true);
    app.set_bottom(true);
    let mut theme = app.get_theme();
    theme.text_scale = 2.;
    app.set_theme(theme);
    app.set_ownership(ui::Ownership::Controlled);
    app.set_value(ui::ColorValue::default());
    app.show().unwrap();
    settle(&app);
    app.invoke_open_picker();
    settle(&app);
    key(&app, Key::Escape);
    assert!(!app.get_open());
    assert!(!app.get_value().present);
    assert_eq!(app.get_proposals(), 0);
    app.invoke_open_picker();
    settle(&app);
    // The popup starts on its real Cancel button. Native Tab enters Apply and
    // Flickable reveals the focused action, without synthetic focus getters.
    key(&app, Key::Tab);
    assert_action_visible(&app, "Apply color");
    key(&app, Key::Return);
    assert!(!app.get_open());
    assert_eq!(app.get_proposals(), 1);
    assert!(!app.get_value().present);
    assert_eq!(app.get_proposal().alpha, 255);
    app.set_value(ui::ColorValue {
        present: true,
        color: app.get_proposal(),
    });
    settle(&app);
    assert!(app.get_value().present);
}
