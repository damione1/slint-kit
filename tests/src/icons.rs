// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{
    AccessibleRole, ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions,
};
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
        assert!(!category.stroke.is_empty());
        assert_eq!(category.viewbox, 24.);
    }
    let arrow = window.get_arrow_up();
    assert_eq!(arrow.stroke, "M 5 12 L 12 5 L 19 12 M 12 19 L 12 5");
    assert_eq!(arrow.fill, "");
    // tag's dot is filled and also inherits the 2-unit stroke: in both layers.
    let tag = window.get_tag();
    assert!(tag.fill.starts_with("M 8 7.5 C "), "{}", tag.fill);
    assert_eq!(tag.fill.matches('Z').count(), 1);
    assert!(tag.stroke.ends_with(tag.fill.as_str()));
    // The kit glyphs share Lucide's 24-unit grid and so the same stroke token.
    let check = window.get_kit_check();
    assert_eq!(
        (check.stroke.as_str(), check.fill.as_str(), check.viewbox),
        ("M 4.5 12 L 9.75 17.25 L 19.5 6.75", "", 24.)
    );
}

#[test]
fn icon_size_follows_control_size_and_text_scale() {
    let window = fixture();
    for scale in [1., 2.] {
        let mut theme = window.get_theme();
        theme.text_scale = scale;
        window.set_theme(theme);
        // The first row; the second holds the fixed-size geometry icons.
        let row = &icons(&window)[..5];
        let sizes: Vec<(f32, f32)> = row
            .iter()
            .map(|icon| (icon.size().width, icon.size().height))
            .collect();
        // compact, medium, comfortable, decorative medium, then the icon button's.
        let expected = [14., 16., 20., 16., 16.].map(|side| (side * scale, side * scale));
        assert_eq!(sizes, expected, "text scale {scale}");
        let button = labelled(&window, "Delete row").unwrap();
        let icon = &row[4];
        let centre = |e: &ElementHandle| {
            (
                e.absolute_position().x + e.size().width / 2.,
                e.absolute_position().y + e.size().height / 2.,
            )
        };
        let (bx, by) = centre(&button);
        let (ix, iy) = centre(icon);
        assert!(close(bx, ix) && close(by, iy), "icon centred in its button");
    }
}

#[test]
fn stroke_follows_the_theme_token_in_both_modes() {
    let window = fixture();
    let mut theme = window.get_theme();
    let lucide = window.get_arrow_up();
    let kit = window.get_kit_check();
    for (size, side) in [
        (ControlSize::Compact, 14.),
        (ControlSize::Medium, 16.),
        (ControlSize::Comfortable, 20.),
    ] {
        for scale in [1., 2.] {
            theme.text_scale = scale;
            for glyph in [&lucide, &kit] {
                let width =
                    |mode| window.invoke_stroke_width(theme.clone(), size, glyph.clone(), mode);
                // scaled: 2 units on the 24-unit grid; absolute: 2px.
                assert!(
                    close(width(IconStroke::Scaled), side * scale * 2. / 24.),
                    "{size:?} x{scale}"
                );
                assert!(close(width(IconStroke::Absolute), 2.), "{size:?} x{scale}");
            }
        }
    }
    theme.text_scale = 1.;
    assert!(close(
        window.invoke_stroke_width(
            theme.clone(),
            ControlSize::Medium,
            lucide.clone(),
            IconStroke::Scaled
        ),
        4. / 3.
    ));
    // The token is the theme's: a heavier theme draws heavier icons.
    theme.shape.icon_stroke = 3.;
    assert!(close(
        window.invoke_stroke_width(
            theme.clone(),
            ControlSize::Medium,
            lucide.clone(),
            IconStroke::Scaled
        ),
        2.
    ));
    assert!(close(
        window.invoke_stroke_width(
            theme.clone(),
            ControlSize::Medium,
            lucide,
            IconStroke::Absolute
        ),
        3.
    ));
    // A glyph without a grid draws nothing instead of dividing by zero.
    assert_eq!(
        window.invoke_stroke_width(
            theme,
            ControlSize::Medium,
            IconGlyph::default(),
            IconStroke::Scaled
        ),
        0.
    );
}

/// Slint 1.18.1 (`i-slint-core` items/path.rs, `fitted_path_events`) fits a
/// Path's view box into its box shrunk by `stroke-width`, offset by half of
/// it. KitIcon compensates so the view box covers exactly the icon's box.
fn fitted_view_box(path: &ElementHandle, stroke_width: f32) -> (f32, f32, f32, f32) {
    let position = path.absolute_position();
    let size = path.size();
    (
        position.x + stroke_width / 2.,
        position.y + stroke_width / 2.,
        size.width - stroke_width,
        size.height - stroke_width,
    )
}

