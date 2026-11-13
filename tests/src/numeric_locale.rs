// SPDX-License-Identifier: GPL-3.0-or-later
use crate::NumericLocaleFixture;
use i_slint_backend_testing::{ElementHandle, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};
use slint_kit::{
    AbsencePolicy, CommitPolicy, CurrencyCode, CurrencyMetadata, DecimalRules, DecimalScale,
    DecimalValue, DraftValue, FieldBinding, FieldSpec, FieldSpecKey, GroupSizes, Grouping,
    HtmlConstraints, HtmlInputKind, Money, NumberInputFormat, NumberSyntax, NumericBounds,
    Ownership, ParseIssue, RoundingPolicy, SignPolicy, SymbolPlacement, format_decimal,
    parse_integer, ui,
};
use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
};
thread_local! { static BACKEND: OnceCell<()> = const { OnceCell::new() }; }
fn decimal(text: &str) -> DecimalValue {
    text.parse().unwrap()
}
fn rules(locale: NumberSyntax) -> DecimalRules {
    DecimalRules {
        syntax: locale,
        scale: DecimalScale::new(2).unwrap(),
        sign: SignPolicy::Signed,
        bounds: Some(NumericBounds::new(decimal("-2000"), decimal("2000")).unwrap()),
    }
}
fn format(locale: NumberSyntax) -> ui::NumberInputFormat {
    if locale.decimal_separator() == '.' {
        ui::NumberInputFormat::Canonical
    } else {
        ui::NumberInputFormat::DecimalComma
    }
}
fn commit(text: &str, locale: NumberSyntax) -> ui::FieldCommit {
    ui::FieldCommit {
        value: ui::FieldValue {
            present: !text.is_empty(),
            text: text.into(),
        },
        constraints: ui::KitInputConstraints {
            kind: ui::HtmlInputKind::Number,
            number_format: format(locale),
            min: "-2000".into(),
            max: "2000".into(),
            step: "0.01".into(),
            min_length: -1,
            max_length: -1,
            ..Default::default()
        },
        ..Default::default()
    }
}
#[test]
fn supported_locale_grammars_cross_exact_constraints_and_typed_codecs_without_losing_values() {
    for locale in [
        NumberSyntax::en_ca(),
        NumberSyntax::en_us(),
        NumberSyntax::fr_ca(),
        NumberSyntax::fr_fr(),
    ] {
        let spec = FieldSpec::decimal(rules(locale));
        let grouped = format_decimal(
            decimal("1234.50"),
            locale,
            DecimalScale::new(2).unwrap(),
            RoundingPolicy::Reject,
        )
        .unwrap();
        for text in [
            grouped.clone(),
            spec.format(&decimal("1234.50")),
            format!(" +{grouped} "),
        ] {
            assert_eq!(
                spec.accept(commit(&text, locale)).unwrap(),
                Some(decimal("1234.50")),
                "{locale:?}: {text}"
            );
            let currency = CurrencyCode::new("CAD").unwrap();
            assert_eq!(
                FieldSpec::money(currency.clone(), rules(locale))
                    .accept(commit(&text, locale))
                    .unwrap()
                    .unwrap()
                    .amount,
                decimal("1234.50")
            );
            assert_eq!(
                FieldSpec::percent(rules(locale))
                    .accept(commit(&text, locale))
                    .unwrap()
                    .unwrap()
                    .0,
                decimal("1234.50")
            );
            assert_eq!(
                FieldSpec::quantity("kg", rules(locale))
                    .accept(commit(&text, locale))
                    .unwrap()
                    .unwrap()
                    .value,
                decimal("1234.50")
            );
        }
        let bad = if locale.decimal_separator() == '.' {
            [
                "1,23.50",
                "12,34,567.89",
                "1,,234.50",
                "1,234.5,0",
                "1.234,50",
            ]
        } else {
            [
                "1 23,50",
                "12 34 567,89",
                "1  234,50",
                "1 234,5,0",
                "1,234.50",
            ]
        };
        for text in bad {
            assert!(
                spec.accept(commit(text, locale)).is_err(),
                "{locale:?}: {text}"
            );
        }
        let mark = locale.decimal_separator();
        for text in [
            format!("1{mark}234"),
            "2001".into(),
            "-2001".into(),
            "-".into(),
            "1e".into(),
        ] {
            assert!(
                spec.accept(commit(&text, locale)).is_err(),
                "{locale:?}: {text}"
            );
        }
        assert_eq!(spec.accept(commit("", locale)).unwrap(), None);
        assert_eq!(
            spec.accept(commit("-12", locale)).unwrap(),
            Some(decimal("-12"))
        );
        let mut nonnegative = rules(locale);
        nonnegative.sign = SignPolicy::NonNegative;
        assert!(
            FieldSpec::decimal(nonnegative)
                .accept(commit("-1", locale))
                .is_err()
        );
        // The direct binding's configured parser takes canonical semantic text.
        // Normalization validates without replacing its native draft projection.
        let binding = FieldBinding::<DecimalValue, ()>::new(
            None,
            Ownership::Local,
            CommitPolicy::OnSubmit,
            AbsencePolicy::Optional,
            |text| match text.parse() {
                Ok(value) => slint_kit::DraftValue::Parsed(value),
                Err(issue) => slint_kit::DraftValue::Invalid(issue),
            },
            ToString::to_string,
            |_| "Invalid amount".into(),
            |_| {},
        );
        binding.set_html_constraints(HtmlConstraints {
            kind: HtmlInputKind::Number,
            number_format: if locale.decimal_separator() == '.' {
                NumberInputFormat::Canonical
            } else {
                NumberInputFormat::DecimalComma
            },
            step: Some("0.01".into()),
            ..Default::default()
        });
        binding.edited(&grouped).unwrap();
        binding.commit(slint_kit::CommitTrigger::Explicit).unwrap();
        assert_eq!(binding.accepted(), Some(decimal("1234.50")));
    }
    assert_eq!(
        FieldSpec::integer(NumberSyntax::en_ca(), None)
            .accept(commit("1,234", NumberSyntax::en_us()))
            .unwrap(),
        Some(1234)
    );
    assert!(
        FieldSpec::integer(NumberSyntax::en_ca(), None)
            .accept(commit("1,234.5", NumberSyntax::en_us()))
            .is_err()
    );
}
#[test]
fn open_syntaxes_share_one_grammar_between_parser_formatter_money_and_integer_codec() {
    let amount = decimal("1234567.50");
    let cad = CurrencyMetadata::cad();
    let money = Money {
        amount,
        currency: cad.code.clone(),
    };
    let negative = Money {
        amount: decimal("-1234.5"),
        currency: cad.code.clone(),
    };
    // (syntax, grouped text, money text, negative money text)
    let cases = [
        (
            NumberSyntax::de_de(),
            "1.234.567,50",
            "1.234.567,50\u{a0}$",
            "-1.234,50\u{a0}$",
        ),
        (
            NumberSyntax::de_ch(),
            "1\u{2019}234\u{2019}567.50",
            "$\u{a0}1\u{2019}234\u{2019}567.50",
            "-$\u{a0}1\u{2019}234.50",
        ),
        (
            NumberSyntax::pt_br(),
            "1.234.567,50",
            "$\u{a0}1.234.567,50",
            "-$\u{a0}1.234,50",
        ),
        (
            NumberSyntax::en_in(),
            "12,34,567.50",
            "$12,34,567.50",
            "-$1,234.50",
        ),
        (
            NumberSyntax::en_us(),
            "1,234,567.50",
            "$1,234,567.50",
            "-$1,234.50",
        ),
        (
            NumberSyntax::fr_fr(),
            "1\u{202f}234\u{202f}567,50",
            "1\u{202f}234\u{202f}567,50\u{a0}$",
            "-1\u{202f}234,50\u{a0}$",
        ),
    ];
    for (syntax, grouped, money_text, negative_text) in cases {
        let scale = DecimalScale::new(2).unwrap();
        assert_eq!(
            format_decimal(amount, syntax, scale, RoundingPolicy::Reject).unwrap(),
            grouped
        );
        let rules = DecimalRules {
            syntax,
            scale,
            sign: SignPolicy::Signed,
            bounds: None,
        };
        assert_eq!(rules.parse(grouped), DraftValue::Parsed(amount));
        assert_eq!(
            money.format(&cad, syntax, RoundingPolicy::Reject).unwrap(),
            money_text
        );
        assert_eq!(
            negative
                .format(&cad, syntax, RoundingPolicy::Reject)
                .unwrap(),
            negative_text
        );
        let ungrouped = grouped.replace(syntax.grouping().unwrap().separator(), "");
        assert_eq!(rules.parse(&ungrouped), DraftValue::Parsed(amount));
        assert_eq!(
            parse_integer(&grouped[..grouped.len() - 3], syntax, None),
            DraftValue::Parsed(1_234_567)
        );
    }
    // Malformed grouping is rejected rather than guessed.
    let de = DecimalRules {
        syntax: NumberSyntax::de_de(),
        scale: DecimalScale::new(2).unwrap(),
        sign: SignPolicy::Signed,
        bounds: None,
    };
    for bad in ["1.23.4,50", "12345.678,50", ".123,50", "1.2345,50"] {
        assert_eq!(
            de.parse(bad),
            DraftValue::Invalid(ParseIssue::InvalidGrouping),
            "{bad}"
        );
    }
    let india = DecimalRules {
        syntax: NumberSyntax::en_in(),
        ..de
    };
    for bad in ["123,456.50", "1,234,567.50", "1,2,345.50"] {
        assert_eq!(
            india.parse(bad),
            DraftValue::Invalid(ParseIssue::InvalidGrouping),
            "{bad}"
        );
    }
    // The ASCII apostrophe is an explicit alternate of de-CH.
    assert_eq!(
        parse_integer("1'234", NumberSyntax::de_ch(), None),
        DraftValue::Parsed(1234)
    );
    // The integer codec takes its syntax and bounds from the caller.
    let bounds = NumericBounds::new(0, 5000).unwrap();
    let spec = FieldSpec::integer(NumberSyntax::de_de(), Some(bounds));
    let text_commit = |text: &str| ui::FieldCommit {
        value: ui::FieldValue {
            present: true,
            text: text.into(),
        },
        constraints: ui::KitInputConstraints {
            kind: ui::HtmlInputKind::Text,
            min_length: -1,
            max_length: -1,
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(spec.accept(text_commit("1.234")).unwrap(), Some(1234));
    assert!(spec.accept(text_commit("5.001")).is_err());
    assert!(spec.accept(text_commit("-1")).is_err());
    assert!(spec.accept(text_commit("1,234")).is_err());
    // A custom syntax is validated once, at construction.
    assert!(
        NumberSyntax::new(
            ',',
            Some(Grouping::new(',', GroupSizes::Thousands)),
            SymbolPlacement::Prefix
        )
        .is_err()
    );
    assert!(NumberSyntax::new('5', None, SymbolPlacement::Prefix).is_err());
    let apostrophe = NumberSyntax::new(
        '.',
        Some(Grouping::new('\'', GroupSizes::Thousands)),
        SymbolPlacement::Prefix,
    )
    .unwrap();
    assert_eq!(
        parse_integer("1'234", apostrophe, None),
        DraftValue::Parsed(1234)
    );
}
fn unshown_fixture(locale: NumberSyntax) -> NumericLocaleFixture {
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
    let app = NumericLocaleFixture::new().unwrap();
    slint_kit::install(&app);
    slint_kit::install_field_specs(
        &app,
        [FieldSpec::decimal(rules(locale)).named(FieldSpecKey::new("amount").unwrap())],
    )
    .unwrap();
    app.set_number_format(format(locale));
    app
}
fn fixture(locale: NumberSyntax) -> NumericLocaleFixture {
    let app = unshown_fixture(locale);
    app.show().unwrap();
    frame(&app);
    app
}
fn frame(app: &NumericLocaleFixture) {
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    slint::platform::update_timers_and_animations();
}
fn key(app: &NumericLocaleFixture, text: impl Into<slint::SharedString>) {
    let text = text.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    frame(app);
}
fn undo(app: &NumericLocaleFixture) {
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    key(app, "z");
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
    frame(app);
}
#[test]
fn native_grouped_paste_keeps_selection_undo_and_controlled_acknowledgment() {
    for locale in [
        NumberSyntax::en_ca(),
        NumberSyntax::en_us(),
        NumberSyntax::fr_ca(),
        NumberSyntax::fr_fr(),
    ] {
        let app = fixture(locale);
        let spec = FieldSpec::decimal(rules(locale));
        let values = Rc::new(RefCell::new(Vec::new()));
        let observed = values.clone();
        app.on_committed(move |value| {
            observed
                .borrow_mut()
                .push(spec.accept(value).unwrap().unwrap())
        });
        let editor = ElementHandle::find_by_accessible_label(&app, "Amount")
            .next()
            .unwrap();
        let grouped = format_decimal(
            decimal("1234.50"),
            locale,
            DecimalScale::new(2).unwrap(),
            RoundingPolicy::Reject,
        )
        .unwrap();
        app.set_clipboard(grouped.clone().into());
        app.invoke_seed_clipboard();
        app.invoke_focus_amount();
        app.invoke_select_amount();
        app.invoke_paste_amount();
        frame(&app);
        assert_eq!(app.get_text(), grouped);
        assert!(editor.is_valid());
        undo(&app);
        undo(&app);
        assert_eq!(app.get_text(), "10");
        app.invoke_select_amount();
        app.invoke_paste_amount();
        frame(&app);
        key(&app, Key::Return);
        assert_eq!(*values.borrow(), [decimal("1234.50")]);
        let accepted = app.get_state().accepted;
        app.set_ownership(ui::Ownership::Controlled);
        app.set_clipboard("12".into());
        app.invoke_seed_clipboard();
        app.invoke_focus_amount();
        app.invoke_select_amount();
        app.invoke_paste_amount();
        frame(&app);
        key(&app, Key::Return);
        assert_eq!(app.get_state().accepted, accepted);
        assert_eq!(*values.borrow(), [decimal("1234.50"), decimal("12")]);
        app.invoke_acknowledge(ui::FieldValue {
            present: true,
            text: "12".into(),
        });
        assert_eq!(app.get_state().accepted.text, "12");
        app.set_access(ui::FieldAccess::ReadOnly);
        key(&app, Key::Return);
        assert_eq!(values.borrow().len(), 2);
        app.set_access(ui::FieldAccess::Editable);
        let malformed = if locale.decimal_separator() == '.' {
            "1,23.50"
        } else {
            "1 23,50"
        };
        app.set_clipboard(malformed.into());
        app.invoke_seed_clipboard();
        app.invoke_focus_amount();
        app.invoke_select_amount();
        app.invoke_paste_amount();
        frame(&app);
        key(&app, Key::Return);
        assert_eq!(app.get_text(), malformed, "malformed syntax stays editable");
        assert_eq!(values.borrow().len(), 2, "malformed grouping cannot commit");
    }
}
#[test]
fn grouped_drafts_step_on_the_exact_lattice_and_render_in_both_themes_and_scales() {
    let app = fixture(NumberSyntax::en_us());
    let values = Rc::new(RefCell::new(Vec::new()));
    let observed = values.clone();
    app.on_committed(move |value| {
        observed.borrow_mut().push(
            FieldSpec::decimal(rules(NumberSyntax::en_us()))
                .accept(value)
                .unwrap()
                .unwrap(),
        )
    });
    app.set_text("1,234.50".into());
    app.invoke_focus_amount();
    key(&app, Key::UpArrow);
    assert_eq!(app.get_text(), "1234.51");
    key(&app, Key::Return);
    assert_eq!(*values.borrow(), [decimal("1234.51")]);
    for dark in [false, true] {
        for scale in [1., 2.] {
            let app = unshown_fixture(NumberSyntax::en_us());
            let mut theme = if dark {
                app.global::<ui::KitThemes>().get_dark()
            } else {
                app.global::<ui::KitThemes>().get_light()
            };
            theme.text_scale = scale;
            app.set_theme(theme);
            app.set_text("1,234.50".into());
            app.show().unwrap();
            frame(&app);
            let editor = ElementHandle::find_by_accessible_label(&app, "Amount")
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
            if let Some(dir) = std::env::var_os("KIT_NUMERIC_LOCALE_RENDER_DIR") {
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
                        "numeric-{}-{scale}.ppm",
                        if dark { "dark" } else { "light" }
                    )),
                    ppm,
                )
                .unwrap();
            }
        }
    }
}
