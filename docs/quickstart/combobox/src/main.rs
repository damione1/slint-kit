// SPDX-License-Identifier: GPL-3.0-or-later
slint::include_modules!();
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Destination { Personal, Shared, Archive, Design, Engineering, Marketing, Research, Community }
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    let choices = slint_kit::Choices::new([(Destination::Personal, "Personal"), (Destination::Shared, "Shared"), (Destination::Archive, "Archive"),
        (Destination::Design, "Design studio"), (Destination::Engineering, "Engineering"), (Destination::Marketing, "Marketing"),
        (Destination::Research, "Research lab"), (Destination::Community, "Community")]).expect("unique destination keys");
    app.set_options(choices.model());
    app.set_selected(choices.selection(&Destination::Personal).expect("destination exists"));
    app.on_destination_chosen(move |row| { if let Some(key) = choices.accept(row) { println!("Destination: {key:?}"); } });
    app.run()
}
