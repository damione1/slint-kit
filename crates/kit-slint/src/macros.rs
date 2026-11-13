// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
/// Bind the conventional window contract to a weak, window-local controller.
///
/// `ui` is the module containing `slint::include_modules!()`. Its window exposes
/// `rows`, `select-state` and select-open/navigate/activate/dismiss/query/typeahead
/// callbacks plus a `select-page-rows` getter projected from the active surface.
/// Payload conversion remains exhaustive and compile-time checked.
///
/// Deprecated compatibility API: use `Choices` and the component's
/// `options`/`selected`/`chosen` contract with `slint_kit::install` instead.
#[macro_export]
macro_rules! bind_select {
    ($ui:ident, $window:expr, $options:expr, $selected:expr, $ownership:expr) => {{
        use slint::ComponentHandle as _;
        let window = &$window;
        let weak = window.as_weak();
        let generation = std::cell::Cell::new(-1);
        let binding =
            $crate::SelectBinding::new($options, $selected, $ownership, move |snapshot| {
                if let Some(window) = weak.upgrade() {
                    if generation.replace(snapshot.generation) != snapshot.generation {
                        let rows = snapshot
                            .rows
                            .iter()
                            .map(|row| $ui::OptionRow {
                                token: row.token,
                                generation: row.generation,
                                label: row.label.as_str().into(),
                                description: row.description.as_str().into(),
                                disabled: row.disabled,
                            })
                            .collect::<Vec<_>>();
                        window.set_rows(slint::ModelRc::new(slint::VecModel::from(rows)));
                    }
                    let selection = |token: Option<i32>| $ui::SelectionRef {
                        present: token.is_some(),
                        token: token.unwrap_or(-1),
                    };
                    window.set_select_state($ui::SelectState {
                        selected: selection(snapshot.selected),
                        active: selection(snapshot.active),
                        open: snapshot.open,
                        active_index: snapshot.active_index,
                        generation: snapshot.generation,
                        selection_availability: match snapshot.selection_availability {
                            $crate::__core::SelectionAvailability::Absent => {
                                $ui::SelectionAvailability::Absent
                            }
                            $crate::__core::SelectionAvailability::Available => {
                                $ui::SelectionAvailability::Available
                            }
                            $crate::__core::SelectionAvailability::Unavailable => {
                                $ui::SelectionAvailability::Unavailable
                            }
                        },
                    });
                }
            });
        let target = binding.clone();
        window.on_select_open(move || target.open());
        let target = binding.clone();
        let weak = window.as_weak();
        window.on_select_navigate(move |step| {
            use $ui::NavigationStep as View;
            let step = match step {
                View::Next => $crate::__core::NavigationStep::Next,
                View::Previous => $crate::__core::NavigationStep::Previous,
                View::First => $crate::__core::NavigationStep::First,
                View::Last => $crate::__core::NavigationStep::Last,
                View::PageNext => $crate::__core::NavigationStep::PageNext,
                View::PagePrevious => $crate::__core::NavigationStep::PagePrevious,
            };
            if let Some(window) = weak.upgrade() {
                let page_rows = usize::try_from(window.get_select_page_rows())
                    .ok()
                    .and_then(std::num::NonZeroUsize::new);
                target.navigate(step, page_rows);
            }
        });
        let target = binding.clone();
        window.on_select_activate(move |token, generation| target.activate(token, generation));
        let target = binding.clone();
        window.on_select_dismiss(move |reason| {
            use $ui::CloseReason as View;
            let reason = match reason {
                View::Escape => $crate::__core::DismissReason::Escape,
                View::OutsidePress => $crate::__core::DismissReason::OutsidePress,
                View::SystemBack => $crate::__core::DismissReason::SystemBack,
                View::TriggerToggle => $crate::__core::DismissReason::TriggerToggle,
                View::CloseButton => $crate::__core::DismissReason::CloseButton,
                View::Programmatic => $crate::__core::DismissReason::Programmatic,
                View::HostClosed => $crate::__core::DismissReason::HostClosed,
            };
            target.dismiss(reason);
        });
        let target = binding.clone();
        window.on_select_query(move |query| target.query(query.as_str()));
        let target = binding.clone();
        window.on_select_typeahead(move |text| target.typeahead(text.as_str()));
        binding
    }};
}

