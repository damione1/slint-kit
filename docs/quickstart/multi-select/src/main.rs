// SPDX-License-Identifier: GPL-3.0-or-later
use slint::{ComponentHandle, ModelRc, VecModel};
use slint_kit::{MultiSelectController, Ownership, SelectOption, NavigationStep};
use std::{cell::RefCell, rc::Rc};
mod ui { slint::include_modules!(); }
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Channel { Email, Push, Sms }
impl Channel {
    fn all() -> [Self; 3] { [Self::Email, Self::Push, Self::Sms] }
    fn token(self) -> i32 { match self { Self::Email => 11, Self::Push => 23, Self::Sms => 38 } }
    fn from_token(token: i32) -> Option<Self> { Self::all().into_iter().find(|key| key.token() == token) }
    fn label(self) -> &'static str { match self { Self::Email => "Email", Self::Push => "Push", Self::Sms => "SMS" } }
}
fn publish(app: &ui::App, controller: &MultiSelectController<Channel>, open: bool) {
    let visible = controller.options().iter().map(|option| option.key).collect::<Vec<_>>();
    let rows = visible.iter().copied().map(|key| ui::OptionRow { token: key.token(), generation: 1, label: key.label().into(), description: "".into(), disabled: false });
    let choices = Channel::all().map(|key| ui::ChoiceRow { token: key.token(), label: key.label().into(), disabled: false, selected: controller.selected().contains(&key) });
    let chips = controller.selected().iter().map(|key| ui::ChipRow { token: key.token(), label: key.label().into(), removable: true }).collect::<Vec<_>>();
    let active = controller.active().copied().unwrap_or(Channel::Email);
    app.set_rows(ModelRc::new(VecModel::from(rows.collect::<Vec<_>>()))); app.set_choices(ModelRc::new(VecModel::from(choices.to_vec()))); app.set_chips(ModelRc::new(VecModel::from(chips)));
    app.set_select_state(ui::SelectState { open, generation: 1, active_index: visible.iter().position(|key| *key == active).map_or(-1, |index| index as i32), active: ui::SelectionRef { present: controller.active().is_some(), token: active.token() }, selected: ui::SelectionRef { present: false, token: -1 }, selection_availability: ui::SelectionAvailability::Absent });
    app.set_result(format!("Selected: {}", controller.selected().iter().map(|key| key.label()).collect::<Vec<_>>().join(", ")).into());
}
fn step(value: ui::NavigationStep) -> NavigationStep { match value { ui::NavigationStep::Next => NavigationStep::Next, ui::NavigationStep::Previous => NavigationStep::Previous, ui::NavigationStep::First => NavigationStep::First, ui::NavigationStep::Last => NavigationStep::Last, ui::NavigationStep::PageNext => NavigationStep::PageNext, ui::NavigationStep::PagePrevious => NavigationStep::PagePrevious } }
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    let controller = Rc::new(RefCell::new(MultiSelectController::new(Channel::all().into_iter().map(|key| SelectOption::new(key, key.label())).collect(), vec![Channel::Email], Ownership::Local, None).expect("unique channel keys")));
    controller.borrow_mut().navigate(NavigationStep::First, 3);
    publish(&app, &controller.borrow(), false);
    let state = controller.clone(); let weak = app.as_weak();
    app.on_open(move || { if let Some(app) = weak.upgrade() { publish(&app, &state.borrow(), true); } });
    let state = controller.clone(); let weak = app.as_weak();
    app.on_close(move || { if let Some(app) = weak.upgrade() { publish(&app, &state.borrow(), false); } });
    let state = controller.clone(); let weak = app.as_weak();
    app.on_navigate(move |direction| { state.borrow_mut().navigate(step(direction), 3); if let Some(app) = weak.upgrade() { publish(&app, &state.borrow(), app.get_select_state().open); } });
    let state = controller.clone(); let weak = app.as_weak();
    app.on_toggle(move |token, generation| {
        if generation != 1 { return; }
        if let Some(key) = Channel::from_token(token) { let _ = state.borrow_mut().toggle(&key); }
        if let Some(app) = weak.upgrade() { publish(&app, &state.borrow(), true); }
    });
    let state = controller.clone(); let weak = app.as_weak();
    let query = Rc::new(RefCell::new(String::new()));
    let draft = query.clone();
    app.on_query(move |text| {
        *draft.borrow_mut() = text.to_string();
        let needle = text.trim().to_lowercase();
        let options = Channel::all().into_iter().filter(|key| key.label().to_lowercase().contains(&needle)).map(|key| SelectOption::new(key, key.label())).collect();
        let mut controller = state.borrow_mut();
        controller.set_options(options).expect("unique channel keys");
        controller.navigate(NavigationStep::First, 3);
        if let Some(app) = weak.upgrade() { publish(&app, &controller, true); }
    });
    let weak = app.as_weak();
    app.on_typeahead(move |text| {
        query.borrow_mut().push_str(&text);
        let text = query.borrow().clone();
        if let Some(app) = weak.upgrade() { app.invoke_query(text.into()); }
    });
    let weak = app.as_weak();
    app.on_remove(move |token| {
        if let Some(key) = Channel::from_token(token) { let accepted = controller.borrow().selected().iter().copied().filter(|selected| *selected != key).collect(); let _ = controller.borrow_mut().set_selected(accepted); }
        if let Some(app) = weak.upgrade() { publish(&app, &controller.borrow(), app.get_select_state().open); }
    });
    app.run()
}
