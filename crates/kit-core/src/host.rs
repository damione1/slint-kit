//! Host I/O contracts: platform adapters perform asynchronous work outside the UI thread.
use crate::FileSelectionMode;
use crate::field::next_owner;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceUri(String);
impl ResourceUri {
    pub fn new(value: String) -> Result<Self, HostError> {
        let Some((scheme, rest)) = value.split_once(':') else {
            return Err(HostError::InvalidResource);
        };
        if scheme.is_empty()
            || rest.is_empty()
            || !scheme.chars().enumerate().all(|(i, c)| {
                c.is_ascii_alphabetic() || (i > 0 && (c.is_ascii_digit() || "+-.".contains(c)))
            })
            || value.chars().any(char::is_control)
        {
            return Err(HostError::InvalidResource);
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FileLocation {
    Path(PathBuf),
    Uri(ResourceUri),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileHandle {
    pub location: FileLocation,
    pub display_name: String,
    pub byte_length: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectoryHandle {
    pub location: FileLocation,
    pub display_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileFilter {
    pub label: String,
    extensions: Vec<String>,
}
impl FileFilter {
    pub fn new(label: String, extensions: Vec<String>) -> Result<Self, HostError> {
        if extensions.is_empty()
            || extensions.iter().any(|value| {
                value.is_empty()
                    || !value
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            })
        {
            return Err(HostError::InvalidFilter);
        }
        Ok(Self { label, extensions })
    }
    pub fn extensions(&self) -> &[String] {
        &self.extensions
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileRequest {
    pub selection: FileSelectionMode,
    pub filters: Vec<FileFilter>,
    pub suggested_name: Option<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DirectoryRequest {
    pub title: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectionOutcome<T> {
    Selected(T),
    Cancelled,
    Failed(HostError),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostError {
    Unsupported,
    PermissionDenied,
    Unavailable,
    InvalidResource,
    InvalidFilter,
    Platform(String),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostCapability {
    Available,
    Unsupported,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostCapabilities {
    pub files: HostCapability,
    pub directories: HostCapability,
    pub drop: HostCapability,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostRequestTicket {
    owner: u64,
    generation: u64,
}
#[derive(Debug)]
pub struct HostRequestController<T> {
    owner: u64,
    generation: u64,
    pending: Option<HostRequestTicket>,
    result: Option<SelectionOutcome<T>>,
}
impl<T> Default for HostRequestController<T> {
    fn default() -> Self {
        Self {
            owner: next_owner(),
            generation: 0,
            pending: None,
            result: None,
        }
    }
}
impl<T> HostRequestController<T> {
    pub fn begin(&mut self) -> HostRequestTicket {
        self.generation = self
            .generation
            .checked_add(1)
            .expect("host request generation exhausted");
        let ticket = HostRequestTicket {
            owner: self.owner,
            generation: self.generation,
        };
        self.pending = Some(ticket);
        self.result = None;
        ticket
    }
    pub fn finish(&mut self, ticket: HostRequestTicket, result: SelectionOutcome<T>) -> bool {
        if self.pending != Some(ticket) {
            return false;
        }
        self.pending = None;
        self.result = Some(result);
        true
    }
    pub fn cancel(&mut self) {
        self.pending = None;
        self.result = Some(SelectionOutcome::Cancelled);
    }
    pub fn pending(&self) -> bool {
        self.pending.is_some()
    }
    pub fn take_result(&mut self) -> Option<SelectionOutcome<T>> {
        self.result.take()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mobile_uri_and_stale_request() {
        let uri = ResourceUri::new("content://provider/document/42".into()).unwrap();
        assert_eq!(uri.as_str(), "content://provider/document/42");
        let mut requests = HostRequestController::<Vec<FileHandle>>::default();
        let old = requests.begin();
        let current = requests.begin();
        assert!(!requests.finish(old, SelectionOutcome::Cancelled));
        assert!(requests.finish(current, SelectionOutcome::Cancelled));
        assert_eq!(requests.take_result(), Some(SelectionOutcome::Cancelled));
    }
    #[test]
    fn invalid_filter_is_rejected() {
        assert!(FileFilter::new("Bad".into(), vec!["*.pdf".into()]).is_err());
    }
}
