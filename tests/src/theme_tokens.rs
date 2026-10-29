// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::{ComponentHandle, Model, SharedString};

fn fixture() -> ThemeTokenFixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = ThemeTokenFixture::new().unwrap();
    window.show().unwrap();
    window
}

/// One shipped token value, typed by what a forgotten field would become.
enum Token {
    Color(slint::Color),
    Length(f32),
    Float(f32),
    Weight(i32),
    Millis(i64),
    Text(SharedString),
    Flag(bool),
    Elevation(ElevationLevel),
    // Both ring styles are real geometry: no variant stands for "unset".
    RingStyle,
}
impl Token {
    fn is_zero(&self) -> bool {
        match self {
            Token::Color(color) => color.alpha() == 0,
            Token::Length(value) | Token::Float(value) => *value == 0.,
            Token::Weight(value) => *value == 0,
            Token::Millis(value) => *value == 0,
            Token::Text(value) => value.is_empty(),
            Token::Flag(value) => !*value,
            Token::Elevation(level) => *level == ElevationLevel::None,
            Token::RingStyle => false,
        }
    }
}

fn shadow(tokens: &mut Vec<(String, Token)>, name: &str, layer: ShadowLayer) {
    let ShadowLayer {
        offset_y,
        blur,
        color,
    } = layer;
    tokens.push((
        format!("elevation.{name}.offset-y"),
        Token::Length(offset_y),
    ));
    tokens.push((format!("elevation.{name}.blur"), Token::Length(blur)));
    tokens.push((format!("elevation.{name}.color"), Token::Color(color)));
}
fn density(tokens: &mut Vec<(String, Token)>, name: &str, value: KitControlDensity) {
    let KitControlDensity {
        height,
        padding_inline,
        padding_block,
    } = value;
    tokens.push((format!("spacing.{name}.height"), Token::Length(height)));
    tokens.push((
        format!("spacing.{name}.padding-inline"),
        Token::Length(padding_inline),
    ));
    tokens.push((
        format!("spacing.{name}.padding-block"),
        Token::Length(padding_block),
    ));
}

