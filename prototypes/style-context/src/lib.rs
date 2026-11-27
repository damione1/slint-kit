// SPDX-License-Identifier: GPL-3.0-or-later
//! Prototype P0 of issue 446: a mutable style context held by a library global.
#![cfg(test)]
mod ui {
    pub use slint_kit::ui::*;
    slint::include_modules!();
}
use slint::ComponentHandle;
use std::{cell::RefCell, rc::Rc};
use ui::*;

type Log = Rc<RefCell<Vec<f32>>>;

fn start() -> (App, Log) {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    // The software renderer draws real frames for `take_snapshot`.
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = App::new().unwrap();
    let log = Log::default();
    let sink = log.clone();
    app.global::<Recorder>().on_radius(move |value| {
        sink.borrow_mut().push(value);
        value
    });
    app.show().unwrap();
    pump();
    (app, log)
}

// One event loop turn: timers, animations, then queued `changed` handlers.
fn pump() {
    slint::platform::update_timers_and_animations();
}

fn frame(app: &App) -> Vec<u8> {
    app.window().take_snapshot().unwrap().as_bytes().to_vec()
}

fn default_radius(app: &App) -> f32 {
    app.global::<KitThemeContext>()
        .get_current()
        .shape
        .radius_container
}

fn differing(a: &[u8], b: &[u8]) -> usize {
    a.chunks(4).zip(b.chunks(4)).filter(|(a, b)| a != b).count()
}

fn pixel(frame: &[u8], (x, y): (usize, usize)) -> [u8; 4] {
    let start = (y * 200 + x) * 4;
    frame[start..start + 4].try_into().unwrap()
}

const CANVAS: (usize, usize) = (190, 110);
const PROBE: (usize, usize) = (60, 40);
const LIGHT_ACCENT: slint::Color = slint::Color::from_rgb_u8(0xe2, 0xa2, 0x2f);
const DARK_ACCENT: slint::Color = slint::Color::from_rgb_u8(0xe9, 0xae, 0x55);

#[test]
fn slint_mutates_a_three_level_field_of_the_library_global() {
    let (app, _) = start();
    let default = default_radius(&app);
    assert_ne!(default, 20.);
    assert_eq!(app.get_probe_radius(), default);
    let before = frame(&app);
    app.invoke_mutate_from_slint();
    assert_eq!(app.get_probe_radius(), 20.);
    assert_eq!(
        app.global::<KitStyleContext>()
            .get_alert()
            .base
            .r#box
            .radius,
        20.
    );
    assert_ne!(
        differing(&frame(&app), &before),
        0,
        "the drawn probe follows the mutation"
    );
}

#[test]
fn rust_mutates_a_three_level_field_of_the_library_global() {
    let (app, _) = start();
    let context = app.global::<KitStyleContext>();
    let before = frame(&app);
    let mut styles = context.get_alert();
    styles.base.r#box.radius = 7.;
    styles.base.title.family = "Proto Sans".into();
    context.set_alert(styles);
    assert_eq!(app.get_probe_radius(), 7.);
    assert_eq!(app.get_probe_family(), "Proto Sans");
    assert_ne!(
        differing(&frame(&app), &before),
        0,
        "the drawn probe follows the mutation"
    );
    // A later mutation from Slint starts from the value Rust wrote.
    app.invoke_mutate_from_slint();
    assert_eq!(app.get_probe_radius(), 20.);
    assert_eq!(app.get_probe_family(), "Proto Sans");
}

#[test]
fn a_field_mutation_detaches_the_style_from_the_tokens_until_reset() {
    let (app, _) = start();
    let theme = app.global::<KitThemeContext>();
    let mut tokens = theme.get_current();
    tokens.shape.radius_container = 31.;
    theme.set_current(tokens.clone());
    assert_eq!(
        app.get_probe_radius(),
        31.,
        "an untouched context follows the tokens"
    );

    app.invoke_mutate_from_slint();
    tokens.shape.radius_container = 33.;
    tokens.typography.family = "Proto Sans".into();
    theme.set_current(tokens);
    pump();
    assert_eq!(app.get_probe_radius(), 20.);
    assert_ne!(
        app.get_probe_family(),
        "Proto Sans",
        "stale: the default binding is gone"
    );

    app.global::<KitStyleContext>().invoke_reset();
    assert_eq!(app.get_probe_radius(), 33., "reset drops the override");
    assert_eq!(app.get_probe_family(), "Proto Sans");
}

