// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
slint::include_modules!();

struct ProjectName(String);
impl ProjectName {
    fn parse(draft: &str) -> Option<Self> {
        let name = draft.trim();
        (!name.is_empty()).then(|| Self(name.into()))
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    let weak = app.as_weak();
    app.on_form_action(move |action, draft| {
        if let Some(app) = weak.upgrade() {
            match action {
                FormAction::Reset => {
                    app.set_draft("Field notes".into());
                    app.set_result("Draft reset to Field notes".into());
                }
                FormAction::Submit => {
                    let result = match ProjectName::parse(&draft) {
                        Some(name) => format!("Accepted in Rust: {}", name.0),
                        None => "Enter a project name before submitting".into(),
                    };
                    app.set_result(result.into());
                }
            }
        }
    });
    app.run()
}