/// Every field of a theme. The destructuring patterns name each field without
/// `..`, so a new token fails to compile here until it is walked.
fn tokens(theme: KitTheme) -> Vec<(String, Token)> {
    let KitTheme {
        canvas,
        surface,
        surface_raised,
        text_primary,
        text_secondary,
        placeholder,
        accent,
        on_accent,
        accent_subtle,
        selection,
        on_selection,
        danger,
        danger_subtle,
        success,
        success_subtle,
        warning,
        warning_subtle,
        information,
        information_subtle,
        border_subtle,
        border_strong,
        hover,
        pressed,
        disabled_fill,
        disabled_text,
        scrim,
        shape,
        elevation,
        focus,
        typography,
        spacing,
        motion,
        reduced_motion,
        text_scale,
    } = theme;
    let mut tokens: Vec<(String, Token)> = [
        ("canvas", canvas),
        ("surface", surface),
        ("surface-raised", surface_raised),
        ("text-primary", text_primary),
        ("text-secondary", text_secondary),
        ("placeholder", placeholder),
        ("accent", accent),
        ("on-accent", on_accent),
        ("accent-subtle", accent_subtle),
        ("selection", selection),
        ("on-selection", on_selection),
        ("danger", danger),
        ("danger-subtle", danger_subtle),
        ("success", success),
        ("success-subtle", success_subtle),
        ("warning", warning),
        ("warning-subtle", warning_subtle),
        ("information", information),
        ("information-subtle", information_subtle),
        ("border-subtle", border_subtle),
        ("border-strong", border_strong),
        ("hover", hover),
        ("pressed", pressed),
        ("disabled-fill", disabled_fill),
        ("disabled-text", disabled_text),
        ("scrim", scrim),
    ]
    .into_iter()
    .map(|(name, color)| (name.to_owned(), Token::Color(color)))
    .collect();

    let KitShape {
        radius_control,
        radius_container,
        radius_overlay,
        radius_pill,
        border_width,
        border_width_strong,
    } = shape;
    for (name, value) in [
        ("shape.radius-control", radius_control),
        ("shape.radius-container", radius_container),
        ("shape.radius-overlay", radius_overlay),
        ("shape.radius-pill", radius_pill),
        ("shape.border-width", border_width),
        ("shape.border-width-strong", border_width_strong),
    ] {
        tokens.push((name.to_owned(), Token::Length(value)));
    }

    let KitElevation {
        control,
        container,
        overlay,
        popover,
        xs,
        sm,
        md,
        lg,
        xl,
    } = elevation;
    for (name, level) in [
        ("elevation.control", control),
        ("elevation.container", container),
        ("elevation.overlay", overlay),
        ("elevation.popover", popover),
    ] {
        tokens.push((name.to_owned(), Token::Elevation(level)));
    }
    for (name, layer) in [("xs", xs), ("sm", sm), ("md", md), ("lg", lg), ("xl", xl)] {
        shadow(&mut tokens, name, layer);
    }

    let KitFocusRing {
        style,
        width,
        offset,
        color,
        invalid_color,
    } = focus;
    tokens.push((
        "focus.style".into(),
        match style {
            FocusRingStyle::Outside | FocusRingStyle::Inside => Token::RingStyle,
        },
    ));
    tokens.push(("focus.width".into(), Token::Length(width)));
    tokens.push(("focus.offset".into(), Token::Length(offset)));
    tokens.push(("focus.color".into(), Token::Color(color)));
    tokens.push(("focus.invalid-color".into(), Token::Color(invalid_color)));

    let KitTypography {
        family,
        weight_regular,
        weight_medium,
        weight_semibold,
        weight_bold,
        size_caption,
        size_control,
        size_body,
        size_subtitle,
        size_title,
        line_height_control,
    } = typography;
    tokens.push(("typography.family".into(), Token::Text(family)));
    for (name, weight) in [
        ("typography.weight-regular", weight_regular),
        ("typography.weight-medium", weight_medium),
        ("typography.weight-semibold", weight_semibold),
        ("typography.weight-bold", weight_bold),
    ] {
        tokens.push((name.to_owned(), Token::Weight(weight)));
    }
    for (name, size) in [
        ("typography.size-caption", size_caption),
        ("typography.size-control", size_control),
        ("typography.size-body", size_body),
        ("typography.size-subtitle", size_subtitle),
        ("typography.size-title", size_title),
        ("typography.line-height-control", line_height_control),
    ] {
        tokens.push((name.to_owned(), Token::Length(size)));
    }

    let KitSpacing {
        unit,
        compact,
        medium,
        comfortable,
    } = spacing;
    tokens.push(("spacing.unit".into(), Token::Length(unit)));
    density(&mut tokens, "compact", compact);
    density(&mut tokens, "medium", medium);
    density(&mut tokens, "comfortable", comfortable);

    let KitMotion { fast, base, slow } = motion;
    tokens.push(("motion.fast".into(), Token::Millis(fast)));
    tokens.push(("motion.base".into(), Token::Millis(base)));
    tokens.push(("motion.slow".into(), Token::Millis(slow)));

    tokens.push(("reduced-motion".into(), Token::Flag(reduced_motion)));
    tokens.push(("text-scale".into(), Token::Float(text_scale)));
    tokens
}

/// Tokens a shipped theme may leave at the zero value, with the reason.
const ZERO_ALLOWED: &[(&str, &str)] = &[
    (
        "typography.family",
        "empty keeps the window's default font family",
    ),
    ("elevation.control", "the default themes draw flat controls"),
    (
        "elevation.container",
        "the default themes draw flat containers",
    ),
    ("elevation.overlay", "the default themes draw flat overlays"),
    ("elevation.popover", "the default themes draw flat popovers"),
    (
        "reduced-motion",
        "motion is on until the host reports the preference",
    ),
];

