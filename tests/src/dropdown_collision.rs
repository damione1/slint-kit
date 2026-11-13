// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{DropdownCollisionFixture, DropdownProbeKind, ui};
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    Color, ComponentHandle, LogicalPosition, Model, ModelRc,
    platform::{Key, PointerEventButton, WindowEvent},
};
use slint_kit::{Choices, SelectOption};
use std::cell::OnceCell;
thread_local! { static BACKEND: OnceCell<()> = const { OnceCell::new() }; }
const COUNT: i32 = 1000;
const MARK: [u8; 3] = [197, 19, 173];
fn kinds() -> [DropdownProbeKind; 3] {
    [
        DropdownProbeKind::Select,
        DropdownProbeKind::Combobox,
        DropdownProbeKind::MultiSelect,
    ]
}
fn fixture(kind: DropdownProbeKind) -> DropdownCollisionFixture {
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
    let app = DropdownCollisionFixture::new().unwrap();
    slint_kit::install(&app);
    app.set_kind(kind);
    let choices = Choices::from_options(
        (0..COUNT)
            .map(|i| SelectOption::new(i, format!("Option {i:04}")))
            .collect(),
    )
    .unwrap();
    app.set_options(choices.model());
    app
}
fn settle(app: &DropdownCollisionFixture) -> slint::SharedPixelBuffer<slint::Rgba8Pixel> {
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap()
}
fn key(app: &DropdownCollisionFixture, text: impl Into<slint::SharedString>) {
    let text = text.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    settle(app);
}
fn open(app: &DropdownCollisionFixture) {
    app.invoke_take_focus();
    key(app, Key::DownArrow);
    assert!(app.get_state().open);
}
fn surface(app: &DropdownCollisionFixture) -> ElementHandle {
    ElementQuery::from_root(app)
        .match_inherits("KitSelectSurface")
        .find_first()
        .expect("presented popup")
}
fn viewport(app: &DropdownCollisionFixture) -> ElementHandle {
    surface(app)
        .query_descendants()
        .match_id("ScrollFrame::flickable")
        .find_first()
        .unwrap()
}
fn anchor(app: &DropdownCollisionFixture) -> ElementHandle {
    let name = match app.get_kind() {
        DropdownProbeKind::Select | DropdownProbeKind::MultiSelect => "KitSelect",
        DropdownProbeKind::Combobox => "KitCombobox",
    };
    ElementQuery::from_root(app)
        .match_inherits(name)
        .find_first()
        .unwrap()
}
// Measure the painted native-window origin. Public ElementHandle positions are
// popup-local for ChildWindow, so they cannot establish collisions by themselves.
fn painted_origin(app: &DropdownCollisionFixture) -> LogicalPosition {
    let natural = app.get_theme();
    let mut marked = natural.clone();
    marked.surface_raised = Color::from_rgb_u8(MARK[0], MARK[1], MARK[2]);
    marked.elevation.popover = ui::ElevationLevel::None;
    app.set_theme(marked);
    let frame = settle(app);
    let width = usize::try_from(frame.width()).unwrap();
    let mut left = u16::MAX;
    let mut top = u16::MAX;
    for (index, p) in frame.as_slice().iter().enumerate() {
        if [p.r, p.g, p.b] == MARK {
            left = left.min(u16::try_from(index % width).unwrap());
            top = top.min(u16::try_from(index / width).unwrap());
        }
    }
    assert_ne!(left, u16::MAX, "the popup surface must actually paint");
    let result = LogicalPosition::new(
        f32::from(left) / app.window().scale_factor() - natural.shape.border_width,
        f32::from(top) / app.window().scale_factor() - natural.shape.border_width,
    );
    app.set_theme(natural);
    settle(app);
    result
}
fn click(app: &DropdownCollisionFixture, position: LogicalPosition) {
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
fn active_row(app: &DropdownCollisionFixture) -> ElementHandle {
    let label = app
        .get_options()
        .row_data(usize::try_from(app.get_state().active_index).unwrap())
        .unwrap()
        .label;
    surface(app)
        .query_descendants()
        .match_inherits("KitOption")
        .match_predicate(move |e| e.accessible_label().as_deref() == Some(label.as_str()))
        .find_first()
        .expect("active row is materialized and visible")
}
fn assert_active_visible(app: &DropdownCollisionFixture) {
    let row = active_row(app);
    let p = row.absolute_position();
    let s = row.size();
    let v = viewport(app);
    let vp = v.absolute_position();
    let vs = v.size();
    assert!(
        p.y >= vp.y - 1. && p.y + s.height <= vp.y + vs.height + 1.,
        "active row {p:?}/{s:?} in {vp:?}/{vs:?}"
    );
    assert!(
        surface(app)
            .query_descendants()
            .match_inherits("KitOption")
            .find_all()
            .len()
            < 100,
        "long lists remain virtualized"
    );
}
fn capture(app: &DropdownCollisionFixture, name: &str) {
    let Some(dir) = std::env::var_os("KIT_DROPDOWN_RENDER_DIR") else {
        return;
    };
    let frame = settle(app);
    std::fs::create_dir_all(&dir).unwrap();
    let mut bytes = format!("P6\n{} {}\n255\n", frame.width(), frame.height()).into_bytes();
    for p in frame.as_slice() {
        bytes.extend([p.r, p.g, p.b]);
    }
    std::fs::write(
        std::path::PathBuf::from(dir).join(format!("{name}.ppm")),
        bytes,
    )
    .unwrap();
}
#[test]
fn popup_bounds_and_long_list_reveal_are_measured_on_all_three_controls_at_both_text_scales() {
    let mut measurements = Vec::new();
    let mut failures = Vec::new();
    for kind in kinds() {
        for (mode, dark) in [("light", false), ("dark", true)] {
            for scale in [2., 1.] {
                for bottom in [false, true] {
                    let app = fixture(kind);
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
                    open(&app);
                    let origin = painted_origin(&app);
                    let size = surface(&app).size();
                    let host = app.window().size().to_logical(app.window().scale_factor());
                    let a = anchor(&app);
                    let ap = a.absolute_position();
                    let az = a.size();
                    let bounded = origin.x >= -1.
                        && origin.y >= -1.
                        && origin.x + size.width <= host.width + 1.
                        && origin.y + size.height <= host.height + 1.;
                    let above = origin.y + size.height <= ap.y + 1.;
                    let below = origin.y >= ap.y + az.height - 1.;
                    let separated = above || below;
                    measurements.push(format!(r#"{{"kind":"{kind:?}","mode":"{mode}","scale":{scale},"bottom":{bottom},"window_width":{},"window_height":{},"popup_x":{},"popup_y":{},"popup_width":{},"popup_height":{},"anchor_y":{},"anchor_height":{},"bounded":{bounded},"separated":{separated}}}"#,host.width,host.height,origin.x,origin.y,size.width,size.height,ap.y,az.height));
                    if let Some(dir) = std::env::var_os("KIT_DROPDOWN_RENDER_DIR") {
                        std::fs::create_dir_all(&dir).unwrap();
                        std::fs::write(
                            std::path::PathBuf::from(dir).join("measurements.json"),
                            format!("[\n{}\n]\n", measurements.join(",\n")),
                        )
                        .unwrap();
                    }
                    if !bounded || !separated {
                        failures.push(format!("{kind:?}/{mode}/{scale}/{bottom}: origin{origin:?}, size{size:?}, host{host:?}, anchor{ap:?}/{az:?}"));
                    }
                    key(&app, Key::End);
                    assert_eq!(app.get_state().active_index, COUNT - 1);
                    assert_active_visible(&app);
                    let page = app.get_page_rows();
                    assert!(page > 0);
                    key(&app, Key::PageUp);
                    assert_eq!(app.get_state().active_index, COUNT - 1 - page);
                    assert_active_visible(&app);
                    key(&app, Key::End);
                    assert_active_visible(&app);
                    capture(&app, &format!("dropdown-{kind:?}-{mode}-{scale}-{bottom}"));
                    // Both native Escape and the restored trigger/editor must still work.
                    key(&app, Key::Escape);
                    assert!(!app.get_state().open);
                    key(&app, Key::DownArrow);
                    assert!(app.get_state().open);
                    key(&app, Key::Escape);
                    app.hide().unwrap();
                }
            }
        }
    }
    if let Some(dir) = std::env::var_os("KIT_DROPDOWN_RENDER_DIR") {
        std::fs::write(
            std::path::PathBuf::from(dir).join("measurements.json"),
            format!("[\n{}\n]\n", measurements.join(",\n")),
        )
        .unwrap();
    }
    assert!(
        failures.is_empty(),
        "popup collisions measured through actual painting and public geometry:\n{}",
        failures.join("\n")
    );
}
#[test]
fn edge_popups_keep_controlled_admission_multi_sessions_and_native_tab_exit() {
    for kind in kinds() {
        let app = fixture(kind);
        app.set_narrow(true);
        app.set_bottom(true);
        app.set_ownership(ui::Ownership::Controlled);
        let mut theme = app.get_theme();
        theme.text_scale = 2.;
        app.set_theme(theme);
        app.show().unwrap();
        settle(&app);
        open(&app);
        key(&app, Key::End);
        assert_active_visible(&app);
        let expected = app
            .get_options()
            .row_data(usize::try_from(COUNT - 1).unwrap())
            .unwrap();
        let origin = painted_origin(&app);
        let row = active_row(&app);
        let p = row.absolute_position();
        let size = row.size();
        click(
            &app,
            LogicalPosition::new(
                origin.x + p.x + size.width / 2.,
                origin.y + p.y + size.height / 2.,
            ),
        );
        assert_eq!(app.get_proposals(), 1);
        assert_eq!(app.get_last_proposal().token, expected.token);
        assert!(!app.get_single_value().present);
        assert_eq!(app.get_multiple_value().row_count(), 0);
        match kind {
            DropdownProbeKind::Select | DropdownProbeKind::Combobox => {
                assert!(!app.get_state().open);
                app.set_single_value(ui::SelectionRef {
                    present: true,
                    token: expected.token,
                });
                settle(&app);
                assert_eq!(app.get_state().selected.token, expected.token);
                key(&app, Key::DownArrow);
            }
            DropdownProbeKind::MultiSelect => {
                assert!(app.get_state().open);
                app.set_multiple_value(ModelRc::new(slint::VecModel::from(vec![
                    ui::SelectionRef {
                        present: true,
                        token: expected.token,
                    },
                ])));
                settle(&app);
                assert_eq!(app.get_multiple_value().row_count(), 1);
            }
        }
        assert!(app.get_state().open);
        if kind != DropdownProbeKind::Combobox {
            key(&app, Key::Tab);
            assert!(
                app.get_state().open,
                "the list's first Tab enters its native search editor"
            );
        }
        key(&app, Key::Tab);
        assert!(!app.get_state().open);
        assert!(
            app.get_outside_focused(),
            "{kind:?}: native Tab must leave focus in the host editor"
        );
        key(&app, "Z");
        assert_eq!(app.get_outside_text(), "Z");
        open(&app);
        click(&app, LogicalPosition::new(1., 1.));
        assert!(!app.get_state().open);
        app.hide().unwrap();
    }
}
fn presented(app: &DropdownCollisionFixture) -> (LogicalPosition, slint::LogicalSize) {
    settle(app);
    open(app);
    (painted_origin(app), surface(app).size())
}
#[test]
fn placement_pins_the_side_and_popup_width_overrides_the_default_on_all_three_controls() {
    for kind in kinds() {
        // The anchor sits near the top edge: below has far more room than above.
        for (placement, side_above) in [
            (ui::DropdownPlacement::Auto, false),
            (ui::DropdownPlacement::Below, false),
            (ui::DropdownPlacement::Above, true),
        ] {
            let app = fixture(kind);
            app.set_placement(placement);
            app.show().unwrap();
            let (origin, size) = presented(&app);
            let a = anchor(&app);
            let (ap, az) = (a.absolute_position(), a.size());
            if side_above {
                assert!(
                    origin.y + size.height <= ap.y + 1.,
                    "{kind:?} above: {origin:?} {size:?} anchor {ap:?}"
                );
                assert!(origin.y >= -1., "{kind:?} above stays in the window");
            } else {
                assert!(
                    origin.y >= ap.y + az.height - 1.,
                    "{kind:?} {placement:?}: {origin:?} anchor {ap:?}"
                );
            }
            app.hide().unwrap();
        }
        for (narrow, wide) in [(false, false), (false, true), (true, true)] {
            let app = fixture(kind);
            app.set_narrow(narrow);
            app.set_wide(wide);
            app.show().unwrap();
            let (origin, size) = presented(&app);
            let host = app.window().size().to_logical(app.window().scale_factor());
            if wide && !narrow {
                assert!(
                    (size.width - 300.).abs() < 1.,
                    "{kind:?} popup-width: {size:?}"
                );
            }
            if narrow {
                assert!(
                    origin.x >= -1. && origin.x + size.width <= host.width + 1.,
                    "{kind:?} wide popup is clamped to the window: {origin:?} {size:?} {host:?}"
                );
            }
            if !wide {
                assert!(size.width < 300., "{kind:?} default width: {size:?}");
            }
            app.hide().unwrap();
        }
    }
}
