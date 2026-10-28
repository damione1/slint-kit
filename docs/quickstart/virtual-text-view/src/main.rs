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
enum LineKey {
    Heading,
    Body,
    Closing,
}
impl LineKey {
    fn token(self) -> i32 {
        match self {
            Self::Heading => 11,
            Self::Body => 23,
            Self::Closing => 37,
        }
    }
}
struct Line {
    key: LineKey,
    text: String,
}
struct State {
    lines: Vec<Line>,
    selected: Option<LineKey>,
}
fn select(state: &State, app: &ui::App) {
    let index = state
        .lines
        .iter()
        .position(|line| Some(line.key) == state.selected);
    let draft = index.map_or("", |i| state.lines[i].text.as_str());
    app.set_active_index(index.map_or(-1, |i| i as i32));
    app.set_draft(draft.into());
    app.set_error("".into());
}
fn publish(state: &RefCell<State>, app: &ui::App) {
    let state = state.borrow();
    let lines = state
        .lines
        .iter()
        .enumerate()
        .map(|(index, line)| ui::TextLine {
            token: line.key.token(),
            number: index as i32 + 1,
            text: line.text.as_str().into(),
            selected: state.selected == Some(line.key),
        })
        .collect::<Vec<_>>();
    drop(state);
    app.set_lines(ModelRc::new(VecModel::from(lines)));
}
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    let state = Rc::new(RefCell::new(State {
        lines: vec![
            Line {
                key: LineKey::Heading,
                text: "# Project notes".into(),
            },
            Line {
                key: LineKey::Body,
                text: "Select and edit this line.".into(),
            },
            Line {
                key: LineKey::Closing,
                text: "Rust owns the saved text.".into(),
            },
        ],
        selected: None,
    }));
    publish(&state, &app);
    let (model, weak) = (state.clone(), app.as_weak());
    app.on_choose(move |token, generation| {
        if generation != 1 {
            return;
        }
        {
            let mut state = model.borrow_mut();
            let Some(key) = state
                .lines
                .iter()
                .find(|line| line.key.token() == token)
                .map(|line| line.key)
            else {
                return;
            };
            state.selected = Some(key);
        }
        if let Some(app) = weak.upgrade() {
            select(&model.borrow(), &app);
            publish(&model, &app);
        }
    });
    let (model, weak) = (state.clone(), app.as_weak());
    app.on_move(move |step| {
        {
            let mut state = model.borrow_mut();
            let current = state
                .lines
                .iter()
                .position(|line| Some(line.key) == state.selected);
            let index = next_index(current, state.lines.len(), step);
            state.selected = Some(state.lines[index].key);
        }
        if let Some(app) = weak.upgrade() {
            select(&model.borrow(), &app);
            publish(&model, &app);
        }
    });
    let (model, weak) = (state.clone(), app.as_weak());
    app.on_save(move |token, generation, text| {
        if generation != 1 {
            return;
        }
        let Some(app) = weak.upgrade() else {
            return;
        };
        if text.contains(['\n', '\r']) {
            app.set_error("Enter one line only".into());
            return;
        }
        {
            let mut state = model.borrow_mut();
            if state.selected.map(LineKey::token) != Some(token) {
                return;
            }
            let Some(line) = state
                .lines
                .iter_mut()
                .find(|line| line.key.token() == token)
            else {
                return;
            };
            line.text = text.to_string();
        }
        app.set_error("".into());
        app.set_status("Line saved in Rust".into());
        publish(&model, &app);
    });
    let weak = app.as_weak();
    app.on_copied(move |_token, generation| {
        if generation == 1 {
            if let Some(app) = weak.upgrade() {
                app.set_status("Selected line copied".into());
            }
        }
    });
    app.run()
}