#[test]
fn every_shipped_theme_sets_every_token() {
    let window = fixture();
    for (name, theme) in [("light", window.get_light()), ("dark", window.get_dark())] {
        let tokens = tokens(theme);
        let zero: Vec<&str> = tokens
            .iter()
            .filter(|(_, token)| token.is_zero())
            .map(|(path, _)| path.as_str())
            .collect();
        let unexpected: Vec<&&str> = zero
            .iter()
            .filter(|path| !ZERO_ALLOWED.iter().any(|(allowed, _)| allowed == *path))
            .collect();
        assert!(
            unexpected.is_empty(),
            "{name} theme leaves tokens at their zero value: {unexpected:?}"
        );
        // An allowlist entry that no longer applies hides nothing; drop it.
        for (allowed, reason) in ZERO_ALLOWED {
            assert!(
                tokens.iter().any(|(path, _)| path == allowed),
                "allowlisted token {allowed} ({reason}) is not a theme token"
            );
        }
    }
}

#[test]
fn components_default_to_the_theme_context_and_keep_local_overrides() {
    let window = fixture();
    let light = window.get_light();
    let dark = window.get_dark();
    assert_eq!(window.get_context_default(), light);
    KitThemeContext::get(&window).set_current(dark.clone());
    assert_eq!(window.get_context_default(), dark);
    assert_eq!(window.get_context_color(), dark.border_subtle);
    assert_eq!(window.get_local_color(), light.border_subtle);
    KitThemeContext::get(&window).set_current(light.clone());
    assert_eq!(window.get_context_color(), light.border_subtle);
}

#[test]
fn metrics_derive_from_tokens_and_follow_text_scale() {
    let window = fixture();
    for mut theme in [window.get_light(), window.get_dark()] {
        assert_eq!(
            window
                .invoke_gaps(theme.clone())
                .iter()
                .collect::<Vec<f32>>(),
            [4., 8., 12., 16., 24.]
        );
        for (space, expected) in [
            (LayoutSpace::None, 0.),
            (LayoutSpace::Tight, 4.),
            (LayoutSpace::Compact, 8.),
            (LayoutSpace::Regular, 16.),
            (LayoutSpace::Roomy, 24.),
            (LayoutSpace::Section, 32.),
        ] {
            assert_eq!(window.invoke_space(theme.clone(), space), expected);
        }
        for scale in [1., 2.] {
            theme.text_scale = scale;
            for (size, height) in [
                (ControlSize::Compact, 32.),
                (ControlSize::Medium, 40.),
                (ControlSize::Comfortable, 48.),
            ] {
                // One scaled 20px line plus 6px block padding on each side.
                let expected = f32::max(height, 20. * scale + 12.);
                assert_eq!(window.invoke_control_height(theme.clone(), size), expected);
                let field = window.invoke_field(
                    theme.clone(),
                    size,
                    FieldAccess::Editable,
                    ValidationPresentation::Pristine,
                );
                assert_eq!(field.height, expected);
                let button = window.invoke_button(theme.clone(), ButtonIntent::Secondary, size);
                assert_eq!(button.height, height);
                assert_eq!(
                    button.padding_inline,
                    if size == ControlSize::Compact {
                        12.
                    } else {
                        16.
                    }
                );
                assert_eq!(field.padding_inline, button.padding_inline);
                assert_eq!(button.padding_block, 6.);
                assert_eq!(
                    button.font_size,
                    if size == ControlSize::Comfortable {
                        16.
                    } else {
                        14.
                    } * scale
                );
                assert_eq!(
                    window.invoke_icon_button(theme.clone(), size).height,
                    expected
                );
            }
            for (role, size) in [
                (TypographyRole::Caption, 12.),
                (TypographyRole::Control, 14.),
                (TypographyRole::Body, 16.),
                (TypographyRole::Subtitle, 20.),
                (TypographyRole::Title, 28.),
            ] {
                assert_eq!(window.invoke_font_size(theme.clone(), role), size * scale);
            }
        }
    }
}

