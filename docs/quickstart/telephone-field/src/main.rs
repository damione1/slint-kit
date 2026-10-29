use slint::ComponentHandle;
use slint_kit::*;
slint::include_modules!();
fn rules() -> PhoneRules {
    PhoneRules::exact(slint_kit::PhoneNumberMode::International, 11).expect("11 digit phone rule")
}
fn parse(text: &str) -> DraftValue<PhoneNumber> { rules().parse(text) }
fn format(value: &PhoneNumber) -> String { value.as_str().into() }
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::init_form_validation!(self, app);
    let initial = rules().validate("+15145550123").unwrap().unwrap();
    app.set_accepted(format(&initial).into());
    let weak = app.as_weak();
    let field = FieldBinding::<PhoneNumber, ()>::new(
        Some(initial),
        Ownership::Local,
        CommitPolicy::OnBlur,
        AbsencePolicy::Required,
        parse,
        format,
        |_| String::new(),
        move |view| {
            if let Some(app) = weak.upgrade() {
                let intrinsic_invalid = view.html_validity.is_some_and(|flags| !flags.valid());
                app.set_draft(view.draft.into());
                // The control presents matching intrinsic rules; Rust keeps business errors.
                app.set_error(if intrinsic_invalid { String::new() } else { view.error }.into());
            }
        },
    );
    field.set_draft_normalizer(|text| normalize_phone(text).map_err(|_| ParseIssue::InvalidSyntax));
    field.set_html_constraints(slint_kit::HtmlConstraints {
        kind: slint_kit::HtmlInputKind::Telephone, required: true,
        pattern: Some(HtmlPattern::compile(r"\+1[0-9]{10}").expect("valid +1 pattern")),
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
