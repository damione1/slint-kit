use slint::ComponentHandle;
use slint_kit::*;
slint::include_modules!();
type Value = DecimalValue;
fn parse(text: &str) -> DraftValue<Value> {
    DecimalRules {
        locale: NumberLocale::EnCa,
        scale: DecimalScale::new(2).unwrap(),
        sign: SignPolicy::NonNegative,
        bounds: None,
    }
    .parse(text)
}
fn format(value: &Value) -> String {
    value.to_string()
}
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::init_form_validation!(self, app);
    let initial = DecimalValue::from_integer(12);
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
                app.set_draft(view.draft.into());
                app.set_error(view.error.into());
            }
        },
    );
    field.set_html_constraints(slint_kit::HtmlConstraints {
        kind: slint_kit::HtmlInputKind::Number, required: true,
        step: Some("any".into()),
        ..Default::default()
    });
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
