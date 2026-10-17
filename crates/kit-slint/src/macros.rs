// SPDX-License-Identifier: GPL-3.0-or-later
/// Bind the conventional window contract to a weak, window-local controller.
///
/// `ui` is the module containing `slint::include_modules!()`. Its window exposes
/// `rows`, `select-state` and select-open/navigate/activate/dismiss/query/typeahead
/// callbacks. Payload conversion remains exhaustive and compile-time checked.
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
            target.navigate(step);
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