#[test]
fn recipes_reproduce_the_default_geometry() {
    let window = fixture();
    for theme in [window.get_light(), window.get_dark()] {
        for intent in [
            ButtonIntent::Primary,
            ButtonIntent::Secondary,
            ButtonIntent::Quiet,
            ButtonIntent::Destructive,
        ] {
            let recipe = window.invoke_button(theme.clone(), intent, ControlSize::Medium);
            assert_eq!(recipe.radius, 8.);
            assert_eq!(
                recipe.border_width,
                if intent == ButtonIntent::Secondary {
                    1.
                } else {
                    0.
                }
            );
            assert_eq!(recipe.focus, theme.focus);
        }
        assert_eq!(
            window
                .invoke_icon_button(theme.clone(), ControlSize::Medium)
                .radius,
            6.
        );

        let field = window.invoke_field(
            theme.clone(),
            ControlSize::Medium,
            FieldAccess::Editable,
            ValidationPresentation::Pristine,
        );
        assert_eq!(field.fill.color(), theme.surface);
        assert_eq!(field.border, theme.border_subtle);
        assert_eq!((field.radius, field.border_width), (8., 1.));
        assert_eq!(field.focus.color, theme.focus.color);
        let invalid = window.invoke_field(
            theme.clone(),
            ControlSize::Medium,
            FieldAccess::Disabled,
            ValidationPresentation::Invalid,
        );
        assert_eq!(invalid.fill.color(), theme.disabled_fill);
        assert_eq!(invalid.border, theme.danger);
        assert_eq!(invalid.focus.color, theme.focus.invalid_color);
        assert_eq!(invalid.focus.width, theme.focus.width);

        for (level, fill, radius) in [
            (SurfaceLevel::Flat, theme.surface, 12.),
            (SurfaceLevel::Raised, theme.surface_raised, 12.),
            (SurfaceLevel::Overlay, theme.surface_raised, 12.),
            (SurfaceLevel::Popover, theme.surface_raised, 12.),
        ] {
            let surface = window.invoke_surface(theme.clone(), level);
            assert_eq!(surface.fill.color(), fill, "{level:?}");
            assert_eq!(surface.radius, radius, "{level:?}");
            assert_eq!(surface.border, theme.border_subtle);
            assert_eq!(surface.border_width, 1.);
            // The default themes are flat: no shadow is painted.
            assert_eq!(surface.elevation, ElevationLevel::None);
            assert_eq!(surface.shadow.color.alpha(), 0);
        }

        let choice = window.invoke_choice(theme.clone());
        assert_eq!(
            (choice.box_size, choice.box_radius, choice.mark_stroke),
            (20., 5., 2.)
        );
        assert_eq!(
            (choice.track_width, choice.track_height, choice.track_radius),
            (38., 22., 11.)
        );
        assert_eq!((choice.thumb_size, choice.thumb_inset), (18., 2.));
        assert_eq!(
            (choice.slider_track_height, choice.slider_thumb_size),
            (4., 16.)
        );
        assert_eq!(choice.transition, theme.motion.fast);
        assert_eq!(choice.focus.style, FocusRingStyle::Inside);
        let mut reduced = theme.clone();
        reduced.reduced_motion = true;
        assert_eq!(window.invoke_choice(reduced).transition, 0);

        let option = window.invoke_option(theme.clone(), ControlSize::Medium);
        assert_eq!(option.active_fill.color(), theme.pressed);
        assert_eq!(option.active_border, theme.focus.color);
        assert_eq!(option.focus.style, FocusRingStyle::Inside);
        let icon = window.invoke_icon_glyph(theme.clone());
        assert_eq!((icon.size, icon.stroke_width), (16., 1.7));
    }
}

