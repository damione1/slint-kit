<div align="center">

<!-- Match GitHub's selected theme even when it differs from the system theme. -->
<img src="assets/branding/slint-kit-stacked.svg#gh-light-mode-only" alt="Slint Kit" width="240">
<img src="assets/branding/slint-kit-stacked-dark.svg#gh-dark-mode-only" alt="Slint Kit" width="240">

<p><strong>Native components. Your visual language.</strong></p>

<p>A composable Rust and Slint toolkit with typed behavior, native text editing,<br>and themes you can shape around your application.</p>

<p>
  <a href="https://github.com/damione1/slint-kit/actions/workflows/check.yml"><img alt="Check" src="https://github.com/damione1/slint-kit/actions/workflows/check.yml/badge.svg"></a>
  <a href="LICENSE"><img alt="License: GPL v3 or later" src="https://img.shields.io/badge/License-GPLv3%2B-blue.svg"></a>
  <img alt="Rust 1.92 or later" src="https://img.shields.io/badge/Rust-1.92%2B-B7410E?logo=rust&amp;logoColor=white">
  <a href="https://slint.dev"><img alt="Slint 1.18.1" src="https://img.shields.io/badge/Slint-1.18.1-818CF8"></a>
  <img alt="Native UI" src="https://img.shields.io/badge/UI-native-0D9488">
</p>

</div>

Buttons, fields, selections, dialogs, calendars and composable display, navigation and layout primitives. Rust owns validated values and identity; Slint owns their presentation and native interaction.

[Website](https://slint-kit.damiengoehrig.ca/) · [Documentation](https://slint-kit.damiengoehrig.ca/guides/getting-started/) · [Components](https://slint-kit.damiengoehrig.ca/components/)

## Try it

Install the toolkit from [crates.io](https://crates.io/crates/slint-kit) with Rust 1.92 or later and the native dependencies required by Slint:

```toml
[dependencies]
slint-kit = "0.1"
slint = "=1.18.1"

[build-dependencies]
slint-kit-build = "0.1"
```

Compile your Slint UI with `slint_kit_build::compile("ui/app.slint")` in `build.rs`. Import components from `@kit/kit.slint`. See the [installation guide](https://slint-kit.damiengoehrig.ca/guides/getting-started/) or [consumption skill](.agents/skills/slint-kit/SKILL.md) for a complete application.

The showcase is a working consumer included in the source repository. From a local checkout, run `cargo run -p kit-showcase`.

## Contribute

See [AGENTS.md](AGENTS.md) for structure and checks. Run `cargo run -p xtask --locked -- check` before submitting changes. Backend support, IME and assistive technology require native qualification on the target device.

## License

Original implementation under [GNU GPL v3.0 or later](LICENSE). Dependencies retain their own licenses.
