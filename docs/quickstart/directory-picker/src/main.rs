use slint::ComponentHandle;
use slint_kit::{
    DirectoryHandle, DirectoryRequest, HostCapability, HostError, HostRequestController,
    SelectionOutcome,
};
use std::{cell::RefCell, rc::Rc};
mod ui {
    slint::include_modules!();
}
fn fail(app: &ui::App, error: HostError) {
    app.set_request_state(if error == HostError::Unsupported {
        ui::HostRequestPresentation::Unsupported
    } else {
        ui::HostRequestPresentation::Failed
    });
    app.set_error(format!("{error:?}").into());
}
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    let selected = Rc::new(RefCell::new(None::<DirectoryHandle>));
    let requests = Rc::new(RefCell::new(
        HostRequestController::<DirectoryHandle>::default(),
    ));
    if slint_kit::native::capabilities().directories == HostCapability::Unsupported {
        app.set_request_state(ui::HostRequestPresentation::Unsupported);
    }
    let weak = app.as_weak();
    let shared = requests.clone();
    let folder = selected.clone();
    app.on_browse(move || {
        let Some(app) = weak.upgrade() else {
            return;
        };
        let ticket = shared.borrow_mut().begin();
        app.set_request_state(ui::HostRequestPresentation::Pending);
        app.set_error("".into());
        let weak_async = weak.clone();
        let shared_async = shared.clone();
        let folder = folder.clone();
        if slint::spawn_local(async move {
            let outcome = slint_kit::native::pick_directory(DirectoryRequest {
                title: Some("Choose a project folder".into()),
            })
            .await;
            let Some(app) = weak_async.upgrade() else {
                return;
            };
            let mut request = shared_async.borrow_mut();
            if !request.finish(ticket, outcome) {
                return;
            }
            match request.take_result().unwrap() {
                SelectionOutcome::Selected(value) => {
                    app.set_selected(value.display_name.as_str().into());
                    *folder.borrow_mut() = Some(value);
                    app.set_request_state(ui::HostRequestPresentation::Idle);
                }
                SelectionOutcome::Cancelled => {
                    app.set_request_state(ui::HostRequestPresentation::Cancelled)
                }
                SelectionOutcome::Failed(error) => fail(&app, error),
            }
        })
        .is_err()
        {
            shared.borrow_mut().cancel();
            fail(&app, HostError::Unavailable);
        }
    });
    let weak = app.as_weak();
    app.on_clear(move || {
        if let Some(app) = weak.upgrade() {
            requests.borrow_mut().cancel();
            *selected.borrow_mut() = None;
            app.set_selected("".into());
            app.set_error("".into());
            app.set_request_state(ui::HostRequestPresentation::Idle);
        }
    });
    app.run()
}
