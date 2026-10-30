// SPDX-License-Identifier: GPL-3.0-or-later
// In-memory host for the reusable workspace composition.
pub mod ui {
    slint::include_modules!();
}
use slint::{ComponentHandle, Model, SharedString};
use slint_kit::{HtmlConstraints, HtmlInputKind, HtmlInputState};
use std::{cell::RefCell, rc::Rc, time::Duration};
use ui::{App, EditorStatus, ProjectFilter, SelectionRef, WorkspaceDraft, WorkspaceProject};

#[derive(Clone, Copy, PartialEq, Eq)]
struct ProjectId(i32);
impl ProjectId {
    fn parse(value: i32) -> Option<Self> {
        (value > 0).then_some(Self(value))
    }
}

fn required_text(draft: SharedString, minimum: usize) -> Option<SharedString> {
    let rules = HtmlConstraints {
        required: true,
        min_length: Some(minimum),
        ..Default::default()
    };
    let value = rules.sanitize(&draft);
    rules
        .validate(
            &value,
            HtmlInputState {
                user_edited: true,
                ..Default::default()
            },
        )
        .valid()
        .then(|| value.into())
}
#[derive(Clone)]
struct ProjectName(SharedString);
impl ProjectName {
    fn parse(draft: SharedString) -> Option<Self> {
        required_text(draft, 3).map(Self)
    }
}
#[derive(Clone)]
struct AccountName(SharedString);
impl AccountName {
    fn parse(draft: SharedString) -> Option<Self> {
        required_text(draft, 1).map(Self)
    }
}
#[derive(Clone)]
struct EmailAddress(SharedString);
impl EmailAddress {
    fn parse(draft: SharedString) -> Option<Self> {
        let rules = HtmlConstraints {
            kind: HtmlInputKind::Email,
            required: true,
            ..Default::default()
        };
        let value = rules.sanitize(&draft);
        rules
            .validate(&value, HtmlInputState::default())
            .valid()
            .then(|| Self(value.into()))
    }
}
struct Account {
    name: AccountName,
    email: EmailAddress,
    notifications: bool,
}
impl Account {
    fn parse(name: SharedString, email: SharedString, notifications: bool) -> Option<Self> {
        Some(Self {
            name: AccountName::parse(name)?,
            email: EmailAddress::parse(email)?,
            notifications,
        })
    }
    fn publish(&self, window: &App) {
        window.set_accepted_name(self.name.0.clone());
        window.set_accepted_email(self.email.0.clone());
        window.set_accepted_notifications(self.notifications);
    }
}
struct Project {
    id: ProjectId,
    name: ProjectName,
    description: SharedString,
    owner: SharedString,
    updated: SharedString,
    archived: bool,
}
impl Project {
    fn from_view(view: WorkspaceProject) -> Self {
        Self {
            id: ProjectId::parse(view.key).expect("positive sample identity"),
            name: ProjectName::parse(view.name).expect("valid sample project name"),
            description: view.description,
            owner: view.owner,
            updated: view.updated,
            archived: view.archived,
        }
    }
    fn view(&self) -> WorkspaceProject {
        WorkspaceProject {
            key: self.id.0,
            name: self.name.0.clone(),
            description: self.description.clone(),
            owner: self.owner.clone(),
            updated: self.updated.clone(),
            archived: self.archived,
        }
    }
}
struct Workspace {
    projects: Vec<Project>,
    account: Account,
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
                    ProjectFilter::Active => !project.archived,
                    ProjectFilter::Archived => project.archived,
                };
                filter
                    && (project.name.0.to_lowercase().contains(&query)
                        || project.description.to_lowercase().contains(&query))
            })
            .map(Project::view)
            .collect::<Vec<_>>();
        let selected = window.get_selection();
        if selected.present && !rows.iter().any(|row| row.key == selected.token) {
            window.set_selection(SelectionRef {
                present: false,
                token: selected.token,
            });
        }
        window.set_projects(slint::ModelRc::new(slint::VecModel::from(rows)));
    }
}

pub fn install(window: &App) {
    let state = Rc::new(RefCell::new(Workspace {
        projects: window
            .get_projects()
            .iter()
            .map(Project::from_view)
            .collect(),
        account: Account::parse(
            window.get_accepted_name(),
            window.get_accepted_email(),
            window.get_accepted_notifications(),
        )
        .expect("valid initial account"),
        query: String::new(),
        filter: ProjectFilter::All,
    }));
    state.borrow().account.publish(window);
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
    let accounts = state.clone();
    window.on_save_account(move |name, email, notifications| {
        let Some(account) = Account::parse(name, email, notifications) else {
            return false;
        };
        accounts.borrow_mut().account = account;
        if let Some(window) = weak.upgrade() {
            accounts.borrow().account.publish(&window);
        }
        true
    });
    let weak = window.as_weak();
    let loading = state.clone();
    window.on_load_project(move |reference| {
        if !reference.present {
            return WorkspaceDraft::default();
        }
        let Some(id) = ProjectId::parse(reference.token) else {
            return WorkspaceDraft::default();
        };
        loading
            .borrow()
            .projects
            .iter()
            .find(|project| project.id == id)
            .map(|project| WorkspaceDraft {
                present: true,
                name: project.name.0.clone(),
                description: project.description.clone(),
            })
            .unwrap_or_default()
    });
    window.on_save_project(move |reference, name, description| {
        let Some(name) = ProjectName::parse(name) else {
            if let Some(window) = weak.upgrade() {
                window.set_editor_status(EditorStatus::Editing);
            }
            return;
        };
        let id = if reference.present {
            let Some(id) = ProjectId::parse(reference.token) else {
                if let Some(window) = weak.upgrade() {
                    window.set_editor_status(EditorStatus::Editing);
                }
                return;
            };
            Some(id)
        } else {
            None
        };
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
                        name: name.clone(),
                        description: description.clone(),
                        owner: "Elliot Alderson".into(),
                        updated: "Just now".into(),
                        archived: false,
                    });
                    id
                };
                if let Some(project) = state.projects.iter_mut().find(|project| project.id == id) {
                    project.name = name;
                    project.description = description;
                    project.updated = "Just now".into();
                    window.set_selection(SelectionRef {
                        present: true,
                        token: id.0,
                    });
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
fn main() -> Result<(), slint::PlatformError> {
    create()?.run()
}
