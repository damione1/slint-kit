// SPDX-License-Identifier: GPL-3.0-or-later
//! In-memory host for the reusable workspace composition.
use crate::ui::{EditorStatus, ProjectFilter, Showcase, WorkspaceProject};
use kit::{HtmlConstraints, HtmlInputKind, HtmlInputState};
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
    fn publish(&self, window: &Showcase) {
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
        window.set_workspace_projects(slint::ModelRc::new(slint::VecModel::from(rows)));
    }
}

fn valid_text(text: &str, minimum: usize) -> bool {
    HtmlConstraints {
        required: true,
        min_length: Some(minimum),
        ..Default::default()
    }
    .validate(
        text,
        HtmlInputState {
            user_edited: true,
            ..Default::default()
        },
    )
    .valid()
}

pub fn install(window: &Showcase) {
    let state = Rc::new(RefCell::new(Workspace {
        projects: window
            .get_workspace_projects()
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
    window.on_workspace_query_changed(move |query, filter| {
        let mut state = filtered.borrow_mut();
        state.query = query.into();
        state.filter = filter;
        if let Some(window) = weak.upgrade() {
            state.publish(&window);
        }
    });
    let weak = window.as_weak();
    window.on_workspace_save_account(move |name, email, notifications| {
        let valid = valid_text(&name, 1)
            && HtmlConstraints {
                kind: HtmlInputKind::Email,
                required: true,
                ..Default::default()
            }
            .validate(&email, HtmlInputState::default())
            .valid();
        if valid {
            if let Some(window) = weak.upgrade() {
                window.set_workspace_accepted_name(name);
                window.set_workspace_accepted_email(email);
                window.set_workspace_accepted_notifications(notifications);
            }
        }
        valid
    });
    let weak = window.as_weak();
    window.on_workspace_save_project(move |reference, name, description| {
        if !valid_text(&name, 3) {
            if let Some(window) = weak.upgrade() {
                window.set_workspace_editor(EditorStatus::Editing);
            }
            return;
        }
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
                    window.set_workspace_selected_key(id.0);
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
        assert_eq!(
            window.get_workspace_projects().row_data(0).unwrap().key,
            412
        );
        click(&window, "All projects");
        click(&window, "Customer portal, Sam Rivera");
        assert_eq!(window.get_workspace_selected_key(), 205);
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
        assert_eq!(window.get_workspace_selected_key(), 205);
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
