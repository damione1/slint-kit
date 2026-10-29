<div align="center">

<!-- Match GitHub's selected theme even when it differs from the system theme. -->
<img src="assets/branding/slint-kit-stacked.svg#gh-light-mode-only" alt="Slint Kit" width="240">
<img src="assets/branding/slint-kit-stacked-dark.svg#gh-dark-mode-only" alt="Slint Kit" width="240">

<p><strong>Native components. Your visual language.</strong></p>

<p>A composable Rust and Slint toolkit with typed behavior, native text editing,<br>and themes you can shape around your application.</p>

<p>
  <a href="https://github.com/damione1/slint-kit/actions/workflows/check.yml"><img alt="Check" src="https://github.com/damione1/slint-kit/actions/workflows/check.yml/badge.svg"></a>
  <a href="https://crates.io/crates/slint-kit"><img alt="crates.io version" src="https://img.shields.io/crates/v/slint-kit.svg"></a>
  <a href="LICENSE"><img alt="License: GPL v3 or later" src="https://img.shields.io/badge/License-GPLv3%2B-blue.svg"></a>
  <img alt="Rust 1.92 or later" src="https://img.shields.io/badge/Rust-1.92%2B-B7410E?logo=rust&amp;logoColor=white">
  <a href="https://slint.dev"><img alt="Slint 1.18.1" src="https://img.shields.io/badge/Slint-1.18.1-818CF8"></a>
  <img alt="Native UI" src="https://img.shields.io/badge/UI-native-0D9488">
</p>

</div>

Buttons, fields, selections, dialogs, calendars and composable display, navigation and layout primitives. Rust owns validated values and identity; Slint owns their presentation and native interaction.

[Website](https://slint-kit.dev/) · [Documentation](https://slint-kit.dev/guides/getting-started/) · [Components](https://slint-kit.dev/components/)

## Try it

Install the toolkit from [crates.io](https://crates.io/crates/slint-kit) with Rust 1.92 or later and the native dependencies required by Slint:

```toml
[dependencies]
slint-kit = "0.3"
slint = "=1.18.1"

[build-dependencies]
slint-kit-build = "0.3"
```

Compile your Slint UI with `slint_kit_build::compile("ui/app.slint")` in `build.rs`. Import components from `@kit/kit.slint`. See the [installation guide](https://slint-kit.dev/guides/getting-started/) or [consumption skill](.agents/skills/slint-kit/SKILL.md) for a complete application.


## Build the showcase

The showcase is a working application that demonstrates the public components. Build it from the source repository with Git and **Rust 1.92+**.

Prepare the desktop dependencies for your OS: Xcode Command Line Tools on macOS, an MSVC Rust toolchain and Visual Studio Build Tools on Windows, or the required development libraries on Linux. See the [Slint 1.18.1 build guide](https://github.com/slint-ui/slint/blob/v1.18.1/docs/building.md#prerequisites) for platform setup.

```sh
git clone https://github.com/damione1/slint-kit.git
cd slint-kit
cargo build --release --locked -p kit-showcase
cargo run --release --locked -p kit-showcase
```

## Contribute

See [AGENTS.md](AGENTS.md) for structure and checks. Run `cargo run -p xtask --locked -- check` before submitting changes. Backend support, IME and assistive technology require native qualification on the target device.

## Documentation hosting

Build with `npm --prefix docs ci` and `npm --prefix docs run build`, then deploy the verified output with `npx wrangler deploy --config docs/wrangler.toml --keep-vars`. The build checks native screenshot provenance against the kit sources; publish from a revision whose captures pass this check.

Eleventy writes content-addressed CSS and JavaScript under `/assets/versioned/`, with one year of immutable browser caching. Replaceable screenshots and branding use a one-day cache. HTML keeps the default browser revalidation policy. Cache rules live in `docs/src/_headers` and are copied into the deployed assets.

## License

Original implementation under [GNU GPL v3.0 or later](LICENSE). Dependencies retain their own licenses.