/// Declare a typed form record and its conventional window ports once.
///
/// Each `field: required(Type, FieldSpec, message)` names a required native draft;
/// `field: checked` converts a checkbox to `bool`. The window exposes the matching
/// `field-field`, `field-candidate`, `report-field`, `focus-field`, `reset-field`
/// and `field-error` ports. Its stable enum has the corresponding CamelCase
/// variants. Missing ports, wrong codecs and unregistered enum variants are
/// compile errors. Global form callbacks and projections are connected here.
///
/// `Record::bind(&window)` returns the transactional `TypedFormBinding` for
/// asynchronous requests and field server errors. `Record::on_submit` is the
/// synchronous convenience: it admits a typed record, calls business code and
/// completes that ticket successfully. Call `install` once before either API.
#[macro_export]
macro_rules! form_schema {
    ($record:ident, $window:path, $keys:path, { $($field:ident: $mode:ident $(($($args:tt)*))?),+ $(,)? }) => {
        $crate::__form_identifiers! {
            pub struct $record {
                $(pub $field: $crate::form_schema!(@type $mode $(($($args)*))?)),+
            }
            impl $record {
                pub fn bind(app: &$window) -> $crate::TypedFormBinding<$window, $keys, Self> {
                    use $keys as FormKey;
                    let form = $crate::FormBinding::new(app, |app: &$window, view| {
                        app.set_valid(view.valid);
                        app.set_dirty(view.dirty);
                        app.set_submitting(view.submitting);
                        app.set_errors(view.errors);
                    });
                    $(let $field = $crate::form_schema!(@bind form, $window, FormKey::[<$field:camel>],
                        [<get_ $field _candidate>], [<get_ $field _state>],
                        $crate::FormFieldPorts {
                            snapshot: <$window>::[<get_ $field _field>],
                            report: <$window>::[<invoke_report_ $field>],
                            focus: <$window>::[<invoke_focus_ $field>],
                            reset: <$window>::[<invoke_reset_ $field>],
                            set_error: <$window>::[<set_ $field _error>],
                        }, $mode $(($($args)*))?);)+
                    form.set_focus_fallback(<$window>::invoke_focus_form);
                    let validate = form.clone();
                    app.on_validate_form(move || validate.report_validity());
                    let edited = form.clone();
                    app.on_field_edited(move |field| match field {
                        $(FormKey::[<$field:camel>] => edited.field_edited(&field)),+
                    });
                    let reset = form.clone();
                    app.on_reset_form(move || reset.reset());
                    let focus = form.clone();
                    app.on_focus_error(move |target| { focus.focus_target(&target); });
                    form.with_schema($crate::form_schema!(@zip $($field),+).map(
                        |$crate::form_schema!(@pattern $($field),+)| Self { $($field),+ }
                    ))
                }
                pub fn on_submit(app: &$window, business: impl Fn(&$window, Self) + 'static) {
                    let form = Self::bind(app);
                    app.on_send_form($crate::action(app, move |app| {
                        if let Ok(submission) = form.submit() {
                            business(&app, submission.payload);
                            form.finish(submission.ticket, Vec::new());
                        }
                    }));
                }
            }
        }
    };
    (@type required($value:ty, $spec:expr, $message:expr)) => { $value };
    (@type checked) => { bool };
    (@bind $form:ident, $window:path, $key:expr, $candidate:ident, $state:ident, $ports:expr,
        required($value:ty, $spec:expr, $message:expr)) => {
        $form.bind_required($key, $spec, <$window>::$candidate, $ports,
            move |_| ($message).into()).expect("schema field keys are unique")
    };
    (@bind $form:ident, $window:path, $key:expr, $candidate:ident, $state:ident, $ports:expr, checked) => {
        $form.bind_checked($key, $ports, <$window>::$state).expect("schema field keys are unique")
    };
    (@zip $field:ident) => { $field };
    (@zip $field:ident, $($rest:ident),+) => { $field.zip($crate::form_schema!(@zip $($rest),+)) };
    (@pattern $field:ident) => { $field };
    (@pattern $field:ident, $($rest:ident),+) => { ($field, $crate::form_schema!(@pattern $($rest),+)) };
}
