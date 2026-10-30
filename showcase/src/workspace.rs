// SPDX-License-Identifier: GPL-3.0-or-later
//! In-memory host for the reusable workspace composition.
use crate::ui::{
    EditorStatus, ProjectFilter, SelectionRef, Showcase, WorkspaceDraft, WorkspaceProject,
};
use kit::{HtmlConstraints, HtmlInputKind, HtmlInputState};
use slint::{ComponentHandle, Model, SharedString};
use std::{cell::RefCell, rc::Rc, time::Duration};

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
    fn publish(&self, window: &Showcase) {
        window.set_workspace_accepted_name(self.name.0.clone());
        window.set_workspace_accepted_email(self.email.0.clone());
        window.set_workspace_accepted_notifications(self.notifications);
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
    fn publish(&self, window: &Showcase) {
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
        let selected = window.get_workspace_selection();
        if selected.present && !rows.iter().any(|row| row.key == selected.token) {
            window.set_workspace_selection(SelectionRef {
                present: false,
                token: selected.token,
            });
        }
        window.set_workspace_projects(slint::ModelRc::new(slint::VecModel::from(rows)));
    }
}

pub fn install(window: &Showcase) {
    let state = Rc::new(RefCell::new(Workspace {
        projects: window
            .get_workspace_projects()
            .iter()
            .map(Project::from_view)
            .collect(),
        account: Account::parse(
            window.get_workspace_accepted_name(),
            window.get_workspace_accepted_email(),
            window.get_workspace_accepted_notifications(),
        )
        .expect("valid initial account"),
        query: String::new(),
        filter: ProjectFilter::All,
    }));
    state.borrow().account.publish(window);
    let weak = window.as_weak();
    let filtered = state.clone();
    window.on_workspace_query_changed(move |query, filter| {
        let mut state = filtered.borrow_mut();
        state.query = query.into();
        state.filter = filter;
        if let Some(window) = weak.upgrade() {
            state.publish(&window);
        }
    });
    let weak = window.as_weak();
    let accounts = state.clone();
    window.on_workspace_save_account(move |name, email, notifications| {
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
    window.on_workspace_load_project(move |reference| {
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
    window.on_workspace_save_project(move |reference, name, description| {
        let Some(name) = ProjectName::parse(name) else {
            if let Some(window) = weak.upgrade() {
                window.set_workspace_editor(EditorStatus::Editing);
            }
            return;
        };
        let id = if reference.present {
            let Some(id) = ProjectId::parse(reference.token) else {
                if let Some(window) = weak.upgrade() {
                    window.set_workspace_editor(EditorStatus::Editing);
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
                        owner: "Alex Morgan".into(),
                        updated: "Just now".into(),
                        archived: false,
                    });
                    id
                };
                if let Some(project) = state.projects.iter_mut().find(|project| project.id == id) {
                    project.name = name;
                    project.description = description;
                    project.updated = "Just now".into();
                    window.set_workspace_selection(SelectionRef {
                        present: true,
                        token: id.0,
                    });
                    state.publish(&window);
                    window.set_workspace_editor(EditorStatus::Saved);
                } else {
                    window.set_workspace_editor(EditorStatus::Editing);
                }
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui;
    use i_slint_backend_testing::{ElementHandle, ElementQuery};
    use slint::platform::{Key, PointerEventButton, WindowEvent};
    fn setup() -> Showcase {
        i_slint_backend_testing::init_no_event_loop();
        let window = Showcase::new().unwrap();
        kit::init_form_validation!(ui, window);
        window.set_product_workspace(true);
        window.set_workspace_page(ui::WorkspacePage::Projects);
        install(&window);
        window.show().unwrap();
        window
    }
    fn click(window: &Showcase, label: &str) {
        ElementHandle::find_by_accessible_label(window, label)
            .next()
            .unwrap_or_else(|| panic!("missing {label}"))
            .mock_single_click(PointerEventButton::Left);
    }
    fn key(window: &Showcase, value: Key) {
        window
            .window()
            .dispatch_event(WindowEvent::KeyPressed { text: value.into() });
        window
            .window()
            .dispatch_event(WindowEvent::KeyReleased { text: value.into() });
    }
    #[test]
    fn filters_empty_state_and_edits_keep_stable_project_identity() {
        let window = setup();
        click(&window, "Archived");
        assert_eq!(window.get_workspace_projects().row_count(), 1);
        assert!(!window.get_workspace_selection().present);
        assert_eq!(
            ElementHandle::find_by_accessible_label(&window, "Edit project")
                .next()
                .unwrap()
                .accessible_enabled(),
            Some(false)
        );
        assert_eq!(
            window.get_workspace_projects().row_data(0).unwrap().key,
            412
        );
        click(&window, "All projects");
        click(&window, "Customer portal, Sam Rivera");
        assert_eq!(window.get_workspace_selection().token, 205);
        window.invoke_workspace_query_changed("missing".into(), ProjectFilter::All);
        assert_eq!(window.get_workspace_projects().row_count(), 0);
        assert!(
            ElementQuery::from_root(&window)
                .match_predicate(
                    |element| element.accessible_label().as_deref() == Some("No projects found")
                )
                .find_first()
                .is_some()
        );
        window.invoke_workspace_query_changed("".into(), ProjectFilter::All);
        window.invoke_workspace_save_project(
            ui::SelectionRef {
                present: true,
                token: 205,
            },
            "Renamed portal".into(),
            "Updated context".into(),
        );
        i_slint_backend_testing::mock_elapsed_time(Duration::from_millis(601));
        slint::platform::update_timers_and_animations();
        assert_eq!(window.get_workspace_selection().token, 205);
        assert!(
            window
                .get_workspace_projects()
                .iter()
                .any(|row| row.key == 205
                    && row.name == "Renamed portal"
                    && row.description == "Updated context")
        );
    }
    #[test]
    fn modal_keyboard_cancellation_restores_the_exact_opener_and_busy_blocks_escape() {
        let window = setup();
        click(&window, "Edit project");
        assert_eq!(window.get_workspace_editor(), EditorStatus::Editing);
        for _ in 0..9 {
            key(&window, Key::Tab);
        }
        key(&window, Key::Escape);
        assert_eq!(window.get_workspace_editor(), EditorStatus::Closed);
        key(&window, Key::Return);
        assert_eq!(window.get_workspace_editor(), EditorStatus::Editing);
        key(&window, Key::Escape);
        click(&window, "New project");
        window.set_workspace_editor(EditorStatus::Saving);
        key(&window, Key::Escape);
        assert_eq!(window.get_workspace_editor(), EditorStatus::Saving);
        window.set_workspace_editor(EditorStatus::Editing);
        key(&window, Key::Escape);
        key(&window, Key::Return);
        assert_eq!(window.get_workspace_editor(), EditorStatus::Editing);
    }
    #[test]
    fn rust_submission_rejects_invalid_name_and_email() {
        let window = setup();
        assert!(!window.invoke_workspace_save_account(
            "Alex".into(),
            "arbitrary nonempty".into(),
            true
        ));
        assert!(window.invoke_workspace_save_account(
            "Alex".into(),
            "alex@example.com".into(),
            true
        ));
        window.invoke_workspace_save_project(
            ui::SelectionRef {
                present: true,
                token: 101,
            },
            "a".into(),
            "draft".into(),
        );
        assert_eq!(window.get_workspace_editor(), EditorStatus::Editing);
        assert_eq!(
            window.get_workspace_projects().row_data(0).unwrap().name,
            "Design system"
        );
    }
}

#[cfg(test)]
mod collection_scale_tests {
    use super::*;
    use i_slint_backend_testing::ElementQuery;
    #[test]
    fn the_collection_virtualizes_a_host_model_larger_than_the_sample() {
        i_slint_backend_testing::init_no_event_loop();
        let app = Showcase::new().unwrap();
        let rows = (1..=10000)
            .map(|key| WorkspaceProject {
                key,
                name: format!("Project {key}").into(),
                description: "Host-provided model".into(),
                ..Default::default()
            })
            .collect::<Vec<_>>();
        app.set_workspace_projects(slint::ModelRc::new(slint::VecModel::from(rows)));
        app.set_product_workspace(true);
        app.set_workspace_page(crate::ui::WorkspacePage::Projects);
        install(&app);
        app.show().unwrap();
        assert_eq!(app.get_workspace_projects().row_count(), 10000);
        let materialized = ElementQuery::from_root(&app)
            .match_id("ProductWorkspace::row")
            .find_all()
            .len();
        assert!(
            materialized > 0 && materialized < 100,
            "finite viewport, materialized {materialized} of10000rows"
        );
        app.invoke_workspace_query_changed("Project 10000".into(), ProjectFilter::All);
        assert_eq!(app.get_workspace_projects().row_count(), 1);
        assert_eq!(app.get_workspace_projects().row_data(0).unwrap().key, 10000);
    }
}

#[cfg(test)]
mod accepted_state_tests {
    use super::*;
    use crate::ui;
    #[test]
    fn creation_has_no_existing_identity_and_account_snapshot_advances_after_save() {
        i_slint_backend_testing::init_no_event_loop();
        let app = Showcase::new().unwrap();
        install(&app);
        assert!(app.invoke_workspace_save_account(
            "Updated name".into(),
            "updated@example.com".into(),
            false
        ));
        assert_eq!(app.get_workspace_accepted_name(), "Updated name");
        assert_eq!(app.get_workspace_accepted_email(), "updated@example.com");
        assert!(!app.get_workspace_accepted_notifications());
        let original = app.get_workspace_projects().row_data(0).unwrap();
        app.invoke_workspace_save_project(
            ui::SelectionRef {
                present: false,
                token: 0,
            },
            "A new project".into(),
            "New context".into(),
        );
        i_slint_backend_testing::mock_elapsed_time(Duration::from_millis(601));
        slint::platform::update_timers_and_animations();
        assert_eq!(app.get_workspace_projects().row_count(), 5);
        assert_eq!(app.get_workspace_projects().row_data(0).unwrap(), original);
        let added = app.get_workspace_projects().row_data(4).unwrap();
        assert_eq!(added.name, "A new project");
        assert!(added.key > 412);
        assert_eq!(app.get_workspace_selection().token, added.key);
    }
}
