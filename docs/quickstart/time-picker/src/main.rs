use slint::ComponentHandle;
use slint_kit::*;
slint::include_modules!();
type Value = LocalTime;
fn parse(text: &str) -> DraftValue<Value> {
    LocalTime::parse(text, slint_kit::TimePrecision::Minute)
}
fn format(value: &Value) -> String {
    value.format(
        slint_kit::TimePrecision::Minute,
        slint_kit::ClockFormat::TwentyFourHour,
    )
}
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    let initial = LocalTime::new(9, 30, 0).unwrap();
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
    let binding = field.clone();
    app.on_adjust(move |segment, direction| {
        let DraftValue::Parsed(value) = parse(&binding.snapshot().draft) else {
            return;
        };
        let delta = match segment {
            TimeSegment::Hour => 3600,
            TimeSegment::Minute => 60,
            TimeSegment::Second => 1,
        } * if direction == TimeAdjustment::Increment {
            1
        } else {
            -1
        };
        let seconds = (value.hour() as i32 * 3600
            + value.minute() as i32 * 60
            + value.second() as i32
            + delta)
            .rem_euclid(86400) as u32;
        let next = LocalTime::new(seconds / 3600, seconds / 60 % 60, seconds % 60).unwrap();
        let _ = binding.edited(&format(&next));
        let _ = binding.commit(CommitTrigger::Explicit);
    });
    app.run()
}
