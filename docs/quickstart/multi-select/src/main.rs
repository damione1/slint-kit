// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
slint::include_modules!();
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] enum Channel { Email, Push, Sms, Desktop, Inbox, Digest, Webhook, Calendar }
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?; slint_kit::install(&app);
    let choices = slint_kit::Choices::new([(Channel::Email, "Email"), (Channel::Push, "Push"), (Channel::Sms, "SMS"), (Channel::Desktop, "Desktop"), (Channel::Inbox, "In-app inbox"), (Channel::Digest, "Weekly digest"), (Channel::Webhook, "Webhook"), (Channel::Calendar, "Calendar")]).unwrap();
    app.set_options(choices.model());
    app.set_selected(choices.selections(&[Channel::Email, Channel::Push]).unwrap());
    app.on_membership_changed(move |row, added| { if let Some(channel) = choices.accept(row) { println!("{channel:?}: selected={added}"); } });
    app.run()
}
