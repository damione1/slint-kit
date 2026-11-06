// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use crate::{
    DirectoryHandle, DirectoryRequest, FileHandle, FileLocation, FileRequest, FileSelectionMode,
    HostCapability, HostError, HostRequestController, KeyedModel, SelectionOutcome, ui,
};
use slint::{ComponentHandle, ModelRc, SharedString};
use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    rc::Rc,
};

/// Explicit application projection; no root-property naming convention is used.
pub struct FilePickerPorts<W> {
    pub entries: fn(&W, ModelRc<ui::FileViewEntry>),
    pub state: fn(&W, ui::HostRequestPresentation),
    pub error: fn(&W, SharedString),
}
struct Files {
    handles: Vec<FileHandle>,
    rows: KeyedModel<FileLocation, ui::FileViewEntry>,
    requests: HostRequestController<Vec<FileHandle>>,
}
type FileHandler = Rc<dyn Fn(Vec<FileHandle>)>;
pub struct FilePickerBinding<W: ComponentHandle + 'static> {
    drop_available: Rc<Cell<bool>>,
    window: slint::Weak<W>,
    ports: Rc<FilePickerPorts<W>>,
    files: Rc<RefCell<Files>>,
    handler: Rc<RefCell<Option<FileHandler>>>,
}
impl<W: ComponentHandle + 'static> Clone for FilePickerBinding<W> {
    fn clone(&self) -> Self {
        Self {
            drop_available: self.drop_available.clone(),
            window: self.window.clone(),
            ports: self.ports.clone(),
            files: self.files.clone(),
            handler: self.handler.clone(),
        }
    }
}
fn error_presentation(error: &HostError) -> (ui::HostRequestPresentation, &'static str) {
    match error {
        HostError::Unsupported => (
            ui::HostRequestPresentation::Unsupported,
            "This platform does not provide this picker.",
        ),
        HostError::PermissionDenied => (
            ui::HostRequestPresentation::Failed,
            "Permission was denied.",
        ),
        HostError::Unavailable => (
            ui::HostRequestPresentation::Failed,
            "The picker is currently unavailable.",
        ),
        HostError::InvalidResource => (
            ui::HostRequestPresentation::Failed,
            "The selected resource is invalid.",
        ),
        HostError::InvalidFilter => (
            ui::HostRequestPresentation::Failed,
            "The file filter is invalid.",
        ),
        HostError::Platform(_) => (
            ui::HostRequestPresentation::Failed,
            "The platform could not complete this request.",
        ),
    }
}
impl<W: ComponentHandle + 'static> FilePickerBinding<W> {
    pub fn new(window: &W, ports: FilePickerPorts<W>) -> Self {
        let result = Self {
            drop_available: Rc::new(Cell::new(false)),
            window: window.as_weak(),
            ports: Rc::new(ports),
            files: Rc::new(RefCell::new(Files {
                handles: vec![],
                rows: KeyedModel::new([]).expect("empty model"),
                requests: HostRequestController::default(),
            })),
            handler: Rc::new(RefCell::new(None)),
        };
        (result.ports.entries)(window, result.files.borrow().rows.model());
        if crate::native::capabilities().files == HostCapability::Unsupported {
            result.fail(HostError::Unsupported);
        }
        result
    }
    pub fn on_selection(&self, handler: impl Fn(Vec<FileHandle>) + 'static) {
        *self.handler.borrow_mut() = Some(Rc::new(handler));
    }
    pub fn selected(&self) -> Vec<FileHandle> {
        self.files.borrow().handles.clone()
    }
    fn fail(&self, error: HostError) {
        if let Some(window) = self.window.upgrade() {
            let (state, message) = error_presentation(&error);
            (self.ports.state)(&window, state);
            (self.ports.error)(&window, message.into());
        }
    }
    fn replace(&self, handles: Vec<FileHandle>) {
        let mut seen = HashSet::new();
        let handles = handles
            .into_iter()
            .filter(|handle| seen.insert(handle.location.clone()))
            .collect::<Vec<_>>();
        {
            let mut state = self.files.borrow_mut();
            state
                .rows
                .replace(handles.iter().map(|handle| {
                    (
                        handle.location.clone(),
                        ui::FileViewEntry {
                            name: handle.display_name.as_str().into(),
                            removable: true,
                            ..Default::default()
                        },
                    )
                }))
                .expect("deduplicated resource locations");
            state.handles = handles.clone();
        }
        if let Some(window) = self.window.upgrade() {
            (self.ports.entries)(&window, self.files.borrow().rows.model());
            (self.ports.state)(&window, ui::HostRequestPresentation::Idle);
            (self.ports.error)(&window, SharedString::default());
        }
        let handler = self.handler.borrow().clone();
        if let Some(handler) = handler {
            handler(handles);
        }
    }
    pub fn browse(&self, mode: ui::FileSelectionMode) {
        let Some(window) = self.window.upgrade() else {
            return;
        };
        let selection = match mode {
            ui::FileSelectionMode::Single => FileSelectionMode::Single,
            ui::FileSelectionMode::Multiple => FileSelectionMode::Multiple,
        };
        let ticket = self.files.borrow_mut().requests.begin();
        (self.ports.state)(&window, ui::HostRequestPresentation::Pending);
        (self.ports.error)(&window, SharedString::default());
        let binding = self.clone();
        if slint::spawn_local(async move {
            let outcome = crate::native::pick_files(FileRequest {
                selection,
                filters: vec![],
                suggested_name: None,
            })
            .await;
            if binding.window.upgrade().is_none() {
                return;
            }
            let outcome = {
                let mut state = binding.files.borrow_mut();
                if !state.requests.finish(ticket, outcome) {
                    return;
                }
                state.requests.take_result().expect("finished request")
            };
            match outcome {
                SelectionOutcome::Selected(handles) => binding.replace(handles),
                SelectionOutcome::Cancelled => {
                    if let Some(window) = binding.window.upgrade() {
                        (binding.ports.state)(&window, ui::HostRequestPresentation::Cancelled);
                    }
                }
                SelectionOutcome::Failed(error) => binding.fail(error),
            }
        })
        .is_err()
        {
            self.files.borrow_mut().requests.cancel();
            self.fail(HostError::Unavailable);
        }
    }
    pub fn browse_handler(&self) -> impl Fn(ui::FileSelectionMode) + 'static {
        let binding = self.clone();
        move |mode| binding.browse(mode)
    }
    pub fn remove(&self, token: i32, generation: i32) {
        let key = self.files.borrow().rows.accept(token, generation);
        let Some(key) = key else {
            return;
        };
        self.files.borrow_mut().requests.cancel();
        let handles = self
            .selected()
            .into_iter()
            .filter(|handle| handle.location != key)
            .collect();
        self.replace(handles);
    }
    pub fn remove_handler(&self) -> impl Fn(i32, i32) + 'static {
        let binding = self.clone();
        move |token, generation| binding.remove(token, generation)
    }
    pub fn can_drop_handler(
        &self,
    ) -> impl Fn(slint::language::DropEvent) -> slint::language::DragAction + 'static {
        let binding = self.clone();
        move |event| {
            if binding.drop_available.get() && event.data.has_file_paths() {
                slint::language::DragAction::Copy
            } else {
                slint::language::DragAction::None
            }
        }
    }
    pub fn drop_handler(
        &self,
    ) -> impl Fn(slint::language::DropEvent) -> slint::language::DragAction + 'static {
        let binding = self.clone();
        move |event| {
            if !binding.drop_available.get() || binding.window.upgrade().is_none() {
                return slint::language::DragAction::None;
            }
            let Ok(paths) = event.data.file_paths() else {
                return slint::language::DragAction::None;
            };
            let paths = paths.map(std::path::Path::to_path_buf).collect::<Vec<_>>();
            if paths.is_empty() {
                return slint::language::DragAction::None;
            }
            binding.accept_drop(paths);
            slint::language::DragAction::Copy
        }
    }
    pub fn accept_drop(&self, paths: impl IntoIterator<Item = std::path::PathBuf>) {
        self.files.borrow_mut().requests.cancel();
        let mut handles = self.selected();
        handles.extend(paths.into_iter().map(|path| {
            FileHandle {
                display_name: path
                    .file_name()
                    .map_or_else(String::new, |name| name.to_string_lossy().into_owned()),
                location: FileLocation::Path(path),
                byte_length: None,
            }
        }));
        self.replace(handles);
    }
}

