// SPDX-License-Identifier: GPL-3.0-or-later
//! Render the exact documentation examples with a native winit/Skia window.
use slint::{
    ComponentHandle,
    platform::{PointerEventButton, WindowEvent},
};

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExampleLocale {
    English,
    French,
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

mod ui {
    slint::include_modules!();
}
struct CaptureOptions {
    locale: ExampleLocale,
    dark: bool,
    enlarged: bool,
    exercise: bool,
    show: bool,
    size: Option<slint::LogicalSize>,
    output: PathBuf,
}
include!(concat!(env!("OUT_DIR"), "/capture_routes.rs"));

fn save(window: &slint::Window, path: &Path) -> Result<()> {
    let pixels = window.take_snapshot()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut encoder = png::Encoder::new(
        std::io::BufWriter::new(std::fs::File::create(path)?),
        pixels.width(),
        pixels.height(),
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()?
        .write_image_data(pixels.as_bytes())?;
    println!("{} {}×{}", path.display(), pixels.width(), pixels.height());
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--print-ui-directory") {
        println!("{}", kit_build::ui_directory().display());
        return Ok(());
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "kit-docs-capture [SLUG ...] [--all] [--check] [--dark] [--text-200] [--show] [--exercise] [--locale en|fr] [--size WIDTHxHEIGHT] [--output-dir PATH]\n\nCompiles complete docs/examples/*.slint files using public @kit imports.\nCaptures actual native winit/Skia windows via Window::take_snapshot.\nOutput: docs/src/assets/components/<slug>[-fr][-dark][-text-200].png\n\n--check validates exact examples without opening native windows.\n--all captures every example; otherwise pass one or more component slugs.\n--locale fr compiles localized authored fixtures; toolkit/host defaults retain their language.\n--show leaves one native example window open for exploration.\n--size sets a logical viewport, independently of the display pixel density.\n--output-dir writes qualification images separately from public posters.\n--text-200 sets theme text scale 2 in a 1000×1000 logical viewport.\n--exercise checks the button example's native pointer and Space callbacks.\n\nExamples use the public compiled kit and install its shared services.\nApplication-owned collections remain explicit presentation fixtures. Capture uses reduced motion. An active native\ngraphical desktop is required. Screenshots do not qualify IME/screen readers.\nAfter public capture: python3 docs/capture/provenance.py"
        );
        return Ok(());
    }
    let all = args.iter().any(|a| a == "--all");
    let check = args.iter().any(|a| a == "--check");
    let dark = args.iter().any(|a| a == "--dark");
    let enlarged = args.iter().any(|a| a == "--text-200");
    let exercise = args.iter().any(|a| a == "--exercise");
    let show = args.iter().any(|a| a == "--show");
    let output_directory = if let Some(index) = args.iter().position(|arg| arg == "--output-dir") {
        PathBuf::from(args.get(index + 1).ok_or("--output-dir requires a path")?)
    } else {
        root().join("docs/src/assets/components")
    };
    let size = if let Some(index) = args.iter().position(|arg| arg == "--size") {
        let value = args.get(index + 1).ok_or("--size requires WIDTHxHEIGHT")?;
        let (width, height) = value
            .split_once('x')
            .ok_or("--size requires WIDTHxHEIGHT")?;
        let width: u16 = width.parse()?;
        let height: u16 = height.parse()?;
        if width == 0 || height == 0 {
            return Err("--size requires positive dimensions".into());
        }
        Some(slint::LogicalSize::new(f32::from(width), f32::from(height)))
    } else {
        None
    };
    let locale = if let Some(index) = args.iter().position(|arg| arg == "--locale") {
        match args.get(index + 1).map(String::as_str) {
            Some("en") => ExampleLocale::English,
            Some("fr") => ExampleLocale::French,
            _ => return Err("--locale requires en or fr".into()),
        }
    } else {
        ExampleLocale::English
    };
    let example_directory = match locale {
        ExampleLocale::English => root().join("docs/examples"),
        ExampleLocale::French => root().join("docs/examples/fr"),
    };
    let names: Vec<_> = args
        .iter()
        .enumerate()
        .filter(|(index, arg)| {
            !arg.starts_with("--")
                && (*index == 0
                    || !matches!(
                        args[*index - 1].as_str(),
                        "--locale" | "--size" | "--output-dir"
                    ))
        })
        .map(|(_, arg)| arg)
        .collect();
    let mut paths = if all || names.is_empty() {
        std::fs::read_dir(&example_directory)?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().is_some_and(|e| e == "slint"))
            .collect::<Vec<_>>()
    } else {
        names
            .iter()
            .map(|name| example_directory.join(format!("{name}.slint")))
            .collect()
    };
    paths.sort();
    if check {
        for path in &paths {
            kit_build::verify_contracts()?;
            println!("Validated {}", path.display());
        }
        println!("Validated {} complete examples", paths.len());
        return Ok(());
    }
    if all || paths.len() > 1 {
        let executable = std::env::current_exe()?;
        for path in paths {
            let mut command = std::process::Command::new(&executable);
            command.arg(path.file_stem().ok_or("missing filename")?);
            if locale == ExampleLocale::French {
                command.args(["--locale", "fr"]);
            }
            if dark {
                command.arg("--dark");
            }
            if enlarged {
                command.arg("--text-200");
            }
            if exercise {
                command.arg("--exercise");
            }
            if let Some(index) = args.iter().position(|arg| arg == "--size") {
                command.args(["--size", &args[index + 1]]);
            }
            command.arg("--output-dir").arg(&output_directory);
            if !command.status()?.success() {
                return Err(format!("Capture failed: {}", path.display()).into());
            }
        }
        return Ok(());
    }
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .renderer_name("skia".into())
        .select()?;
    let path = paths.first().ok_or("No examples found")?;
    let slug = path
        .file_stem()
        .ok_or("missing filename")?
        .to_string_lossy();
    if exercise && slug != "button" {
        return Err(
            "--exercise currently qualifies the button example; pass button --exercise".into(),
        );
    }
    let suffix = format!(
        "{}{}{}{}",
        if locale == ExampleLocale::French {
            "-fr"
        } else {
            ""
        },
        if dark { "-dark" } else { "" },
        if enlarged { "-text-200" } else { "" },
        if exercise { "-interaction" } else { "" }
    );
    let output = output_directory.join(format!("{slug}{suffix}.png"));
    capture(
        path,
        CaptureOptions {
            locale,
            dark,
            enlarged,
            exercise,
            show,
            size,
            output,
        },
    )
}

