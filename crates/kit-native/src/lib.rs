// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
//! Optional asynchronous host services, independent of any UI framework.
//! Construct/poll the picker on the host's UI executor (for Slint, spawn_local).
//! The returned handles do not read files or assume every resource is a path.
use kit_core::{
    DirectoryHandle, DirectoryRequest, FileHandle, FileRequest, HostCapabilities, HostCapability,
    HostError, SaveFileRequest, SelectionOutcome,
};

/// Picker availability of this adapter. Drag and drop is supplied separately by
/// the application's window backend, not implied by a working native picker.
pub const fn capabilities() -> HostCapabilities {
    let picker = if cfg!(any(
        target_os = "macos",
        target_os = "windows",
        target_os = "linux",
        target_os = "freebsd"
    )) {
        HostCapability::Available
    } else {
        HostCapability::Unsupported
    };
    HostCapabilities {
        files: picker,
        save_files: picker,
        directories: picker,
        drop: HostCapability::Unsupported,
    }
}

#[cfg(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd"
))]
pub async fn pick_files(request: FileRequest) -> SelectionOutcome<Vec<FileHandle>> {
    finish_files(file_picker(&request), request.selection).await
}
#[cfg(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd"
))]
fn file_picker(request: &FileRequest) -> rfd::AsyncFileDialog {
    let mut picker = rfd::AsyncFileDialog::new();
    for filter in &request.filters {
        picker = picker.add_filter(&filter.label, filter.extensions());
    }
    if let Some(name) = &request.suggested_name {
        picker = picker.set_file_name(name);
    }
    if let Some(title) = &request.title {
        picker = picker.set_title(title);
    }
    if let Some(directory) = &request.directory {
        picker = picker.set_directory(directory);
    }
    picker
}
#[cfg(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd"
))]
async fn finish_files(
    picker: rfd::AsyncFileDialog,
    selection: kit_core::FileSelectionMode,
) -> SelectionOutcome<Vec<FileHandle>> {
    use kit_core::FileSelectionMode;
    let result = match selection {
        FileSelectionMode::Single => picker.pick_file().await.map(|handle| vec![handle]),
        FileSelectionMode::Multiple => picker.pick_files().await,
    };
    match result {
        Some(files) => {
            inspect_file_paths(
                files
                    .into_iter()
                    .map(|handle| handle.path().to_owned())
                    .collect(),
            )
            .await
        }
        None => SelectionOutcome::Cancelled,
    }
}
#[cfg(not(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd"
)))]
pub async fn pick_files(_request: FileRequest) -> SelectionOutcome<Vec<FileHandle>> {
    SelectionOutcome::Failed(HostError::Unsupported)
}

#[cfg(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd"
))]
pub async fn pick_directory(request: DirectoryRequest) -> SelectionOutcome<DirectoryHandle> {
    finish_directory(directory_picker(&request)).await
}
#[cfg(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd"
))]
fn directory_picker(request: &DirectoryRequest) -> rfd::AsyncFileDialog {
    let mut picker = rfd::AsyncFileDialog::new();
    if let Some(title) = &request.title {
        picker = picker.set_title(title);
    }
    if let Some(directory) = &request.directory {
        picker = picker.set_directory(directory);
    }
    picker
}
#[cfg(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd"
))]
async fn finish_directory(picker: rfd::AsyncFileDialog) -> SelectionOutcome<DirectoryHandle> {
    match picker.pick_folder().await {
        Some(handle) => SelectionOutcome::Selected(DirectoryHandle {
            display_name: handle.file_name(),
            location: kit_core::FileLocation::Path(handle.path().to_owned()),
        }),
        None => SelectionOutcome::Cancelled,
    }
}

#[cfg(not(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd"
)))]
pub async fn pick_directory(_request: DirectoryRequest) -> SelectionOutcome<DirectoryHandle> {
    SelectionOutcome::Failed(HostError::Unsupported)
}

