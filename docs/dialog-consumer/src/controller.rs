// SPDX-License-Identifier: GPL-3.0-or-later
use super::ui;
use slint::ComponentHandle;
use slint_kit::{
    DialogActivity, DialogFocusRequest, DialogFocusSpec, DialogHost, DialogHostPorts,
    DialogLayerSpec, DismissReason, FocusDirection,
};
use std::time::Duration;

// Slint declares the presentation-only layer set once. The generated enum's
// structural equality is total, so it is a stable identity for the host.
impl Eq for ui::DialogLayer {}
pub type Layer = ui::DialogLayer;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    Toolbar,
    Card,
    Name,
    Cancel,
    Archive,
    Keep,
    Confirm,
}

pub type Host = DialogHost<ui::DialogApp, Layer, Target>;
fn status(language: ui::DialogLanguage, project: &str, archived: bool) -> String {
    match (language, archived) {
        (ui::DialogLanguage::English, false) => format!("{project} is active."),
        (ui::DialogLanguage::English, true) => format!("{project} is archived."),
        (ui::DialogLanguage::French, false) => format!("{project} est actif."),
        (ui::DialogLanguage::French, true) => format!("{project} est archivé."),
    }
}

/// The application declares identities, topology and commands; the kit owns
/// focus eligibility, traps, nested restoration and safe publication ordering.
pub fn create_app_with_host(
    language: ui::DialogLanguage,
) -> Result<(ui::DialogApp, Host), slint::PlatformError> {
    let app = ui::DialogApp::new()?;
    slint_kit::install(&app);
    app.set_language(language);
    app.set_status(status(language, &app.get_project_name(), false).into());
    let host = DialogHost::new(
        &app,
        [
            DialogLayerSpec {
                key: Layer::Archive,
                parent: None,
                initial_focus: Target::Name,
            },
            DialogLayerSpec {
                key: Layer::Confirm,
                parent: Some(Layer::Archive),
                initial_focus: Target::Keep,
            },
        ],
        [
            (Target::Toolbar, None),
            (Target::Card, None),
            (Target::Name, Some(Layer::Archive)),
            (Target::Cancel, Some(Layer::Archive)),
            (Target::Archive, Some(Layer::Archive)),
            (Target::Keep, Some(Layer::Confirm)),
            (Target::Confirm, Some(Layer::Confirm)),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, (key, layer))| DialogFocusSpec {
            key,
            layer,
            token: u16::try_from(index + 1).expect("seven targets"),
            enabled: true,
        }),
        DialogHostPorts {
            slots: ui::DialogApp::set_slots,
            state: |app, layer, state| match layer {
                Layer::Archive => app.set_archive(state),
                Layer::Confirm => app.set_confirm(state),
            },
            focus: |app, request| match request {
                DialogFocusRequest::Control(request) => app.set_focus_request(request),
                DialogFocusRequest::Layer(layer) => {
                    app.set_focus_request(ui::FocusRequest::default());
                    app.invoke_focus_layer(layer);
                }
            },
        },
    )
    .expect("valid modal topology");
    app.on_open_requested({
        let host = host.clone();
        move |layer, token| {
            if let Some(target) = host.target(token) {
                host.open(layer, target);
            }
        }
    });
    app.on_dismiss({
        let host = host.clone();
        move |layer, reason| host.close(layer, DismissReason::from(reason))
    });
    app.on_focus_next({
        let host = host.clone();
        move |backward| {
            host.cycle(if backward {
                FocusDirection::Backward
            } else {
                FocusDirection::Forward
            })
        }
    });
    app.on_focus_changed({
        let host = host.clone();
        move |token, focused| host.observe_focus(token, focused)
    });
    app.on_confirm_requested({
        let (weak, host) = (app.as_weak(), host.clone());
        move || {
            if host
                .state(Layer::Confirm)
                .is_none_or(|state| !state.open || !state.topmost || state.busy)
                || !host.eligible(Target::Confirm)
            {
                return;
            }
            host.set_activity(Layer::Confirm, DialogActivity::Busy(Target::Confirm));
            let (weak, host) = (weak.clone(), host.clone());
            slint::Timer::single_shot(Duration::from_millis(1200), move || {
                let Some(app) = weak.upgrade() else {
                    return;
                };
                host.close(Layer::Archive, DismissReason::Programmatic);
                app.set_status(status(language, &app.get_project_name(), true).into());
            });
        }
    });
    Ok((app, host))
}
pub fn create_app(language: ui::DialogLanguage) -> Result<ui::DialogApp, slint::PlatformError> {
    create_app_with_host(language).map(|(app, _)| app)
}