#[test]
fn reset_then_mutations_in_a_changed_handler_draw_no_intermediate_frame() {
    let (app, log) = start();
    let identity = app.global::<Identity>();
    identity.invoke_apply();
    let light = frame(&app);
    assert_eq!(app.get_probe_radius(), 12.);
    assert_eq!(app.get_probe_frame(), LIGHT_ACCENT);
    let light_fill = app.get_probe_fill();
    log.borrow_mut().clear();

    identity.set_dark(true);
    // The handler is queued: tokens and styles are both still the old ones.
    assert_eq!(app.get_probe_frame(), LIGHT_ACCENT);
    assert_eq!(app.get_probe_fill(), light_fill);
    assert_eq!(identity.get_applies(), 1);

    // An event loop turn runs the handler, then the frame is drawn.
    pump();
    assert_eq!(identity.get_applies(), 2);
    let first = frame(&app);
    pump();
    assert_eq!(
        differing(&first, &frame(&app)),
        0,
        "the first frame is the final one"
    );
    assert_eq!(
        differing(&first, &light),
        200 * 120,
        "canvas, probe and bar all changed"
    );
    assert_eq!(app.get_probe_radius(), 12.);
    assert_eq!(app.get_probe_frame(), DARK_ACCENT);
    assert_ne!(app.get_probe_fill(), light_fill);
    assert_eq!(identity.get_applies(), 2);
    // Between reset() and the mutations the radius is the kit default. No
    // binding was evaluated there.
    assert!(!log.borrow().is_empty());
    assert!(
        log.borrow().iter().all(|radius| *radius == 12.),
        "{:?}",
        log.borrow()
    );
}

// Observed with the software renderer: it reads the window background before
// it runs the queued handlers, so that one frame mixes old and new values.
#[test]
fn a_draw_that_runs_the_queued_handler_itself_keeps_the_old_window_background() {
    let (app, log) = start();
    let identity = app.global::<Identity>();
    identity.invoke_apply();
    let light = frame(&app);
    log.borrow_mut().clear();

    identity.set_dark(true);
    let first = frame(&app);
    assert_eq!(identity.get_applies(), 2, "drawing ran the queued handler");
    assert_eq!(
        pixel(&first, CANVAS),
        pixel(&light, CANVAS),
        "old window background"
    );
    assert_ne!(pixel(&first, PROBE), pixel(&light, PROBE), "new probe fill");
    let second = frame(&app);
    assert_ne!(pixel(&second, CANVAS), pixel(&light, CANVAS));
    assert_eq!(pixel(&second, PROBE), pixel(&first, PROBE));
    assert!(
        log.borrow().iter().all(|radius| *radius == 12.),
        "{:?}",
        log.borrow()
    );
}

#[test]
fn a_synchronous_apply_leaves_nothing_queued_for_the_first_frame() {
    let (app, _) = start();
    let identity = app.global::<Identity>();
    identity.invoke_apply();
    let light = frame(&app);

    identity.invoke_show_dark(true);
    assert_eq!(app.get_probe_frame(), DARK_ACCENT);
    let first = frame(&app);
    pump();
    assert_eq!(differing(&first, &frame(&app)), 0);
    assert_eq!(differing(&first, &light), 200 * 120);
}

#[test]
fn tokens_written_directly_from_rust_leave_a_mixed_state_until_the_handler_runs() {
    let (app, _) = start();
    let identity = app.global::<Identity>();
    identity.set_follow_tokens(true);
    identity.invoke_apply();
    pump();
    let applies = identity.get_applies();
    let light_fill = app.get_probe_fill();

    let theme = app.global::<KitThemeContext>();
    let dark = app.global::<KitThemes>().get_dark();
    theme.set_current(dark);
    // Readable from Rust before the handler: new tokens, old styles.
    assert_eq!(app.get_probe_fill(), light_fill);
    let first = frame(&app);
    pump();
    assert_eq!(
        differing(&first, &frame(&app)),
        0,
        "no frame is drawn from the mixed state"
    );
    assert_ne!(app.get_probe_fill(), light_fill);
    assert_eq!(app.get_probe_radius(), 12.);
    assert!(
        identity.get_applies() - applies <= 2,
        "{}",
        identity.get_applies() - applies
    );
}

