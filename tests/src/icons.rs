// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{AccessibleRole, ElementHandle, ElementQuery};
use slint::{ComponentHandle, Model};

fn fixture() -> IconFixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = IconFixture::new().unwrap();
    window.show().unwrap();
    window
}

fn labelled(window: &IconFixture, label: &str) -> Option<ElementHandle> {
    ElementHandle::find_by_accessible_label(window, label).next()
}

fn icons(window: &IconFixture) -> Vec<ElementHandle> {
    ElementQuery::from_root(window)
        .match_inherits("KitIcon")
        .find_all()
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.001
}

#[test]
fn every_category_global_carries_the_same_glyphs_as_lucide_icons() {
    let window = fixture();
    let categories = window.get_category_glyphs();
    let all = window.get_all_glyphs();
    let ui = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../crates/kit-slint/ui/icons/lucide");
    let files = std::fs::read_dir(ui)
        .unwrap()
        .filter(|entry| {
            let name = entry.as_ref().unwrap().file_name();
            name != "all.slint" && name.to_string_lossy().ends_with(".slint")
        })
        .count();
    // The fixture imports and calls every category file, so each one compiles.
    assert_eq!(categories.row_count(), files);
    assert_eq!(all.row_count(), files);
    for (index, (category, all)) in categories.iter().zip(all.iter()).enumerate() {
        assert_eq!(category, all, "category glyph {index}");
        assert!(!category.commands.is_empty());
        assert_eq!((category.viewbox, category.stroke), (24., 2.));
    }
    let arrow = window.get_arrow_up();
    assert_eq!(arrow.commands, "M 5 12 L 12 5 L 19 12 M 12 19 L 12 5");
    let check = window.get_kit_check();
    assert_eq!((check.viewbox, check.stroke), (16., 1.7));
}

#[test]
fn icon_size_follows_control_size_and_text_scale() {
    let window = fixture();
    for scale in [1., 2.] {
        let mut theme = window.get_theme();
        theme.text_scale = scale;
        window.set_theme(theme);
        let sizes: Vec<(f32, f32)> = icons(&window)
            .iter()
            .map(|icon| (icon.size().width, icon.size().height))
            .collect();
        // compact, medium, comfortable, decorative medium, then the icon button's.
        let expected = [14., 16., 20., 16., 16.].map(|side| (side * scale, side * scale));
        assert_eq!(sizes, expected, "text scale {scale}");
        let button = labelled(&window, "Delete row").unwrap();
        let icon = icons(&window).pop().unwrap();
        let centre = |e: &ElementHandle| {
            (
                e.absolute_position().x + e.size().width / 2.,
                e.absolute_position().y + e.size().height / 2.,
            )
        };
        let (bx, by) = centre(&button);
        let (ix, iy) = centre(&icon);
        assert!(close(bx, ix) && close(by, iy), "icon centred in its button");
    }
}

#[test]
fn stroke_keeps_each_glyph_grid_ratio_at_every_size() {
    let window = fixture();
    let mut theme = window.get_theme();
    let lucide = window.get_arrow_up();
    let kit = window.get_kit_check();
    // Lucide draws 2 units on 24; the kit glyphs 1.7 on 16.
    for (size, side) in [
        (ControlSize::Compact, 14.),
        (ControlSize::Medium, 16.),
        (ControlSize::Comfortable, 20.),
    ] {
        for scale in [1., 2.] {
            theme.text_scale = scale;
            let lucide_width = window.invoke_stroke_width(theme.clone(), size, lucide.clone());
            let kit_width = window.invoke_stroke_width(theme.clone(), size, kit.clone());
            assert!(
                close(lucide_width, side * scale * 2. / 24.),
                "{size:?} x{scale}"
            );
            assert!(
                close(kit_width, side * scale * 1.7 / 16.),
                "{size:?} x{scale}"
            );
        }
    }
    theme.text_scale = 1.;
    assert!(close(
        window.invoke_stroke_width(theme.clone(), ControlSize::Medium, lucide),
        4. / 3.
    ));
    // A glyph without a grid draws nothing instead of dividing by zero.
    assert_eq!(
        window.invoke_stroke_width(theme, ControlSize::Medium, IconGlyph::default()),
        0.
    );
}

#[test]
fn informative_icons_are_named_images_and_decorative_icons_stay_silent() {
    let window = fixture();
    for label in ["Sort ascending", "Saved", "Warning"] {
        let image = labelled(&window, label).unwrap_or_else(|| panic!("missing {label}"));
        assert_eq!(
            image.accessible_role(),
            Some(AccessibleRole::Image),
            "{label}"
        );
    }
    let images = || {
        ElementQuery::from_root(&window)
            .match_accessible_role(AccessibleRole::Image)
            .find_all()
            .len()
    };
    // The decorative glyph and the icon button's glyph expose no node; the
    // button itself is named by its own accessible name.
    assert_eq!(images(), 3);
    assert_eq!(
        labelled(&window, "Delete row").unwrap().accessible_role(),
        Some(AccessibleRole::Button)
    );
    window.set_arrow_purpose(IconPurpose::Decorative);
    assert!(labelled(&window, "Sort ascending").is_none());
    assert_eq!(images(), 2);
    window.set_arrow_purpose(IconPurpose::Informative);
    assert_eq!(
        labelled(&window, "Sort ascending")
            .unwrap()
            .accessible_role(),
        Some(AccessibleRole::Image)
    );
}
