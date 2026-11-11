//! Typed file selection rules shared by Rust acceptance and HTML form controls.
use crate::{
    FileFilter, FileHandle, HostError, HtmlConstraints, HtmlInputKind, HtmlInputState, HtmlValidity,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct FileExtension(String);
#[derive(Clone, Debug, PartialEq, Eq)]
enum AcceptSpecifier {
    Extension(FileExtension),
    MediaType(mime_guess::mime::Mime),
}
/// Parsed HTML accept specifiers. MIME matching is an extension hint, never a
/// claim that file contents were read or verified to have that media type.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileAccept(Vec<AcceptSpecifier>);
impl FileAccept {
    pub fn parse(source: &str) -> Result<Self, HostError> {
        if source.trim().is_empty() {
            return Ok(Self::default());
        }
        let mut result = Vec::new();
        for token in source.split(',').map(str::trim) {
            let token = token.to_ascii_lowercase();
            let specifier = if let Some(extension) = token.strip_prefix('.') {
                FileFilter::new(String::new(), vec![extension.into()])?;
                AcceptSpecifier::Extension(FileExtension(extension.into()))
            } else {
                let media: mime_guess::mime::Mime =
                    token.parse().map_err(|_| HostError::InvalidFilter)?;
                if media.params().next().is_some()
                    || media.type_().as_str() == "*"
                    || (media.subtype().as_str() == "*"
                        && !matches!(media.type_().as_str(), "audio" | "image" | "video"))
                {
                    return Err(HostError::InvalidFilter);
                }
                AcceptSpecifier::MediaType(media)
            };
            if !result.contains(&specifier) {
                result.push(specifier);
            }
        }
        Ok(Self(result))
    }
    pub fn description(&self) -> String {
        self.0
            .iter()
            .map(|specifier| match specifier {
                AcceptSpecifier::Extension(extension) => format!(".{}", extension.0),
                AcceptSpecifier::MediaType(media) => media.to_string(),
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
    pub fn matches_name(&self, name: &str) -> bool {
        let lower = name.to_ascii_lowercase();
        self.0.is_empty()
            || self.0.iter().any(|specifier| match specifier {
                AcceptSpecifier::Extension(extension) => {
                    lower.ends_with(&format!(".{}", extension.0))
                }
                AcceptSpecifier::MediaType(expected) => {
                    mime_guess::from_path(name).iter().any(|actual| {
                        (expected.subtype().as_str() == "*" && actual.type_() == expected.type_())
                            || &actual == expected
                    })
                }
            })
    }
    /// Convert supported MIME hints to native extension filters. Unknown MIME
    /// hints return Unsupported rather than silently showing an unfiltered dialog.
    pub fn native_filter(&self, label: String) -> Result<Option<FileFilter>, HostError> {
        if self.0.is_empty() {
            return Ok(None);
        }
        let mut extensions = Vec::new();
        for specifier in &self.0 {
            match specifier {
                AcceptSpecifier::Extension(extension) => extensions.push(extension.0.clone()),
                AcceptSpecifier::MediaType(media) => {
                    let known =
                        mime_guess::get_mime_extensions(media).ok_or(HostError::Unsupported)?;
                    extensions.extend(known.iter().map(|extension| (*extension).into()));
                }
            }
        }
        extensions.sort();
        extensions.dedup();
        FileFilter::new(label, extensions).map(Some)
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileConstraints {
    pub accept: FileAccept,
    pub max_files: Option<usize>,
    /// Per-resource bytes, exact even beyond a renderer float's precision.
    pub max_size: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileViolation {
    TypeMismatch { name: String },
    TooMany { maximum: usize },
    TooLarge { name: String, maximum: u64 },
    UnknownSize { name: String },
}
#[derive(Clone, Copy, Debug)]
pub struct FileMetadata<'a> {
    pub name: &'a str,
    pub byte_length: Option<u64>,
}
impl<'a> From<&'a FileHandle> for FileMetadata<'a> {
    fn from(file: &'a FileHandle) -> Self {
        Self {
            name: &file.display_name,
            byte_length: file.byte_length,
        }
    }
}
impl FileConstraints {
    pub fn validate(&self, files: &[FileMetadata<'_>]) -> Result<(), FileViolation> {
        if let Some(maximum) = self.max_files
            && files.len() > maximum
        {
            return Err(FileViolation::TooMany { maximum });
        }
        for file in files {
            if !self.accept.matches_name(file.name) {
                return Err(FileViolation::TypeMismatch {
                    name: file.name.into(),
                });
            }
            if let Some(maximum) = self.max_size {
                let Some(size) = file.byte_length else {
                    return Err(FileViolation::UnknownSize {
                        name: file.name.into(),
                    });
                };
                if size > maximum {
                    return Err(FileViolation::TooLarge {
                        name: file.name.into(),
                        maximum,
                    });
                }
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct HtmlFileValidation {
    pub validity: HtmlValidity,
    pub violation: Option<FileViolation>,
}
impl HtmlConstraints {
    /// File-specific validation through the same HTML constraint boundary used
    /// for required, disabled/read-only and custom validity, also callable in Rust.
    pub fn validate_file_selection(
        &self,
        rules: &FileConstraints,
        files: &[FileMetadata<'_>],
    ) -> HtmlFileValidation {
        let mut validity = self.validate(
            "",
            HtmlInputState {
                selected_count: files.len(),
                ..Default::default()
            },
        );
        if self.kind != HtmlInputKind::File || !self.will_validate() {
            return HtmlFileValidation {
                validity,
                violation: None,
            };
        }
        let violation = rules.validate(files).err();
        if let Some(issue) = &violation {
            match issue {
                FileViolation::TypeMismatch { .. } => validity.type_mismatch = true,
                FileViolation::TooMany { .. } | FileViolation::TooLarge { .. } => {
                    validity.range_overflow = true
                }
                FileViolation::UnknownSize { .. } => validity.bad_input = true,
            }
        }
        HtmlFileValidation {
            validity,
            violation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accept_parses_extensions_mime_and_compound_names_once() {
        let accept = FileAccept::parse(".PDF, image/*, application/json, .tar.gz, .pdf").unwrap();
        for name in [
            "report.PDF",
            "photo.png",
            "icon.svg",
            "data.json",
            "archive.TAR.GZ",
        ] {
            assert!(accept.matches_name(name), "{name}");
        }
        for name in ["report.pdf.exe", "notes.txt", "picture.unknown"] {
            assert!(!accept.matches_name(name), "{name}");
        }
        assert_eq!(FileAccept::parse("").unwrap(), FileAccept::default());
        assert!(FileAccept::default().matches_name("untyped-resource"));
    }
    #[test]
    fn malformed_accept_is_not_silently_treated_as_unrestricted() {
        for source in [
            "*.pdf",
            ".",
            ".pdf,",
            "text/*",
            "*/*",
            "image/png; charset=utf-8",
            "image",
            ".bad/extension",
            ".tar..gz",
        ] {
            assert_eq!(
                FileAccept::parse(source),
                Err(HostError::InvalidFilter),
                "{source}"
            );
        }
    }
    #[test]
    fn native_hints_expand_known_mime_and_report_unknown_mapping() {
        let filter = FileAccept::parse("image/*,.pdf")
            .unwrap()
            .native_filter("Images and PDF".into())
            .unwrap()
            .unwrap();
        assert!(filter.extensions().contains(&"png".into()));
        assert!(filter.extensions().contains(&"pdf".into()));
        assert_eq!(
            FileAccept::parse("application/x-kit-unknown-test")
                .unwrap()
                .native_filter("Custom".into()),
            Err(HostError::Unsupported)
        );
    }
    #[test]
    fn count_and_exact_byte_bounds_reject_unknown_sizes_without_guessing() {
        let rules = FileConstraints {
            accept: FileAccept::parse(".pdf").unwrap(),
            max_files: Some(1),
            max_size: Some(9_007_199_254_740_993),
        };
        let item = |bytes| FileMetadata {
            name: "report.pdf",
            byte_length: bytes,
        };
        assert!(rules.validate(&[item(Some(9_007_199_254_740_993))]).is_ok());
        assert!(matches!(
            rules.validate(&[item(Some(9_007_199_254_740_994))]),
            Err(FileViolation::TooLarge { .. })
        ));
        assert!(matches!(
            rules.validate(&[item(None)]),
            Err(FileViolation::UnknownSize { .. })
        ));
        assert_eq!(
            rules.validate(&[item(Some(0)), item(Some(0))]),
            Err(FileViolation::TooMany { maximum: 1 })
        );
        assert!(
            FileConstraints {
                max_size: Some(0),
                ..Default::default()
            }
            .validate(&[item(Some(0))])
            .is_ok()
        );
    }
    #[test]
    fn html_file_boundary_combines_required_metadata_and_access() {
        let mut html = HtmlConstraints {
            kind: HtmlInputKind::File,
            required: true,
            ..Default::default()
        };
        let rules = FileConstraints {
            accept: FileAccept::parse(".pdf").unwrap(),
            ..Default::default()
        };
        assert!(
            html.validate_file_selection(&rules, &[])
                .validity
                .value_missing
        );
        let files = [FileMetadata {
            name: "script.exe",
            byte_length: Some(1),
        }];
        let result = html.validate_file_selection(&rules, &files);
        assert!(result.validity.type_mismatch);
        assert!(matches!(
            result.violation,
            Some(FileViolation::TypeMismatch { .. })
        ));
        html.custom_error = "Business constraint".into();
        assert!(
            html.validate_file_selection(&rules, &files)
                .validity
                .custom_error
        );
        html.disabled = true;
        assert!(!html.will_validate());
        assert_eq!(html.validate_file_selection(&rules, &files).violation, None);
    }
}
