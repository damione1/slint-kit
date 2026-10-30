// SPDX-License-Identifier: GPL-3.0-or-later
use crate::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::ComponentHandle;

#[test]
fn local_presentation_keeps_state_resolution_and_siblings_independent() {
    i_slint_backend_testing::init_no_event_loop();
    let app = ProductStyleFixture::new().unwrap();
    app.show().unwrap();
    assert_eq!(app.get_button_radius(), 3.);
    let sibling = app.get_sibling_radius();
    let width = app.get_button_width();
    let height = app.get_button_height();
    app.set_busy(true);
    assert_eq!(
        app.get_button_width(),
        width,
        "busy reserves both icon and label geometry"
    );
    assert_eq!(app.get_button_height(), height);
    app.set_local_padding(30.);
    assert!(app.get_button_height() > height);
    assert_eq!(app.get_sibling_radius(), sibling);
    app.set_local_radius(17.);
    assert_eq!(app.get_button_radius(), 17.);
    assert_eq!(app.get_sibling_radius(), sibling);
    app.set_validation(ValidationPresentation::Invalid);
    assert!(app.get_field_invalid());
    app.set_access(FieldAccess::ReadOnly);
    assert!(
        app.get_field_invalid(),
        "local geometry must not bypass validation"
    );
    let field = ElementHandle::find_by_accessible_label(&app, "Local field")
        .next()
        .unwrap();
    assert!(field.size().height > 0.);
    let before = app.get_choice_height();
    let nav_before = app.get_navigation_height();
    app.set_local_size(30.);
    assert!(app.get_navigation_height() > nav_before);
    assert!(
        app.get_choice_height() > before,
        "inherited choice text-size participates in measurement"
    );
}

#[test]
fn icons_and_explicit_theme_regions_follow_their_own_context() {
    i_slint_backend_testing::init_no_event_loop();
    let app = ProductStyleFixture::new().unwrap();
    app.show().unwrap();
    let global = app.global::<KitThemeContext>();
    let light = app.get_theme();
    let dark = app.get_nested_theme();
    assert_eq!(app.get_explicit_child_theme(), dark);
    assert_eq!(app.get_popup_theme(), dark);
    assert_eq!(app.get_implicit_child_theme(), light);
    assert_eq!(app.get_sister_theme(), light);
    global.set_current(dark.clone());
    assert_eq!(app.get_implicit_child_theme(), dark);
    assert_eq!(app.get_sister_theme(), dark);
    assert_eq!(app.get_explicit_child_theme(), dark);
    assert_eq!(app.get_popup_theme(), dark);
    // The explicit fixture controls still use their supplied light theme.
    assert_eq!(
        app.get_icon_button_fill(),
        slint::Brush::SolidColor(light.danger)
    );
    let action = ElementQuery::from_root(&app)
        .match_id("ProductStyleFixture::action")
        .find_first()
        .unwrap();
    let icons: Vec<_> = action
        .query_descendants()
        .match_inherits("KitIcon")
        .find_all();
    assert_eq!(icons.len(), 2);
    let with_icons = app.get_button_width();
    app.set_leading_icon(IconGlyph::default());
    app.set_trailing_icon(IconGlyph::default());
    assert!(app.get_button_width() < with_icons);
}

#[test]
fn shipped_icon_sets_are_complete_and_the_optional_pack_replaces_semantic_roles() {
    i_slint_backend_testing::init_no_event_loop();
    let app = ThemeTokenFixture::new().unwrap();
    fn roles(set: KitIconSet) -> [IconGlyph; 15] {
        let KitIconSet {
            close,
            check,
            minus,
            plus,
            chevron_down,
            chevron_up,
            chevron_left,
            chevron_right,
            eye,
            eye_hidden,
            search,
            information,
            warning,
            bell,
            article,
        } = set;
        [
            close,
            check,
            minus,
            plus,
            chevron_down,
            chevron_up,
            chevron_left,
            chevron_right,
            eye,
            eye_hidden,
            search,
            information,
            warning,
            bell,
            article,
        ]
    }
    for theme in [
        app.get_light(),
        app.get_dark(),
        app.get_tailwind_light(),
        app.get_tailwind_dark(),
    ] {
        for glyph in roles(theme.icons) {
            assert!(glyph.viewbox > 0.);
            assert!(!glyph.stroke.is_empty() || !glyph.fill.is_empty());
        }
    }
    assert_eq!(roles(app.get_light().icons), roles(app.get_dark().icons));
    assert_eq!(
        roles(app.get_tailwind_light().icons),
        roles(app.get_tailwind_dark().icons)
    );
    assert_ne!(
        app.get_light().icons.search,
        app.get_tailwind_light().icons.search
    );
    assert_ne!(
        app.get_light().icons.close,
        app.get_tailwind_light().icons.close
    );
}

#[test]
fn unlabelled_divider_paints_one_continuous_line() {
    i_slint_backend_testing::init_no_event_loop();
    let app = ProductStyleFixture::new().unwrap();
    app.show().unwrap();
    let divider = ElementQuery::from_root(&app)
        .match_id("ProductStyleFixture::divider")
        .find_first()
        .unwrap();
    let lines = divider
        .query_descendants()
        .match_type_name("Rectangle")
        .find_all();
    assert_eq!(lines.len(), 2);
    let painted: f32 = lines.iter().map(|line| line.size().width).sum();
    assert!(
        (painted - divider.size().width).abs() < 0.01,
        "an empty label must not leave a central gap"
    );
}
