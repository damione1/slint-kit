// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::{KeyedModel, event, ui};
slint::include_modules!();
#[derive(Clone, Debug, PartialEq, Eq, Hash)] struct ArticleId(&'static str);
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?; slint_kit::install(&app);
    let articles = [("setup", "Install Slint Kit", "Import a component and connect its typed callback.", "Getting started · 5 min read", app.get_alpine()), ("state", "Keep selection stable", "Use an application key when the display order changes.", "Engineering · 8 min read", app.get_desert()), ("accessibility", "Accessible controls", "Preserve keyboard navigation and clear focus states.", "Design · 6 min read", app.get_coast()), ("themes", "Make the theme your own", "Bring your brand to surfaces, typography, and interaction states.", "Design · 6 min read", app.get_alpine()), ("forms", "Build a better form", "Give every field a label, useful help, and a clear recovery path.", "Engineering · 8 min read", app.get_desert()), ("shipping", "From prototype to desktop", "Package the same Rust and Slint application for your users.", "Getting started · 5 min read", app.get_coast())];
    let rows = KeyedModel::new(articles.map(|(key, title, excerpt, metadata, thumbnail)| (ArticleId(key), ui::ContentItem {title: title.into(), excerpt: excerpt.into(), metadata: metadata.into(), thumbnail, link_label: "Select article".into(), ..Default::default()}))).unwrap();
    rows.bind(&app, App::set_items, App::set_generation, App::set_active_index);
    app.on_choose(rows.activation(event(&app, |app, key: ArticleId| app.set_selected_label(format!("Selected: {}", key.0).into()))));
    app.on_move(rows.navigation());
    app.run()
}
