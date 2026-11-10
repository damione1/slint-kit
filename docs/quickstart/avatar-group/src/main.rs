// SPDX-License-Identifier: GPL-3.0-or-later
slint::include_modules!();
use slint_kit::{KeyedModel, ui::{AvatarEntry, Tone}};
#[derive(Clone, PartialEq, Eq, Hash)] struct ProfileId(u16);
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new()?;
    slint_kit::install(&app);
    let rows = [(ProfileId(101), "Elliot Alderson", "EA", Tone::Accent), (ProfileId(203), "Darlene Alderson", "DA", Tone::Success), (ProfileId(307), "Angela Moss", "AM", Tone::Information), (ProfileId(409), "Tyrell Wellick", "TW", Tone::Warning)].into_iter().map(|(id, label, initials, tone)| (id, AvatarEntry { label: label.into(), initials: initials.into(), tone, ..Default::default() }));
    let members = KeyedModel::new(rows)?;
    app.set_entries(members.model());
    app.on_profile_requested(slint_kit::event2(&app, move |app, token, generation| { if let Some(id) = members.accept(token, generation) { app.set_status(format!("Profile #{} opened by Rust.", id.0).into()); } }));
    app.on_more_requested(slint_kit::action(&app, |app| app.set_status("More profiles requested by Rust.".into())));
    app.run()?; Ok(())
}
