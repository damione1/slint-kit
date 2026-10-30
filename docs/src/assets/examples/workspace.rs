// SPDX-License-Identifier: GPL-3.0-or-later
// In-memory host for the reusable workspace composition.
pub mod ui { slint::include_modules!(); }
use ui::{EditorStatus, ProjectFilter, App, WorkspaceProject};
use slint_kit::{HtmlConstraints, HtmlInputKind, HtmlInputState};
use slint::{ComponentHandle, Model};
use std::{cell::RefCell, rc::Rc, time::Duration};

#[derive(Clone, Copy, PartialEq, Eq)]
struct ProjectId(i32);
impl ProjectId {
    fn parse(value: i32) -> Option<Self> {
        (value > 0).then_some(Self(value))
    }
}
#[derive(Clone)]
struct Project {
    id: ProjectId,
    view: WorkspaceProject,
}
struct Workspace {
    projects: Vec<Project>,
    query: String,
    filter: ProjectFilter,
}
impl Workspace {
    fn publish(&self, window: &App) {
        let query = self.query.to_lowercase();
        let rows = self
            .projects
            .iter()
            .filter(|project| {
                let filter = match self.filter {
                    ProjectFilter::All => true,
                    ProjectFilter::Active => !project.view.archived,
                    ProjectFilter::Archived => project.view.archived,
                };
                filter
                    && (project.view.name.to_lowercase().contains(&query)
                        || project.view.description.to_lowercase().contains(&query))
            })
            .map(|project| project.view.clone())
            .collect::<Vec<_>>();
        window.set_projects(slint::ModelRc::new(slint::VecModel::from(rows)));
    }
}

fn valid_text(text: &str, minimum: usize) -> bool {
    HtmlConstraints {
        required: true,
        min_length: Some(minimum),
        ..Default::default()
    }
    .validate(text, HtmlInputState { user_edited: true, ..Default::default() })
    .valid()
}

pub fn install(window: &App) {
    let state = Rc::new(RefCell::new(Workspace {
        projects: window
            .get_projects()
            .iter()
            .map(|view| Project {
                id: ProjectId::parse(view.key).expect("positive sample identity"),
                view,
            })
            .collect(),
        query: String::new(),
        filter: ProjectFilter::All,
    }));
    let weak = window.as_weak();
    let filtered = state.clone();
    window.on_query_changed(move |query, filter| {
        let mut state = filtered.borrow_mut();
        state.query = query.into();
        state.filter = filter;
        if let Some(window) = weak.upgrade() {
            state.publish(&window);
        }
    });
    let weak = window.as_weak();
    window.on_save_account(move |name, email, notifications| {
        let valid = valid_text(&name, 1) && HtmlConstraints { kind: HtmlInputKind::Email, required: true, ..Default::default() }.validate(&email, HtmlInputState::default()).valid();
        if valid { if let Some(window) = weak.upgrade() { window.set_accepted_name(name); window.set_accepted_email(email); window.set_accepted_notifications(notifications); } }
        valid
    });
    let weak = window.as_weak();
    window.on_save_project(move |reference, name, description| {
        if !valid_text(&name, 3) {
            if let Some(window) = weak.upgrade() {
                window.set_editor_status(EditorStatus::Editing);
            }
            return;
        }
        let id = if reference.present {
            let Some(id) = ProjectId::parse(reference.token) else { if let Some(window) = weak.upgrade() { window.set_editor_status(EditorStatus::Editing); } return; };
            Some(id)
        } else { None };
        let weak = weak.clone();
        let state = state.clone();
        slint::Timer::single_shot(Duration::from_millis(600), move || {
            if let Some(window) = weak.upgrade() {
                let mut state = state.borrow_mut();
                let id = if let Some(id) = id {
                    id
                } else {
                    let next = state
                        .projects
                        .iter()
                        .map(|project| project.id.0)
                        .max()
                        .unwrap_or(0)
                        .checked_add(1)
                        .expect("demo identity capacity");
                    let id = ProjectId(next);
                    state.projects.push(Project {
                        id,
                        view: WorkspaceProject {
                            key: next,
                            owner: "Alex Morgan".into(),
                            updated: "Just now".into(),
                            ..Default::default()
                        },
                    });
                    id
                };
                if let Some(project) = state.projects.iter_mut().find(|project| project.id == id) {
                    project.view.name = name;
                    project.view.description = description;
                    project.view.updated = "Just now".into();
                    window.set_selected_key(id.0);
                    state.publish(&window);
                    window.set_editor_status(EditorStatus::Saved);
                } else {
                    window.set_editor_status(EditorStatus::Editing);
                }
            }
        });
    });
}

pub fn create() -> Result<ui::App, slint::PlatformError> {
    let app = ui::App::new()?;
    slint_kit::init_form_validation!(ui, app);
    install(&app);
    Ok(app)
}
#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), slint::PlatformError> { create()?.run() }
