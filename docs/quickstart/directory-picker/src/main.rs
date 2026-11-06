// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::{DirectoryPickerBinding, DirectoryPickerPorts};
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    let picker = DirectoryPickerBinding::new(&app, DirectoryPickerPorts {selected: App::set_selected, state: App::set_request_state, error: App::set_error});
    picker.on_selection(|directory| println!("Directory: {directory:?}"));
    app.on_browse(picker.browse_handler());
    app.on_clear(picker.clear_handler());
    app.run()
}
