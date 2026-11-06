// SPDX-License-Identifier: GPL-3.0-or-later
slint::include_modules!();
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Section { Permissions, Notifications }
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    let choices = slint_kit::Choices::from_options(vec![
        slint_kit::SelectOption { key: Section::Permissions, label: "Project permissions".into(), description: "Members can read and edit all project notes.".into(), disabled: false },
        slint_kit::SelectOption { key: Section::Notifications, label: "Notification settings".into(), description: "Choose how you receive updates.".into(), disabled: false },
    ]).expect("unique section keys");
    app.set_options(choices.model());
    app.on_section_expanded(move |row, expanded| { if let Some(key) = choices.accept(row) { println!("Section {key:?}: expanded={expanded}"); } });
    app.run()
}
