// SPDX-License-Identifier: GPL-3.0-or-later
use kit_build::config;
use slint_build::DefaultTranslationContext;

fn main() {
    // Catalogues live in lang/<code>/LC_MESSAGES/<crate-name>.po and are compiled into the binary.
    let configuration = config()
        .with_bundled_translations("lang")
        .with_default_translation_context(DefaultTranslationContext::None);
    kit_build::compile_with_config("ui/app.slint", configuration)
        .expect("compile the translations example");
}