fn outlines(window: &ThemeTokenFixture) -> Vec<ElementHandle> {
    ElementQuery::from_root(window)
        .match_inherits("FocusOutline")
        .find_all()
}
fn labelled(window: &ThemeTokenFixture, label: &str) -> ElementHandle {
    let wanted = label.to_owned();
    ElementQuery::from_root(window)
        .match_predicate(move |e| e.accessible_label().as_deref() == Some(wanted.as_str()))
        .find_first()
        .unwrap_or_else(|| panic!("missing element {label}"))
}
fn assert_ring(outline: &ElementHandle, host: &ElementHandle, outset: f32) {
    let (ring, host_position) = (outline.absolute_position(), host.absolute_position());
    let (ring_size, host_size) = (outline.size(), host.size());
    assert!(
        (ring.x - (host_position.x - outset)).abs() < 0.01,
        "{ring:?}"
    );
    assert!(
        (ring.y - (host_position.y - outset)).abs() < 0.01,
        "{ring:?}"
    );
    assert!((ring_size.width - (host_size.width + 2. * outset)).abs() < 0.01);
    assert!((ring_size.height - (host_size.height + 2. * outset)).abs() < 0.01);
}

#[test]
fn one_focus_outline_serves_buttons_fields_and_inset_hosts() {
    let window = fixture();
    assert!(outlines(&window).is_empty());

    // Programmatic focus is keyboard-visible: offset 2px plus width 2px.
    window.invoke_focus_button();
    let button = labelled(&window, "Focus geometry");
    let rings = outlines(&window);
    assert_eq!(rings.len(), 1);
    assert_ring(&rings[0], &button, 4.);

    // A focused field uses the same ring geometry as a button.
    window.set_field_focused(true);
    let field = labelled(&window, "Field outline");
    let rings = outlines(&window);
    assert_eq!(rings.len(), 2);
    let field_ring = rings
        .iter()
        .find(|ring| (ring.absolute_position().y - (field.absolute_position().y - 4.)).abs() < 0.01)
        .expect("field ring");
    assert_ring(field_ring, &field, 4.);
    window.set_field_focused(false);

    // An inset host paints its ring over its own edge.
    window.invoke_focus_slider();
    let rings = outlines(&window);
    assert_eq!(rings.len(), 1);
    let slider = labelled(&window, "Slider outline");
    let ring = rings[0].size();
    assert_eq!((ring.width, ring.height), (16., 16.));
    let thumb_y = rings[0].absolute_position().y;
    assert!(
        (thumb_y - (slider.absolute_position().y + (slider.size().height - 16.) / 2.)).abs() < 0.01
    );
}

#[test]
fn every_public_component_is_in_the_theme_matrix() {
    let ui = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../crates/kit-slint/ui");
    let mut components = std::collections::BTreeSet::new();
    for entry in std::fs::read_dir(&ui).unwrap() {
        let source = std::fs::read_to_string(entry.unwrap().path()).unwrap();
        for declaration in source.split("export component ").skip(1) {
            if let Some(name) = declaration.split_whitespace().next() {
                components.insert(name.to_owned());
            }
        }
    }
    let kit = std::fs::read_to_string(ui.join("kit.slint")).unwrap();
    let exported: Vec<&str> = kit
        .split("export {")
        .skip(1)
        .filter_map(|list| list.split('}').next())
        .flat_map(|list| list.split(','))
        .map(str::trim)
        .filter(|name| components.contains(*name))
        .collect();
    assert!(
        exported.len() > 100,
        "public component scan found {exported:?}"
    );
    let matrix = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/theme-matrix.slint"),
    )
    .unwrap();
    let missing: Vec<&&str> = exported
        .iter()
        .filter(|name| !matrix.contains(&format!("    {name} {{")))
        .collect();
    assert!(
        missing.is_empty(),
        "add to tests/ui/theme-matrix.slint: {missing:?}"
    );

    // The debug constructor of a window holding every component needs more
    // than the default test-thread stack; release builds do not.
    std::thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(|| {
            i_slint_backend_testing::init_no_event_loop();
            let window = ThemeMatrixFixture::new().unwrap();
            window.show().unwrap();
            window.hide().unwrap();
        })
        .unwrap()
        .join()
        .unwrap();
}
