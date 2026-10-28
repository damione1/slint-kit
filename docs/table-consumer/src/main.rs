// SPDX-License-Identifier: GPL-3.0-or-later
use slint::{ComponentHandle, ModelRc, VecModel};
use std::{cell::RefCell, rc::Rc};

pub mod ui {
    slint::include_modules!();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ProjectId(i32);

#[derive(Clone, Copy, PartialEq, Eq)]
struct ViewGeneration(i32);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Column {
    Name,
    Tasks,
}

impl Column {
    fn from_token(token: i32) -> Option<Self> {
        match token {
            1 => Some(Self::Name),
            2 => Some(Self::Tasks),
            _ => None,
        }
    }
}

struct Project {
    id: ProjectId,
    name: &'static str,
    tasks: u32,
    archived: bool,
}

struct Catalogue {
    projects: Vec<Project>,
    order: Vec<ProjectId>,
    selected: Option<ProjectId>,
    active: Option<ProjectId>,
    column: Column,
    direction: ui::DataSortDirection,
    generation: ViewGeneration,
}

impl Catalogue {
    fn new() -> Self {
        let projects = vec![
            Project {
                id: ProjectId(101),
                name: "Atlas",
                tasks: 12,
                archived: false,
            },
            Project {
                id: ProjectId(203),
                name: "Boreal",
                tasks: 3,
                archived: false,
            },
            Project {
                id: ProjectId(307),
                name: "Cedar",
                tasks: 24,
                archived: true,
            },
            Project {
                id: ProjectId(409),
                name: "Delta",
                tasks: 8,
                archived: false,
            },
        ];
        Self {
            order: projects.iter().map(|project| project.id).collect(),
            projects,
            selected: None,
            active: None,
            column: Column::Name,
            direction: ui::DataSortDirection::Unsorted,
            generation: ViewGeneration(1),
        }
    }

    fn project(&self, id: ProjectId) -> &Project {
        self.projects
            .iter()
            .find(|project| project.id == id)
            .expect("known project key")
    }

    fn sort(&mut self, token: i32) {
        let Some(column) = Column::from_token(token) else {
            return;
        };
        // Reject exhaustion instead of reusing an old generation.
        let Some(generation) = self.generation.0.checked_add(1) else {
            return;
        };
        self.direction = if column != self.column {
            ui::DataSortDirection::Ascending
        } else {
            match self.direction {
                ui::DataSortDirection::Unsorted => ui::DataSortDirection::Ascending,
                ui::DataSortDirection::Ascending => ui::DataSortDirection::Descending,
                ui::DataSortDirection::Descending => ui::DataSortDirection::Unsorted,
            }
        };
        self.column = column;
        let mut order: Vec<_> = self.projects.iter().map(|project| project.id).collect();
        if self.direction != ui::DataSortDirection::Unsorted {
            order.sort_by(|left, right| {
                let left = self.project(*left);
                let right = self.project(*right);
                let comparison = match column {
                    Column::Name => left.name.cmp(right.name),
                    Column::Tasks => left.tasks.cmp(&right.tasks),
                }
                .then(left.id.0.cmp(&right.id.0));
                if self.direction == ui::DataSortDirection::Descending {
                    comparison.reverse()
                } else {
                    comparison
                }
            });
        }
        self.order = order;
        self.generation = ViewGeneration(generation);
    }

    fn activate(&mut self, token: i32, generation: i32) {
        let id = ProjectId(token);
        if ViewGeneration(generation) == self.generation
            && self.order.contains(&id)
            && !self.project(id).archived
        {
            self.selected = Some(id);
            self.active = Some(id);
        }
    }

