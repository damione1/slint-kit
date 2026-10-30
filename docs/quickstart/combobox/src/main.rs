use slint::ComponentHandle;
use slint_kit::{Ownership, SelectOption, SelectionIntent};

mod ui {
    slint::include_modules!();
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Workspace {
    Personal,
    Shared,
    Archive,
    Design,
    Engineering,
    Marketing,
    Research,
    Community,
}

impl Workspace {
    fn label(self) -> &'static str {
        match self {
            Self::Personal => "Personal",
            Self::Shared => "Shared",
            Self::Archive => "Archive",
            Self::Design => "Design studio",
            Self::Engineering => "Engineering",
            Self::Marketing => "Marketing",
            Self::Research => "Research lab",
            Self::Community => "Community",
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    slint_kit::init_form_validation!(ui, app);
    let selection = slint_kit::bind_select!(
        ui,
        app,
        vec![
            SelectOption::new(Workspace::Personal, "Personal"),
            SelectOption::new(Workspace::Shared, "Shared"),
            SelectOption::new(Workspace::Archive, "Archive"),
            SelectOption::new(Workspace::Design, "Design studio"),
            SelectOption::new(Workspace::Engineering, "Engineering"),
            SelectOption::new(Workspace::Marketing, "Marketing"),
            SelectOption::new(Workspace::Research, "Research lab"),
            SelectOption::new(Workspace::Community, "Community"),
        ],
        Some(Workspace::Personal),
        Ownership::Local
    );
    let weak = app.as_weak();
    selection.on_intent(move |intent| {
        if let Some(app) = weak.upgrade() {
            let label = match intent {
                SelectionIntent::Commit(workspace) => workspace.label(),
                SelectionIntent::Clear => "Choose a workspace",
            };
            app.set_selected_label(label.into());
        }
    });
    app.run()
}
