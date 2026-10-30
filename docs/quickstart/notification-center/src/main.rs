// SPDX-License-Identifier: GPL-3.0-or-later
use slint::{ComponentHandle, ModelRc, VecModel};
use std::{cell::RefCell, rc::Rc};
mod ui {
    slint::include_modules!();
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Notice {
    SharedDocument,
    ExportReady,
    JoinedWorkspace,
}
impl Notice {
    fn token(self) -> i32 {
        match self {
            Self::SharedDocument => 11,
            Self::ExportReady => 23,
            Self::JoinedWorkspace => 37,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::SharedDocument => "Document shared",
            Self::ExportReady => "Export ready",
            Self::JoinedWorkspace => "Taylor joined the workspace",
        }
    }
}
fn publish(app: &ui::App, notices: &[(Notice, bool)]) {
    app.set_notices(ModelRc::new(VecModel::from(
        notices
            .iter()
            .map(|(key, read)| ui::NotificationRow {
                item: ui::OptionRow {
                    token: key.token(),
                    generation: 1,
                    label: key.label().into(),
                    description: "Review the latest project update.".into(),
                    disabled: false,
                },
                read: *read,
            })
            .collect::<Vec<_>>(),
    )));
}
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    slint_kit::init_form_validation!(ui, app);
    let notices = Rc::new(RefCell::new(vec![
        (Notice::SharedDocument, false),
        (Notice::ExportReady, true),
        (Notice::JoinedWorkspace, false),
    ]));
    publish(&app, &notices.borrow());
    let weak = app.as_weak();
    let state = notices.clone();
    app.on_open_notice(move |token, generation| {
        if generation != 1 {
            return;
        }
        let label = state
            .borrow()
            .iter()
            .find(|(key, _)| key.token() == token)
            .map(|(key, _)| key.label());
        if let (Some(app), Some(label)) = (weak.upgrade(), label) {
            app.set_result(format!("Preview: {label}").into());
        }
    });
    let weak = app.as_weak();
    let state = notices.clone();
    app.on_mark_read(move |token, generation| {
        if generation != 1 {
            return;
        }
        if let Some(app) = weak.upgrade() {
            if let Some((_, read)) = state
                .borrow_mut()
                .iter_mut()
                .find(|(key, _)| key.token() == token)
            {
                *read = true;
            }
            let snapshot = state.borrow().clone();
            publish(&app, &snapshot);
            app.set_result("Notification marked read".into());
        }
    });
    let weak = app.as_weak();
    app.on_dismiss_notice(move |token, generation| {
        if generation != 1 {
            return;
        }
        if let Some(app) = weak.upgrade() {
            notices.borrow_mut().retain(|(key, _)| key.token() != token);
            let snapshot = notices.borrow().clone();
            publish(&app, &snapshot);
            app.set_result("Notification dismissed".into());
        }
    });
    app.run()
}
