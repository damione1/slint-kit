// SPDX-License-Identifier: GPL-3.0-or-later
use slint::{ComponentHandle, ModelRc, VecModel};
use slint_kit::{CellContent, KeyedModel, table_cells};
use std::{cell::Cell, cmp::Ordering, collections::BTreeMap, rc::Rc};

pub mod ui {
    pub use slint_kit::ui::*;
    slint::include_modules!();
}

/// The stable key of a project. Selection and the active row follow it,
/// never the row's visual position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ProjectId(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Column {
    Name,
    Tasks,
}

impl Column {
    const ALL: [Self; 2] = [Self::Name, Self::Tasks];
    fn token(self) -> i32 {
        match self {
            Self::Name => 1,
            Self::Tasks => 2,
        }
    }
    fn from_token(token: i32) -> Option<Self> {
        Self::ALL.into_iter().find(|column| column.token() == token)
    }
    fn label(self, language: ui::TableLanguage) -> &'static str {
        match (self, language) {
            (Self::Name, ui::TableLanguage::English) => "Project",
            (Self::Name, ui::TableLanguage::French) => "Projet",
            (Self::Tasks, ui::TableLanguage::English) => "Tasks",
            (Self::Tasks, ui::TableLanguage::French) => "Tâches",
        }
    }
    fn alignment(self) -> ui::DataAlignment {
        match self {
            Self::Name => ui::DataAlignment::Leading,
            Self::Tasks => ui::DataAlignment::Trailing,
        }
    }
}

struct Project {
    name: &'static str,
    tasks: u32,
    archived: bool,
}

/// Host-owned data: the projects by key, the key of the selected one and
/// the order shown, so a filtered row set keeps the current order.
struct Catalogue {
    projects: BTreeMap<ProjectId, Project>,
    selected: Cell<Option<ProjectId>>,
    order: Cell<Option<(Column, ui::DataSortDirection)>>,
}

impl Catalogue {
    fn new() -> Self {
        let project = |name, tasks, archived| Project {
            name,
            tasks,
            archived,
        };
        Self {
            projects: BTreeMap::from([
                (ProjectId(101), project("Atlas", 12, false)),
                (ProjectId(203), project("Boreal", 3, false)),
                (ProjectId(307), project("Cedar", 24, true)),
                (ProjectId(409), project("Delta", 8, false)),
            ]),
            selected: Cell::new(None),
            order: Cell::new(None),
        }
    }
    /// Domain comparison of two keys; the key itself breaks ties.
    fn compare(&self, column: Column, left: ProjectId, right: ProjectId) -> Ordering {
        let (first, second) = (&self.projects[&left], &self.projects[&right]);
        match column {
            Column::Name => first.name.cmp(second.name),
            Column::Tasks => first.tasks.cmp(&second.tasks),
        }
        .then(left.cmp(&right))
    }
    /// The rows to show, in the current order. Tokens are stamped by the model.
    fn rows(&self, show_archived: bool) -> Vec<(ProjectId, ui::TableRow)> {
        let mut rows: Vec<_> = self
            .projects
            .iter()
            .filter(|(_, project)| show_archived || !project.archived)
            .map(|(id, project)| {
                let row = ui::TableRow {
                    content: table_cells([
                        CellContent::text(project.name),
                        CellContent::text(project.tasks.to_string()),
                    ]),
                    selected: self.selected.get() == Some(*id),
                    disabled: project.archived,
                    ..Default::default()
                };
                (*id, row)
            })
            .collect();
        if let Some((column, direction)) = self.order.get() {
            rows.sort_by(|(left, _), (right, _)| match direction {
                ui::DataSortDirection::Ascending => self.compare(column, *left, *right),
                ui::DataSortDirection::Descending => self.compare(column, *right, *left),
                ui::DataSortDirection::Unsorted => Ordering::Equal,
            });
        }
        rows
    }
    fn selection_label(&self, language: ui::TableLanguage) -> String {
        match (self.selected.get(), language) {
            (Some(id), ui::TableLanguage::English) => {
                format!("Selected: {} · #{}", self.projects[&id].name, id.0)
            }
            (Some(id), ui::TableLanguage::French) => {
                format!("Sélection : {} · #{}", self.projects[&id].name, id.0)
            }
            (None, ui::TableLanguage::English) => "No selection".into(),
            (None, ui::TableLanguage::French) => "Aucune sélection".into(),
        }
    }
}

/// The header row; `sorted` names the column whose indicator is shown.
fn columns(
    language: ui::TableLanguage,
    sorted: Option<(Column, ui::DataSortDirection)>,
) -> Vec<ui::TableColumn> {
    Column::ALL
        .into_iter()
        .map(|column| ui::TableColumn {
            token: column.token(),
            label: column.label(language).into(),
            sortable: true,
            sort: sorted
                .filter(|(sorted, _)| *sorted == column)
                .map_or(ui::DataSortDirection::Unsorted, |(_, direction)| direction),
            alignment: column.alignment(),
            width: ui::ColumnWidth::default(),
        })
        .collect()
}