#[test]
fn both_layers_map_the_view_box_onto_the_icon_in_both_modes() {
    let window = fixture();
    let all = icons(&window);
    let geometry = &all[5..];
    let strokes: Vec<f32> = window.get_geometry_strokes().iter().collect();
    // scaled 16, absolute 16, scaled 24, absolute 24.
    let expected = [16. * 2. / 24., 2., 2., 2.];
    assert_eq!(geometry.len(), expected.len());
    for (index, (icon, (&stroke, expected))) in geometry
        .iter()
        .zip(strokes.iter().zip(expected))
        .enumerate()
    {
        assert!(close(stroke, expected), "icon {index}: {stroke}");
        let paths = icon.query_descendants().match_type_name("Path").find_all();
        // tag has a fill layer; minus has none.
        assert_eq!(paths.len(), if index < 2 { 2 } else { 1 }, "icon {index}");
        let origin = icon.absolute_position();
        let side = icon.size();
        for path in &paths {
            let (x, y, width, height) = fitted_view_box(path, stroke);
            assert!(
                close(x, origin.x) && close(y, origin.y),
                "icon {index}: view box at {x},{y}, icon at {},{}",
                origin.x,
                origin.y
            );
            assert!(
                close(width, side.width) && close(height, side.height),
                "icon {index}: view box {width}x{height}, icon {}x{}",
                side.width,
                side.height
            );
        }
    }
}

/// Ink coverage (0 for white, 1 for black) of each pixel of a snapshot.
struct Coverage {
    width: usize,
    ink: Vec<f32>,
}

impl Coverage {
    fn at(&self, x: usize, y: usize) -> f32 {
        self.ink[y * self.width + x]
    }
    /// Total ink in the columns `x` of rows `rows`.
    fn column(&self, x: usize, rows: std::ops::Range<usize>) -> f32 {
        rows.map(|y| self.at(x, y)).sum()
    }
    /// Total ink in a rectangle of pixels.
    fn area(&self, columns: std::ops::Range<usize>, rows: std::ops::Range<usize>) -> f32 {
        columns.map(|x| self.column(x, rows.clone())).sum()
    }
}

fn render_icons() -> Coverage {
    // Each test thread has its own platform: this one rasterizes headlessly.
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = IconRenderFixture::new().unwrap();
    window.show().unwrap();
    let snapshot = window.window().take_snapshot().unwrap();
    let width = snapshot.width() as usize;
    let ink = snapshot
        .as_slice()
        .iter()
        .map(|pixel| 1. - (f32::from(pixel.r) + f32::from(pixel.g) + f32::from(pixel.b)) / 765.)
        .collect();
    Coverage { width, ink }
}

#[test]
fn rendered_icons_match_their_svg_geometry() {
    let image = render_icons();
    // Straight edges on whole pixels rasterize exactly. The rasterizer
    // approximates round caps and discs, so curved coverage gets a margin.
    let near_within = |actual: f32, expected: f32, margin: f32, what: &str| {
        assert!(
            (actual - expected).abs() <= margin,
            "{what}: {actual} instead of {expected}"
        );
    };
    let near = |actual: f32, expected: f32, what: &str| near_within(actual, expected, 0.02, what);
    let cap = |actual: f32, what: &str| near_within(actual, std::f32::consts::FRAC_PI_2, 0.2, what);
    // minus at (36, 4), 24px: "M5 12h14" with a 2px round-capped stroke
    // covers x 4..20 and y 11..13. Each cap is a half disc of area pi/2 in
    // the column next to the line. Slint's own fitting would start the line
    // 0.58px further right and end it 0.58px earlier, leaving the first and
    // last line columns partly empty.
    let (x, y) = (36, 4);
    let rows = y + 9..y + 15;
    near(image.column(x + 3, rows.clone()), 0., "left of the cap");
    cap(image.column(x + 4, rows.clone()), "left cap");
    for column in x + 5..x + 19 {
        near(image.column(column, rows.clone()), 2., "line");
    }
    cap(image.column(x + 19, rows.clone()), "right cap");
    near(image.column(x + 20, rows.clone()), 0., "right of the cap");
    near(image.at(x + 12, y + 11), 1., "line top row");
    near(image.at(x + 12, y + 12), 1., "line bottom row");
    near(image.at(x + 12, y + 10), 0., "above the line");
    // The same minus at 48px by 24px: the 24-unit view box is centred, 12px in.
    let (x, y) = (4, 32);
    let rows = y + 9..y + 15;
    near(
        image.column(x + 12 + 3, rows.clone()),
        0.,
        "wide: left of the cap",
    );
    cap(image.column(x + 12 + 4, rows.clone()), "wide: left cap");
    cap(image.column(x + 12 + 19, rows.clone()), "wide: right cap");
    // Absolute mode at 48px: the line doubles in length (x 10..38) but keeps a
    // 2px stroke, rows 23 and 24.
    let (x, y) = (100, 4);
    near(image.at(x + 24, y + 22), 0., "absolute: above");
    near(image.at(x + 24, y + 23), 1., "absolute: top row");
    near(image.at(x + 24, y + 24), 1., "absolute: bottom row");
    near(image.at(x + 24, y + 25), 0., "absolute: below");
    let rows = y + 20..y + 28;
    near(
        image.column(x + 8, rows.clone()),
        0.,
        "absolute: left of the cap",
    );
    cap(image.column(x + 9, rows.clone()), "absolute: left cap");
    cap(image.column(x + 38, rows.clone()), "absolute: right cap");
    near(image.column(x + 39, rows), 0., "absolute: right of the cap");
    // The filled square at (68, 4): exactly 16px by 16px of ink from 4 to 20,
    // so the fill layer lands on the same mapping as the stroke layer.
    let (x, y) = (68, 4);
    near(image.area(x..x + 24, y..y + 24), 256., "square area");
    near(
        image.area(x + 4..x + 20, y + 4..y + 20),
        256.,
        "square position",
    );
    // tag at (4, 4): its filled dot at (7.5, 7.5) with the inherited stroke is
    // a solid disc of radius 1.5; the tag body inside its outline stays empty.
    let (x, y) = (4, 4);
    near(image.at(x + 7, y + 7), 1., "tag dot");
    near_within(
        image.area(x + 5..x + 10, y + 5..y + 10),
        9. * std::f32::consts::PI / 4.,
        0.5,
        "tag dot area",
    );
    near(image.at(x + 13, y + 13), 0., "tag body");
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

fn advance(milliseconds: u64) {
    i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(milliseconds));
    slint::platform::update_timers_and_animations();
}

