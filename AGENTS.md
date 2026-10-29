# Slint Kit

These rules apply to every change, including pull requests from contributors and their coding agents. A fix or a new component follows the same design as the existing components; a change that works but diverges from it is not accepted.

Use familiar web control behavior as the default interaction reference, including numeric input filtering, keyboard navigation, selection, disabled/read-only states and form actions. Preserve native editing and accessibility; document and verify platform differences instead of silently inventing a different interaction. Intrinsic form rules belong in the shared HTML constraint engine; email/URL examples must not accept arbitrary nonempty strings. Initialize validation once per window and guard Rust submission paths as well. Document shared properties and composition primitives distinctly, with compatibility links derived from actual inherited APIs.

The kit is standalone: never name or depend on a consuming application. A defect found while integrating the kit is fixed here, with a regression check, rather than hidden in application code. Distinguish kit defects from application and upstream Slint defects before choosing what to change.

## Structure

- `crates/kit-core`: domain values, state machines and generation guards; no UI backend.
- `crates/kit-slint`: public `.slint` components and weak-window Rust adapters.
- `crates/kit-build`: register `@kit` imports and verify generated contracts.
- `crates/kit`: consumer facade; optional `native` feature exposes host services.
- `crates/kit-native`: native host integration and explicit capability outcomes.
- `showcase`: consume public APIs as an application would.
- `tests`: public component, keyboard, pointer, identity and geometry checks.

For app integration, read [.agents/skills/slint-kit/SKILL.md](.agents/skills/slint-kit/SKILL.md). Use public Slint APIs and `@kit` imports; the testing backend is the intentional test-only exception. Preserve native text selection, caret, undo and IME. Keep component geometry intrinsic unless its caller requests filling available space. Use a virtualized model/view for large collections.

## Strict types, decided early

Be precise once, in the type, so the compiler checks every consumer.

1. **Parse once at the boundary.** Input becomes a typed value where it enters (Rust adapter, parser, controller); pass the typed value on instead of re-validating at call sites or discarding what the parser produced. What the type cannot express is validated once, at that boundary.
2. **Closed sets are enums, never strings.** Variants, intents, sizes, states, policies and modes are enums. Domain enums shared by Rust and Slint are declared once through `contract_enums!` in `kit-core` and regenerated with `cargo run -p xtask -- generate-contracts`; presentation-only enums are declared once in Slint and reused through their generated Rust types, as with `ControlSize`. Strings remain only for labels, user text and external input.
3. **Open sets are newtypes, not enums.** Identifiers, keys and validated values use newtypes so they cannot be mixed with other strings. Application identity is a stable key, never a visual row index. No speculative variants, no `All` or "reserved" sentinels.
4. **Make invalid states unrepresentable.** Replace clusters of booleans, or a mode plus optional fields, with an enum whose variants carry only their valid data. A boolean is yes or no; a real third state gets a name (`Option` or a dedicated enum, as `FieldAccess` does for editable/read-only/disabled).
5. **Exhaustive matching.** Match every variant of a domain enum; no `_ =>` or catch-all arm, so adding a variant breaks the build at every site to update. External enums built to evolve are mapped into a domain enum through an explicit conversion with a deliberate unknown-value policy.
6. **Exact arithmetic.** Money and decimals stay exact; renderer floats are projections only.
7. **The compiler finds the impact.** Patch the sites the Rust and Slint compilers report, rerun, iterate. Never silence a diagnostic to get green (`#[allow]`, `as` casts that hide a type problem, wildcard arms, unused fallbacks): fix the type.
8. **Generated files are never hand-edited.** `shared-contracts.slint` and every file whose header says it is generated come from their generator; change the source and regenerate.
9. **Types do not prove behavior.** Keyboard, pointer, focus, identity and geometry still need tests.

## Component design

- **Behavior and presentation stay separate.** Interaction lives in shared primitives and Rust controllers; appearance comes from the theme. A component never hard-codes colors, radii, borders, shadows, focus rings, typography, spacing or durations: it reads them from `KitTheme` tokens and the recipe of its family. A missing token is added to the theme (and to every shipped theme) rather than written as a literal.
- **Themes are complete.** Every shipped theme is a complete struct literal; omitted Slint struct fields silently become zero, so the theme completeness test must stay green.
- **Reuse before adding.** Extend the existing primitive, recipe or enum of the family before creating a parallel one. Follow the property naming, callbacks and composition (`@children`, explicit header/body/footer parts) of neighbouring components.
- **Icons** come from the generated icon set and `KitIcon`; do not inline path data in a component.
- **Every state renders.** Hover, pressed, focus, disabled, read-only, invalid, selected and open states are visible in light and dark themes and at 200% text.

## Change and verify

Run `cargo run -p xtask --locked -- check` before submitting; it formats, verifies generated files, runs the tests and clippy with warnings denied, and checks for hard-coded design values. New behavior comes with a test in `tests` (keyboard, pointer, identity or geometry as relevant). For UI changes, exercise the actual interaction and inspect the rendering in light/dark themes and at 200% text. Report backend, device and accessibility qualification honestly; a passing headless test does not qualify IME or a screen reader.

## Documentation

A change to a public API is not complete until its documentation is updated in the same pull request, in English and French:

- the component reference in `docs/content/components.json` (properties, callbacks, states, accessibility);
- the example in `docs/examples/<slug>.slint` and its French counterpart in `docs/examples/fr/`;
- the affected guides in `docs/src/en/guides/` and `docs/src/fr/guides/`;
- the integration skill [.agents/skills/slint-kit/SKILL.md](.agents/skills/slint-kit/SKILL.md) when integration guidance changes;
- the changelog, naming every breaking change.

Documentation in this repository addresses developers who use the kit. Do not add maintainer runbooks or notes about the kit's own development process.

## License

Keep implementation original and licensed `GPL-3.0-or-later`. Design references inspire behavior and component coverage; they do not authorize copying code or assets. Preserve notices belonging to dependencies and bundled assets.
