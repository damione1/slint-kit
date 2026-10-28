// SPDX-License-Identifier: GPL-3.0-or-later
use super::example::ui::{TableApp, TableAppearance, TableLanguage};
use slint::{
    ComponentHandle, Model,
    platform::{Key, PointerEventButton, WindowEvent},
};
use std::{path::PathBuf, time::Duration};

fn key(app: &TableApp, text: impl Into<slint::SharedString>) {
    let text = text.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
}

fn click(app: &TableApp, x: f32, y: f32) {
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

fn snapshot(app: &TableApp, path: PathBuf) {
    let path = match app.window().scale_factor() {
        1.0 => path,
        2.0 => path.with_file_name(
            path.file_name()
                .expect("capture filename")
                .to_str()
                .expect("UTF-8 capture filename")
                .replace(".png", "@2x.png"),
        ),
        scale => panic!("Documentation captures support 1x or 2x, got {scale}"),
    };
    std::fs::create_dir_all(path.parent().expect("capture directory")).expect("capture directory");
    let image = app.window().take_snapshot().expect("native Skia snapshot");
    let mut encoder = png::Encoder::new(
        std::io::BufWriter::new(std::fs::File::create(path).expect("capture")),
        image.width(),
        image.height(),
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .expect("PNG header")
        .write_image_data(image.as_bytes())
        .expect("PNG pixels");
}

pub fn schedule(app: &TableApp) {
    let dark = std::env::args().any(|arg| arg == "--dark");
    let enlarged = std::env::args().any(|arg| arg == "--text-200");
    app.set_appearance(if dark {
        TableAppearance::Dark
    } else {
        TableAppearance::Light
    });
    let mut theme = app.get_theme();
    theme.text_scale = if enlarged { 2. } else { 1. };
    theme.reduced_motion = true;
    app.set_theme(theme);
    app.window().set_size(slint::LogicalSize::new(
        if enlarged { 1000. } else { 720. },
        if enlarged { 900. } else { 460. },
    ));
    let weak = app.as_weak();
    slint::Timer::single_shot(Duration::from_millis(500), move || {
        let app = weak.upgrade().expect("live table example");
        let french = app.get_language() == TableLanguage::French;
        let docs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let filename = format!(
            "table-{}-{}-{}.png",
            if french { "fr" } else { "en" },
            if dark { "dark" } else { "light" },
            if enlarged { "200" } else { "100" }
        );
        snapshot(&app, docs.join(".verification/native").join(&filename));
        if !dark && !enlarged {
            snapshot(
                &app,
                docs.join("src/assets/components").join(if french {
                    "table-integration-fr.png"
                } else {
                    "table-integration.png"
                }),
            );
        }
        app.invoke_focus_table();
        key(&app, Key::DownArrow);
        key(&app, Key::Return);
        assert!(app.get_selection_label().contains("#101"));
        let generation = app.get_generation();
        if enlarged {
            // Traverse the actual header buttons from the table's key scope.
            key(&app, Key::Tab);
            key(&app, Key::Tab);
            key(&app, Key::Space);
        } else {
            // Pointer route through the Tasks header in the documented window.
            click(&app, 550., 134.);
        }
        assert!(
            app.get_generation() > generation,
            "header sort callback must run"
        );
        assert_eq!(app.get_rows().row_data(0).expect("first row").token, 203);
        assert!(
            app.get_selection_label().contains("#101"),
            "selection survives reorder"
        );
        app.invoke_activate(409, generation);
        app.invoke_activate(307, app.get_generation());
        assert!(
            app.get_selection_label().contains("#101"),
            "stale and disabled activation rejected"
        );
        app.invoke_focus_table();
        key(&app, Key::End);
        assert_eq!(app.get_active_index(), 2, "archived last row is skipped");
        key(&app, Key::Home);
        key(&app, Key::Return);
        assert!(app.get_selection_label().contains("#203"));
        snapshot(
            &app,
            docs.join(".verification/native")
                .join(filename.replace(".png", "-selected.png")),
        );
        println!(
            "Native table PASS: sort, stable selection, stale/disabled rejection, Home/End and Enter; winit/Skia; fr={french}, dark={dark}, text200={enlarged}"
        );
        slint::quit_event_loop().expect("finish table verification");
    });
}
