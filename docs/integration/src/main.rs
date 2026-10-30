// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::{
    AbsencePolicy, CommitPolicy, CommitTrigger, DraftValue, FieldAccess, FieldBinding, FieldIntent,
    Ownership, ParseIssue, SelectOption, SelectionIntent, ValidationPresentation,
};

mod verify;

mod ui {
    slint::include_modules!();
}

#[derive(Clone, PartialEq, Eq)]
struct ProjectName(String);

#[derive(Clone, Copy)]
enum NameIssue {
    TooLong,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum WorkspaceKey {
    Personal,
    Shared,
    Archive,
}

fn main() -> Result<(), slint::PlatformError> {
    let french = std::env::args().any(|arg| arg == "--fr");
    let app = ui::App::new()?;
    slint_kit::init_form_validation!(ui, app);
    app.set_language(if french {
        ui::ExampleLanguage::French
    } else {
        ui::ExampleLanguage::English
    });
    app.set_accepted_label("Atlas".into());
    let weak = app.as_weak();
    let field = FieldBinding::<ProjectName, NameIssue>::new(
        Some(ProjectName("Atlas".into())),
        Ownership::Local,
        CommitPolicy::OnBlur,
        AbsencePolicy::Required,
        |draft| {
            if draft.is_empty() {
                DraftValue::Empty
            } else if draft.chars().any(char::is_control) {
                DraftValue::Invalid(ParseIssue::InvalidSyntax)
            } else {
                DraftValue::Parsed(ProjectName(draft.into()))
            }
        },
        |name| name.0.clone(),
        move |issue| match issue {
            NameIssue::TooLong => if french {
                "Utilisez au plus 40 graphèmes"
            } else {
                "Use at most 40 graphemes"
            }
            .into(),
        },
        move |projection| {
            if let Some(app) = weak.upgrade() {
                app.set_draft(projection.draft.into());
                app.set_error(projection.error.into());
                app.set_field_access(match projection.access {
                    FieldAccess::Editable => ui::FieldAccess::Editable,
                    FieldAccess::ReadOnly => ui::FieldAccess::ReadOnly,
                    FieldAccess::Disabled => ui::FieldAccess::Disabled,
                });
                app.set_validation(match projection.validation {
                    ValidationPresentation::Pristine => ui::ValidationPresentation::Pristine,
                    ValidationPresentation::Checking => ui::ValidationPresentation::Checking,
                    ValidationPresentation::Valid => ui::ValidationPresentation::Valid,
                    ValidationPresentation::Invalid => ui::ValidationPresentation::Invalid,
                });
            }
        },
    );
    field.set_validator(|name| {
        if slint_kit::grapheme_count(&name.0) > 40 {
            vec![NameIssue::TooLong]
        } else {
            Vec::new()
        }
    });
    field.set_html_constraints(slint_kit::HtmlConstraints {
        kind: slint_kit::HtmlInputKind::Text,
        required: true,
        ..Default::default()
    });
    let weak = app.as_weak();
    field.on_intent(move |intent| {
        if let Some(app) = weak.upgrade() {
            match intent {
                FieldIntent::Commit(name) => app.set_accepted_label(name.0.into()),
                FieldIntent::Clear => app.set_accepted_label("".into()),
                FieldIntent::Cancel => {}
            }
        }
    });
    let target = field.clone();
    app.on_name_edited(move |draft| {
        let _ = target.edited(draft.as_str());
    });
    let target = field.clone();
    app.on_name_commit(move || {
        let _ = target.commit(CommitTrigger::Explicit);
    });
    let target = field.clone();
    app.on_name_blur(move || {
        let _ = target.commit(CommitTrigger::Blur);
    });
    let target = field.clone();
    app.on_name_cancel(move || target.cancel());

    let mut archived = SelectOption::new(
        WorkspaceKey::Archive,
        if french {
            "Archives (lecture seule)"
        } else {
            "Archive (read only)"
        },
    );
    archived.disabled = true;
    let selection = slint_kit::bind_select!(
        ui,
        app,
        vec![
            SelectOption::new(
                WorkspaceKey::Personal,
                if french { "Personnel" } else { "Personal" }
            ),
            SelectOption::new(
                WorkspaceKey::Shared,
                if french { "Partagé" } else { "Shared" }
            ),
            archived,
        ],
        Some(WorkspaceKey::Personal),
        Ownership::Local
    );
    app.set_selected_label(if french { "Personnel" } else { "Personal" }.into());
    let weak = app.as_weak();
    selection.on_intent(move |intent| {
        if let Some(app) = weak.upgrade() {
            let label = match intent {
                SelectionIntent::Commit(WorkspaceKey::Personal) => {
                    if french {
                        "Personnel"
                    } else {
                        "Personal"
                    }
                }
                SelectionIntent::Commit(WorkspaceKey::Shared) => {
                    if french {
                        "Partagé"
                    } else {
                        "Shared"
                    }
                }
                SelectionIntent::Commit(WorkspaceKey::Archive) => {
                    if french {
                        "Archives (lecture seule)"
                    } else {
                        "Archive (read only)"
                    }
                }
                SelectionIntent::Clear => {
                    if french {
                        "Choisir un espace"
                    } else {
                        "Choose a workspace"
                    }
                }
            };
            app.set_selected_label(label.into());
        }
    });
    if std::env::args().any(|arg| arg == "--verify") {
        verify::schedule(&app);
    }
    // Keep both window-local bindings alive for the complete event loop.
    app.run()
}
