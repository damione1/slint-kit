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
    Accessibility,
    Themes,
    Forms,
    Shipping,
}
impl Article {
    fn token(self) -> i32 {
        match self {
            Self::Setup => 11,
            Self::State => 23,
            Self::Accessibility => 37,
            Self::Themes => 51,
            Self::Forms => 66,
            Self::Shipping => 80,
        }
    }
    fn title(self) -> &'static str {
        match self {
            Self::Setup => "Install Slint Kit",
            Self::State => "Keep selection stable",
            Self::Accessibility => "Accessible controls",
            Self::Themes => "Make the theme your own",
            Self::Forms => "Build a better form",
            Self::Shipping => "From prototype to desktop",
        }
    }
    fn excerpt(self) -> &'static str {
        match self {
            Self::Setup => "Import a component and connect its Rust callback.",
            Self::State => "Use an application key when the display order changes.",
            Self::Accessibility => "Preserve keyboard navigation and clear focus states.",
            Self::Themes => "Bring your brand to surfaces, typography, and interaction states.",
            Self::Forms => "Give every field a label, useful help, and a clear recovery path.",
            Self::Shipping => "Package the same Rust and Slint application for your users.",
        }
    }
}
const ARTICLES: [Article; 6] = [Article::Setup, Article::State, Article::Accessibility, Article::Themes, Article::Forms, Article::Shipping];
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
            metadata: match article { Article::Setup | Article::Shipping => "Getting started · 5 min read", Article::State | Article::Forms => "Engineering · 8 min read", Article::Accessibility | Article::Themes => "Design · 6 min read" }.into(),
            link_label: "Select article".into(),
            thumbnail: landscape(match article { Article::Setup | Article::Themes => Landscape::Alpine, Article::State | Article::Forms => Landscape::Desert, Article::Accessibility | Article::Shipping => Landscape::Coast }),
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
    slint_kit::init_form_validation!(ui, app);
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

// Original landscape illustrations, rendered locally; no external asset or network dependency.
#[derive(Clone, Copy)]
enum Landscape { Alpine, Desert, Coast }
fn landscape(scene: Landscape) -> slint::Image {
    let (sky, ridge, near, sun): ([u8; 3], [u8; 3], [u8; 3], [u8; 3]) = match scene {
        Landscape::Alpine => ([204, 230, 239], [90, 139, 148], [30, 81, 95], [255, 246, 212]),
        Landscape::Desert => ([249, 222, 192], [205, 134, 111], [136, 74, 78], [255, 250, 222]),
        Landscape::Coast => ([210, 229, 235], [99, 158, 165], [43, 107, 123], [255, 246, 215]),
    };
    let mut buffer = slint::SharedPixelBuffer::<slint::Rgb8Pixel>::new(640, 260);
    for (index, pixel) in buffer.make_mut_slice().iter_mut().enumerate() {
        let x = i32::try_from(index % 640).expect("image width fits i32");
        let y = i32::try_from(index / 640).expect("image height fits i32");
        let distant = 145 - (130 - (x - 210).abs()).max(0) / 2;
        let foreground = 222 - (180 - (x - 495).abs()).max(0) / 3;
        let color = if (x - 493).pow(2) + (y - 62).pow(2) < 28 * 28 {
            sun
        } else if y > foreground { near } else if y > distant { ridge } else { sky };
        *pixel = slint::Rgb8Pixel::new(color[0], color[1], color[2]);
    }
    slint::Image::from_rgb8(buffer)
}
