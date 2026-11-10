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
type ErrorMapper = Rc<dyn Fn(&HostError) -> SharedString>;
fn default_error_text(error: &HostError) -> SharedString {
    let (_, message) = error_presentation(error);
    match error {
        HostError::Platform(detail) => format!("{message} {detail}").into(),
        HostError::Unsupported
        | HostError::PermissionDenied
        | HostError::Unavailable
        | HostError::InvalidResource
        | HostError::InvalidFilter => message.into(),
    }
}
fn mapped_error(
    error: &HostError,
    mapper: &RefCell<ErrorMapper>,
) -> (ui::HostRequestPresentation, SharedString) {
    let (state, _) = error_presentation(error);
    if state == ui::HostRequestPresentation::Unsupported {
        return (state, SharedString::default());
    }
    let mapper = mapper.borrow().clone();
    (state, mapper(error))
}
pub struct FilePickerBinding<W: ComponentHandle + 'static> {
    drop_available: Rc<Cell<bool>>,
    selection_mode: Rc<Cell<FileSelectionMode>>,
    access: Rc<Cell<crate::FieldAccess>>,
    window: slint::Weak<W>,
    ports: Rc<FilePickerPorts<W>>,
    error_mapper: Rc<RefCell<ErrorMapper>>,
    files: Rc<RefCell<Files>>,
    handler: Rc<RefCell<Option<FileHandler>>>,
}
impl<W: ComponentHandle + 'static> Clone for FilePickerBinding<W> {
    fn clone(&self) -> Self {
        Self {
            drop_available: self.drop_available.clone(),
            selection_mode: self.selection_mode.clone(),
            access: self.access.clone(),
            window: self.window.clone(),
            ports: self.ports.clone(),
            error_mapper: self.error_mapper.clone(),
            files: self.files.clone(),
            handler: self.handler.clone(),
        }
    }
}
fn error_presentation(error: &HostError) -> (ui::HostRequestPresentation, &'static str) {
    match error {
        HostError::Unsupported => (ui::HostRequestPresentation::Unsupported, ""),
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
            selection_mode: Rc::new(Cell::new(FileSelectionMode::Single)),
            access: Rc::new(Cell::new(crate::FieldAccess::Editable)),
            window: window.as_weak(),
            ports: Rc::new(ports),
            error_mapper: Rc::new(RefCell::new(Rc::new(default_error_text))),
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
    /// Set the same mode as the picker. Single-file drops replace the selection.
    pub fn set_mode(&self, mode: FileSelectionMode) {
        self.selection_mode.set(mode);
    }
    /// Set the current field access before forwarding native window events.
    pub fn set_access(&self, access: crate::FieldAccess) {
        self.access.set(access);
        if access != crate::FieldAccess::Editable {
            self.files.borrow_mut().requests.cancel();
        }
    }
    pub fn on_selection(&self, handler: impl Fn(Vec<FileHandle>) + 'static) {
        *self.handler.borrow_mut() = Some(Rc::new(handler));
    }
    pub fn selected(&self) -> Vec<FileHandle> {
        self.files.borrow().handles.clone()
    }
    /// Map typed host failures to localized UI text. Unsupported remains a
    /// capability state and never publishes a field error.
    pub fn set_error_mapper(&self, mapper: impl Fn(&HostError) -> SharedString + 'static) {
        *self.error_mapper.borrow_mut() = Rc::new(mapper);
    }
    fn fail(&self, error: HostError) {
        if let Some(window) = self.window.upgrade() {
            let (state, message) = mapped_error(&error, &self.error_mapper);
            (self.ports.state)(&window, state);
            (self.ports.error)(&window, message);
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
        if self.access.get() != crate::FieldAccess::Editable {
            return;
        }
        let Some(window) = self.window.upgrade() else {
            return;
        };
        let selection = match mode {
            ui::FileSelectionMode::Single => FileSelectionMode::Single,
            ui::FileSelectionMode::Multiple => FileSelectionMode::Multiple,
        };
        self.set_mode(selection);
        let ticket = self.files.borrow_mut().requests.begin();
        (self.ports.state)(&window, ui::HostRequestPresentation::Pending);
        (self.ports.error)(&window, SharedString::default());
        let binding = self.clone();
        if slint::spawn_local(async move {
            let future = {
                let Some(window) = binding.window.upgrade() else {
                    return;
                };
                crate::native::pick_files_with_parent(
                    FileRequest {
                        selection,
                        filters: vec![],
                        suggested_name: None,
                    },
                    &window.window().window_handle(),
                )
            };
            let outcome = future.await;
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
            if binding.drop_available.get()
                && binding.access.get() == crate::FieldAccess::Editable
                && event.data.has_file_paths()
            {
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
            if !binding.drop_available.get()
                || binding.access.get() != crate::FieldAccess::Editable
                || binding.window.upgrade().is_none()
            {
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
        if self.access.get() != crate::FieldAccess::Editable || self.window.upgrade().is_none() {
            return;
        }
        let incoming: Vec<_> = paths
            .into_iter()
            .map(|path| FileHandle {
                display_name: path
                    .file_name()
                    .map_or_else(String::new, |name| name.to_string_lossy().into_owned()),
                location: FileLocation::Path(path),
                byte_length: None,
            })
            .collect();
        if incoming.is_empty() {
            return;
        }
        self.files.borrow_mut().requests.cancel();
        let handles = match self.selection_mode.get() {
            FileSelectionMode::Single => incoming.into_iter().last().into_iter().collect(),
            FileSelectionMode::Multiple => self.selected().into_iter().chain(incoming).collect(),
        };
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
    error_mapper: Rc<RefCell<ErrorMapper>>,
    requests: Rc<RefCell<HostRequestController<DirectoryHandle>>>,
    selected: Rc<RefCell<Option<DirectoryHandle>>>,
    handler: Rc<RefCell<Option<DirectoryHandler>>>,
}
impl<W: ComponentHandle + 'static> Clone for DirectoryPickerBinding<W> {
    fn clone(&self) -> Self {
        Self {
            window: self.window.clone(),
            ports: self.ports.clone(),
            error_mapper: self.error_mapper.clone(),
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
            error_mapper: Rc::new(RefCell::new(Rc::new(default_error_text))),
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
    /// Localize typed host failures; platform details are available to the mapper.
    pub fn set_error_mapper(&self, mapper: impl Fn(&HostError) -> SharedString + 'static) {
        *self.error_mapper.borrow_mut() = Rc::new(mapper);
    }
    fn fail(&self, error: HostError) {
        if let Some(window) = self.window.upgrade() {
            let (state, message) = mapped_error(&error, &self.error_mapper);
            (self.ports.state)(&window, state);
            (self.ports.error)(&window, message);
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
            let future = {
                let Some(window) = binding.window.upgrade() else {
                    return;
                };
                crate::native::pick_directory_with_parent(
                    DirectoryRequest::default(),
                    &window.window().window_handle(),
                )
            };
            let outcome = future.await;
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
    fn unsupported_picker_uses_capability_state_without_a_field_error() {
        let (state, message) = error_presentation(&HostError::Unsupported);
        assert_eq!(state, ui::HostRequestPresentation::Unsupported);
        assert!(
            message.is_empty(),
            "the picker renders its capability explanation"
        );
        let (state, message) = error_presentation(&HostError::PermissionDenied);
        assert_eq!(state, ui::HostRequestPresentation::Failed);
        assert!(
            !message.is_empty(),
            "actual failures retain an error message"
        );
    }

    #[test]
    fn both_picker_bindings_localize_typed_errors_and_preserve_platform_detail() {
        i_slint_backend_testing::init_no_event_loop();
        let window = ui::KitVirtualTextView::new().unwrap();
        let files = FilePickerBinding::new(
            &window,
            FilePickerPorts {
                entries: |_, _| {},
                state: |_, _| {},
                error: ui::KitVirtualTextView::set_error,
            },
        );
        files.fail(HostError::Platform("device unavailable".into()));
        assert!(window.get_error().contains("device unavailable"));
        files.set_error_mapper(|error| format!("Échec : {error:?}").into());
        files.fail(HostError::PermissionDenied);
        assert_eq!(window.get_error(), "Échec : PermissionDenied");
        files.fail(HostError::Unsupported);
        assert!(window.get_error().is_empty());
        let directory = DirectoryPickerBinding::new(
            &window,
            DirectoryPickerPorts {
                selected: |_, _| {},
                state: |_, _| {},
                error: ui::KitVirtualTextView::set_error,
            },
        );
        directory.set_error_mapper(|error| format!("Dossier : {error:?}").into());
        directory.fail(HostError::Platform("accès refusé".into()));
        assert!(window.get_error().starts_with("Dossier : "));
        assert!(window.get_error().contains("accès refusé"));
        directory.fail(HostError::Unsupported);
        assert!(window.get_error().is_empty());
    }

    #[test]
    fn dropped_files_respect_single_selection_and_field_access() {
        i_slint_backend_testing::init_no_event_loop();
        let window = ui::KitVirtualTextView::new().unwrap();
        let binding = FilePickerBinding::new(
            &window,
            FilePickerPorts {
                entries: |_, _| {},
                state: |_, _| {},
                error: |_, _| {},
            },
        );
        let first = std::path::PathBuf::from("/example/first.txt");
        let last = std::path::PathBuf::from("/example/last.txt");
        binding.accept_drop([first.clone(), last.clone()]);
        assert_eq!(binding.selected().len(), 1);
        assert_eq!(
            binding.selected()[0].location,
            FileLocation::Path(last.clone())
        );
        for access in [crate::FieldAccess::ReadOnly, crate::FieldAccess::Disabled] {
            binding.set_access(access);
            binding.accept_drop([first.clone()]);
            assert_eq!(
                binding.selected()[0].location,
                FileLocation::Path(last.clone())
            );
        }
        binding.set_access(crate::FieldAccess::Editable);
        binding.accept_drop([first.clone()]);
        assert_eq!(binding.selected()[0].location, FileLocation::Path(first));
        binding.accept_drop([]);
        assert_eq!(
            binding.selected().len(),
            1,
            "an empty drop preserves selection"
        );
    }

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
        binding.set_mode(FileSelectionMode::Multiple);
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