/// Inspect selected paths without reading their contents. Metadata work runs
/// off the UI executor; an unavailable size stays None, never a fabricated zero.
#[cfg(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd"
))]
pub async fn inspect_file_paths(
    paths: Vec<std::path::PathBuf>,
) -> SelectionOutcome<Vec<FileHandle>> {
    SelectionOutcome::Selected(
        blocking::unblock(move || {
            path_handles_with(paths, |path| {
                std::fs::metadata(path)
                    .ok()
                    .filter(std::fs::Metadata::is_file)
                    .map(|value| value.len())
            })
        })
        .await,
    )
}
#[cfg(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd"
))]
fn path_handles_with(
    paths: Vec<std::path::PathBuf>,
    byte_length: impl Fn(&std::path::Path) -> Option<u64>,
) -> Vec<FileHandle> {
    paths
        .into_iter()
        .map(|path| FileHandle {
            display_name: path
                .file_name()
                .map_or_else(String::new, |name| name.to_string_lossy().into_owned()),
            byte_length: byte_length(&path),
            location: kit_core::FileLocation::Path(path),
        })
        .collect()
}
#[cfg(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd"
))]
fn save_picker(request: &SaveFileRequest) -> rfd::AsyncFileDialog {
    file_picker(&FileRequest {
        selection: kit_core::FileSelectionMode::Single,
        filters: request.filters.clone(),
        suggested_name: request.suggested_name.clone(),
        title: request.title.clone(),
        directory: request.directory.clone(),
    })
}
/// Select a save destination. This function never writes or creates a file.
#[cfg(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd"
))]
pub async fn save_file(request: SaveFileRequest) -> SelectionOutcome<FileHandle> {
    finish_save(save_picker(&request)).await
}
#[cfg(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd"
))]
async fn finish_save(picker: rfd::AsyncFileDialog) -> SelectionOutcome<FileHandle> {
    match picker.save_file().await {
        Some(handle) => SelectionOutcome::Selected(FileHandle {
            display_name: handle.file_name(),
            location: kit_core::FileLocation::Path(handle.path().to_owned()),
            byte_length: None,
        }),
        None => SelectionOutcome::Cancelled,
    }
}

#[cfg(not(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd"
)))]
pub async fn inspect_file_paths(
    _paths: Vec<std::path::PathBuf>,
) -> SelectionOutcome<Vec<FileHandle>> {
    unsupported()
}
#[cfg(not(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd"
)))]
pub async fn save_file(_request: SaveFileRequest) -> SelectionOutcome<FileHandle> {
    unsupported()
}

/// A native parent provider independent of the UI framework.
pub trait DialogParent:
    raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle
{
}
impl<T: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + ?Sized>
    DialogParent for T
{
}
/// Host-executor future. Parent handles are copied before returning; no window
/// reference is retained across the asynchronous request.
pub type PickerFuture<T> =
    std::pin::Pin<Box<dyn std::future::Future<Output = SelectionOutcome<T>>>>;

/// Parent-aware file selection. Show the native window before requesting its
/// handle; backend support and modal behavior still need platform qualification.
pub fn pick_files_with_parent(
    request: FileRequest,
    parent: &impl DialogParent,
) -> PickerFuture<Vec<FileHandle>> {
    #[cfg(any(
        target_os = "macos",
        target_os = "windows",
        target_os = "linux",
        target_os = "freebsd"
    ))]
    {
        let picker = file_picker(&request).set_parent(parent);
        Box::pin(finish_files(picker, request.selection))
    }
    #[cfg(not(any(
        target_os = "macos",
        target_os = "windows",
        target_os = "linux",
        target_os = "freebsd"
    )))]
    {
        let _ = (request, parent);
        Box::pin(async { SelectionOutcome::Failed(HostError::Unsupported) })
    }
}
/// Parent-aware directory selection with the same weak-host lifetime policy.
pub fn pick_directory_with_parent(
    request: DirectoryRequest,
    parent: &impl DialogParent,
) -> PickerFuture<DirectoryHandle> {
    #[cfg(any(
        target_os = "macos",
        target_os = "windows",
        target_os = "linux",
        target_os = "freebsd"
    ))]
    {
        Box::pin(finish_directory(
            directory_picker(&request).set_parent(parent),
        ))
    }
    #[cfg(not(any(
        target_os = "macos",
        target_os = "windows",
        target_os = "linux",
        target_os = "freebsd"
    )))]
    {
        let _ = (request, parent);
        Box::pin(async { SelectionOutcome::Failed(HostError::Unsupported) })
    }
}

