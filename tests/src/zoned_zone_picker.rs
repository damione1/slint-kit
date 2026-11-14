// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{ZonedReportFocusFixture, ui};
use slint::{
    ComponentHandle, Model,
    platform::{Key, WindowEvent},
};

fn fixture() -> ZonedReportFocusFixture {
    i_slint_backend_testing::init_no_event_loop();
    let app = ZonedReportFocusFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    app
}
fn key(app: &ZonedReportFocusFixture, text: impl Into<slint::SharedString>) {
    let text = text.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    slint::platform::update_timers_and_animations();
}
fn rows(app: &ZonedReportFocusFixture) -> Vec<ui::OptionRow> {
    let options = app.global::<ui::KitTimeService>().invoke_zone_options();
    (0..options.row_count())
        .filter_map(|index| options.row_data(index))
        .collect()
}
fn focus_zone(app: &ZonedReportFocusFixture) {
    app.set_date_text("2026-11-10".into());
    app.set_time_text("09:00".into());
    assert!(!app.invoke_report_zoned());
    assert!(app.invoke_focus_invalid_zoned());
}

#[test]
fn the_catalogue_feeds_one_option_per_zone_with_an_offset_hint() {
    let app = fixture();
    let rows = rows(&app);
    assert_eq!(rows.len(), kit_core::TimeZoneId::all().len());
    let toronto = rows
        .iter()
        .find(|row| row.label == "America/Toronto")
        .expect("the catalogue lists America/Toronto");
    assert!(toronto.description.starts_with("UTC-0"));
    assert!(toronto.description.ends_with("Toronto"));
    let tokens: std::collections::HashSet<_> = rows.iter().map(|row| row.token).collect();
    assert_eq!(tokens.len(), rows.len(), "every option has its own token");
}

#[test]
fn a_zone_name_maps_back_to_its_option_and_an_unknown_name_to_none() {
    let app = fixture();
    let service = app.global::<ui::KitTimeService>();
    let options = service.invoke_zone_options();
    let paris = service.invoke_zone_selection(options.clone(), "Europe/Paris".into());
    assert!(paris.present);
    let row = options
        .row_data(usize::try_from(paris.token).unwrap())
        .unwrap();
    assert_eq!(row.label, "Europe/Paris");
    assert!(
        !service
            .invoke_zone_selection(options, "Mars/Olympus".into())
            .present
    );
}

#[test]
fn typing_a_lowercase_place_name_finds_the_zone_and_enter_chooses_it() {
    let app = fixture();
    focus_zone(&app);
    for character in "toronto".chars() {
        key(&app, character.to_string());
    }
    assert_eq!(app.get_zone_text(), "", "typing only filters the options");
    key(&app, Key::DownArrow);
    key(&app, Key::Return);
    assert_eq!(app.get_zone_text(), "America/Toronto");
    assert!(app.invoke_report_zoned(), "a chosen zone is valid");
}

#[test]
fn a_zone_set_by_the_host_is_a_valid_choice() {
    let app = fixture();
    app.set_date_text("2026-11-10".into());
    app.set_time_text("09:00".into());
    app.set_zone_text("Europe/Paris".into());
    slint::platform::update_timers_and_animations();
    assert!(app.invoke_report_zoned());
    assert_eq!(app.get_zone_text(), "Europe/Paris");
    app.set_zone_text("Mars/Olympus".into());
    slint::platform::update_timers_and_animations();
    assert!(
        !app.invoke_report_zoned(),
        "a name outside the catalogue stays invalid"
    );
}
