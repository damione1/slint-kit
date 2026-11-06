// SPDX-License-Identifier: GPL-3.0-or-later
use std::path::{Path, PathBuf};
use syn::visit::{self, Visit};

// These collections intentionally receive host-owned models. Other families
// project their domain values inside the kit, rather than in the quickstart.
const HOST_COLLECTIONS: &[&str] = &[
    "content-list",
    "data-table",
    "directory-picker",
    "file-drop-zone",
    "file-list",
    "file-picker",
    "notification-center",
    "tag-input",
    "toast",
    "toast-host",
    "virtual-text-view",
];

#[derive(Default)]
struct Wiring {
    weak_window: bool,
    model: bool,
    ui_match: bool,
    legacy_binding: bool,
    ui_aliases: Vec<String>,
}

struct UiPath<'a> {
    found: bool,
    aliases: &'a [String],
}
impl<'ast> Visit<'ast> for UiPath<'_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.found |= path.segments.iter().any(|segment| {
            segment.ident == "ui" || self.aliases.iter().any(|alias| segment.ident == alias)
        });
        visit::visit_path(self, path);
    }
}

fn ui_imports(tree: &syn::UseTree, in_ui: bool, aliases: &mut Vec<String>) {
    match tree {
        syn::UseTree::Path(path) => {
            ui_imports(&path.tree, in_ui || path.ident == "ui", aliases);
        }
        syn::UseTree::Name(name) => {
            if in_ui {
                aliases.push(name.ident.to_string());
            }
        }
        syn::UseTree::Rename(rename) => {
            if in_ui {
                aliases.push(rename.rename.to_string());
            }
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                ui_imports(item, in_ui, aliases);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}

#[derive(Default)]
struct UiImports(Vec<String>);
impl<'ast> Visit<'ast> for UiImports {
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        ui_imports(&item.tree, false, &mut self.0);
        visit::visit_item_use(self, item);
    }
}

impl<'ast> Visit<'ast> for Wiring {
    fn visit_ident(&mut self, identifier: &'ast syn::Ident) {
        self.model |= identifier == "ModelRc";
        visit::visit_ident(self, identifier);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        self.weak_window |= call.method == "as_weak";
        visit::visit_expr_method_call(self, call);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.model |= path
            .segments
            .iter()
            .any(|segment| segment.ident == "ModelRc");
        self.weak_window |= path
            .segments
            .iter()
            .any(|segment| segment.ident == "as_weak");
        self.legacy_binding |= path.segments.iter().any(|segment| {
            segment.ident == "bind_select" || segment.ident == "init_form_validation"
        });
        visit::visit_path(self, path);
    }

    fn visit_expr_match(&mut self, expression: &'ast syn::ExprMatch) {
        let mut paths = UiPath {
            found: false,
            aliases: &self.ui_aliases,
        };
        paths.visit_expr_match(expression);
        self.ui_match |= paths.found;
        visit::visit_expr_match(self, expression);
    }
}

fn inspect(slug: &str, source: &str) -> Result<Vec<String>, String> {
    let syntax = syn::parse_file(source).map_err(|error| error.to_string())?;
    let mut imports = UiImports::default();
    imports.visit_file(&syntax);
    let mut wiring = Wiring {
        ui_aliases: imports.0,
        ..Default::default()
    };
    wiring.visit_file(&syntax);
    let mut errors = Vec::new();
    if wiring.weak_window {
        errors.push("window wiring (as_weak) belongs in the kit".into());
    }
    if wiring.model && !HOST_COLLECTIONS.contains(&slug) {
        errors.push("ModelRc is reserved for host-owned collections (F4)".into());
    }
    if wiring.ui_match {
        errors.push("UI enum matching belongs in the kit's typed conversions".into());
    }
    if wiring.legacy_binding {
        errors.push("use install and component props instead of legacy binding macros".into());
    }
    Ok(errors)
}

fn rust_sources(directory: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(directory).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        if entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            paths.extend(rust_sources(&entry.path())?);
        } else if entry
            .path()
            .extension()
            .is_some_and(|extension| extension == "rs")
        {
            paths.push(entry.path());
        }
    }
    paths.sort();
    Ok(paths)
}

pub fn run() -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask is in the workspace")
        .join("docs");
    let inventory: serde_json::Map<String, serde_json::Value> = serde_json::from_slice(
        &std::fs::read(root.join("content/starters.json")).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let mut errors = Vec::new();
    for slug in inventory.keys() {
        let source_directory = root.join("quickstart").join(slug).join("src");
        if !source_directory.join("main.rs").is_file() {
            errors.push(format!("{slug}: missing src/main.rs"));
            continue;
        }
        let mut lines = 0;
        for path in rust_sources(&source_directory)? {
            let source = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
            lines += source.lines().count();
            match inspect(slug, &source) {
                Ok(diagnostics) => errors.extend(diagnostics.into_iter().map(|diagnostic| {
                    format!(
                        "{}: {diagnostic}",
                        path.strip_prefix(&root).unwrap_or(&path).display()
                    )
                })),
                Err(error) => errors.push(format!("{}: {error}", path.display())),
            }
        }
        if lines > 15 {
            errors.push(format!(
                "{slug}: {lines} Rust source lines; quickstarts must fit in 15 lines including any helper modules"
            ));
        }
    }
    if errors.is_empty() {
        println!(
            "Checked {} import + props + callback quickstarts",
            inventory.len()
        );
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn business_callbacks_and_comments_do_not_count_as_wiring() {
        let source = r#"
            // The kit owns as_weak, ModelRc and match ui::CalendarNavigation.
            fn main() {
                slint_kit::install(&app);
                app.on_chosen(|day| println!("as_weak ModelRc {day:?}"));
                app.on_save(|value| match value { Some(value) => save(value), None => {} });
            }
        "#;
        assert!(inspect("calendar", source).unwrap().is_empty());
    }

    #[test]
    fn rejects_wiring_and_ui_matches_without_flagging_domain_matches() {
        let source = r#"
            fn main() {
                let weak = app.as_weak();
                let rows = slint::ModelRc::new(model);
                match step { slint_kit::ui::CalendarNavigation::NextDay => next() }
                slint_kit::bind_select!(self, app);
            }
        "#;
        assert_eq!(inspect("calendar", source).unwrap().len(), 4);
    }

    #[test]
    fn only_host_collections_may_construct_models() {
        let source = "fn main() { let rows = slint::ModelRc::new(model); }";
        for slug in HOST_COLLECTIONS {
            assert!(inspect(slug, source).unwrap().is_empty(), "{slug}");
        }
        for slug in ["calendar", "select", "text-field", "color-picker"] {
            assert_eq!(inspect(slug, source).unwrap().len(), 1, "{slug}");
        }
    }

    #[test]
    fn imported_aliases_do_not_hide_model_or_ui_enum_wiring() {
        let source = r#"
            use slint::ModelRc as Rows;
            use slint_kit::ui::{CalendarNavigation as Move};
            fn main() {
                let rows = Rows::new(model);
                match step { Move::NextDay => next() }
            }
        "#;
        assert_eq!(inspect("calendar", source).unwrap().len(), 2);
    }
}