/// Parent-aware save target selection, with the same explicit outcomes as open.
pub fn save_file_with_parent(
    request: SaveFileRequest,
    parent: &impl DialogParent,
) -> PickerFuture<FileHandle> {
    #[cfg(any(
        target_os = "macos",
        target_os = "windows",
        target_os = "linux",
        target_os = "freebsd"
    ))]
    {
        Box::pin(finish_save(save_picker(&request).set_parent(parent)))
    }
    #[cfg(not(any(
        target_os = "macos",
        target_os = "windows",
        target_os = "linux",
        target_os = "freebsd"
    )))]
    {
        let _ = (request, parent);
        Box::pin(async { unsupported() })
    }
}

/// Native APIs that cannot distinguish cancellation from platform failure must
/// not fabricate a permission error. Hosts with richer APIs can implement the
/// same core contract and return its typed failure variants.
pub fn unsupported<T>() -> SelectionOutcome<T> {
    SelectionOutcome::Failed(HostError::Unsupported)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(any(
        target_os = "macos",
        target_os = "windows",
        target_os = "linux",
        target_os = "freebsd"
    ))]
    #[test]
    fn parent_handles_are_requested_before_returning_the_picker_future() {
        use raw_window_handle::{HandleError, HasDisplayHandle, HasWindowHandle};
        struct Parent {
            window: std::cell::Cell<u32>,
            display: std::cell::Cell<u32>,
        }
        impl HasWindowHandle for Parent {
            fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, HandleError> {
                self.window.set(self.window.get() + 1);
                Err(HandleError::Unavailable)
            }
        }
        impl HasDisplayHandle for Parent {
            fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>, HandleError> {
                self.display.set(self.display.get() + 1);
                Err(HandleError::Unavailable)
            }
        }
        let (file, directory, save) = {
            let parent = Parent {
                window: 0.into(),
                display: 0.into(),
            };
            let file = pick_files_with_parent(
                FileRequest {
                    selection: kit_core::FileSelectionMode::Single,
                    filters: vec![],
                    suggested_name: None,
                    ..Default::default()
                },
                &parent,
            );
            let directory = pick_directory_with_parent(DirectoryRequest::default(), &parent);
            let save = save_file_with_parent(SaveFileRequest::default(), &parent);
            assert_eq!(parent.window.get(), 3);
            assert_eq!(parent.display.get(), 3);
            (file, directory, save)
        };
        drop((file, directory, save));
    }

    #[test]
    fn picker_capabilities_do_not_claim_drag_and_drop() {
        assert_eq!(capabilities().drop, HostCapability::Unsupported);
        assert!(matches!(
            unsupported::<()>(),
            SelectionOutcome::Failed(HostError::Unsupported)
        ));
    }
}

#[cfg(all(
    test,
    any(
        target_os = "macos",
        target_os = "windows",
        target_os = "linux",
        target_os = "freebsd"
    )
))]
mod metadata_tests {
    use super::*;
    #[test]
    fn path_metadata_projection_uses_injected_sizes_without_opening_any_file() {
        let paths = vec![
            std::path::PathBuf::from("/virtual/empty.pdf"),
            "/virtual/large.pdf".into(),
            "/virtual/unknown.pdf".into(),
        ];
        let handles = path_handles_with(paths.clone(), |path| {
            if path == paths[0] {
                Some(0)
            } else if path == paths[1] {
                Some(9_007_199_254_740_993)
            } else {
                None
            }
        });
        assert_eq!(handles[0].byte_length, Some(0));
        assert_eq!(handles[1].byte_length, Some(9_007_199_254_740_993));
        assert_eq!(handles[2].byte_length, None);
        assert_eq!(
            handles[1].location,
            kit_core::FileLocation::Path(paths[1].clone())
        );
    }
    #[test]
    fn native_builders_retain_typed_options_without_opening_dialogs() {
        let request = SaveFileRequest {
            title: Some("Export report".into()),
            directory: Some("/virtual/output".into()),
            suggested_name: Some("report.pdf".into()),
            filters: vec![kit_core::FileFilter::new("PDF".into(), vec!["pdf".into()]).unwrap()],
        };
        let configured = format!("{:?}", save_picker(&request));
        for expected in [
            "Export report",
            "/virtual/output",
            "report.pdf",
            "PDF",
            "pdf",
        ] {
            assert!(configured.contains(expected), "{configured}");
        }
        let directory = format!(
            "{:?}",
            directory_picker(&DirectoryRequest {
                title: Some("Workspace".into()),
                directory: Some("/virtual/workspace".into())
            })
        );
        assert!(directory.contains("Workspace") && directory.contains("/virtual/workspace"));
        assert_eq!(capabilities().save_files, capabilities().files);
    }
}