pub struct DirectoryPickerPorts<W> {
    pub selected: fn(&W, SharedString),
    pub state: fn(&W, ui::HostRequestPresentation),
    pub error: fn(&W, SharedString),
}
type DirectoryHandler = Rc<dyn Fn(Option<DirectoryHandle>)>;
pub struct DirectoryPickerBinding<W: ComponentHandle + 'static> {
    window: slint::Weak<W>,
    ports: Rc<DirectoryPickerPorts<W>>,
    requests: Rc<RefCell<HostRequestController<DirectoryHandle>>>,
    selected: Rc<RefCell<Option<DirectoryHandle>>>,
    handler: Rc<RefCell<Option<DirectoryHandler>>>,
}
impl<W: ComponentHandle + 'static> Clone for DirectoryPickerBinding<W> {
    fn clone(&self) -> Self {
        Self {
            window: self.window.clone(),
            ports: self.ports.clone(),
            requests: self.requests.clone(),
            selected: self.selected.clone(),
            handler: self.handler.clone(),
        }
    }
}
impl<W: ComponentHandle + 'static> DirectoryPickerBinding<W> {
    pub fn new(window: &W, ports: DirectoryPickerPorts<W>) -> Self {
        let result = Self {
            window: window.as_weak(),
            ports: Rc::new(ports),
            requests: Rc::new(RefCell::new(HostRequestController::default())),
            selected: Rc::new(RefCell::new(None)),
            handler: Rc::new(RefCell::new(None)),
        };
        if crate::native::capabilities().directories == HostCapability::Unsupported {
            result.fail(HostError::Unsupported);
        }
        result
    }
    pub fn on_selection(&self, handler: impl Fn(Option<DirectoryHandle>) + 'static) {
        *self.handler.borrow_mut() = Some(Rc::new(handler));
    }
    pub fn selected(&self) -> Option<DirectoryHandle> {
        self.selected.borrow().clone()
    }
    fn fail(&self, error: HostError) {
        if let Some(window) = self.window.upgrade() {
            let (state, message) = error_presentation(&error);
            (self.ports.state)(&window, state);
            (self.ports.error)(&window, message.into());
        }
    }
    fn publish(&self, selected: Option<DirectoryHandle>) {
        *self.selected.borrow_mut() = selected.clone();
        if let Some(window) = self.window.upgrade() {
            (self.ports.selected)(
                &window,
                selected
                    .as_ref()
                    .map_or_else(SharedString::default, |value| {
                        value.display_name.as_str().into()
                    }),
            );
            (self.ports.state)(&window, ui::HostRequestPresentation::Idle);
            (self.ports.error)(&window, SharedString::default());
        }
        let handler = self.handler.borrow().clone();
        if let Some(handler) = handler {
            handler(selected);
        }
    }
    pub fn browse(&self) {
        let Some(window) = self.window.upgrade() else {
            return;
        };
        let ticket = self.requests.borrow_mut().begin();
        (self.ports.state)(&window, ui::HostRequestPresentation::Pending);
        (self.ports.error)(&window, SharedString::default());
        let binding = self.clone();
        if slint::spawn_local(async move {
            let outcome = crate::native::pick_directory(DirectoryRequest::default()).await;
            if binding.window.upgrade().is_none() {
                return;
            }
            let outcome = {
                let mut request = binding.requests.borrow_mut();
                if !request.finish(ticket, outcome) {
                    return;
                }
                request.take_result().expect("finished request")
            };
            match outcome {
                SelectionOutcome::Selected(handle) => binding.publish(Some(handle)),
                SelectionOutcome::Cancelled => {
                    if let Some(window) = binding.window.upgrade() {
                        (binding.ports.state)(&window, ui::HostRequestPresentation::Cancelled);
                    }
                }
                SelectionOutcome::Failed(error) => binding.fail(error),
            }
        })
        .is_err()
        {
            self.requests.borrow_mut().cancel();
            self.fail(HostError::Unavailable);
        }
    }
    pub fn browse_handler(&self) -> impl Fn() + 'static {
        let binding = self.clone();
        move || binding.browse()
    }
    pub fn clear(&self) {
        self.requests.borrow_mut().cancel();
        self.publish(None);
    }
    pub fn clear_handler(&self) -> impl Fn() + 'static {
        let binding = self.clone();
        move || binding.clear()
    }
}

