# Slint Kit

Use strict domain types: enums for finite policies and states, newtypes for IDs and validated values. Keep money and decimal arithmetic exact; renderer floats are projections. Application identity is a stable key, never a visual row index.

Use familiar web control behavior as the default interaction reference, including numeric input filtering, keyboard navigation, selection, disabled/read-only states and form actions. Preserve native editing and accessibility; document and verify platform differences instead of silently inventing a different interaction. Intrinsic form rules belong in the shared HTML constraint engine; email/URL examples must not accept arbitrary nonempty strings. Initialize validation once per window and guard Rust submission paths as well. Document shared properties and composition primitives distinctly, with compatibility links derived from actual inherited APIs.

## Structure

- `crates/kit-core`: domain values, state machines and generation guards; no UI backend.
- `crates/kit-slint`: public `.slint` components and weak-window Rust adapters.
- `crates/kit-build`: register `@kit` imports and verify generated contracts.
- `crates/kit`: consumer facade; optional `native` feature exposes host services.
- `crates/kit-native`: native host integration and explicit capability outcomes.
- `showcase`: consume public APIs as an application would.
- `tests`: public component, keyboard, pointer, identity and geometry checks.

For app integration, read [.agents/skills/slint-kit/SKILL.md](.agents/skills/slint-kit/SKILL.md). Use public Slint APIs and `@kit` imports; the testing backend is the intentional test-only exception. Preserve native text selection, caret, undo and IME. Keep component geometry intrinsic unless its caller requests filling available space. Use a virtualized model/view for large collections.

## Change and verify

Define domain enums shared by Rust and Slint through `contract_enums!` in `kit-core`; regenerate with `cargo run -p xtask -- generate-contracts`. Do not hand-edit `shared-contracts.slint`. Define presentation-only enums once in Slint and reuse their generated Rust types, as with `ControlSize`.

Run `cargo run -p xtask --locked -- check`. For UI changes, exercise the actual interaction and inspect native screenshots in light/dark themes and at 200% text. Report backend, device and accessibility qualification honestly; a passing headless test does not qualify IME or a screen reader.

Keep implementation original and licensed `GPL-3.0-or-later`. Design references inspire behavior and component coverage; they do not authorize copying code or assets. Preserve notices belonging to dependencies.

## Integration POC and repository permissions

Homeward is the first concrete integration POC for this kit, which was built independently from Slint experience in another project. Treat integration findings as evidence for improving the core. Keep the default UI simple and the primitives extensible and customizable. Complex components, including the initial home-automation component set, belong in the private Slint Kit Pro repository; they may extend these primitives or be implemented directly when appropriate. Application transport, real installation identity and server authorization remain in the host application.

Fix confirmed kit defects in this repository, with a suitable regression check; Homeward consumes the correction rather than hiding the defect in application code. Distinguish kit defects from application and upstream Slint defects before choosing the source to change.

For this campaign Damien authorizes local commits here, but no push. Record tested source revisions and keep the existing official gate.
