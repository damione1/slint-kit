// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::{KeyedModel, event2, ui};
slint::include_modules!();
#[derive(Clone, Debug, PartialEq, Eq, Hash)] struct NoticeId(&'static str);
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    let notices = KeyedModel::new([(NoticeId("review"), ui::NotificationRow {item: ui::OptionRow {label: "Review ready".into(), description: "Your design is ready for review.".into(), ..Default::default()}, read: false}), (NoticeId("release"), ui::NotificationRow {item: ui::OptionRow {label: "Release complete".into(), ..Default::default()}, read: false})]).unwrap();
    app.set_notices(notices.model());
    let read = notices.clone(); app.on_mark_read(move |token, generation| { read.update(token, generation, |row| row.read = true); });
    let open = notices.clone(); app.on_open_notice(event2(&app, move |app, token, generation| { if let Some(key) = open.accept(token, generation) { app.set_result(format!("Opened: {}", key.0).into()); } }));
    app.on_dismiss_notice(move |token, generation| { notices.remove_fenced(token, generation); });
    app.run()
}
