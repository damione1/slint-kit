// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::{KeyedModel, event2, event3, ui};
slint::include_modules!();
#[derive(Clone, Debug, PartialEq, Eq, Hash)] struct LineId(i32);
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?; slint_kit::install(&app);
    let text = ["# Project notes", "A shared notebook for the autumn workspace project.", "Select a line below to refine the plan.", "## The goal", "Help small teams turn scattered ideas into a clear plan.", "Keep the first release focused, calm, and easy to learn.", "[x] Collect interview notes and everyday frustrations", "[x] Build the first interactive prototype", "[ ] Invite the team to the design review", "[ ] Walk through the full flow with the keyboard", "[ ] Check contrast in light and dark themes", "[ ] Respect reduced-motion preferences", "## Ready to share", "Capture what we learn and choose the next small improvement.", "Good software leaves room for people to think."];
    let rows = KeyedModel::new(text.into_iter().zip(1..).map(|(text, number)| (LineId(number), ui::TextLine {number, text: text.into(), ..Default::default()}))).unwrap();
    rows.bind(&app, App::set_lines, App::set_generation, App::set_active_index); rows.bind_active(&app, |app, row| { app.set_draft(row.map_or_else(Default::default, |row| row.text)); app.set_error("".into()); });
    app.on_choose(rows.activation(|key| println!("Selected line: {key:?}"))); app.on_move(rows.navigation());
    let edit = rows.clone(); app.on_save(event3(&app, move |app, token, generation, text: slint::SharedString| { if text.contains(['\n', '\r']) { app.set_error("Enter one line only".into()); return; } if edit.update_active(token, generation, |row| row.text = text).is_some() { app.set_status("Line saved in Rust".into()); } }));
    app.on_copied(event2(&app, move |app, token, generation| { if rows.accept(token, generation).is_some() { app.set_status("Selected line copied".into()); } }));
    app.run()
}
