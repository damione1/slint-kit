// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{SliderAccessibilityFixture, ui};
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::ComponentHandle;

fn fixture(install: bool) -> SliderAccessibilityFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = SliderAccessibilityFixture::new().unwrap();
    if install {
        slint_kit::install(&app);
    }
    app.show().unwrap();
    app.window().take_snapshot().unwrap();
    app
}
fn element(app: &SliderAccessibilityFixture, id: &str) -> ElementHandle {
    ElementQuery::from_root(app)
        .match_id(format!("SliderAccessibilityFixture::{id}"))
        .find_first()
        .unwrap()
}
fn thumb(app: &SliderAccessibilityFixture, label: &str) -> ElementHandle {
    let label = label.to_owned();
    element(app, "range")
        .query_descendants()
        .match_predicate(move |element| {
            element.accessible_label().as_deref() == Some(label.as_str())
        })
        .find_first()
        .unwrap()
}
fn bounds(element: &ElementHandle, min: f32, max: f32, step: f32, value: &str) {
    assert_eq!(element.accessible_value_minimum(), Some(min));
    assert_eq!(element.accessible_value_maximum(), Some(max));
    assert_eq!(element.accessible_value_step(), Some(step));
    assert_eq!(element.accessible_value().as_deref(), Some(value));
}

