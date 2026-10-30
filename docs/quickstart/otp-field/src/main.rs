use slint::ComponentHandle;
use slint_kit::*;
slint::include_modules!();

fn rules(length: u8, pattern: &str) -> OtpRules {
    OtpRules::new(
        OtpLength::new(length).expect("valid code slots"),
        HtmlPattern::compile(pattern).expect("valid application pattern"),
    )
}
fn format(value: &OtpValue) -> String {
    value.as_str().into()
}
fn grouped(value: &OtpValue) -> String {
    let first: String = value.as_str().chars().take(2).collect();
    let second: String = value.as_str().chars().skip(2).collect();
    format!("{first}‑{second}")
}
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::init_form_validation!(self, app);
    // Configure the same complete-draft regex in Slint and Rust. Rust guards
    // acceptance even when a submission does not originate from the button.
    let numeric_rules = rules(6, "[0-9]*");
    let weak = app.as_weak();
    let numeric = FieldBinding::<OtpValue, ()>::new(
        None,
        Ownership::Local,
        CommitPolicy::OnSubmit,
        AbsencePolicy::Required,
        move |text| numeric_rules.parse(text),
        format,
        |_| String::new(),
        move |view| {
            if let Some(app) = weak.upgrade() {
                app.set_draft(view.draft.into());
            }
        },
    );
    numeric.set_html_constraints(HtmlConstraints {
        required: true,
        pattern: Some(HtmlPattern::compile("[0-9]*").unwrap()),
        min_length: Some(6),
        max_length: Some(6),
        ..Default::default()
    });
    let weak = app.as_weak();
    numeric.on_intent(move |intent| {
        if let (Some(app), FieldIntent::Commit(value)) = (weak.upgrade(), intent) {
            app.set_accepted(format(&value).into());
        }
    });
    let binding = numeric.clone();
    app.on_edited(move |text| {
        let _ = binding.edited(&text);
    });
    let binding = numeric.clone();
    app.on_commit(move || {
        let _ = binding.commit(CommitTrigger::Submit);
    });
    app.on_cancel(move || numeric.cancel());

    let alpha_rules = rules(6, "[A-Za-z0-9]*");
    let initial = match alpha_rules.parse("ABC123") {
        DraftValue::Parsed(value) => value,
        _ => unreachable!("complete example code"),
    };
    app.set_alpha_accepted(format(&initial).into());
    let weak = app.as_weak();
    let alpha = FieldBinding::<OtpValue, ()>::new(
        Some(initial),
        Ownership::Local,
        CommitPolicy::OnSubmit,
        AbsencePolicy::Required,
        move |text| alpha_rules.parse(text),
        format,
        |_| String::new(),
        move |view| {
            if let Some(app) = weak.upgrade() {
                app.set_alpha_draft(view.draft.into());
            }
        },
    );
    alpha.set_html_constraints(HtmlConstraints {
        required: true,
        pattern: Some(HtmlPattern::compile("[A-Za-z0-9]*").unwrap()),
        min_length: Some(6),
        max_length: Some(6),
        ..Default::default()
    });
    let weak = app.as_weak();
    alpha.on_intent(move |intent| {
        if let (Some(app), FieldIntent::Commit(value)) = (weak.upgrade(), intent) {
            app.set_alpha_accepted(format(&value).into());
        }
    });
    let binding = alpha.clone();
    app.on_alpha_edited(move |text| {
        let _ = binding.edited(&text);
    });
    let binding = alpha.clone();
    app.on_alpha_commit(move || {
        let _ = binding.commit(CommitTrigger::Submit);
    });
    app.on_alpha_cancel(move || alpha.cancel());
    let grouped_rules = rules(7, "[A-Z]{2}[0-9]{5}");
    let initial = match grouped_rules.parse("FG76657") {
        DraftValue::Parsed(value) => value,
        _ => unreachable!("complete example code"),
    };
    app.set_formatted_accepted(format(&initial).into());
    app.set_formatted_display(grouped(&initial).into());
    let weak = app.as_weak();
    let grouped_field = FieldBinding::<OtpValue, ()>::new(
        Some(initial),
        Ownership::Local,
        CommitPolicy::OnSubmit,
        AbsencePolicy::Required,
        move |text| grouped_rules.parse(text),
        format,
        |_| String::new(),
        move |view| {
            if let Some(app) = weak.upgrade() {
                app.set_formatted_draft(view.draft.into());
            }
        },
    );
    grouped_field.set_html_constraints(HtmlConstraints {
        required: true,
        pattern: Some(HtmlPattern::compile("[A-Z]{2}[0-9]{5}").unwrap()),
        min_length: Some(7),
        max_length: Some(7),
        ..Default::default()
    });
    let weak = app.as_weak();
    grouped_field.on_intent(move |intent| {
        if let (Some(app), FieldIntent::Commit(value)) = (weak.upgrade(), intent) {
            app.set_formatted_accepted(format(&value).into());
            app.set_formatted_display(grouped(&value).into());
        }
    });
    let binding = grouped_field.clone();
    app.on_formatted_edited(move |text| {
        let _ = binding.edited(&text);
    });
    let binding = grouped_field.clone();
    app.on_formatted_commit(move || {
        let _ = binding.commit(CommitTrigger::Submit);
    });
    app.on_formatted_cancel(move || grouped_field.cancel());
    app.run()
}
