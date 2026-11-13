// SPDX-License-Identifier: GPL-3.0-or-later
use slint_kit::{
    CurrencyCode, CurrencyMetadata, DecimalScale, FieldSpec, Money, NumberSyntax, ParseIssue,
    RoundingPolicy, SignPolicy, ui,
};

fn commit(text: &str) -> ui::FieldCommit {
    ui::FieldCommit {
        value: ui::FieldValue {
            present: !text.is_empty(),
            text: text.into(),
        },
        constraints: ui::KitInputConstraints {
            kind: ui::HtmlInputKind::Number,
            number_format: ui::NumberInputFormat::Canonical,
            step: "any".into(),
            min_length: -1,
            max_length: -1,
            ..Default::default()
        },
        ..Default::default()
    }
}
fn code(text: &str) -> CurrencyCode {
    CurrencyCode::new(text).unwrap()
}
fn scale_of(text: &str) -> u32 {
    CurrencyMetadata::iso(&code(text)).unwrap().scale.get()
}

#[test]
fn iso_table_gives_the_standard_minor_unit_scale_and_rejects_unlisted_codes() {
    assert_eq!(scale_of("USD"), 2);
    assert_eq!(scale_of("JPY"), 0);
    assert_eq!(scale_of("KWD"), 3);
    assert_eq!(scale_of("CLF"), 4);
    assert_eq!(
        CurrencyMetadata::iso(&code("CAD")),
        Some(CurrencyMetadata::cad())
    );
    assert_eq!(
        CurrencyMetadata::iso(&code("EUR")),
        Some(CurrencyMetadata::eur())
    );
    assert_eq!(CurrencyMetadata::iso(&code("XXX")), None);
}

#[test]
fn custom_metadata_carries_its_own_code_symbol_and_scale() {
    let metadata = CurrencyMetadata::new(code("XAF"), "FCFA", DecimalScale::new(0).unwrap());
    assert_eq!(metadata.code.as_str(), "XAF");
    assert_eq!(metadata.symbol, "FCFA");
    assert_eq!(metadata.scale.get(), 0);
}

#[test]
fn money_in_takes_its_scale_and_currency_from_the_metadata() {
    let syntax = NumberSyntax::en_ca();
    let dollars = FieldSpec::money_in(
        CurrencyMetadata::cad(),
        syntax,
        SignPolicy::NonNegative,
        None,
    );
    let value = dollars.accept(commit("1234.5")).unwrap().unwrap();
    assert_eq!(value.currency.as_str(), "CAD");
    assert_eq!(dollars.format(&value), "1234.5");
    assert!(dollars.accept(commit("1.234")).is_err());
    let yen = FieldSpec::money_in(
        CurrencyMetadata::iso(&code("JPY")).unwrap(),
        syntax,
        SignPolicy::NonNegative,
        None,
    );
    assert_eq!(
        yen.accept(commit("500")).unwrap().unwrap().amount,
        "500".parse().unwrap()
    );
    assert!(yen.accept(commit("0.5")).is_err());
    let dinars = FieldSpec::money_in(
        CurrencyMetadata::iso(&code("KWD")).unwrap(),
        syntax,
        SignPolicy::NonNegative,
        None,
    );
    assert!(dinars.accept(commit("1.234")).is_ok());
    assert!(dinars.accept(commit("1.2345")).is_err());
}

#[test]
fn formatting_with_iso_metadata_shows_symbol_grouping_and_rejects_finer_amounts() {
    let metadata = CurrencyMetadata::iso(&code("USD")).unwrap();
    let money = Money {
        amount: "1234567.5".parse().unwrap(),
        currency: code("USD"),
    };
    assert_eq!(
        money
            .format(&metadata, NumberSyntax::en_us(), RoundingPolicy::Reject)
            .unwrap(),
        "$1,234,567.50"
    );
    let finer = Money {
        amount: "1.005".parse().unwrap(),
        currency: code("USD"),
    };
    assert_eq!(
        finer.format(&metadata, NumberSyntax::en_us(), RoundingPolicy::Reject),
        Err(ParseIssue::ScaleExceeded)
    );
}
