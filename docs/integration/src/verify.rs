// SPDX-License-Identifier: GPL-3.0-or-later
//! Native interaction verification for the documentation's runnable consumer.
use crate::ui::App;
use slint::{
    ComponentHandle,
    platform::{Key, PointerEventButton, WindowEvent},
};
use std::{path::Path, time::Duration};

fn key(app: &App, text: impl Into<slint::SharedString>) {
    let text = text.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
}

fn click(app: &App, x: f32, y: f32) {
    let position = slint::LogicalPosition::new(x, y);
    app.window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    app.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    app.window().dispatch_event(WindowEvent::PointerReleased {
        position,
        button: PointerEventButton::Left,
    });
}

fn snapshot(app: &App, name: &str) {
    let name = if app.get_language() == crate::ui::ExampleLanguage::French {
        name.replace(".png", "-fr.png")
    } else {
        name.into()
    };
    let image = app.window().take_snapshot().expect("native snapshot");
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../src/assets/components")
        .join(name);
    let mut encoder = png::Encoder::new(
        std::io::BufWriter::new(std::fs::File::create(path).expect("snapshot file")),
        image.width(),
        image.height(),
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .expect("PNG header")
        .write_image_data(image.as_bytes())
        .expect("PNG image");
}

pub fn schedule(app: &App) {
    let weak = app.as_weak();
    slint::Timer::single_shot(Duration::from_millis(400), move || {
        let app = weak.upgrade().expect("live example window");
        snapshot(&app, "typed-integration.png");
        app.invoke_focus_name();
        // Native key input edits the real TextInput; the binding validates its draft.
        key(&app, " II");
        let valid_draft = app.get_draft();
        assert!(valid_draft.contains("Atlas") && valid_draft.contains(" II"));
        assert_eq!(app.get_accepted_label(), "Atlas");
        key(&app, Key::Return);
        assert_eq!(app.get_accepted_label(), valid_draft);
        // Open through the actual trigger pointer route, navigate through its
        // keyboard handler, then commit a stable WorkspaceKey through bind_select!.
        click(&app, 100., 230.);
        assert!(
            app.get_select_state().open,
            "trigger opens controlled presentation"
        );
        key(&app, Key::DownArrow);
        key(&app, Key::Return);
        assert!(!app.get_select_state().open);
        assert_eq!(
            app.get_selected_label(),
            if app.get_language() == crate::ui::ExampleLanguage::French {
                "Partagé"
            } else {
                "Shared"
            }
        );
        // Invalid commit preserves the last accepted domain value.
        app.invoke_focus_name();
        key(&app, "a".repeat(45));
        key(&app, Key::Return);
        assert!(!app.get_error().is_empty());
        assert_eq!(app.get_accepted_label(), valid_draft);
        snapshot(&app, "typed-integration-validated.png");
        println!(
            "Native typed example: editor draft/commit, invalid value preservation, pointer-open selection, keyboard navigation and stable keyed commit PASS"
        );
        slint::quit_event_loop().expect("close native verification");
    });
}