#[test]
fn a_token_written_from_rust_after_init_needs_a_second_apply() {
    let (app, _) = start();
    let identity = app.global::<Identity>();
    identity.invoke_apply();
    let theme = app.global::<KitThemeContext>();
    let mut tokens = theme.get_current();
    tokens.typography.family = "Proto Sans".into();
    theme.set_current(tokens);
    pump();
    assert_ne!(
        app.get_probe_family(),
        "Proto Sans",
        "stale without a second apply"
    );

    // A bare second reset() takes the token but drops the overrides.
    app.global::<KitStyleContext>().invoke_reset();
    assert_eq!(app.get_probe_family(), "Proto Sans");
    assert_eq!(app.get_probe_radius(), default_radius(&app));

    identity.invoke_apply();
    assert_eq!(app.get_probe_family(), "Proto Sans");
    assert_eq!(app.get_probe_radius(), 12.);

    // The token was written outside the apply sequence, so a theme switch loses it.
    identity.set_dark(true);
    pump();
    assert_ne!(app.get_probe_family(), "Proto Sans");
}

#[test]
fn a_rust_owned_token_inside_the_apply_sequence_needs_one_order_only() {
    let (app, _) = start();
    let identity = app.global::<Identity>();
    identity.invoke_apply();
    identity.set_family("Proto Sans".into());
    let first = frame(&app);
    pump();
    assert_eq!(differing(&first, &frame(&app)), 0);
    assert_eq!(app.get_probe_family(), "Proto Sans");
    assert_eq!(app.get_probe_radius(), 12.);
    assert_eq!(identity.get_applies(), 2);

    identity.set_dark(true);
    pump();
    assert_eq!(
        app.get_probe_family(),
        "Proto Sans",
        "the token survives a theme switch"
    );
    assert_eq!(app.get_probe_frame(), DARK_ACCENT);
    assert_eq!(identity.get_applies(), 3);
}

#[test]
fn following_the_tokens_reapplies_without_a_loop() {
    let (app, _) = start();
    let identity = app.global::<Identity>();
    identity.set_follow_tokens(true);
    identity.invoke_apply();
    pump();
    let applies = identity.get_applies();

    let theme = app.global::<KitThemeContext>();
    let mut tokens = theme.get_current();
    tokens.typography.family = "Proto Sans".into();
    theme.set_current(tokens);
    pump();
    assert_eq!(app.get_probe_family(), "Proto Sans");
    assert_eq!(app.get_probe_radius(), 12.);
    let after_token = identity.get_applies();
    assert!(after_token - applies <= 2, "{}", after_token - applies);

    // Settled: further turns apply nothing.
    pump();
    pump();
    assert_eq!(identity.get_applies(), after_token);

    identity.set_dark(true);
    pump();
    assert_eq!(app.get_probe_frame(), DARK_ACCENT);
    assert!(
        identity.get_applies() - after_token <= 2,
        "{}",
        identity.get_applies() - after_token
    );
}

// Fallback: a `pure callback` of a library global, assigned from Rust.
#[test]
fn fallback_a_style_function_assigned_from_rust() {
    let (app, _) = start();
    let default = default_radius(&app);
    assert_eq!(
        app.get_hook_radius(),
        default,
        "the kit handler is the default"
    );

    let weak = app.as_weak();
    let radius = Rc::new(std::cell::Cell::new(12.));
    let shared = radius.clone();
    app.global::<KitStyleHooks>().on_alert_styles(move |theme| {
        let mut styles = weak
            .unwrap()
            .global::<KitStyles>()
            .invoke_alert_styles(theme);
        styles.base.r#box.radius = shared.get();
        styles
    });
    assert_eq!(
        app.get_hook_radius(),
        12.,
        "assigning the handler re-evaluates the binding"
    );
    let theme = app.global::<KitThemeContext>();
    let mut tokens = theme.get_current();
    tokens.typography.family = "Proto Sans".into();
    theme.set_current(tokens);
    // Follows the tokens with no reset and keeps the override.
    assert_eq!(app.get_hook_family(), "Proto Sans");
    assert_eq!(app.get_hook_radius(), 12.);
    radius.set(16.);
    assert_eq!(
        app.get_hook_radius(),
        12.,
        "stale: Rust state is not a dependency"
    );
    let mut tokens = theme.get_current();
    tokens.text_scale += 0.25;
    theme.set_current(tokens);
    assert_eq!(app.get_hook_radius(), 16.);
}

// Added after the kit grew `KitBox.min-height`: the application inherits the
// kit default through reset() and its own overrides stay in place.
#[test]
fn a_field_added_to_a_shared_atom_reaches_the_application_through_reset() {
    let (app, _) = start();
    app.global::<Identity>().invoke_apply();
    let styles = app.global::<KitStyleContext>().get_alert();
    let tokens = app.global::<KitThemeContext>().get_current();
    assert_ne!(tokens.spacing.medium.height, 0.);
    assert_eq!(styles.base.r#box.min_height, tokens.spacing.medium.height);
    assert_eq!(styles.base.r#box.radius, 12.);
}
