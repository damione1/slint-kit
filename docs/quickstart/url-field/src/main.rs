use slint::ComponentHandle;
use slint_kit::*;
slint::include_modules!();
#[derive(Clone, PartialEq)]
struct Value(String);
fn parse(text: &str) -> DraftValue<Value> {
    let value = text.replace(['\r', '\n'], "");
    let value = value.trim_matches(|c| matches!(c, ' ' | '\t' | '\u{c}'));
    if value.is_empty() { DraftValue::Empty }
    else if url::Url::parse(value).is_ok() { DraftValue::Parsed(Value(value.into())) }
    else { DraftValue::Invalid(ParseIssue::InvalidSyntax) }
}
fn format(value: &Value) -> String {
    value.0.clone()
}
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    let initial = Value("https://example.com".into());
    app.set_accepted(format(&initial).into());
    let weak = app.as_weak();
    let field = FieldBinding::<Value, ()>::new(
        Some(initial),
        Ownership::Local,
        CommitPolicy::OnBlur,
        AbsencePolicy::Required,
        parse,
        format,
        |_| String::new(),
        move |view| {
            if let Some(app) = weak.upgrade() {
                let draft_empty = view.draft.is_empty();
                app.set_draft(view.draft.into());
                app.set_error(if view.error.is_empty() { String::new() } else if draft_empty { "Please fill out this field.".into() } else { "Please enter a valid absolute URL.".into() }.into());
            }
        },
    );
    let weak = app.as_weak();
    field.on_intent(move |intent| {
        if let Some(app) = weak.upgrade() {
            if let FieldIntent::Commit(value) = intent {
                app.set_accepted(format(&value).into());
            }
        }
    });
    let binding = field.clone();
    app.on_edited(move |text| {
        let _ = binding.edited(&text);
    });
    let binding = field.clone();
    app.on_commit(move || {
        let _ = binding.commit(CommitTrigger::Submit);
    });
    let binding = field.clone();
    app.on_blurred(move || {
        let _ = binding.commit(CommitTrigger::Blur);
    });
    let binding = field.clone();
    app.on_cancel(move || binding.cancel());

    app.run()
}
