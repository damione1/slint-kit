// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::{FilePickerBinding, FilePickerPorts, event};
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    let picker = FilePickerBinding::new(&app, FilePickerPorts {entries: App::set_entries, state: App::set_request_state, error: App::set_error});
    picker.on_selection(event(&app, |app, files: Vec<slint_kit::FileHandle>| app.set_status(format!("{} selected resources", files.len()).into())));
    app.on_browse(picker.browse_handler());
    app.on_remove(picker.remove_handler());
    app.run()
}
