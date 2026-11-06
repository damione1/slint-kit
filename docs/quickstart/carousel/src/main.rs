// SPDX-License-Identifier: GPL-3.0-or-later
slint::include_modules!();
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Destination { Alpine, Desert, Coast }
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    let choices = slint_kit::Choices::new([(Destination::Alpine, "Alpine morning"), (Destination::Desert, "Desert light"), (Destination::Coast, "Coastal calm")]).expect("unique destination keys");
    app.set_options(choices.model());
    app.set_selected(choices.selection(&Destination::Alpine).expect("destination exists"));
    app.set_desert_key(choices.selection(&Destination::Desert).expect("destination exists"));
    app.set_coast_key(choices.selection(&Destination::Coast).expect("destination exists"));
    app.on_destination_chosen(move |row| { if let Some(key) = choices.accept(row) { println!("Destination: {key:?}"); } });
    app.run()
}
