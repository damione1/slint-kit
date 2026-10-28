// SPDX-License-Identifier: GPL-3.0-or-later
use slint::{ComponentHandle, ModelRc, VecModel};
use std::{cell::{Cell, RefCell}, rc::Rc};
mod ui { slint::include_modules!(); }
#[derive(Clone, Copy, PartialEq, Eq)]
struct TagId(i32);
#[derive(Clone)]
struct Tag { id: TagId, label: String }
fn publish(app: &ui::App, tags: &[Tag]) {
    app.set_chips(ModelRc::new(VecModel::from(tags.iter().map(|tag| ui::ChipRow { token: tag.id.0, label: tag.label.as_str().into(), removable: true }).collect::<Vec<_>>())));
}
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    let tags = Rc::new(RefCell::new(vec![Tag { id: TagId(1), label: "Design".into() }]));
    let next = Cell::new(2_i32);
    publish(&app, &tags.borrow());
    let weak = app.as_weak();
    let state = tags.clone();
    app.on_create(move |draft| { if let Some(app) = weak.upgrade() {
        let label = draft.trim();
        if label.is_empty() || label.chars().count() > 40 { app.set_error("Use 1 to 40 characters".into()); return; }
        if state.borrow().iter().any(|tag| tag.label == label) { app.set_error("Tag already exists".into()); return; }
        let token = next.get();
        next.set(token.checked_add(1).expect("tag identity capacity"));
        state.borrow_mut().push(Tag { id: TagId(token), label: label.into() });
        let snapshot = state.borrow().clone();
        publish(&app, &snapshot); app.set_draft("".into()); app.set_error("".into());
    } });
    let weak = app.as_weak();
    app.on_remove(move |token| { if let Some(app) = weak.upgrade() {
        tags.borrow_mut().retain(|tag| tag.id != TagId(token));
        let snapshot = tags.borrow().clone(); publish(&app, &snapshot);
    } });
    app.run()
}