#[test]
fn single_slider_exports_domain_bounds_step_and_numeric_value_with_separate_description() {
    let app = fixture(true);
    let single = element(&app, "single");
    bounds(&single, -12.5, 37.5, 0.25, "12.5");
    assert_eq!(single.accessible_description().as_deref(), Some("12.5 °C"));
    single.invoke_accessible_increment_action();
    assert_eq!(app.get_value(), "12.75");
    bounds(&single, -12.5, 37.5, 0.25, "12.75");
    single.invoke_accessible_decrement_action();
    assert_eq!(app.get_value(), "12.5");
}
#[test]
fn accessible_set_value_rounds_clamps_and_emits_one_exact_commit_only_for_a_change() {
    let app = fixture(true);
    let single = element(&app, "single");
    for (proposed, expected) in [("12.63", "12.75"), ("99", "37.5"), ("-99", "-12.5")] {
        let chosen = app.get_chosen_count();
        let committed = app.get_commit_count();
        let positioned = app.get_position_count();
        single.set_accessible_value(proposed);
        assert_eq!(app.get_value(), expected);
        assert_eq!(app.get_chosen_value(), expected);
        assert_eq!(app.get_chosen_count(), chosen + 1);
        assert_eq!(app.get_commit_count(), committed + 1);
        assert_eq!(app.get_position_count(), positioned + 1);
    }
    let counts = (
        app.get_chosen_count(),
        app.get_commit_count(),
        app.get_position_count(),
    );
    for proposed in ["-12.5", "-12.49", "NaN", "inf", "1e2", "", "twelve"] {
        single.set_accessible_value(proposed);
        assert_eq!(app.get_value(), "-12.5");
        assert_eq!(
            (
                app.get_chosen_count(),
                app.get_commit_count(),
                app.get_position_count()
            ),
            counts
        );
    }
}
#[test]
fn controlled_accessible_set_value_preserves_exact_proposal_until_host_acknowledgement() {
    let app = fixture(true);
    let controlled = element(&app, "controlled");
    controlled.set_accessible_value("1000000000000.266");
    assert_eq!(app.get_controlled_value(), "1000000000000.25");
    assert_eq!(
        controlled.accessible_value().as_deref(),
        Some("1000000000000.25")
    );
    assert_eq!(app.get_chosen_value(), "1000000000000.27");
    assert_eq!(app.get_chosen_count(), 1);
    assert_eq!(app.get_commit_count(), 1);
    app.set_controlled_value(app.get_chosen_value());
    assert_eq!(
        controlled.accessible_value().as_deref(),
        Some("1000000000000.27")
    );
}
#[test]
fn each_range_thumb_uses_the_other_domain_endpoint_and_set_value_cannot_cross_it() {
    let app = fixture(true);
    let lower = thumb(&app, "Interval — lower");
    let upper = thumb(&app, "Interval — upper");
    bounds(&lower, -10., 30., 2., "10");
    bounds(&upper, 10., 50., 2., "30");
    assert_eq!(
        lower.accessible_description().as_deref(),
        Some("Lower bound")
    );
    lower.set_accessible_value("13");
    assert_eq!(app.get_range_value().lower, "14");
    bounds(&upper, 14., 50., 2., "30");
    lower.set_accessible_value("99");
    assert_eq!(
        app.get_range_value(),
        ui::DecimalRangeView {
            lower: "30".into(),
            upper: "30".into()
        }
    );
    bounds(&lower, -10., 30., 2., "30");
    bounds(&upper, 30., 50., 2., "30");
    let chosen = app.get_chosen_count();
    upper.set_accessible_value("-99");
    assert_eq!(app.get_chosen_count(), chosen);
    upper.set_accessible_value("44.9");
    assert_eq!(app.get_range_value().upper, "44");
    bounds(&lower, -10., 44., 2., "30");
    app.set_ownership(ui::Ownership::Controlled);
    lower.set_accessible_value("20");
    assert_eq!(app.get_range_value().lower, "30");
    assert_eq!(app.get_chosen_count(), chosen + 2);
}
#[test]
fn accessible_actions_respect_read_only_disabled_and_uninstalled_services() {
    let app = fixture(false);
    let single = element(&app, "single");
    single.set_accessible_value("25");
    assert_eq!(app.get_value(), "12.5");
    assert_eq!(single.accessible_enabled(), Some(false));
    slint_kit::install(&app);
    for access in [ui::FieldAccess::ReadOnly, ui::FieldAccess::Disabled] {
        app.set_access(access);
        assert_eq!(
            single.accessible_read_only(),
            Some(access == ui::FieldAccess::ReadOnly)
        );
        let before = (
            app.get_value(),
            app.get_range_value(),
            app.get_accepted_color(),
        );
        for element in [
            single.clone(),
            thumb(&app, "Interval — lower"),
            thumb(&app, "Interval — upper"),
        ] {
            element.set_accessible_value("25");
            element.invoke_accessible_increment_action();
            element.invoke_accessible_decrement_action();
        }
        for slider in element(&app, "picker")
            .query_descendants()
            .match_inherits("KitSlider")
            .find_all()
        {
            slider.set_accessible_value("123");
        }
        assert_eq!(
            (
                app.get_value(),
                app.get_range_value(),
                app.get_accepted_color()
            ),
            before
        );
        assert_eq!(app.get_chosen_count(), 0);
        assert_eq!(app.get_commit_count(), 0);
        assert_eq!(app.get_position_count(), 0);
    }
}
#[test]
fn color_channel_sliders_publish_byte_domains_and_accept_accessible_values() {
    let app = fixture(true);
    let sliders = element(&app, "picker")
        .query_descendants()
        .match_inherits("KitSlider")
        .find_all();
    assert_eq!(sliders.len(), 4);
    for (slider, initial) in sliders.iter().zip(["40", "60", "80", "200"]) {
        bounds(slider, 0., 255., 1., initial);
        slider.set_accessible_value("123.6");
        bounds(slider, 0., 255., 1., "124");
    }
    assert_eq!(
        app.get_accepted_color(),
        ui::SrgbColorView {
            red: 124,
            green: 124,
            blue: 124,
            alpha: 124
        }
    );
}
#[test]
fn accessible_actions_keep_controls_visible_in_light_dark_and_doubled_text() {
    let app = fixture(true);
    element(&app, "single").set_accessible_value("20.5");
    thumb(&app, "Interval — lower").set_accessible_value("14");
    for (name, mut theme) in [("light", app.get_theme()), ("dark", app.get_dark_theme())] {
        for scale in [1., 2.] {
            theme.text_scale = scale;
            app.set_theme(theme.clone());
            let frame = app.window().take_snapshot().unwrap();
            for id in ["single", "range"] {
                let host = element(&app, id);
                let size = host.size();
                assert!(size.width > 0. && size.height > 0.);
                assert!(
                    host.absolute_position().y + size.height
                        <= app
                            .window()
                            .size()
                            .to_logical(app.window().scale_factor())
                            .height
                );
            }
            crate::choice_presentation::save_capture(
                &frame,
                &format!("slider-accessible-domain-{name}-{scale}"),
            );
        }
    }
}

#[test]
fn color_field_draft_popup_sliders_use_the_same_byte_domain() {
    let app = fixture(true);
    app.invoke_open_draft();
    app.window().take_snapshot().unwrap();
    let red = ElementQuery::from_root(&app)
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Red draft"))
        .find_first()
        .unwrap();
    bounds(&red, 0., 255., 1., "40");
    red.set_accessible_value("123.6");
    bounds(&red, 0., 255., 1., "124");
    // A popup draft remains separate from the accepted field value.
    assert_eq!(app.get_draft_color().red, 40);
}
