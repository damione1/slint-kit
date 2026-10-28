// SPDX-License-Identifier: GPL-3.0-or-later
use slint::{ComponentHandle, ModelRc, VecModel};
use std::{cell::RefCell, rc::Rc};
mod ui {
    slint::include_modules!();
}

fn next_index(current: Option<usize>, length: usize, step: ui::NavigationStep) -> usize {
    use ui::NavigationStep::*;
    let current = current.unwrap_or(0);
    match step {
        First => 0,
        Last => length.saturating_sub(1),
        Previous | PagePrevious => current.saturating_sub(1),
        Next | PageNext => (current + 1).min(length.saturating_sub(1)),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Person {
    Zoe,
    Ada,
    Lina,
}
impl Person {
    fn token(self) -> i32 {
        match self {
            Self::Zoe => 11,
            Self::Ada => 23,
            Self::Lina => 37,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Zoe => "Zoe",
            Self::Ada => "Ada",
            Self::Lina => "Lina",
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct Generation(i32);
struct State {
    order: Vec<Person>,
    selected: Option<Person>,
    active: Option<Person>,
    direction: ui::DataSortDirection,
    generation: Generation,
}
fn publish(state: &RefCell<State>, app: &ui::App) {
    let state = state.borrow();
    let rows = state
        .order
        .iter()
        .map(|person| ui::TableRow {
            token: person.token(),
            cells: ModelRc::new(VecModel::from(vec![person.label().into()])),
            selected: state.selected == Some(*person),
            disabled: false,
        })
        .collect::<Vec<_>>();
    let columns = vec![ui::TableColumn {
        token: 1,
        label: "Name".into(),
        sortable: true,
        sort: state.direction,
        alignment: ui::DataAlignment::Leading,
    }];
    let active = state
        .order
        .iter()
        .position(|p| Some(*p) == state.active)
        .map_or(-1, |i| i as i32);
    let label = state.selected.map_or("No selection".to_owned(), |p| {
        format!("Selected: {}", p.label())
    });
    let generation = state.generation.0;
    drop(state);
    app.set_generation(generation);
    app.set_active_index(active);
    app.set_rows(ModelRc::new(VecModel::from(rows)));
    app.set_columns(ModelRc::new(VecModel::from(columns)));
    app.set_selected_label(label.into());
}
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    let state = Rc::new(RefCell::new(State {
        order: vec![Person::Zoe, Person::Ada, Person::Lina],
        selected: None,
        active: None,
        direction: ui::DataSortDirection::Unsorted,
        generation: Generation(1),
    }));
    publish(&state, &app);
    let (model, weak) = (state.clone(), app.as_weak());
    app.on_sort(move |column| {
        if column != 1 {
            return;
        }
        {
            let mut state = model.borrow_mut();
            let Some(next) = state.generation.0.checked_add(1) else {
                return;
            };
            state.generation = Generation(next);
            state.direction = if state.direction == ui::DataSortDirection::Ascending {
                ui::DataSortDirection::Descending
            } else {
                ui::DataSortDirection::Ascending
            };
            state.order.sort_by_key(|p| p.label());
            if state.direction == ui::DataSortDirection::Descending {
                state.order.reverse();
            }
        }
        if let Some(app) = weak.upgrade() {
            publish(&model, &app);
        }
    });
    let (model, weak) = (state.clone(), app.as_weak());
    app.on_choose(move |token, generation| {
        {
            let mut state = model.borrow_mut();
            if generation != state.generation.0 {
                return;
            }
            let Some(person) = state.order.iter().copied().find(|p| p.token() == token) else {
                return;
            };
            state.selected = Some(person);
            state.active = Some(person);
        }
        if let Some(app) = weak.upgrade() {
            publish(&model, &app);
        }
    });
    let weak = app.as_weak();
    app.on_move(move |step| {
        {
            let mut state = state.borrow_mut();
            let current = state.order.iter().position(|p| Some(*p) == state.active);
            let index = next_index(current, state.order.len(), step);
            state.active = Some(state.order[index]);
        }
        if let Some(app) = weak.upgrade() {
            publish(&state, &app);
        }
    });
    app.run()
}
