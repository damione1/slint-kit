// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
slint::include_modules!();
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] enum Channel { Email, Push, Sms }
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?; slint_kit::install(&app);
    let choices = slint_kit::Choices::from_options(vec![slint_kit::SelectOption::new(Channel::Email, "Email"), slint_kit::SelectOption::new(Channel::Push, "Push"), slint_kit::SelectOption { disabled: true, ..slint_kit::SelectOption::new(Channel::Sms, "SMS") }]).unwrap();
    app.set_options(choices.model());
    app.set_selected(choices.selections(&[Channel::Email]).unwrap());
    app.on_membership_changed(move |row, added| { if let Some(channel) = choices.accept(row) { println!("{channel:?}: selected={added}"); } });
    app.run()
}
