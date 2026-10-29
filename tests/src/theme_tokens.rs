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
    // A zero shadow spread is real geometry (Tailwind's sm and xs key layers),
    // so a forgotten spread cannot be told apart; the literals name every one.
    Spread,
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
            Token::RingStyle | Token::Spread => false,
        }
    }
}

fn shadow(tokens: &mut Vec<(String, Token)>, name: &str, value: KitShadow) {
    let KitShadow { key, ambient } = value;
    for (layer_name, layer) in [("key", key), ("ambient", ambient)] {
        let ShadowLayer {
            offset_y,
            blur,
            spread: _,
            color,
        } = layer;
        let path = format!("elevation.{name}.{layer_name}");
        tokens.push((format!("{path}.offset-y"), Token::Length(offset_y)));
        tokens.push((format!("{path}.blur"), Token::Length(blur)));
        tokens.push((format!("{path}.spread"), Token::Spread));
        tokens.push((format!("{path}.color"), Token::Color(color)));
    }
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
        accent_hover,
        accent_pressed,
        accent_subtle,
        selection,
        on_selection,
        danger,
        danger_hover,
        danger_pressed,
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
        ("accent-hover", accent_hover),
        ("accent-pressed", accent_pressed),
        ("accent-subtle", accent_subtle),
        ("selection", selection),
        ("on-selection", on_selection),
        ("danger", danger),
        ("danger-hover", danger_hover),
        ("danger-pressed", danger_pressed),
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
        icon_stroke,
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
    tokens.push(("shape.icon-stroke".into(), Token::Float(icon_stroke)));

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

/// The shipped theme families. Each allows its own zero tokens.
#[derive(Clone, Copy, Debug)]
enum ThemeFamily {
    Kit,
    Tailwind,
}

/// Tokens every shipped theme may leave at the zero value, with the reason.
const ZERO_ALLOWED: &[(&str, &str)] = &[
    (
        "elevation.xs.ambient.offset-y",
        "shadow-xs is a single layer",
    ),
    ("elevation.xs.ambient.blur", "shadow-xs is a single layer"),
    ("elevation.xs.ambient.color", "shadow-xs is a single layer"),
    (
        "reduced-motion",
        "motion is on until the host reports the preference",
    ),
];
/// Tokens only the Kit themes may leave at the zero value.
const KIT_ZERO_ALLOWED: &[(&str, &str)] = &[
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
];
/// Tokens only the Tailwind themes may leave at the zero value.
const TAILWIND_ZERO_ALLOWED: &[(&str, &str)] = &[(
    "elevation.control",
    "a drop-shadow on a control repeated in a layout inflates its measured height",
)];

fn zero_allowed(
    family: ThemeFamily,
) -> impl Iterator<Item = &'static (&'static str, &'static str)> {
    let own = match family {
        ThemeFamily::Kit => KIT_ZERO_ALLOWED,
        ThemeFamily::Tailwind => TAILWIND_ZERO_ALLOWED,
    };
    ZERO_ALLOWED.iter().chain(own)
}

