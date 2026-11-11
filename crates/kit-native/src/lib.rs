// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
//! Optional asynchronous host services, independent of any UI framework.
//! Construct/poll the picker on the host's UI executor (for Slint, spawn_local).
//! The returned handles do not read files or assume every resource is a path.
use kit_core::{
    DirectoryHandle, DirectoryRequest, FileHandle, FileRequest, HostCapabilities, HostCapability,
    HostError, SelectionOutcome,
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
    use kit_core::{FileLocation, FileSelectionMode};
    let result = match selection {
        FileSelectionMode::Single => picker.pick_file().await.map(|handle| vec![handle]),
        FileSelectionMode::Multiple => picker.pick_files().await,
    };
    match result {
        Some(files) => SelectionOutcome::Selected(
            files
                .into_iter()
                .map(|handle| FileHandle {
                    display_name: handle.file_name(),
                    location: FileLocation::Path(handle.path().to_owned()),
                    byte_length: None,
                })
                .collect(),
        ),
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
    use kit_core::FileLocation;
    let mut picker = rfd::AsyncFileDialog::new();
    if let Some(title) = request.title {
        picker = picker.set_title(title);
    }
    if let Some(directory) = request.directory {
        picker = picker.set_directory(directory);
    }
    match picker.pick_folder().await {
        Some(handle) => SelectionOutcome::Selected(DirectoryHandle {
            display_name: handle.file_name(),
            location: FileLocation::Path(handle.path().to_owned()),
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
        use kit_core::FileLocation;
        let mut picker = rfd::AsyncFileDialog::new().set_parent(parent);
        if let Some(title) = request.title {
            picker = picker.set_title(title);
        }
        if let Some(directory) = request.directory {
            picker = picker.set_directory(directory);
        }
        Box::pin(async move {
            match picker.pick_folder().await {
                Some(handle) => SelectionOutcome::Selected(DirectoryHandle {
                    display_name: handle.file_name(),
                    location: FileLocation::Path(handle.path().to_owned()),
                }),
                None => SelectionOutcome::Cancelled,
            }
        })
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
        let (file, directory) = {
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
            assert_eq!(parent.window.get(), 2);
            assert_eq!(parent.display.get(), 2);
            (file, directory)
        };
        drop((file, directory));
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
