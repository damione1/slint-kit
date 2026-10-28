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
enum Article {
    Setup,
    State,
}
impl Article {
    fn token(self) -> i32 {
        match self {
            Self::Setup => 11,
            Self::State => 23,
        }
    }
    fn title(self) -> &'static str {
        match self {
            Self::Setup => "Install Slint Kit",
            Self::State => "Keep selection stable",
        }
    }
    fn excerpt(self) -> &'static str {
        match self {
            Self::Setup => "Import a component and connect its Rust callback.",
            Self::State => "Use an application key when the display order changes.",
        }
    }
}
const ARTICLES: [Article; 2] = [Article::Setup, Article::State];
#[derive(Default)]
struct State {
    selected: Option<Article>,
    active: Option<Article>,
}
fn publish(state: &RefCell<State>, app: &ui::App) {
    let state = state.borrow();
    let items = ARTICLES
        .iter()
        .map(|article| ui::ContentItem {
            token: article.token(),
            title: article.title().into(),
            excerpt: article.excerpt().into(),
            metadata: "Guide".into(),
            link_label: "Select article".into(),
            thumbnail: slint::Image::default(),
            selected: state.selected == Some(*article),
            disabled: false,
        })
        .collect::<Vec<_>>();
    let index = ARTICLES
        .iter()
        .position(|a| Some(*a) == state.active)
        .map_or(-1, |i| i as i32);
    let label = state.selected.map_or("Choose an article".to_owned(), |a| {
        format!("Selected: {}", a.title())
    });
    drop(state);
    app.set_items(ModelRc::new(VecModel::from(items)));
    app.set_active_index(index);
    app.set_selected_label(label.into());
}
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    let state = Rc::new(RefCell::new(State::default()));
    publish(&state, &app);
    let (model, weak) = (state.clone(), app.as_weak());
    app.on_choose(move |token, generation| {
        if generation != 1 {
            return;
        }
        let Some(article) = ARTICLES.iter().copied().find(|a| a.token() == token) else {
            return;
        };
        {
            let mut state = model.borrow_mut();
            state.selected = Some(article);
            state.active = Some(article);
        }
        if let Some(app) = weak.upgrade() {
            publish(&model, &app);
        }
    });
    let weak = app.as_weak();
    app.on_move(move |step| {
        {
            let mut state = state.borrow_mut();
            let current = ARTICLES.iter().position(|a| Some(*a) == state.active);
            state.active = Some(ARTICLES[next_index(current, ARTICLES.len(), step)]);
        }
        if let Some(app) = weak.upgrade() {
            publish(&state, &app);
        }
    });
    app.run()
}
