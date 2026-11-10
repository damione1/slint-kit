// SPDX-License-Identifier: GPL-3.0-or-later
slint::include_modules!();
use slint_kit::{KeyedModel, ui::{StepEntry, StepState}};
#[derive(Clone, PartialEq, Eq, Hash)] struct StepId(u16);
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    let rows = [(StepId(101), "Account", "Credentials and security", StepState::Complete, true), (StepId(203), "Workspace", "Project details", StepState::Current, true), (StepId(307), "Invite team", "Available after setup", StepState::Pending, false)].into_iter().map(|(key, label, description, state, enabled)| (key, StepEntry { label: label.into(), description: description.into(), state, enabled, ..Default::default() }));
    let steps = KeyedModel::new(rows).expect("unique step keys");
    app.set_steps(steps.model());
    app.on_step_requested(slint_kit::event2(&app, move |app, token, generation| { if let Some(id) = steps.accept(token, generation) { app.set_status(format!("Step #{} requested by Rust.", id.0).into()); } }));
    app.run()
}
