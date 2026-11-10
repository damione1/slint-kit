// SPDX-License-Identifier: GPL-3.0-or-later
slint::include_modules!();
use slint_kit::{ KeyedModel, ui::{ButtonGroupEntry, ButtonIntent}, };
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Command { Back, Next, }
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    let actions = KeyedModel::new([(Command::Back, "Back"), (Command::Next, "Next")].map( |(key, text)| { ( key, ButtonGroupEntry { text: text.into(), intent: ButtonIntent::Outline, ..Default::default() }, ) }, )) .expect("unique commands");
    app.set_actions(actions.model());
    app.on_action_requested(slint_kit::event2(&app, move |app, token, generation| {
        if let Some(command) = actions.accept(token, generation) { app.set_status(format!("Rust received {command:?}.").into()); }
    }));
    app.run()
}
