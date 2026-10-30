use slint::ComponentHandle;
use std::{cell::RefCell, rc::Rc};
mod ui {
    slint::include_modules!();
}
#[derive(Clone, Copy, PartialEq)]
struct FileId(i32);
struct Entry {
    id: FileId,
    name: &'static str,
}
struct Files {
    generation: i32,
    entries: Vec<Entry>,
}
fn publish(app: &ui::App, state: &Files) {
    app.set_entries(slint::ModelRc::new(slint::VecModel::from(
        state
            .entries
            .iter()
            .map(|entry| ui::FileViewEntry {
                token: entry.id.0,
                generation: state.generation,
                name: entry.name.into(),
                detail: "Sample resource".into(),
                removable: true,
            })
            .collect::<Vec<_>>(),
    )));
    app.set_status(format!("{} selected files", state.entries.len()).into());
}
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    slint_kit::init_form_validation!(ui, app);
    let state = Rc::new(RefCell::new(Files {
        generation: 1,
        entries: vec![
            Entry {
                id: FileId(11),
                name: "brief.txt",
            },
            Entry {
                id: FileId(23),
                name: "notes.md",
            },
            Entry {
                id: FileId(37),
                name: "interface-preview.png",
            },
        ],
    }));
    publish(&app, &state.borrow());
    let weak = app.as_weak();
    app.on_remove(move |token, generation| {
        if let Some(app) = weak.upgrade() {
            let mut state = state.borrow_mut();
            if state.generation != generation {
                return;
            }
            state.entries.retain(|entry| entry.id != FileId(token));
            state.generation = state.generation.checked_add(1).expect("view generation");
            publish(&app, &state);
        }
    });
    app.run()
}