#[cfg(feature = "native-winit")]
impl<W: ComponentHandle + 'static> FilePickerBinding<W> {
    /// Install the winit file-drop bridge. Availability is reported from the
    /// actual window backend; a Slint DropArea alone does not imply OS support.
    pub fn install_winit_drop(
        &self,
        capability: fn(&W, ui::DropCapability),
        hovered: fn(&W, bool),
    ) {
        let Some(window) = self.window.upgrade() else {
            return;
        };
        capability(&window, ui::DropCapability::Unsupported);
        #[cfg(not(target_arch = "wasm32"))]
        {
            use slint::winit_030::{WinitWindowAccessor, winit::event::WindowEvent};
            let binding = self.clone();
            window.window().on_winit_window_event(move |native, event| {
                if let Some(window) = binding.window.upgrade() {
                    if cfg!(any(target_os = "macos", target_os = "windows"))
                        && native.has_winit_window()
                    {
                        binding.drop_available.set(true);
                        capability(&window, ui::DropCapability::Available);
                    }
                    match event {
                        WindowEvent::HoveredFile(_) => hovered(&window, true),
                        WindowEvent::HoveredFileCancelled => hovered(&window, false),
                        WindowEvent::DroppedFile(path) => {
                            hovered(&window, false);
                            if binding.drop_available.get() {
                                binding.accept_drop([path.clone()]);
                            }
                        }
                        // Winit events are an evolving external set. Other OS
                        // events remain owned by Slint and propagate below.
                        _ => {}
                    }
                }
                slint::winit_030::EventResult::Propagate
            });
        }
        #[cfg(target_arch = "wasm32")]
        hovered(&window, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::Model;
    #[test]
    fn file_collection_deduplicates_paths_fences_removal_and_releases_window() {
        i_slint_backend_testing::init_no_event_loop();
        let window = ui::KitVirtualTextView::new().unwrap();
        let weak = window.as_weak();
        let binding = FilePickerBinding::new(
            &window,
            FilePickerPorts {
                entries: |_, _| {},
                state: |_, _| {},
                error: |_, _| {},
            },
        );
        let path = std::path::PathBuf::from("/example/project.txt");
        binding.accept_drop([path.clone(), path]);
        assert_eq!(binding.selected().len(), 1);
        let row = binding.files.borrow().rows.model().row_data(0).unwrap();
        binding.accept_drop([std::path::PathBuf::from("/example/second.txt")]);
        binding.remove(row.token, row.generation);
        assert_eq!(
            binding.selected().len(),
            2,
            "stale gesture cannot remove a replacement projection"
        );
        drop(window);
        assert!(
            weak.upgrade().is_none(),
            "native adapter must not retain the window"
        );
        assert_eq!(binding.selected().len(), 2);
    }
}
