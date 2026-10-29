use slint::ComponentHandle;
use slint_kit::*;
slint::include_modules!();
type Value = YearMonth;
fn parse(text: &str) -> DraftValue<Value> {
    if text.is_empty() {
        return DraftValue::Empty;
    }
    if text.len() != 7
        || !(text.bytes().enumerate().all(|(i, b)| {
            if i == 4 {
                b == b'-'
            } else {
                b.is_ascii_digit()
            }
        }))
    {
        return DraftValue::Invalid(ParseIssue::InvalidDate);
    }
    let Some((year, month)) = text.split_once('-') else {
        return DraftValue::Incomplete;
    };
    match year
        .parse()
        .ok()
        .zip(month.parse().ok())
        .and_then(|(y, m)| YearMonth::new(y, m).ok())
    {
        Some(value) => DraftValue::Parsed(value),
        None => DraftValue::Invalid(ParseIssue::InvalidDate),
    }
}
fn format(value: &Value) -> String {
    format!("{:04}-{:02}", value.year(), value.month())
}
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::init_form_validation!(self, app);
    let initial = YearMonth::new(2026, 10).unwrap();
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
        kind: slint_kit::HtmlInputKind::Month, required: true,
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
