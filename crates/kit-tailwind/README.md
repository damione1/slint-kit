# Slint Kit Tailwind

Optional presentation pack for Slint Kit, GPL-3.0-or-later. This 0.4 API is in development.
Add `slint-kit-tailwind` beside `slint-kit-build` in build-dependencies, using matching versions.
Call `slint_kit_tailwind::compile("ui/app.slint")` in build.rs.
Import `TailwindThemes` from `@kit-tailwind/theme.slint` and assign its light/dark value to `KitThemeContext.current`.
For application libraries, pass their aliases to `config_with_libraries` and compile with slint-build.
Inter is optional: import `@kit-tailwind/fonts/inter.slint` and set the Window's `default-font-family: "Inter"`.
Without that import the pack embeds no font and retains the application's default font family.
Migration: the former `@kit/themes/tailwind.slint` import has moved; the core package contains no Tailwind assets.
