use slint::ComponentHandle;
use slint_kit::{
    FileHandle, FileRequest, HostCapability, HostError, HostRequestController, SelectionOutcome,
};
use std::{cell::RefCell, rc::Rc};
mod ui {
    slint::include_modules!();
}
#[derive(Clone, Copy, PartialEq)]
struct FileId(i32);
struct Entry {
    id: FileId,
    handle: FileHandle,
}
struct Files {
    generation: i32,
    next_id: i32,
    entries: Vec<Entry>,
    requests: HostRequestController<Vec<FileHandle>>,
}
fn publish(app: &ui::App, state: &Files) {
    app.set_entries(slint::ModelRc::new(slint::VecModel::from(
        state
            .entries
            .iter()
            .map(|entry| ui::FileViewEntry {
                token: entry.id.0,
                generation: state.generation,
                name: entry.handle.display_name.as_str().into(),
                detail: "Selected resource".into(),
                removable: true,
            })
            .collect::<Vec<_>>(),
    )));
    app.set_status(format!("{} selected files", state.entries.len()).into());
}
fn replace(state: &mut Files, files: Vec<FileHandle>) {
    state.entries.clear();
    for handle in files {
        let id = FileId(state.next_id);
        state.next_id = state.next_id.checked_add(1).expect("resource identity");
        state.entries.push(Entry { id, handle });
    }
    state.generation = state.generation.checked_add(1).expect("view generation");
}
fn failed(app: &ui::App, error: HostError) {
    app.set_request_state(if error == HostError::Unsupported {
        ui::HostRequestPresentation::Unsupported
    } else {
        ui::HostRequestPresentation::Failed
    });
    app.set_error(format!("{error:?}").into());
}
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    slint_kit::init_form_validation!(ui, app);
    let state = Rc::new(RefCell::new(Files {
        generation: 1,
        next_id: 1,
        entries: Vec::new(),
        requests: HostRequestController::default(),
    }));
    publish(&app, &state.borrow());
    if slint_kit::native::capabilities().files == HostCapability::Unsupported {
        app.set_request_state(ui::HostRequestPresentation::Unsupported);
    }
    let weak = app.as_weak();
    let shared = state.clone();
    app.on_browse(move |mode| {
        let Some(app) = weak.upgrade() else {
            return;
        };
        let ticket = shared.borrow_mut().requests.begin();
        app.set_request_state(ui::HostRequestPresentation::Pending);
        app.set_error("".into());
        let shared_async = shared.clone();
        let weak_async = weak.clone();
        if slint::spawn_local(async move {
            let outcome = slint_kit::native::pick_files(FileRequest {
                selection: if mode == ui::FileSelectionMode::Single {
                    slint_kit::FileSelectionMode::Single
                } else {
                    slint_kit::FileSelectionMode::Multiple
                },
                filters: Vec::new(),
                suggested_name: None,
            })
            .await;
            let Some(app) = weak_async.upgrade() else {
                return;
            };
            let mut state = shared_async.borrow_mut();
            if !state.requests.finish(ticket, outcome) {
                return;
            }
            match state.requests.take_result().unwrap() {
                SelectionOutcome::Selected(files) => {
                    replace(&mut state, files);
                    app.set_request_state(ui::HostRequestPresentation::Idle);
                    publish(&app, &state);
                }
                SelectionOutcome::Cancelled => {
                    app.set_request_state(ui::HostRequestPresentation::Cancelled)
                }
                SelectionOutcome::Failed(error) => failed(&app, error),
            }
        })
        .is_err()
        {
            shared.borrow_mut().requests.cancel();
            failed(&app, HostError::Unavailable);
        }
    });
    let weak = app.as_weak();
    let shared = state.clone();
    app.on_remove(move |token, generation| {
        if let Some(app) = weak.upgrade() {
            let mut state = shared.borrow_mut();
            if state.generation != generation {
                return;
            }
            state.requests.cancel();
            state.entries.retain(|entry| entry.id != FileId(token));
            state.generation = state.generation.checked_add(1).expect("view generation");
            app.set_request_state(ui::HostRequestPresentation::Idle);
            publish(&app, &state);
        }
    });
    use slint::language::DragAction;
    #[cfg(not(target_arch = "wasm32"))]
    {
        use slint::winit_030::{WinitWindowAccessor, winit::event::WindowEvent};
        let weak = app.as_weak();
        let shared = state.clone();
        app.window().on_winit_window_event(move |native, event| {
            if let Some(app) = weak.upgrade() {
                if cfg!(any(target_os = "macos", target_os = "windows"))
                    && native.has_winit_window()
                {
                    app.set_drop_capability(ui::DropCapability::Available);
                }
                match event {
                    WindowEvent::HoveredFile(_) => app.set_hovered(true),
                    WindowEvent::HoveredFileCancelled => app.set_hovered(false),
                    WindowEvent::DroppedFile(path) => {
                        let mut state = shared.borrow_mut();
                        state.requests.cancel();
                        let id = FileId(state.next_id);
                        state.next_id = state.next_id.checked_add(1).expect("file identity");
                        state.entries.push(Entry {
                            id,
                            handle: slint_kit::FileHandle {
                                display_name: path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .into_owned(),
                                location: slint_kit::FileLocation::Path(path.to_owned()),
                                byte_length: None,
                            },
                        });
                        state.generation =
                            state.generation.checked_add(1).expect("view generation");
                        app.set_hovered(false);
                        publish(&app, &state);
                    }
                    _ => {}
                }
            }
            slint::winit_030::EventResult::Propagate
        });
    }
    let weak = app.as_weak();
    app.on_can_drop(move |event| {
        if weak
            .upgrade()
            .is_some_and(|app| app.get_drop_capability() == ui::DropCapability::Available)
            && event.data.has_file_paths()
        {
            DragAction::Copy
        } else {
            DragAction::None
        }
    });
    let weak = app.as_weak();
    let shared = state.clone();
    app.on_dropped(move |event| {
        let Some(app) = weak.upgrade() else {
            return DragAction::None;
        };
        if app.get_drop_capability() != ui::DropCapability::Available {
            return DragAction::None;
        }
        let Ok(paths) = event.data.file_paths() else {
            return DragAction::None;
        };
        let files = paths
            .map(|path| slint_kit::FileHandle {
                display_name: path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                location: slint_kit::FileLocation::Path(path.to_owned()),
                byte_length: None,
            })
            .collect::<Vec<_>>();
        if files.is_empty() {
            return DragAction::None;
        }
        let mut state = shared.borrow_mut();
        state.requests.cancel();
        for handle in files {
            let id = FileId(state.next_id);
            state.next_id = state.next_id.checked_add(1).expect("file identity");
            state.entries.push(Entry { id, handle });
        }
        state.generation = state.generation.checked_add(1).expect("view generation");
        publish(&app, &state);
        DragAction::Copy
    });
    app.run()
}
