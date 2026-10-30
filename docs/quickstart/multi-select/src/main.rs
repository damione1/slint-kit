// SPDX-License-Identifier: GPL-3.0-or-later
use slint::{ComponentHandle, ModelRc, VecModel};
use slint_kit::{MultiSelectController, Ownership, SelectOption, NavigationStep};
use std::{cell::RefCell, rc::Rc};
mod ui { slint::include_modules!(); }
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Channel { Email, Push, Sms, Desktop, InApp, Digest, Webhook, Calendar }
impl Channel {
    fn all() -> [Self; 8] { [Self::Email, Self::Push, Self::Sms, Self::Desktop, Self::InApp, Self::Digest, Self::Webhook, Self::Calendar] }
    fn token(self) -> i32 { match self { Self::Email => 11, Self::Push => 23, Self::Sms => 38, Self::Desktop => 52, Self::InApp => 67, Self::Digest => 81, Self::Webhook => 94, Self::Calendar => 108 } }
    fn from_token(token: i32) -> Option<Self> { Self::all().into_iter().find(|key| key.token() == token) }
    fn label(self) -> &'static str { match self { Self::Email => "Email", Self::Push => "Push", Self::Sms => "SMS", Self::Desktop => "Desktop", Self::InApp => "In-app inbox", Self::Digest => "Weekly digest", Self::Webhook => "Webhook", Self::Calendar => "Calendar" } }
}
fn publish(app: &ui::App, controller: &mut MultiSelectController<Channel>, query: &str, open: bool) {
    let needle = query.trim().to_lowercase();
    let options = Channel::all().into_iter().filter(|key| !controller.selected().contains(key) && key.label().to_lowercase().contains(&needle)).map(|key| SelectOption::new(key, key.label())).collect();
    controller.set_options(options).expect("unique channel keys");
    if controller.active().is_none() { controller.navigate(NavigationStep::First, 6); }
    let visible = controller.options().iter().map(|option| option.key).collect::<Vec<_>>();
    let rows = visible.iter().copied().map(|key| ui::OptionRow { token: key.token(), generation: 1, label: key.label().into(), description: "Available channel".into(), disabled: false }).collect::<Vec<_>>();
    let chips = controller.selected().iter().map(|key| ui::ChipRow { token: key.token(), label: key.label().into(), removable: true }).collect::<Vec<_>>();
    let active = controller.active().copied();
    app.set_rows(ModelRc::new(VecModel::from(rows)));
    app.set_chips(ModelRc::new(VecModel::from(chips)));
    app.set_select_state(ui::SelectState { open, generation: 1, active_index: active.and_then(|key| visible.iter().position(|row| *row == key)).map_or(-1, |index| i32::try_from(index).expect("bounded option count")), active: ui::SelectionRef { present: active.is_some(), token: active.map_or(-1, Channel::token) }, selected: ui::SelectionRef { present: false, token: -1 }, selection_availability: ui::SelectionAvailability::Absent });
    app.set_result(format!("{} channels selected: {}", controller.selected().len(), controller.selected().iter().map(|key| key.label()).collect::<Vec<_>>().join(", ")).into());
}
fn step(value: ui::NavigationStep) -> NavigationStep { match value { ui::NavigationStep::Next => NavigationStep::Next, ui::NavigationStep::Previous => NavigationStep::Previous, ui::NavigationStep::First => NavigationStep::First, ui::NavigationStep::Last => NavigationStep::Last, ui::NavigationStep::PageNext => NavigationStep::PageNext, ui::NavigationStep::PagePrevious => NavigationStep::PagePrevious } }
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    slint_kit::init_form_validation!(ui, app);
    let controller = Rc::new(RefCell::new(MultiSelectController::new(Channel::all().into_iter().map(|key| SelectOption::new(key, key.label())).collect(), vec![Channel::Email, Channel::Push], Ownership::Local, None).expect("unique channel keys")));
    let query = Rc::new(RefCell::new(String::new()));
    publish(&app, &mut controller.borrow_mut(), "", false);
    let state = controller.clone(); let draft = query.clone(); let weak = app.as_weak();
    app.on_open(move || { if let Some(app) = weak.upgrade() { publish(&app, &mut state.borrow_mut(), &draft.borrow(), true); } });
    let state = controller.clone(); let draft = query.clone(); let weak = app.as_weak();
    app.on_close(move || { if let Some(app) = weak.upgrade() { publish(&app, &mut state.borrow_mut(), &draft.borrow(), false); } });
    let state = controller.clone(); let draft = query.clone(); let weak = app.as_weak();
    app.on_navigate(move |direction| { state.borrow_mut().navigate(step(direction), 6); if let Some(app) = weak.upgrade() { publish(&app, &mut state.borrow_mut(), &draft.borrow(), app.get_select_state().open); } });
    let state = controller.clone(); let draft = query.clone(); let weak = app.as_weak();
    app.on_toggle(move |token, generation| {
        if generation != 1 { return; }
        if let Some(key) = Channel::from_token(token) { let _ = state.borrow_mut().toggle(&key); }
        if let Some(app) = weak.upgrade() { publish(&app, &mut state.borrow_mut(), &draft.borrow(), true); }
    });
    let state = controller.clone(); let draft = query.clone(); let weak = app.as_weak();
    app.on_query(move |text| {
        *draft.borrow_mut() = text.to_string();
        if let Some(app) = weak.upgrade() { publish(&app, &mut state.borrow_mut(), &draft.borrow(), true); }
    });
    let weak = app.as_weak(); let draft = query.clone();
    app.on_typeahead(move |text| {
        draft.borrow_mut().push_str(&text);
        let text = draft.borrow().clone();
        if let Some(app) = weak.upgrade() { app.invoke_query(text.into()); }
    });
    let weak = app.as_weak();
    app.on_remove(move |token| {
        if let Some(key) = Channel::from_token(token) { let accepted = controller.borrow().selected().iter().copied().filter(|selected| *selected != key).collect(); let _ = controller.borrow_mut().set_selected(accepted); }
        if let Some(app) = weak.upgrade() { publish(&app, &mut controller.borrow_mut(), &query.borrow(), app.get_select_state().open); }
    });
    app.run()
}