type StatusPorts<W> = (fn(&W) -> slint::SharedString, fn(&W, slint::SharedString));

fn run_example<W>(
    app: W,
    get_theme: fn(&W) -> slint_kit::ui::KitTheme,
    set_theme: fn(&W, slint_kit::ui::KitTheme),
    status: Option<StatusPorts<W>>,
    options: CaptureOptions,
) -> Result<()>
where
    W: ComponentHandle + 'static,
    for<'a> slint_kit::ui::KitThemes<'a>: slint::Global<'a, W>,
    for<'a> slint_kit::ui::KitThemeContext<'a>: slint::Global<'a, W>,
{
    let mut theme = if options.dark {
        app.global::<slint_kit::ui::KitThemes>().get_dark()
    } else {
        get_theme(&app)
    };
    theme.reduced_motion = true;
    if options.enlarged {
        theme.text_scale = 2.;
    }
    app.global::<slint_kit::ui::KitThemeContext>()
        .set_current(theme.clone());
    set_theme(&app, theme);
    if let Some(size) = options.size.or_else(|| {
        options
            .enlarged
            .then(|| slint::LogicalSize::new(1000., 1000.))
    }) {
        app.window().set_size(size);
    }
    if options.show {
        return Ok(app.run()?);
    }
    app.show()?;
    let weak = app.as_weak();
    slint::Timer::single_shot(Duration::from_millis(300), move || {
        if let Some(app) = weak.upgrade() {
            if options.exercise {
                let (get_status, set_status) = status.expect("button exposes status");
                let position = slint::LogicalPosition::new(70., 96.);
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
                let text = get_status(&app);
                assert!(!text.is_empty(), "native pointer activation updates status");
                set_status(&app, "".into());
                app.window().dispatch_event(WindowEvent::KeyPressed {
                    text: slint::platform::Key::Space.into(),
                });
                app.window().dispatch_event(WindowEvent::KeyReleased {
                    text: slint::platform::Key::Space.into(),
                });
                assert_eq!(
                    get_status(&app),
                    text,
                    "native Space activation updates status"
                );
                println!("Verified pointer activation and keyboard Space activation");
            }
            if let Err(error) = save(app.window(), &options.output) {
                eprintln!("Snapshot failed: {error}");
                std::process::exit(1);
            }
        }
        slint::quit_event_loop().expect("finish capture");
    });
    app.run()?;
    Ok(())
}