#[test]
fn turning_icons_rest_unrotated_on_the_glyph_of_their_state() {
    i_slint_backend_testing::init_no_event_loop();
    let window = TurningIconFixture::new().unwrap();
    window.show().unwrap();
    // At rest each icon draws its state's glyph without rotation, so a
    // renderer that cannot rotate paths still points the right way.
    assert_eq!(window.get_chevron_glyph(), window.get_down());
    assert_eq!(window.get_disclosure_glyph(), window.get_right());
    assert_eq!(window.get_chevron_rotation(), 0.);
    assert_eq!(window.get_disclosure_rotation(), 0.);

    window.set_open(true);
    // The new glyph appears at once, rotated back onto the old direction:
    // up turned half a turn looks down; down turned a quarter back looks right.
    assert_eq!(window.get_chevron_glyph(), window.get_up());
    assert_eq!(window.get_disclosure_glyph(), window.get_down());
    assert!(close(window.get_chevron_rotation(), -180.));
    assert!(close(window.get_disclosure_rotation(), -90.));
    advance(75);
    let halfway = window.get_chevron_rotation();
    assert!(
        -180. < halfway && halfway < 0.,
        "mid-turn rotation {halfway}"
    );
    advance(200);
    assert_eq!(window.get_chevron_rotation(), 0.);
    assert_eq!(window.get_disclosure_rotation(), 0.);

    window.set_open(false);
    assert_eq!(window.get_chevron_glyph(), window.get_down());
    assert_eq!(window.get_disclosure_glyph(), window.get_right());
    // Right turned a quarter clockwise looks down, the state just left.
    assert!(close(window.get_disclosure_rotation(), 90.));
    advance(300);
    assert_eq!(window.get_disclosure_rotation(), 0.);
}

#[test]
fn turning_icons_snap_with_reduced_motion() {
    i_slint_backend_testing::init_no_event_loop();
    let window = TurningIconFixture::new().unwrap();
    window.show().unwrap();
    let mut theme = window.get_theme();
    theme.reduced_motion = true;
    window.set_theme(theme);
    window.set_open(true);
    advance(0);
    assert_eq!(window.get_chevron_glyph(), window.get_up());
    assert_eq!(window.get_chevron_rotation(), 0.);
    assert_eq!(window.get_disclosure_rotation(), 0.);
}

#[test]
fn chips_mark_removal_with_the_close_glyph() {
    i_slint_backend_testing::init_no_event_loop();
    let window = TurningIconFixture::new().unwrap();
    window.show().unwrap();
    let chip = ElementQuery::from_root(&window)
        .match_inherits("KitChip")
        .find_first()
        .expect("chip");
    assert_eq!(chip.accessible_label().as_deref(), Some("Remove Alpha"));
    let labels: Vec<_> = ElementQuery::from_root(&window)
        .match_type_name("Text")
        .find_all()
        .into_iter()
        .filter_map(|text| text.accessible_value().or_else(|| text.accessible_label()))
        .collect();
    assert_eq!(labels, ["Alpha"], "the label carries no text glyph");
    let icon = ElementQuery::from_root(&window)
        .match_inherits("KitIcon")
        .find_all()
        .into_iter()
        .find(|icon| {
            let (position, chip_position) = (icon.absolute_position(), chip.absolute_position());
            position.y >= chip_position.y && position.x > chip_position.x
        })
        .expect("close glyph inside the chip");
    // The compact icon size, inside the chip's trailing padding.
    assert!(close(icon.size().width, 14.));
    let right = icon.absolute_position().x + icon.size().width;
    assert!(right < chip.absolute_position().x + chip.size().width);
    assert!(window.get_chip_width() > 14.);
}
