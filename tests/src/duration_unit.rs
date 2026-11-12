// SPDX-License-Identifier: GPL-3.0-or-later
use crate::DurationUnitFixture;
use i_slint_backend_testing::{ElementHandle, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};
use slint_kit::{
    DecimalRules, DecimalScale, DecimalValue, DurationUnit, DurationValue, FieldSpec, FieldSpecKey,
    NumberLocale, NumericBounds, ParseIssue, SignPolicy, ui,
};
use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
};
thread_local! { static BACKEND: OnceCell<()> = const { OnceCell::new() }; }
fn decimal(text: &str) -> DecimalValue {
    text.parse().unwrap()
}
fn rules(locale: NumberLocale, scale: u32) -> DecimalRules {
    DecimalRules {
        locale,
        scale: DecimalScale::new(scale).unwrap(),
        sign: SignPolicy::NonNegative,
        bounds: None,
    }
}
fn commit(text: &str, locale: NumberLocale) -> ui::FieldCommit {
    ui::FieldCommit {
        value: ui::FieldValue {
            present: !text.is_empty(),
            text: text.into(),
        },
        constraints: ui::KitInputConstraints {
            kind: ui::HtmlInputKind::Number,
            number_format: match locale {
                NumberLocale::EnCa | NumberLocale::EnUs => ui::NumberInputFormat::Canonical,
                NumberLocale::FrCa | NumberLocale::FrFr => ui::NumberInputFormat::DecimalComma,
            },
            step: "any".into(),
            min_length: -1,
            max_length: -1,
            ..Default::default()
        },
        ..Default::default()
    }
}
#[test]
fn explicit_units_round_trip_exact_quantities_through_the_public_codec() {
    for locale in [
        NumberLocale::EnCa,
        NumberLocale::EnUs,
        NumberLocale::FrCa,
        NumberLocale::FrFr,
    ] {
        for (unit, milliseconds) in [
            (DurationUnit::Milliseconds, 5),
            (DurationUnit::Seconds, 5_000),
            (DurationUnit::Minutes, 300_000),
            (DurationUnit::Hours, 18_000_000),
        ] {
            let spec = FieldSpec::duration(unit, rules(locale, 3));
            let value = spec.accept(commit("5", locale)).unwrap().unwrap();
            assert_eq!(value.milliseconds(), milliseconds);
            assert_eq!(value.quantity(unit).unwrap(), decimal("5"));
            let formatted = spec.format(&value).unwrap();
            assert_eq!(
                spec.accept(commit(&formatted, locale)).unwrap(),
                Some(value)
            );
            let (plain, grouped) = match unit {
                DurationUnit::Milliseconds => (
                    "1234",
                    match locale {
                        NumberLocale::EnCa | NumberLocale::EnUs => "+1,234",
                        NumberLocale::FrCa | NumberLocale::FrFr => "+1\u{202f}234",
                    },
                ),
                DurationUnit::Seconds | DurationUnit::Minutes | DurationUnit::Hours => (
                    "1234.5",
                    match locale {
                        NumberLocale::EnCa | NumberLocale::EnUs => "+1,234.5",
                        NumberLocale::FrCa | NumberLocale::FrFr => "+1\u{202f}234,5",
                    },
                ),
            };
            let grouped_value = spec.accept(commit(grouped, locale)).unwrap().unwrap();
            assert_eq!(grouped_value.quantity(unit).unwrap(), decimal(plain));
            assert_eq!(
                spec.accept(commit(&spec.format(&grouped_value).unwrap(), locale))
                    .unwrap(),
                Some(grouped_value)
            );
        }
        let spec = FieldSpec::duration(DurationUnit::Seconds, rules(locale, 3));
        let fraction = match locale {
            NumberLocale::EnCa | NumberLocale::EnUs => "1.5",
            NumberLocale::FrCa | NumberLocale::FrFr => "1,5",
        };
        assert_eq!(
            spec.accept(commit(fraction, locale))
                .unwrap()
                .unwrap()
                .milliseconds(),
            1500
        );
        let tiny = match locale {
            NumberLocale::EnCa | NumberLocale::EnUs => "0.001",
            NumberLocale::FrCa | NumberLocale::FrFr => "0,001",
        };
        assert_eq!(
            spec.accept(commit(tiny, locale))
                .unwrap()
                .unwrap()
                .milliseconds(),
            1
        );
        assert_eq!(spec.accept(commit("", locale)).unwrap(), None);
    }
}
#[test]
fn duration_representability_scale_bounds_and_overflow_are_explicit() {
    let locale = NumberLocale::EnCa;
    let seconds = FieldSpec::duration(DurationUnit::Seconds, rules(locale, 4));
    for text in [
        "0.0001",
        "1.00001",
        "-1",
        "1e",
        "1,23",
        "79228162514264337593543950335",
    ] {
        assert!(seconds.accept(commit(text, locale)).is_err(), "{text}");
    }
    let one = DurationValue::from_milliseconds(1);
    assert_eq!(
        one.quantity(DurationUnit::Minutes),
        Err(ParseIssue::ScaleExceeded)
    );
    assert_eq!(
        one.quantity(DurationUnit::Hours),
        Err(ParseIssue::ScaleExceeded)
    );
    assert_eq!(
        FieldSpec::duration(DurationUnit::Minutes, rules(locale, 28)).format(&one),
        Err(ParseIssue::ScaleExceeded)
    );
    assert_eq!(
        FieldSpec::duration(DurationUnit::Hours, rules(locale, 28)).format(&one),
        Err(ParseIssue::ScaleExceeded)
    );
    assert_eq!(
        FieldSpec::duration(DurationUnit::Seconds, rules(locale, 2)).format(&one),
        Err(ParseIssue::ScaleExceeded)
    );
    assert_eq!(seconds.format(&one).unwrap(), "0.0010");
    assert_eq!(
        FieldSpec::duration(DurationUnit::Minutes, rules(locale, 5))
            .format(&DurationValue::from_milliseconds(3))
            .unwrap(),
        "0.00005"
    );
    assert_eq!(
        FieldSpec::duration(DurationUnit::Hours, rules(locale, 7))
            .format(&DurationValue::from_milliseconds(9))
            .unwrap(),
        "0.0000025"
    );
    let maximum = DurationValue::from_milliseconds(u64::MAX);
    let milliseconds = FieldSpec::duration(DurationUnit::Milliseconds, rules(locale, 0));
    assert_eq!(
        milliseconds
            .accept(commit(&u64::MAX.to_string(), locale))
            .unwrap(),
        Some(maximum)
    );
    assert_eq!(milliseconds.format(&maximum).unwrap(), u64::MAX.to_string());
    assert!(
        milliseconds
            .accept(commit("18446744073709551616", locale))
            .is_err()
    );
    let mut limited = rules(locale, 2);
    limited.bounds = Some(NumericBounds::new(decimal("1"), decimal("5")).unwrap());
    let spec = FieldSpec::duration(DurationUnit::Seconds, limited);
    assert_eq!(
        spec.format(&DurationValue::from_milliseconds(6000)),
        Err(ParseIssue::AboveMaximum)
    );
    assert!(spec.accept(commit("0.5", locale)).is_err());
    let mut c = commit("5", locale);
    c.constraints.max = "4".into();
    assert!(spec.accept(c).is_err());
    let mut c = commit("1.5", locale);
    c.constraints.step = "1".into();
    assert!(spec.accept(c).is_err());
    let mut c = commit("", locale);
    c.constraints.required = true;
    assert!(spec.accept(c).is_err());
    let zero = FieldSpec::duration(DurationUnit::Seconds, rules(locale, 3))
        .accept(commit("0", locale))
        .unwrap()
        .unwrap();
    assert_eq!(zero.milliseconds(), 0);
}
fn fixture() -> DurationUnitFixture {
    BACKEND.with(|b| {
        b.get_or_init(|| {
            slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
                mock_time: true,
                renderer_name: Some("software".into()),
                ..Default::default()
            })))
            .unwrap()
        });
    });
    let app = DurationUnitFixture::new().unwrap();
    slint_kit::install(&app);
    slint_kit::install_field_specs(
        &app,
        [
            FieldSpec::duration(DurationUnit::Seconds, rules(NumberLocale::EnCa, 3))
                .named(FieldSpecKey::new("amount").unwrap()),
        ],
    )
    .unwrap();
    app
}
fn frame(app: &DurationUnitFixture) {
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    slint::platform::update_timers_and_animations();
}
fn key(app: &DurationUnitFixture, text: impl Into<slint::SharedString>) {
    let text = text.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    frame(app);
}
#[test]
fn native_seconds_commit_step_undo_and_controlled_acknowledgment_keep_the_unit() {
    let app = fixture();
    app.show().unwrap();
    frame(&app);
    let observed = Rc::new(RefCell::new(Vec::new()));
    let values = observed.clone();
    let spec = FieldSpec::duration(DurationUnit::Seconds, rules(NumberLocale::EnCa, 3));
    app.on_committed(move |commit| {
        values
            .borrow_mut()
            .push(spec.accept(commit).unwrap().unwrap());
    });
    app.set_clipboard("1.5".into());
    app.invoke_seed_clipboard();
    app.invoke_focus_amount();
    app.invoke_select_amount();
    app.invoke_paste_amount();
    frame(&app);
    assert_eq!(app.get_text(), "1.5");
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    key(&app, "z");
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    key(&app, "z");
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
    assert_eq!(app.get_text(), "5");
    app.invoke_select_amount();
    app.invoke_paste_amount();
    frame(&app);
    key(&app, Key::UpArrow);
    key(&app, Key::Return);
    assert_eq!(observed.borrow()[0].milliseconds(), 2000);
    app.set_ownership(ui::Ownership::Controlled);
    app.set_clipboard("5".into());
    app.invoke_seed_clipboard();
    app.invoke_focus_amount();
    app.invoke_select_amount();
    app.invoke_paste_amount();
    frame(&app);
    key(&app, Key::Return);
    assert_eq!(observed.borrow()[1].milliseconds(), 5000);
    assert_eq!(app.get_state().accepted.text, "2");
    app.invoke_acknowledge(ui::FieldValue {
        present: true,
        text: "5".into(),
    });
    assert_eq!(app.get_state().accepted.text, "5");
    app.set_clipboard("7".into());
    app.invoke_seed_clipboard();
    app.invoke_focus_amount();
    app.invoke_select_amount();
    app.invoke_paste_amount();
    frame(&app);
    key(&app, Key::Escape);
    assert_eq!(app.get_text(), "7", "the default Escape policy bubbles");
    app.invoke_cancel_amount();
    frame(&app);
    assert_eq!(app.get_text(), "5");
    assert_eq!(app.get_state().accepted.text, "5");
    assert_eq!(observed.borrow().len(), 2, "cancel does not commit");
    app.set_access(ui::FieldAccess::ReadOnly);
    key(&app, Key::UpArrow);
    assert_eq!(app.get_text(), "5");
    app.set_access(ui::FieldAccess::Disabled);
    key(&app, Key::Return);
    assert_eq!(observed.borrow().len(), 2);
}
#[test]
fn duration_unit_suffix_and_quantity_render_in_both_themes_at_both_text_scales() {
    for dark in [false, true] {
        for scale in [1., 2.] {
            let app = fixture();
            let mut theme = if dark {
                app.global::<ui::KitThemes>().get_dark()
            } else {
                app.global::<ui::KitThemes>().get_light()
            };
            theme.text_scale = scale;
            app.set_theme(theme);
            app.show().unwrap();
            frame(&app);
            app.invoke_focus_amount();
            key(&app, Key::UpArrow);
            key(&app, Key::Return);
            assert_eq!(app.get_state().accepted.text, "5.5");
            let editor = ElementHandle::find_by_accessible_label(&app, "Duration")
                .next()
                .unwrap();
            let point = editor.absolute_position();
            let size = editor.size();
            let window = app.window().size().to_logical(app.window().scale_factor());
            assert!(
                size.width > 0.
                    && size.height > 0.
                    && point.x >= 0.
                    && point.y >= 0.
                    && point.x + size.width <= window.width + 1.
                    && point.y + size.height <= window.height + 1.
            );
            if let Some(dir) = std::env::var_os("KIT_DURATION_UNIT_RENDER_DIR") {
                let image = app.window().take_snapshot().unwrap();
                let dir = std::path::PathBuf::from(dir);
                std::fs::create_dir_all(&dir).unwrap();
                let mut ppm =
                    format!("P6\n{} {}\n255\n", image.width(), image.height()).into_bytes();
                for pixel in image.as_slice() {
                    ppm.extend([pixel.r, pixel.g, pixel.b]);
                }
                std::fs::write(
                    dir.join(format!(
                        "duration-{}-{scale}.ppm",
                        if dark { "dark" } else { "light" }
                    )),
                    ppm,
                )
                .unwrap();
            }
        }
    }
}