/// A header action: the model reorders by the domain comparator, keeps keys,
/// selection and the active row, advances the generation, then reports the
/// direction, which the host records and shows on this column's header.
fn sorter(
    model: &KeyedModel<ProjectId, ui::TableRow>,
    catalogue: &Rc<Catalogue>,
    headers: &Rc<VecModel<ui::TableColumn>>,
    column: Column,
    language: ui::TableLanguage,
) -> impl Fn() + 'static {
    let compare = {
        let catalogue = catalogue.clone();
        move |left: &ProjectId, right: &ProjectId| catalogue.compare(column, *left, *right)
    };
    let (catalogue, headers) = (catalogue.clone(), headers.clone());
    model.sorting_by(compare, move |direction| {
        catalogue.order.set(Some((column, direction)));
        headers.set_vec(columns(language, catalogue.order.get()));
    })
}

pub fn create_app(language: ui::TableLanguage) -> Result<ui::TableApp, slint::PlatformError> {
    let app = ui::TableApp::new()?;
    slint_kit::install(&app);
    app.set_language(language);
    let catalogue = Rc::new(Catalogue::new());
    let model = KeyedModel::new(catalogue.rows(true)).expect("project keys are unique");
    // Rows, generation and active index reach the window through a weak reference.
    model.bind(
        &app,
        ui::TableApp::set_rows,
        ui::TableApp::set_generation,
        ui::TableApp::set_active_index,
    );
    let headers = Rc::new(VecModel::from(columns(language, None)));
    app.set_columns(ModelRc::from(headers.clone()));
    app.set_selection_label(catalogue.selection_label(language).into());
    app.on_navigate(model.navigation(&app, ui::TableApp::get_page_rows));
    app.on_activate(model.activation({
        let (weak, catalogue) = (app.as_weak(), catalogue.clone());
        move |id| {
            catalogue.selected.set(Some(id));
            if let Some(app) = weak.upgrade() {
                app.set_selection_label(catalogue.selection_label(language).into());
            }
        }
    }));
    let by_name = sorter(&model, &catalogue, &headers, Column::Name, language);
    let by_tasks = sorter(&model, &catalogue, &headers, Column::Tasks, language);
    app.on_sort_requested(move |token| match Column::from_token(token) {
        Some(Column::Name) => by_name(),
        Some(Column::Tasks) => by_tasks(),
        None => {}
    });
    app.on_show_archived_changed({
        let (model, catalogue) = (model.clone(), catalogue.clone());
        move |show_archived| {
            model
                .replace(catalogue.rows(show_archived))
                .expect("project keys are unique");
        }
    });
    Ok(app)
}

pub fn main() -> Result<(), slint::PlatformError> {
    let language = if std::env::args().any(|arg| arg == "--fr") {
        ui::TableLanguage::French
    } else {
        ui::TableLanguage::English
    };
    create_app(language)?.run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::Model;

    fn names(model: &KeyedModel<ProjectId, ui::TableRow>) -> Vec<String> {
        model
            .model()
            .iter()
            .map(|row| row.content.row_data(0).expect("name cell").text.to_string())
            .collect()
    }
    fn token(model: &KeyedModel<ProjectId, ui::TableRow>, index: usize) -> i32 {
        model.model().row_data(index).expect("row").token
    }

    #[test]
    fn selection_and_active_row_follow_the_key_through_sort_and_filter() {
        let language = ui::TableLanguage::English;
        let catalogue = Rc::new(Catalogue::new());
        let model = KeyedModel::new(catalogue.rows(true)).unwrap();
        let headers = Rc::new(VecModel::from(columns(language, None)));
        let by_tasks = sorter(&model, &catalogue, &headers, Column::Tasks, language);
        let select = model.activation({
            let catalogue = catalogue.clone();
            move |id| catalogue.selected.set(Some(id))
        });
        let generation = model.generation();
        select(token(&model, 0), generation);
        assert_eq!(catalogue.selected.get(), Some(ProjectId(101)));
        assert_eq!(
            catalogue.selection_label(language),
            "Selected: Atlas · #101"
        );

        by_tasks();
        assert_eq!(names(&model), ["Boreal", "Delta", "Atlas", "Cedar"]);
        assert!(model.model().row_data(2).unwrap().selected);
        assert_eq!(model.active_index(), 2);
        assert_eq!(
            headers.row_data(1).unwrap().sort,
            ui::DataSortDirection::Ascending
        );
        assert_eq!(
            headers.row_data(0).unwrap().sort,
            ui::DataSortDirection::Unsorted
        );

        // A gesture from the previous generation and a disabled row are rejected.
        select(token(&model, 1), generation);
        select(token(&model, 3), model.generation());
        assert_eq!(catalogue.selected.get(), Some(ProjectId(101)));

        // Hiding archived projects keeps the selected and active key.
        model.replace(catalogue.rows(false)).unwrap();
        assert_eq!(names(&model), ["Boreal", "Delta", "Atlas"]);
        assert!(model.model().row_data(2).unwrap().selected);
        assert_eq!(model.active_index(), 2);
        model.replace(catalogue.rows(true)).unwrap();
        assert_eq!(model.active_index(), 2);

        // Navigation skips the archived row; descending order follows the key.
        model.navigate(slint_kit::NavigationStep::Last, 1);
        assert_eq!(model.active_index(), 2);
        by_tasks();
        assert_eq!(names(&model), ["Cedar", "Atlas", "Delta", "Boreal"]);
        assert_eq!(model.active_index(), 1);
        assert!(model.model().row_data(1).unwrap().selected);
        assert_eq!(
            headers.row_data(1).unwrap().sort,
            ui::DataSortDirection::Descending
        );
    }
}
