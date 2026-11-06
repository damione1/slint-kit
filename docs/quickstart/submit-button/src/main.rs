// SPDX-License-Identifier: GPL-3.0-or-later
slint::include_modules!();
struct ProjectName(String);
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    app.on_submit_draft(slint_kit::event(&app, |app, draft: slint::SharedString| {
        let draft = draft.trim();
        let name = (!draft.is_empty()).then(|| ProjectName(draft.into()));
        app.set_result(name.map_or("Enter a project name before submitting".into(), |name| format!("Accepted in Rust: {}", name.0)).into());
    }));
    app.run()
}
