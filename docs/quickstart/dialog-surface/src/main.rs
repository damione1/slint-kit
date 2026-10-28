// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::{OverlayStack, OverlaySpec, OverlayId, DismissReason, CloseResult};
mod ui { slint::include_modules!(); }
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    let overlays = std::rc::Rc::new(std::cell::RefCell::new(OverlayStack::new()));
    let weak = app.as_weak();
    let stack = overlays.clone();
    app.on_open(move || {
        if let Some(app) = weak.upgrade() {
            if stack.borrow().is_empty() {
                stack.borrow_mut().push(OverlaySpec::new(OverlayId(1), None, true)).expect("single modal");
                app.set_showing(true);
                app.invoke_focus_dialog();
            }
        }
    });
    let weak = app.as_weak();
    let stack = overlays.clone();
    app.on_close(move |reason| {
        let reason = match reason {
            ui::CloseReason::Escape => DismissReason::Escape,
            ui::CloseReason::OutsidePress => DismissReason::OutsidePress,
            ui::CloseReason::SystemBack => DismissReason::SystemBack,
            ui::CloseReason::TriggerToggle => DismissReason::TriggerToggle,
            ui::CloseReason::Programmatic => DismissReason::Programmatic,
            ui::CloseReason::HostClosed => DismissReason::HostClosed,
        };
        let closed = matches!(stack.borrow_mut().request_close(reason), CloseResult::Closed(_));
        if closed { if let Some(app) = weak.upgrade() { app.set_showing(false); app.invoke_focus_opener(); } }
    });
    let weak = app.as_weak();
    app.on_archive(move || { if let Some(app) = weak.upgrade() { app.set_result("Project archived".into()); app.invoke_close(ui::CloseReason::Programmatic); } });
    app.run()
}
