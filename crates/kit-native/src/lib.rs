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
    use kit_core::{FileLocation, FileSelectionMode};
    let mut picker = rfd::AsyncFileDialog::new();
    for filter in &request.filters {
        picker = picker.add_filter(&filter.label, filter.extensions());
    }
    if let Some(name) = request.suggested_name {
        picker = picker.set_file_name(name);
    }
    let result = match request.selection {
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

/// Native APIs that cannot distinguish cancellation from platform failure must
/// not fabricate a permission error. Hosts with richer APIs can implement the
/// same core contract and return its typed failure variants.
pub fn unsupported<T>() -> SelectionOutcome<T> {
    SelectionOutcome::Failed(HostError::Unsupported)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn picker_capabilities_do_not_claim_drag_and_drop() {
        assert_eq!(capabilities().drop, HostCapability::Unsupported);
        assert!(matches!(
            unsupported::<()>(),
            SelectionOutcome::Failed(HostError::Unsupported)
        ));
    }
}