fn shipped_themes(window: &ThemeTokenFixture) -> [(&'static str, ThemeFamily, KitTheme); 4] {
    [
        ("light", ThemeFamily::Kit, window.get_light()),
        ("dark", ThemeFamily::Kit, window.get_dark()),
        (
            "tailwind-light",
            ThemeFamily::Tailwind,
            window.get_tailwind_light(),
        ),
        (
            "tailwind-dark",
            ThemeFamily::Tailwind,
            window.get_tailwind_dark(),
        ),
    ]
}

#[test]
fn every_shipped_theme_sets_every_token() {
    let window = fixture();
    for (name, family, theme) in shipped_themes(&window) {
        let tokens = tokens(theme);
        let zero: Vec<&str> = tokens
            .iter()
            .filter(|(_, token)| token.is_zero())
            .map(|(path, _)| path.as_str())
            .collect();
        let unexpected: Vec<&&str> = zero
            .iter()
            .filter(|path| !zero_allowed(family).any(|(allowed, _)| allowed == *path))
            .collect();
        assert!(
            unexpected.is_empty(),
            "{name} theme leaves tokens at their zero value: {unexpected:?}"
        );
        // An allowlist entry that no longer applies hides nothing; drop it.
        for (allowed, reason) in zero_allowed(family) {
            assert!(
                zero.contains(allowed),
                "allowlisted token {allowed} ({reason}) is set in {name}; drop the entry"
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
            ButtonIntent::Outline,
            ButtonIntent::Quiet,
            ButtonIntent::Destructive,
        ] {
            let recipe = window.invoke_button(theme.clone(), intent, ControlSize::Medium);
            assert_eq!(recipe.radius, 8.);
            let (border_width, hover, pressed) = match intent {
                ButtonIntent::Primary => (0., theme.accent_hover, theme.accent_pressed),
                ButtonIntent::Destructive => (0., theme.danger_hover, theme.danger_pressed),
                ButtonIntent::Secondary | ButtonIntent::Outline => (1., theme.hover, theme.pressed),
                ButtonIntent::Quiet => (0., theme.hover, theme.pressed),
            };
            assert_eq!(recipe.border_width, border_width, "{intent:?}");
            // Hover and press read explicit tokens, never a derived darkening.
            assert_eq!(recipe.hover_fill.color(), hover, "{intent:?}");
            assert_eq!(recipe.pressed_fill.color(), pressed, "{intent:?}");
            assert_eq!((recipe.busy_mark_size, recipe.busy_mark_stroke), (14., 2.));
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
        let icon = window.invoke_icon_glyph(theme.clone(), ControlSize::Medium);
        assert_eq!(
            (icon.size, icon.color, icon.disabled_color, icon.stroke),
            (16., theme.text_secondary, theme.disabled_text, 2.)
        );
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
        let path = entry.unwrap().path();
        // Public components live at the top level; `themes/` holds token values.
        if path.is_dir() {
            continue;
        }
        let source = std::fs::read_to_string(path).unwrap();
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

#[test]
fn outline_is_a_bordered_transparent_intent_distinct_from_secondary() {
    let window = fixture();
    for theme in [window.get_light(), window.get_dark()] {
        for size in [
            ControlSize::Compact,
            ControlSize::Medium,
            ControlSize::Comfortable,
        ] {
            let outline = window.invoke_button(theme.clone(), ButtonIntent::Outline, size);
            let secondary = window.invoke_button(theme.clone(), ButtonIntent::Secondary, size);
            // Same geometry, so swapping the intent never moves a layout.
            assert_eq!(
                (
                    outline.height,
                    outline.padding_inline,
                    outline.padding_block
                ),
                (
                    secondary.height,
                    secondary.padding_inline,
                    secondary.padding_block
                )
            );
            assert_eq!(
                (outline.radius, outline.border_width, outline.font_size),
                (
                    secondary.radius,
                    secondary.border_width,
                    secondary.font_size
                )
            );
            // Distinct surface: secondary is filled, outline shows its parent.
            assert_eq!(secondary.fill.color(), theme.surface);
            assert_eq!(outline.fill.color().alpha(), 0);
            assert_eq!(outline.disabled_fill.color().alpha(), 0);
            assert_eq!(secondary.border, theme.border_subtle);
            assert_eq!(outline.border, theme.border_strong);
            assert_eq!(outline.foreground, theme.text_primary);
            assert_eq!(outline.disabled_foreground, theme.disabled_text);
        }
    }
}

#[test]
fn field_and_trigger_recipes_render_read_only_and_hover() {
    let window = fixture();
    for theme in [window.get_light(), window.get_dark()] {
        let field = |access, validation| {
            window.invoke_field(theme.clone(), ControlSize::Medium, access, validation)
        };
        let editable = field(FieldAccess::Editable, ValidationPresentation::Pristine);
        assert_eq!(editable.hover_border, theme.border_strong);
        let read_only = field(FieldAccess::ReadOnly, ValidationPresentation::Pristine);
        assert_eq!(read_only.fill.color(), theme.canvas);
        assert_eq!(read_only.foreground, theme.text_primary);
        // A read-only field offers no editing affordance on hover.
        assert_eq!(read_only.hover_border, read_only.border);
        let invalid = field(FieldAccess::Editable, ValidationPresentation::Invalid);
        assert_eq!(invalid.hover_border, theme.danger);
        assert_eq!(editable.placeholder, theme.placeholder);
        assert_eq!(
            (editable.selection, editable.on_selection),
            (theme.selection, theme.on_selection)
        );

        for (access, answers_pointer) in [
            (FieldAccess::Editable, true),
            (FieldAccess::ReadOnly, false),
            (FieldAccess::Disabled, false),
        ] {
            let trigger = window.invoke_trigger(
                theme.clone(),
                ControlSize::Medium,
                access,
                ValidationPresentation::Pristine,
            );
            let surface = field(access, ValidationPresentation::Pristine);
            assert_eq!(trigger.fill, surface.fill, "{access:?}");
            assert_eq!(
                (trigger.radius, trigger.height, trigger.padding_inline),
                (surface.radius, surface.height, surface.padding_inline)
            );
            assert_eq!(
                trigger.hover_fill.color() != trigger.fill.color(),
                answers_pointer,
                "{access:?} hover"
            );
            assert_eq!(
                trigger.pressed_fill.color() != trigger.fill.color(),
                answers_pointer,
                "{access:?} press"
            );
        }
        let invalid_trigger = window.invoke_trigger(
            theme.clone(),
            ControlSize::Medium,
            FieldAccess::Editable,
            ValidationPresentation::Invalid,
        );
        assert_eq!(invalid_trigger.border, theme.danger);
        assert_eq!(invalid_trigger.focus.color, theme.focus.invalid_color);

        let choice = window.invoke_choice(theme.clone());
        assert_eq!((choice.check_size, choice.mixed_width), (14., 12.));
        assert_eq!(
            (choice.group_radius, choice.group_padding, choice.group_gap),
            (10., 3., 6.)
        );
    }
}

#[test]
fn input_group_reports_hover_without_taking_the_press() {
    let window = fixture();
    let group = labelled(&window, "Field outline");
    let inside = group.absolute_position();
    // The group's first row: later siblings cover the rest of its height.
    let point = slint::LogicalPosition::new(inside.x + group.size().width / 2., inside.y + 20.);
    assert!(!window.get_field_hovered());
    window
        .window()
        .dispatch_event(slint::platform::WindowEvent::PointerMoved { position: point });
    assert!(window.get_field_hovered());
    window
        .window()
        .dispatch_event(slint::platform::WindowEvent::PointerMoved {
            position: slint::LogicalPosition::new(470., 350.),
        });
    assert!(!window.get_field_hovered());
}

#[test]
fn read_only_select_keeps_focus_but_never_opens() {
    use slint::platform::{Key, PointerEventButton, WindowEvent};
    let window = fixture();
    let press = |key: Key| {
        window
            .window()
            .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
        window
            .window()
            .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    };
    let trigger = labelled(&window, "Access select");
    for (access, focusable, opens) in [
        (FieldAccess::Editable, true, true),
        (FieldAccess::ReadOnly, true, false),
        (FieldAccess::Disabled, false, false),
    ] {
        // Move focus away first: Slint 1.18.1 ignores FocusOut on a
        // FocusScope that is disabled while focused, leaving has-focus stale.
        window.invoke_focus_button();
        window.set_select_access(access);
        window.set_select_opens(0);
        window.invoke_focus_select();
        assert_eq!(window.get_select_focused(), focusable, "{access:?} focus");
        assert_eq!(
            trigger.accessible_enabled(),
            Some(access != FieldAccess::Disabled),
            "{access:?} enabled"
        );
        if focusable {
            press(Key::DownArrow);
            press(Key::Space);
            press(Key::Return);
        }
        trigger.mock_single_click(PointerEventButton::Left);
        let expected = if opens { 4 } else { 0 };
        assert_eq!(window.get_select_opens(), expected, "{access:?} opens");
    }
}

fn contrast(a: slint::Color, b: slint::Color) -> f64 {
    fn luminance(color: slint::Color) -> f64 {
        let linear = |c: u8| {
            let s = f64::from(c) / 255.;
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        linear(color.red()) * 0.2126
            + linear(color.green()) * 0.7152
            + linear(color.blue()) * 0.0722
    }
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

#[test]
fn every_alert_severity_has_its_own_legible_tint_edge_and_mark() {
    let window = fixture();
    let severities = [
        AlertSeverity::Information,
        AlertSeverity::Success,
        AlertSeverity::Warning,
        AlertSeverity::Error,
    ];
    for (_, _, theme) in shipped_themes(&window) {
        let recipes: Vec<AlertRecipe> = severities
            .iter()
            .map(|severity| window.invoke_alert(theme.clone(), *severity))
            .collect();
        for (index, recipe) in recipes.iter().enumerate() {
            for other in &recipes[index + 1..] {
                assert_ne!(recipe.fill, other.fill, "tints must differ");
                assert_ne!(recipe.border, other.border, "edges must differ");
                assert_ne!(recipe.mark, other.mark, "marks must differ");
            }
            for text in [theme.text_primary, theme.text_secondary] {
                assert!(
                    contrast(text, recipe.fill) >= 4.5,
                    "copy on tint: {}",
                    contrast(text, recipe.fill)
                );
            }
            assert!(contrast(recipe.mark, recipe.fill) >= 3., "mark on tint");
            assert_eq!(recipe.radius, theme.shape.radius_container);
            assert_eq!(recipe.border_width, theme.shape.border_width);
        }
        assert_eq!(recipes[3].border, theme.danger);
        assert_eq!(recipes[0].border, theme.information);
    }
}

#[test]
fn the_selected_tab_carries_an_underline_and_no_outline() {
    i_slint_backend_testing::init_no_event_loop();
    let window = TabsIndicatorFixture::new().unwrap();
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    let theme = window.get_theme_probe();
    let tabs: Vec<ElementHandle> = ElementQuery::from_root(&window)
        .match_inherits("KitNavigationAction")
        .find_all();
    assert_eq!(tabs.len(), 3);
    for tab in &tabs {
        let selected = tab.accessible_label().as_deref() == Some("Activity");
        assert_eq!(tab.accessible_item_selected(), Some(selected));
        let indicator = tab
            .query_descendants()
            .match_predicate(move |e| {
                e.type_name().as_deref() == Some("Rectangle")
                    && (e.size().height - theme.shape.border_width_strong).abs() < 0.01
            })
            .find_first()
            .expect("every tab owns an indicator slot");
        let opacity = indicator.computed_opacity();
        assert_eq!(opacity, if selected { 1. } else { 0. });
        // The indicator spans the tab and sits on its bottom edge.
        assert!((indicator.size().width - tab.size().width).abs() < 0.01);
        assert!((indicator.absolute_position().x - tab.absolute_position().x).abs() < 0.01);
        let bottom = indicator.absolute_position().y + indicator.size().height;
        assert!((bottom - (tab.absolute_position().y + tab.size().height)).abs() < 0.01);
    }
}

#[test]
fn scroll_bar_recipe_reads_theme_tokens() {
    let window = fixture();
    for (_, _, theme) in shipped_themes(&window) {
        let recipe = window.invoke_scroll_bar(theme.clone());
        assert_eq!(recipe.thumb, theme.border_strong);
        assert_eq!(recipe.active_thumb, theme.text_secondary);
        assert_eq!(recipe.active_track, theme.hover);
        assert_eq!(recipe.track.alpha(), 0);
        assert!(recipe.active_thumb_thickness > recipe.thumb_thickness);
        assert!(recipe.thumb_thickness + recipe.inset <= recipe.thickness);
        assert!(
            contrast(recipe.active_thumb, theme.surface) >= 3.,
            "hovered thumb"
        );
        let mut reduced = theme.clone();
        reduced.reduced_motion = true;
        assert_eq!(window.invoke_scroll_bar(reduced).transition, 0);
    }
}

#[test]
fn every_shipped_theme_keeps_text_rings_and_states_legible() {
    let window = fixture();
    for (name, _, theme) in shipped_themes(&window) {
        let pairs = [
            ("text-primary/canvas", theme.text_primary, theme.canvas, 4.5),
            (
                "text-primary/surface",
                theme.text_primary,
                theme.surface,
                4.5,
            ),
            (
                "text-primary/surface-raised",
                theme.text_primary,
                theme.surface_raised,
                4.5,
            ),
            ("text-primary/hover", theme.text_primary, theme.hover, 4.5),
            (
                "text-primary/pressed",
                theme.text_primary,
                theme.pressed,
                4.5,
            ),
            (
                "text-primary/accent-subtle",
                theme.text_primary,
                theme.accent_subtle,
                4.5,
            ),
            (
                "text-secondary/canvas",
                theme.text_secondary,
                theme.canvas,
                4.5,
            ),
            (
                "text-secondary/surface",
                theme.text_secondary,
                theme.surface,
                4.5,
            ),
            (
                "text-secondary/surface-raised",
                theme.text_secondary,
                theme.surface_raised,
                4.5,
            ),
            (
                "text-secondary/hover",
                theme.text_secondary,
                theme.hover,
                4.5,
            ),
            ("placeholder/surface", theme.placeholder, theme.surface, 4.5),
            ("placeholder/canvas", theme.placeholder, theme.canvas, 4.5),
            ("on-accent/accent", theme.on_accent, theme.accent, 4.5),
            (
                "on-accent/accent-hover",
                theme.on_accent,
                theme.accent_hover,
                4.5,
            ),
            (
                "on-accent/accent-pressed",
                theme.on_accent,
                theme.accent_pressed,
                4.5,
            ),
            ("on-accent/danger", theme.on_accent, theme.danger, 4.5),
            (
                "on-accent/danger-hover",
                theme.on_accent,
                theme.danger_hover,
                4.5,
            ),
            (
                "on-accent/danger-pressed",
                theme.on_accent,
                theme.danger_pressed,
                4.5,
            ),
            (
                "on-selection/selection",
                theme.on_selection,
                theme.selection,
                4.5,
            ),
            ("danger/surface", theme.danger, theme.surface, 4.5),
            ("danger/canvas", theme.danger, theme.canvas, 4.5),
            // Rings and status marks are non-text indicators: 3:1.
            ("focus/canvas", theme.focus.color, theme.canvas, 3.),
            ("focus/surface", theme.focus.color, theme.surface, 3.),
            (
                "focus/surface-raised",
                theme.focus.color,
                theme.surface_raised,
                3.,
            ),
            (
                "invalid-focus/surface",
                theme.focus.invalid_color,
                theme.surface,
                3.,
            ),
            ("success/surface", theme.success, theme.surface, 3.),
            ("warning/surface", theme.warning, theme.surface, 3.),
            ("information/surface", theme.information, theme.surface, 3.),
        ];
        for (pair, foreground, background, minimum) in pairs {
            let ratio = contrast(foreground, background);
            assert!(ratio >= minimum, "{name} {pair}: {ratio:.2} < {minimum}");
        }
    }
}

#[test]
fn tailwind_themes_use_role_radii_density_inter_and_shadows() {
    let window = fixture();
    for mut theme in [window.get_tailwind_light(), window.get_tailwind_dark()] {
        assert_eq!(theme.typography.family, "Inter");
        let button =
            window.invoke_button(theme.clone(), ButtonIntent::Outline, ControlSize::Medium);
        assert_eq!(button.radius, 6.);
        assert_eq!(button.border, theme.border_strong);
        let field = window.invoke_field(
            theme.clone(),
            ControlSize::Medium,
            FieldAccess::Editable,
            ValidationPresentation::Pristine,
        );
        assert_eq!(field.radius, 6.);
        for (level, radius, elevation) in [
            (SurfaceLevel::Flat, 8., ElevationLevel::Sm),
            (SurfaceLevel::Raised, 8., ElevationLevel::Sm),
            (SurfaceLevel::Overlay, 12., ElevationLevel::Lg),
            (SurfaceLevel::Popover, 12., ElevationLevel::Md),
        ] {
            let surface = window.invoke_surface(theme.clone(), level);
            assert_eq!(surface.radius, radius, "{level:?}");
            assert_eq!(surface.elevation, elevation, "{level:?}");
            assert!(surface.shadow.color.alpha() > 0, "{level:?} key layer");
            assert!(
                surface.ambient_shadow.color.alpha() > 0,
                "{level:?} ambient layer"
            );
            // The ambient layer sits tighter and closer than the key layer.
            assert!(surface.ambient_shadow.offset_y <= surface.shadow.offset_y);
            assert!(surface.ambient_shadow.blur < surface.shadow.blur);
            assert!(surface.ambient_shadow.spread < 0.);
        }
        for scale in [1., 2.] {
            theme.text_scale = scale;
            for (size, height, padding_inline, padding_block) in [
                (ControlSize::Compact, 32., 12., 6.),
                (ControlSize::Medium, 36., 16., 8.),
                (ControlSize::Comfortable, 40., 24., 8.),
            ] {
                let button = window.invoke_button(theme.clone(), ButtonIntent::Primary, size);
                assert_eq!(button.height, height);
                assert_eq!(
                    (button.padding_inline, button.padding_block),
                    (padding_inline, padding_block)
                );
                // One scaled 20px line never clips inside the block padding.
                assert_eq!(
                    window.invoke_control_height(theme.clone(), size),
                    f32::max(height, 20. * scale + 2. * padding_block)
                );
            }
            assert_eq!(
                window.invoke_font_size(theme.clone(), TypographyRole::Title),
                30. * scale
            );
        }
    }
}

#[test]
fn dialogs_stack_both_shadow_layers_under_their_surface() {
    i_slint_backend_testing::init_no_event_loop();
    let window = ElevationFixture::new().unwrap();
    window.show().unwrap();
    let panel = ElementQuery::from_root(&window)
        .match_inherits("KitDialogPanel")
        .find_first()
        .expect("panel");
    let surface = panel
        .query_descendants()
        .match_inherits("ElevatedSurface")
        .find_first()
        .expect("the panel paints an elevated surface");
    assert_eq!(surface.size(), panel.size());
    assert_eq!(surface.absolute_position(), panel.absolute_position());
    let theme = window.get_tailwind_light();
    let lg = theme.elevation.lg;
    let origin = panel.absolute_position();
    let size = panel.size();
    let rectangles = surface
        .query_descendants()
        .match_type_name("Rectangle")
        .find_all();
    // Ambient, key, then the surface itself on top.
    assert_eq!(rectangles.len(), 3);
    for (rectangle, spread) in rectangles
        .iter()
        .zip([lg.ambient.spread, lg.key.spread, 0.])
    {
        let position = rectangle.absolute_position();
        assert!((position.x - (origin.x - spread)).abs() < 0.01, "{spread}");
        assert!((position.y - (origin.y - spread)).abs() < 0.01, "{spread}");
        assert!((rectangle.size().width - (size.width + 2. * spread)).abs() < 0.01);
        assert!((rectangle.size().height - (size.height + 2. * spread)).abs() < 0.01);
    }
    assert!(lg.ambient.spread < lg.key.spread && lg.key.spread < 0.);
}

#[test]
fn third_party_notices_ship_unchanged_with_the_published_crates() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let notices = std::fs::read_to_string(root.join("THIRD-PARTY-NOTICES.md")).unwrap();
    for copy in [
        "crates/kit-slint/THIRD-PARTY-NOTICES.md",
        "crates/kit/THIRD-PARTY-NOTICES.md",
    ] {
        assert_eq!(
            std::fs::read_to_string(root.join(copy)).unwrap(),
            notices,
            "{copy} must be a copy of THIRD-PARTY-NOTICES.md"
        );
    }
    // The font license travels with the fonts and is quoted verbatim.
    let license =
        std::fs::read_to_string(root.join("crates/kit-slint/ui/themes/fonts/Inter-OFL.txt"))
            .unwrap();
    assert!(notices.contains(&license));
    assert!(license.contains("SIL Open Font License, Version 1.1"));
    assert_eq!(
        license,
        std::fs::read_to_string(root.join("assets/branding/Inter-OFL.txt")).unwrap()
    );
    // Lucide's ISC notice and Feather's MIT notice are quoted verbatim from
    // the vendored snapshot the icon globals are generated from.
    let snapshot = std::fs::read_dir(root.join("vendor/lucide"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.is_dir())
        .unwrap();
    let lucide = std::fs::read_to_string(snapshot.join("LICENSE")).unwrap();
    assert!(notices.contains(lucide.trim_end()));
    assert!(lucide.contains("ISC License") && lucide.contains("Cole Bemis"));
    let version = snapshot.file_name().unwrap().to_string_lossy().into_owned();
    assert!(notices.contains(&format!("vendor/lucide/{version}/")));
}

#[test]
fn every_shipped_theme_draws_lucide_weight_icon_strokes() {
    let window = fixture();
    // Lucide draws 2 units on its 24-unit grid; every shipped theme keeps it,
    // so a scaled stroke is 1.33px at 16px and an absolute one 2px.
    for (name, _, theme) in shipped_themes(&window) {
        assert_eq!(theme.shape.icon_stroke, 2., "{name}");
    }
}
