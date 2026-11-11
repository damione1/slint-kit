// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use crate::{
    DirectoryHandle, DirectoryRequest, FileConstraints, FileHandle, FileLocation, FileMetadata,
    FileRequest, FileSelectionMode, FileViolation, HostCapability, HostError,
    HostRequestController, KeyedModel, SelectionOutcome, ui,
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
type DetailFormatter = Rc<dyn Fn(&FileHandle) -> SharedString>;
type ConstraintMapper = Rc<dyn Fn(&FileViolation) -> SharedString>;
fn constraint_message(issue: &FileViolation) -> SharedString {
    match issue {
        FileViolation::TypeMismatch { name } => {
            format!("{name}: file type is not accepted.").into()
        }
        FileViolation::TooMany { maximum } => format!("Choose at most {maximum} files.").into(),
        FileViolation::TooLarge { name, maximum } => {
            format!("{name}: choose a file of at most {maximum} bytes.").into()
        }
        FileViolation::UnknownSize { name } => format!("{name}: file size is unavailable.").into(),
    }
}
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
    request: Rc<RefCell<FileRequest>>,
    constraints: Rc<RefCell<FileConstraints>>,
    detail: Rc<RefCell<DetailFormatter>>,
    constraint_mapper: Rc<RefCell<ConstraintMapper>>,
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
            request: self.request.clone(),
            constraints: self.constraints.clone(),
            detail: self.detail.clone(),
            constraint_mapper: self.constraint_mapper.clone(),
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
            request: Rc::new(RefCell::new(FileRequest::default())),
            constraints: Rc::new(RefCell::new(FileConstraints::default())),
            detail: Rc::new(RefCell::new(Rc::new(|file: &FileHandle| {
                file.byte_length
                    .map_or_else(SharedString::default, |bytes| format!("{bytes} B").into())
            }))),
            constraint_mapper: Rc::new(RefCell::new(Rc::new(constraint_message))),
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
        self.request.borrow_mut().selection = mode;
    }
    /// Configure future native requests. Filters are parsed once by FileFilter.
    pub fn set_request(&self, request: FileRequest) {
        *self.request.borrow_mut() = request;
    }
    pub fn request(&self) -> FileRequest {
        self.request.borrow().clone()
    }
    /// Configure and launch one request, retaining options for the browse handler.
    pub fn browse_with(&self, request: FileRequest) {
        let mode = match request.selection {
            FileSelectionMode::Single => ui::FileSelectionMode::Single,
            FileSelectionMode::Multiple => ui::FileSelectionMode::Multiple,
        };
        self.set_request(request);
        self.browse(mode);
    }
    /// Guard accepted resources with the shared HTML file rules. Set equivalent
    /// accept/max-files/max-size on the component for form reporting.
    pub fn set_constraints(&self, constraints: FileConstraints) {
        *self.constraints.borrow_mut() = constraints;
    }
    pub fn constraints(&self) -> FileConstraints {
        self.constraints.borrow().clone()
    }
    pub fn set_constraint_mapper(&self, mapper: impl Fn(&FileViolation) -> SharedString + 'static) {
        *self.constraint_mapper.borrow_mut() = Rc::new(mapper);
    }
    /// Format known or unavailable metadata in the caller's locale. Formatting
    /// updates existing row identities; it does not reselect accepted resources.
    pub fn set_detail_formatter(&self, formatter: impl Fn(&FileHandle) -> SharedString + 'static) {
        *self.detail.borrow_mut() = Rc::new(formatter);
        let (handles, rows) = {
            let files = self.files.borrow();
            (files.handles.clone(), files.rows.clone())
        };
        let formatter = self.detail.borrow().clone();
        for handle in handles {
            let detail = formatter(&handle);
            rows.update_by_key(&handle.location, |row| row.detail = detail);
        }
    }
    fn fail_constraint(&self, issue: FileViolation) {
        let mapper = self.constraint_mapper.borrow().clone();
        let message = mapper(&issue);
        if let Some(window) = self.window.upgrade() {
            (self.ports.state)(&window, ui::HostRequestPresentation::Failed);
            (self.ports.error)(&window, message);
        }
    }
    /// Set the current field access before forwarding native window events.
    pub fn set_access(&self, access: crate::FieldAccess) {
        self.access.set(access);
        if access != crate::FieldAccess::Editable {
            let pending = {
                let mut files = self.files.borrow_mut();
                let pending = files.requests.pending();
                files.requests.cancel();
                pending
            };
            if pending && let Some(window) = self.window.upgrade() {
                (self.ports.state)(&window, ui::HostRequestPresentation::Cancelled);
            }
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
        let metadata = handles.iter().map(FileMetadata::from).collect::<Vec<_>>();
        if let Err(issue) = self.constraints().validate(&metadata) {
            self.fail_constraint(issue);
            return;
        }
        let formatter = self.detail.borrow().clone();
        let entries = handles
            .iter()
            .map(|handle| {
                (
                    handle.location.clone(),
                    ui::FileViewEntry {
                        name: handle.display_name.as_str().into(),
                        detail: formatter(handle),
                        byte_length: handle
                            .byte_length
                            .map_or_else(SharedString::default, |bytes| bytes.to_string().into()),
                        removable: true,
                        ..Default::default()
                    },
                )
            })
            .collect::<Vec<_>>();
        {
            let mut state = self.files.borrow_mut();
            state
                .rows
                .replace(entries)
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
        let Some(operation) = self.browse_operation(mode, |request, window| {
            crate::native::pick_files_with_parent(request, &window.window().window_handle())
        }) else {
            return;
        };
        if slint::spawn_local(operation).is_err() {
            self.files.borrow_mut().requests.cancel();
            self.fail(HostError::Unavailable);
        }
    }
    // The production and mock host drive this exact request snapshot/ticket path.
    fn browse_operation(
        &self,
        mode: ui::FileSelectionMode,
        launch: impl FnOnce(FileRequest, &W) -> crate::native::PickerFuture<Vec<FileHandle>> + 'static,
    ) -> Option<impl std::future::Future<Output = ()> + 'static> {
        if self.access.get() != crate::FieldAccess::Editable {
            return None;
        }
        let Some(window) = self.window.upgrade() else {
            return None;
        };
        let selection = match mode {
            ui::FileSelectionMode::Single => FileSelectionMode::Single,
            ui::FileSelectionMode::Multiple => FileSelectionMode::Multiple,
        };
        self.set_mode(selection);
        let mut request = self.request();
        if request.filters.is_empty() {
            let constraints = self.constraints();
            match constraints
                .accept
                .native_filter(constraints.accept.description())
            {
                Ok(Some(filter)) => request.filters.push(filter),
                Ok(None) => {}
                Err(error) => {
                    self.fail(error);
                    return None;
                }
            }
        }
        let ticket = self.files.borrow_mut().requests.begin();
        (self.ports.state)(&window, ui::HostRequestPresentation::Pending);
        (self.ports.error)(&window, SharedString::default());
        let binding = self.clone();
        Some(async move {
            let future = {
                let Some(window) = binding.window.upgrade() else {
                    return;
                };
                launch(request, &window)
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
    /// Resolve path metadata off the UI thread before accepting a drop. A
    /// superseding selection, removal, blocked field or dead window fences it.
    pub fn accept_drop(&self, paths: impl IntoIterator<Item = std::path::PathBuf>) {
        let paths = paths.into_iter().collect::<Vec<_>>();
        if paths.is_empty() {
            return;
        }
        let mode = self.request.borrow().selection;
        let previous = match mode {
            FileSelectionMode::Single => Vec::new(),
            FileSelectionMode::Multiple => self.selected(),
        };
        let view_mode = match mode {
            FileSelectionMode::Single => ui::FileSelectionMode::Single,
            FileSelectionMode::Multiple => ui::FileSelectionMode::Multiple,
        };
        let Some(operation) = self.browse_operation(view_mode, move |_, _| {
            Box::pin(async move {
                match crate::native::inspect_file_paths(paths).await {
                    SelectionOutcome::Selected(incoming) => {
                        SelectionOutcome::Selected(match mode {
                            FileSelectionMode::Single => {
                                incoming.into_iter().last().into_iter().collect()
                            }
                            FileSelectionMode::Multiple => {
                                previous.into_iter().chain(incoming).collect()
                            }
                        })
                    }
                    SelectionOutcome::Cancelled => SelectionOutcome::Cancelled,
                    SelectionOutcome::Failed(error) => SelectionOutcome::Failed(error),
                }
            })
        }) else {
            return;
        };
        if slint::spawn_local(operation).is_err() {
            self.files.borrow_mut().requests.cancel();
            self.fail(HostError::Unavailable);
        }
    }
    /// Accept already-resolved resources from a host provider, using the same
    /// access, identity, count, type and size checks as native picker results.
    pub fn accept_handles(&self, incoming: Vec<FileHandle>) {
        if incoming.is_empty()
            || self.access.get() != crate::FieldAccess::Editable
            || self.window.upgrade().is_none()
        {
            return;
        }
        self.files.borrow_mut().requests.cancel();
        let handles = match self.request.borrow().selection {
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
    request: Rc<RefCell<DirectoryRequest>>,
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
            request: self.request.clone(),
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
            request: Rc::new(RefCell::new(DirectoryRequest::default())),
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
    pub fn set_request(&self, request: DirectoryRequest) {
        *self.request.borrow_mut() = request;
    }
    pub fn request(&self) -> DirectoryRequest {
        self.request.borrow().clone()
    }
    pub fn browse_with(&self, request: DirectoryRequest) {
        self.set_request(request);
        self.browse();
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
        let Some(operation) = self.browse_operation(|request, window| {
            crate::native::pick_directory_with_parent(request, &window.window().window_handle())
        }) else {
            return;
        };
        if slint::spawn_local(operation).is_err() {
            self.requests.borrow_mut().cancel();
            self.fail(HostError::Unavailable);
        }
    }
    fn browse_operation(
        &self,
        launch: impl FnOnce(DirectoryRequest, &W) -> crate::native::PickerFuture<DirectoryHandle>
        + 'static,
    ) -> Option<impl std::future::Future<Output = ()> + 'static> {
        let Some(window) = self.window.upgrade() else {
            return None;
        };
        let request = self.request();
        let ticket = self.requests.borrow_mut().begin();
        (self.ports.state)(&window, ui::HostRequestPresentation::Pending);
        (self.ports.error)(&window, SharedString::default());
        let binding = self.clone();
        Some(async move {
            let future = {
                let Some(window) = binding.window.upgrade() else {
                    return;
                };
                launch(request, &window)
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
    use crate::native_test_ui::PickerTestWindow;
    use slint::Model;

    fn handles(paths: impl IntoIterator<Item = std::path::PathBuf>) -> Vec<FileHandle> {
        paths
            .into_iter()
            .map(|path| FileHandle {
                display_name: path.file_name().unwrap().to_string_lossy().into_owned(),
                location: FileLocation::Path(path),
                byte_length: Some(12),
            })
            .collect()
    }
    #[test]
    fn configured_picker_requests_survive_clones_and_mode_updates() {
        i_slint_backend_testing::init_no_event_loop();
        let window = PickerTestWindow::new().unwrap();
        let files = FilePickerBinding::new(
            &window,
            FilePickerPorts {
                entries: |_, _| {},
                state: |_, _| {},
                error: |_, _| {},
            },
        );
        let request = FileRequest {
            selection: FileSelectionMode::Multiple,
            filters: vec![crate::FileFilter::new("Documents".into(), vec!["pdf".into()]).unwrap()],
            title: Some("Attach a document".into()),
            directory: Some(std::path::PathBuf::from("/tmp/documents")),
            ..Default::default()
        };
        files.set_request(request.clone());
        assert_eq!(files.clone().request(), request);
        files.clone().set_mode(FileSelectionMode::Single);
        let configured = files.request();
        assert_eq!(configured.selection, FileSelectionMode::Single);
        assert_eq!(configured.filters, request.filters);
        assert_eq!(configured.title, request.title);
        assert_eq!(configured.directory, request.directory);
        let directories = DirectoryPickerBinding::new(
            &window,
            DirectoryPickerPorts {
                selected: |_, _| {},
                state: |_, _| {},
                error: |_, _| {},
            },
        );
        let request = DirectoryRequest {
            title: Some("Project folder".into()),
            directory: Some(std::path::PathBuf::from("/tmp/projects")),
        };
        directories.set_request(request.clone());
        assert_eq!(directories.clone().request(), request);
    }

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
        let window = PickerTestWindow::new().unwrap();
        let files = FilePickerBinding::new(
            &window,
            FilePickerPorts {
                entries: |_, _| {},
                state: |_, _| {},
                error: PickerTestWindow::set_error,
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
                error: PickerTestWindow::set_error,
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
        let window = PickerTestWindow::new().unwrap();
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
        binding.accept_handles(handles([first.clone(), last.clone()]));
        assert_eq!(binding.selected().len(), 1);
        assert_eq!(
            binding.selected()[0].location,
            FileLocation::Path(last.clone())
        );
        for access in [crate::FieldAccess::ReadOnly, crate::FieldAccess::Disabled] {
            binding.set_access(access);
            binding.accept_handles(handles([first.clone()]));
            assert_eq!(
                binding.selected()[0].location,
                FileLocation::Path(last.clone())
            );
        }
        binding.set_access(crate::FieldAccess::Editable);
        binding.accept_handles(handles([first.clone()]));
        assert_eq!(binding.selected()[0].location, FileLocation::Path(first));
        binding.accept_handles(handles([]));
        assert_eq!(
            binding.selected().len(),
            1,
            "an empty drop preserves selection"
        );
    }

    #[test]
    fn file_collection_deduplicates_paths_fences_removal_and_releases_window() {
        i_slint_backend_testing::init_no_event_loop();
        let window = PickerTestWindow::new().unwrap();
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
        binding.accept_handles(handles([path.clone(), path]));
        assert_eq!(binding.selected().len(), 1);
        let row = binding.files.borrow().rows.model().row_data(0).unwrap();
        binding.accept_handles(handles([std::path::PathBuf::from("/example/second.txt")]));
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

#[cfg(test)]
mod request_tests {
    use super::*;
    use std::{
        future::Future,
        pin::Pin,
        task::{Context, Poll, Waker},
    };
    type Reply<T> = Rc<RefCell<Option<SelectionOutcome<T>>>>;
    fn poll(future: Pin<&mut impl Future<Output = ()>>) -> Poll<()> {
        future.poll(&mut Context::from_waker(Waker::noop()))
    }
    fn pending<T: 'static>(reply: Reply<T>) -> crate::native::PickerFuture<T> {
        Box::pin(std::future::poll_fn(move |_| {
            match reply.borrow_mut().take() {
                Some(outcome) => Poll::Ready(outcome),
                None => Poll::Pending,
            }
        }))
    }
    fn file(name: &str) -> FileHandle {
        FileHandle {
            location: FileLocation::Path(name.into()),
            display_name: name.into(),
            byte_length: Some(12),
        }
    }
    #[test]
    fn configured_file_request_reaches_mock_host_and_late_results_are_fenced() {
        i_slint_backend_testing::init_no_event_loop();
        let window = PickerTestWindow::new().unwrap();
        let binding = FilePickerBinding::new(
            &window,
            FilePickerPorts {
                entries: |_, _| {},
                state: |_, _| {},
                error: |_, _| {},
            },
        );
        let request = FileRequest {
            title: Some("Attach report".into()),
            directory: Some("/chosen/start".into()),
            filters: vec![crate::FileFilter::new("PDF".into(), vec!["pdf".into()]).unwrap()],
            suggested_name: Some("report.pdf".into()),
            ..Default::default()
        };
        binding.set_request(request.clone());
        let recorded = Rc::new(RefCell::new(Vec::new()));
        let first: Reply<Vec<FileHandle>> = Rc::default();
        let first_reply = first.clone();
        let requests = recorded.clone();
        let mut first_operation = Box::pin(
            binding
                .browse_operation(ui::FileSelectionMode::Multiple, move |request, _| {
                    requests.borrow_mut().push(request);
                    pending(first_reply)
                })
                .unwrap(),
        );
        let mut updated = request.clone();
        updated.title = Some("Later request".into());
        binding.set_request(updated.clone());
        let second: Reply<Vec<FileHandle>> = Rc::default();
        let second_reply = second.clone();
        let requests = recorded.clone();
        let mut second_operation = Box::pin(
            binding
                .browse_operation(ui::FileSelectionMode::Single, move |request, _| {
                    requests.borrow_mut().push(request);
                    pending(second_reply)
                })
                .unwrap(),
        );
        assert!(poll(first_operation.as_mut()).is_pending());
        assert!(poll(second_operation.as_mut()).is_pending());
        let mut first_expected = request;
        first_expected.selection = FileSelectionMode::Multiple;
        assert_eq!(*recorded.borrow(), vec![first_expected, updated]);
        *second.borrow_mut() = Some(SelectionOutcome::Selected(vec![file("current.pdf")]));
        assert!(poll(second_operation.as_mut()).is_ready());
        *first.borrow_mut() = Some(SelectionOutcome::Selected(vec![file("obsolete.pdf")]));
        assert!(poll(first_operation.as_mut()).is_ready());
        assert_eq!(binding.selected(), vec![file("current.pdf")]);
    }
    #[test]
    fn directory_request_snapshot_clear_and_window_lifetime_share_the_real_path() {
        i_slint_backend_testing::init_no_event_loop();
        let window = PickerTestWindow::new().unwrap();
        let binding = DirectoryPickerBinding::new(
            &window,
            DirectoryPickerPorts {
                selected: |_, _| {},
                state: |_, _| {},
                error: |_, _| {},
            },
        );
        let request = DirectoryRequest {
            title: Some("Workspace".into()),
            directory: Some("/chosen/workspace".into()),
        };
        binding.set_request(request.clone());
        let recorded = Rc::new(RefCell::new(Vec::new()));
        let requests = recorded.clone();
        let reply: Reply<DirectoryHandle> = Rc::default();
        let response = reply.clone();
        let mut operation = Box::pin(
            binding
                .browse_operation(move |request, _| {
                    requests.borrow_mut().push(request);
                    pending(response)
                })
                .unwrap(),
        );
        binding.set_request(DirectoryRequest::default());
        assert!(poll(operation.as_mut()).is_pending());
        assert_eq!(*recorded.borrow(), vec![request]);
        binding.clear();
        *reply.borrow_mut() = Some(SelectionOutcome::Selected(DirectoryHandle {
            location: FileLocation::Path("/late".into()),
            display_name: "Late".into(),
        }));
        assert!(poll(operation.as_mut()).is_ready());
        assert_eq!(binding.selected(), None);
        let launches = Rc::new(Cell::new(0));
        let count = launches.clone();
        let mut destroyed = Box::pin(
            binding
                .browse_operation(move |_, _| {
                    count.set(count.get() + 1);
                    Box::pin(async { SelectionOutcome::Cancelled })
                })
                .unwrap(),
        );
        let weak = window.as_weak();
        drop(window);
        assert!(weak.upgrade().is_none());
        assert!(poll(destroyed.as_mut()).is_ready());
        assert_eq!(launches.get(), 0);
    }
    #[test]
    fn access_change_invalidates_pending_file_response_without_retaining_window() {
        i_slint_backend_testing::init_no_event_loop();
        let window = PickerTestWindow::new().unwrap();
        let binding = FilePickerBinding::new(
            &window,
            FilePickerPorts {
                entries: |_, _| {},
                state: |_, _| {},
                error: |_, _| {},
            },
        );
        let reply: Reply<Vec<FileHandle>> = Rc::default();
        let response = reply.clone();
        let mut operation = Box::pin(
            binding
                .browse_operation(ui::FileSelectionMode::Single, move |_, _| pending(response))
                .unwrap(),
        );
        assert!(poll(operation.as_mut()).is_pending());
        binding.set_access(crate::FieldAccess::ReadOnly);
        *reply.borrow_mut() = Some(SelectionOutcome::Selected(vec![file("blocked.pdf")]));
        assert!(poll(operation.as_mut()).is_ready());
        assert!(binding.selected().is_empty());
        let weak = window.as_weak();
        drop(window);
        assert!(weak.upgrade().is_none());
    }
}

#[cfg(test)]
mod file_constraint_tests {
    use super::*;
    use slint::Model as _;
    fn handle(name: &str, bytes: Option<u64>) -> FileHandle {
        FileHandle {
            location: FileLocation::Path(name.into()),
            display_name: name.into(),
            byte_length: bytes,
        }
    }
    fn binding() -> (
        ui::KitVirtualTextView,
        FilePickerBinding<ui::KitVirtualTextView>,
    ) {
        i_slint_backend_testing::init_no_event_loop();
        let window = PickerTestWindow::new().unwrap();
        let binding = FilePickerBinding::new(
            &window,
            FilePickerPorts {
                entries: |_, _| {},
                state: |_, _| {},
                error: ui::KitVirtualTextView::set_error,
            },
        );
        (window, binding)
    }
    #[test]
    fn accepted_resources_share_count_type_size_guards_and_preserve_prior_selection() {
        let (window, binding) = binding();
        binding.set_constraints(FileConstraints {
            accept: crate::FileAccept::parse(".pdf").unwrap(),
            max_files: Some(1),
            max_size: Some(100),
        });
        binding.accept_handles(vec![handle("report.pdf", Some(100))]);
        let accepted = binding.selected();
        for incoming in [
            handle("wrong.exe", Some(1)),
            handle("large.pdf", Some(101)),
            handle("unknown.pdf", None),
        ] {
            binding.accept_handles(vec![incoming]);
            assert_eq!(binding.selected(), accepted);
            assert!(!window.get_error().is_empty());
        }
        binding.set_mode(FileSelectionMode::Multiple);
        binding.accept_handles(vec![handle("second.pdf", Some(1))]);
        assert_eq!(binding.selected(), accepted);
        assert!(window.get_error().contains("at most 1"));
    }
    #[test]
    fn detail_formatter_uses_exact_metadata_without_changing_row_identity() {
        let (_window, binding) = binding();
        let bytes = 9_007_199_254_740_993;
        binding.accept_handles(vec![handle("large.pdf", Some(bytes))]);
        let before = binding.files.borrow().rows.model().row_data(0).unwrap();
        assert_eq!(before.byte_length, bytes.to_string());
        assert_eq!(before.detail, format!("{bytes} B"));
        binding.set_detail_formatter(|file| {
            file.byte_length.map_or_else(
                || "Taille inconnue".into(),
                |value| format!("{value} octets").into(),
            )
        });
        let after = binding.files.borrow().rows.model().row_data(0).unwrap();
        assert_eq!(
            (before.token, before.generation),
            (after.token, after.generation)
        );
        assert_eq!(after.detail, format!("{bytes} octets"));
    }
    #[test]
    fn configured_accept_becomes_native_filter_and_mock_result_is_checked_on_return() {
        use std::{
            future::Future as _,
            task::{Context, Waker},
        };
        let (window, binding) = binding();
        binding.set_constraints(FileConstraints {
            accept: crate::FileAccept::parse(".pdf").unwrap(),
            max_files: Some(1),
            max_size: Some(2),
        });
        binding.set_constraint_mapper(|issue| match issue {
            FileViolation::TooLarge { .. } => "Fichier trop grand".into(),
            FileViolation::TypeMismatch { .. } => "Type incorrect".into(),
            FileViolation::TooMany { .. } => "Trop de fichiers".into(),
            FileViolation::UnknownSize { .. } => "Taille inconnue".into(),
        });
        let launched = Rc::new(RefCell::new(None));
        let request = launched.clone();
        let mut operation = Box::pin(
            binding
                .browse_operation(ui::FileSelectionMode::Single, move |options, _| {
                    *request.borrow_mut() = Some(options);
                    Box::pin(async {
                        SelectionOutcome::Selected(vec![handle("large.pdf", Some(3))])
                    })
                })
                .unwrap(),
        );
        assert!(
            operation
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_ready()
        );
        assert_eq!(
            launched.borrow().as_ref().unwrap().filters[0].extensions(),
            &["pdf"]
        );
        assert!(binding.selected().is_empty());
        assert_eq!(window.get_error(), "Fichier trop grand");
    }
}

#[cfg(test)]
mod cancelled_presentation_tests {
    use super::*;
    use std::{
        future::Future as _,
        task::{Context, Poll, Waker},
    };
    #[test]
    fn blocking_a_pending_picker_clears_busy_state_and_late_response_stays_cancelled() {
        i_slint_backend_testing::init_no_event_loop();
        let window = ui::KitFilePicker::new().unwrap();
        let binding = FilePickerBinding::new(
            &window,
            FilePickerPorts {
                entries: ui::KitFilePicker::set_entries,
                state: ui::KitFilePicker::set_request_state,
                error: ui::KitFilePicker::set_error,
            },
        );
        let ready = Rc::new(Cell::new(false));
        let response = ready.clone();
        let mut operation = Box::pin(
            binding
                .browse_operation(ui::FileSelectionMode::Single, move |_, _| {
                    Box::pin(std::future::poll_fn(move |_| {
                        if response.get() {
                            Poll::Ready(SelectionOutcome::Cancelled)
                        } else {
                            Poll::Pending
                        }
                    }))
                })
                .unwrap(),
        );
        assert!(
            operation
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending()
        );
        assert_eq!(
            window.get_request_state(),
            ui::HostRequestPresentation::Pending
        );
        binding.set_access(crate::FieldAccess::ReadOnly);
        assert_eq!(
            window.get_request_state(),
            ui::HostRequestPresentation::Cancelled
        );
        ready.set(true);
        assert!(
            operation
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_ready()
        );
        assert_eq!(
            window.get_request_state(),
            ui::HostRequestPresentation::Cancelled
        );
    }
}
