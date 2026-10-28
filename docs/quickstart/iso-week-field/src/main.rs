use slint::ComponentHandle;
use slint_kit::*;
slint::include_modules!();
type Value = IsoWeek;
fn parse(text: &str) -> DraftValue<Value> {
    if text.is_empty() {
        return DraftValue::Empty;
    }
    if text.len() != 8
        || !(text.bytes().enumerate().all(|(i, b)| match i {
            4 => b == b'-',
            5 => b == b'W',
            _ => b.is_ascii_digit(),
        }))
    {
        return DraftValue::Invalid(ParseIssue::InvalidDate);
    }
    let Some((year, week)) = text.split_once("-W") else {
        return DraftValue::Incomplete;
    };
    match year
        .parse()
        .ok()
        .zip(week.parse().ok())
        .and_then(|(y, w)| IsoWeek::new(y, w).ok())
    {
        Some(value) => DraftValue::Parsed(value),
        None => DraftValue::Invalid(ParseIssue::InvalidDate),
    }
}
fn format(value: &Value) -> String {
    format!("{:04}-W{:02}", value.year().get(), value.week())
}
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    let initial = IsoWeek::new(2026, 42).unwrap();
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