    fn navigate(&mut self, step: ui::NavigationStep) {
        let enabled: Vec<_> = self
            .order
            .iter()
            .copied()
            .filter(|id| !self.project(*id).archived)
            .collect();
        if enabled.is_empty() {
            return;
        }
        let current = enabled.iter().position(|id| Some(*id) == self.active);
        let last = enabled.len() - 1;
        let index = match step {
            ui::NavigationStep::Next => current.map_or(0, |index| (index + 1).min(last)),
            ui::NavigationStep::Previous => current.map_or(last, |index| index.saturating_sub(1)),
            ui::NavigationStep::First => 0,
            ui::NavigationStep::Last => last,
            ui::NavigationStep::PageNext => current.map_or(0, |index| (index + 5).min(last)),
            ui::NavigationStep::PagePrevious => {
                current.map_or(last, |index| index.saturating_sub(5))
            }
        };
        self.active = Some(enabled[index]);
    }
}

struct Projection {
    generation: i32,
    active_index: i32,
    rows: Vec<ui::TableRow>,
    columns: Vec<ui::TableColumn>,
    selection_label: String,
}

fn project(state: &Catalogue, french: bool) -> Projection {
    let generation = state.generation.0;
    let active_index = state
        .order
        .iter()
        .position(|id| Some(*id) == state.active)
        .map_or(-1, |index| index as i32);
    let rows = state
        .order
        .iter()
        .map(|id| {
            let project = state.project(*id);
            ui::TableRow {
                token: id.0,
                cells: ModelRc::new(VecModel::from(vec![
                    project.name.into(),
                    project.tasks.to_string().into(),
                ])),
                selected: state.selected == Some(*id),
                disabled: project.archived,
            }
        })
        .collect();
    let columns = [Column::Name, Column::Tasks]
        .iter()
        .map(|column| ui::TableColumn {
            token: if *column == Column::Name { 1 } else { 2 },
            label: match (column, french) {
                (Column::Name, false) => "Project",
                (Column::Name, true) => "Projet",
                (Column::Tasks, false) => "Tasks",
                (Column::Tasks, true) => "Tâches",
            }
            .into(),
            sortable: true,
            sort: if *column == state.column {
                state.direction
            } else {
                ui::DataSortDirection::Unsorted
            },
            alignment: if *column == Column::Name {
                ui::DataAlignment::Leading
            } else {
                ui::DataAlignment::Trailing
            },
        })
        .collect();
    let selection_label = match state.selected {
        Some(id) => format!(
            "{}: {} · #{}",
            if french { "Sélection" } else { "Selected" },
            state.project(id).name,
            id.0
        ),
        None => {
            if french {
                "Aucune sélection".into()
            } else {
                "No selection".into()
            }
        }
    };
    Projection {
        generation,
        active_index,
        rows,
        columns,
        selection_label,
    }
}

fn publish(
    state: &RefCell<Catalogue>,
    app: &ui::TableApp,
    rows: &VecModel<ui::TableRow>,
    french: bool,
) {
    let projection = project(&state.borrow(), french);
    // No state borrow is held while model notifications or window updates run.
    app.set_generation(projection.generation);
    app.set_active_index(projection.active_index);
    rows.set_vec(projection.rows);
    app.set_columns(ModelRc::new(VecModel::from(projection.columns)));
    app.set_selection_label(projection.selection_label.into());
}

pub fn create_app(french: bool) -> Result<ui::TableApp, slint::PlatformError> {
    let app = ui::TableApp::new()?;
    app.set_language(if french {
        ui::TableLanguage::French
    } else {
        ui::TableLanguage::English
    });
    let state = Rc::new(RefCell::new(Catalogue::new()));
    let rows = Rc::new(VecModel::default());
    app.set_rows(ModelRc::from(rows.clone()));
    publish(&state, &app, &rows, french);
    let weak = app.as_weak();
    let sort_state = state.clone();
    let sort_rows = rows.clone();
    app.on_sort_requested(move |token| {
        {
            sort_state.borrow_mut().sort(token);
        }
        if let Some(app) = weak.upgrade() {
            publish(&sort_state, &app, &sort_rows, french);
        }
    });
    let weak = app.as_weak();
    let activate_state = state.clone();
    let activate_rows = rows.clone();
    app.on_activate(move |token, generation| {
        {
            activate_state.borrow_mut().activate(token, generation);
        }
        if let Some(app) = weak.upgrade() {
            publish(&activate_state, &app, &activate_rows, french);
        }
    });
    let weak = app.as_weak();
    app.on_navigate(move |step| {
        {
            state.borrow_mut().navigate(step);
        }
        if let Some(app) = weak.upgrade() {
            publish(&state, &app, &rows, french);
        }
    });
    Ok(app)
}

pub fn main() -> Result<(), slint::PlatformError> {
    let app = create_app(std::env::args().any(|arg| arg == "--fr"))?;
    app.run()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selection_survives_sort_and_stale_disabled_or_unknown_rows_are_rejected() {
        let mut state = Catalogue::new();
        state.activate(101, state.generation.0);
        let previous = state.generation;
        state.sort(2);
        assert_eq!(state.order[0], ProjectId(203));
        assert_eq!(state.selected, Some(ProjectId(101)));
        state.activate(409, previous.0);
        state.activate(307, state.generation.0);
        state.activate(-1, state.generation.0);
        assert_eq!(state.selected, Some(ProjectId(101)));
        state.navigate(ui::NavigationStep::Last);
        assert_eq!(state.active, Some(ProjectId(101)));
        state.navigate(ui::NavigationStep::First);
        state.activate(state.active.expect("active project").0, state.generation.0);
        assert_eq!(state.selected, Some(ProjectId(203)));
        state.generation = ViewGeneration(i32::MAX);
        state.sort(1);
        assert_eq!(state.generation.0, i32::MAX);
        assert_eq!(state.order[0], ProjectId(203));
    }
}
