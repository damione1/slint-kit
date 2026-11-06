// SPDX-License-Identifier: GPL-3.0-or-later
slint::include_modules!();
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Destination { Overview, Activity, Settings }
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    let choices = slint_kit::Choices::new([(Destination::Overview, "Overview"), (Destination::Activity, "Activity"), (Destination::Settings, "Settings")]).expect("unique destination keys");
    app.set_options(choices.model());
    app.set_selected(choices.selection(&Destination::Overview).expect("destination exists"));
    app.on_destination_chosen(move |row| { if let Some(key) = choices.accept(row) { println!("Destination: {key:?}"); } });
    app.run()
}
