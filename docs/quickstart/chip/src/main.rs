// SPDX-License-Identifier: GPL-3.0-or-later
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::rc::Rc;
mod ui { slint::include_modules!(); }
fn tags() -> Vec<ui::ChipRow> {
    [(11, "Design"), (23, "Research"), (38, "Accessibility"), (52, "Rust"), (67, "Mobile"), (81, "Documentation")]
        .into_iter().map(|(token, label)| ui::ChipRow { token, label: label.into(), removable: true }).collect()
}
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    slint_kit::init_form_validation!(ui, app);
    let chips = Rc::new(VecModel::from(tags()));
    app.set_chips(ModelRc::from(chips.clone()));
    let model = chips.clone();
    app.on_remove(move |token| {
        let index = (0..model.row_count()).find(|index| model.row_data(*index).is_some_and(|chip| chip.token == token));
        if let Some(index) = index { model.remove(index); }
    });
    app.on_reset(move || chips.set_vec(tags()));
    app.run()
}
