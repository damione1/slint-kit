// SPDX-License-Identifier: GPL-3.0-or-later
use kit_core::{
    DismissReason, ModelError, NavigationStep, Ownership, RemovalPolicy, SelectController,
    SelectOption, SelectionAvailability, SelectionIntent,
};
use std::{cell::RefCell, collections::HashMap, hash::Hash, rc::Rc};
use web_time::Instant;

#[derive(Clone, Debug)]
pub struct ProjectedOption {
    pub token: i32,
    pub generation: i32,
    pub label: String,
    pub description: String,
    pub disabled: bool,
}

/// A coherent, immutable projection. Tokens are positions valid only within this
/// generation; the controller retains the application's actual stable keys.
#[derive(Clone, Debug)]
pub struct SelectProjection {
    pub rows: Rc<Vec<ProjectedOption>>,
    pub selected: Option<i32>,
    pub selection_availability: SelectionAvailability,
    pub active: Option<i32>,
    pub active_index: i32,
    pub open: bool,
    pub generation: i32,
}

type IntentHandler<K> = Rc<dyn Fn(SelectionIntent<K>)>;
struct Inner<K> {
    controller: SelectController<K>,
    source: Vec<SelectOption<K>>,
    source_index: HashMap<K, i32>,
    query: String,
    rows: Rc<Vec<ProjectedOption>>,
    rows_generation: u64,
}

/// Window-local controller binding. Its publisher should hold a weak window
/// handle. No I/O, runtime, global registry, or strong window reference is needed.
///
/// Editable search performs Unicode lowercase substring matching (accent-sensitive).
/// Non-editable typeahead is a separate event and uses an injected monotonic clock.
pub struct SelectBinding<K> {
    inner: Rc<RefCell<Inner<K>>>,
    publish: Rc<dyn Fn(SelectProjection)>,
    intent: Rc<RefCell<Option<IntentHandler<K>>>>,
    epoch: Instant,
}
impl<K> Clone for SelectBinding<K> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            publish: self.publish.clone(),
            intent: self.intent.clone(),
            epoch: self.epoch,
        }
    }
}
impl<K: Clone + Eq + Hash + 'static> SelectBinding<K> {
    pub fn new(
        options: Vec<SelectOption<K>>,
        selected: Option<K>,
        ownership: Ownership,
        publish: impl Fn(SelectProjection) + 'static,
    ) -> Self {
        let mut controller = SelectController::new(options.clone(), selected, ownership);
        // Filtering is a view operation, not removal of the application value.
        controller.set_removal_policy(RemovalPolicy::PreserveUnavailable);
        let result = Self {
            inner: Rc::new(RefCell::new(Inner {
                controller,
                source_index: source_index(&options),
                source: options,
                query: String::new(),
                rows: Rc::new(Vec::new()),
                rows_generation: 0,
            })),
            publish: Rc::new(publish),
            intent: Rc::new(RefCell::new(None)),
            epoch: Instant::now(),
        };
        result.refresh();
        result
    }
    pub fn on_intent(&self, callback: impl Fn(SelectionIntent<K>) + 'static) {
        *self.intent.borrow_mut() = Some(Rc::new(callback));
    }
    pub fn snapshot(&self) -> SelectProjection {
        let inner = self.inner.borrow();
        let state = inner.controller.state();
        let token = |key: &Option<K>| {
            key.as_ref()
                .and_then(|key| inner.source_index.get(key).copied())
        };
        let active = token(&state.active);
        let selected = token(&state.selected);
        let selection_availability = match (&state.selected, selected) {
            (None, _) => SelectionAvailability::Absent,
            (Some(_), Some(_)) => SelectionAvailability::Available,
            (Some(_), None) => SelectionAvailability::Unavailable,
        };
        SelectProjection {
            rows: inner.rows.clone(),
            selected,
            selection_availability,
            active,
            active_index: state
                .active
                .as_ref()
                .and_then(|key| {
                    inner
                        .controller
                        .options()
                        .iter()
                        .position(|row| &row.key == key)
                })
                .and_then(|index| i32::try_from(index).ok())
                .unwrap_or(-1),
            open: state.open,
            generation: i32::try_from(inner.controller.generation()).unwrap_or(-1),
        }
    }
    pub fn refresh(&self) {
        {
            let mut inner = self.inner.borrow_mut();
            let generation = inner.controller.generation();
            if inner.rows_generation != generation {
                inner.rows = Rc::new(
                    inner
                        .controller
                        .options()
                        .iter()
                        .map(|row| ProjectedOption {
                            token: inner.source_index[&row.key],
                            generation: i32::try_from(generation).unwrap_or(-1),
                            label: row.label.clone(),
                            description: row.description.clone(),
                            disabled: row.disabled,
                        })
                        .collect(),
                );
                inner.rows_generation = generation;
            }
        }
        (self.publish)(self.snapshot());
    }
    fn apply(&self, action: impl FnOnce(&mut SelectController<K>) -> Option<SelectionIntent<K>>) {
        let intent = action(&mut self.inner.borrow_mut().controller);
        self.refresh();
        if let Some(intent) = intent {
            self.emit_intent(intent);
        }
    }
    fn emit_intent(&self, intent: SelectionIntent<K>) {
        let callback = self.intent.borrow().clone();
        if let Some(callback) = callback {
            callback(intent);
        }
    }
    pub fn open(&self) {
        self.apply(|c| {
            c.open();
            None
        });
    }
    pub fn navigate(&self, step: NavigationStep) {
        self.apply(|c| {
            c.navigate(step, 7);
            None
        });
    }
    pub fn activate(&self, token: i32, generation: i32) {
        if token < 0 || generation < 0 {
            return;
        }
        let key = {
            let inner = self.inner.borrow();
            if inner.controller.generation() != generation as u64 {
                return;
            }
            inner.source.get(token as usize).map(|row| row.key.clone())
        };
        if let Some(key) = key {
            self.apply(|c| c.activate(&key));
        }
    }
    pub fn commit(&self) {
        self.apply(SelectController::commit);
    }
    pub fn dismiss(&self, reason: DismissReason) {
        self.apply(|c| {
            c.dismiss(reason);
            None
        });
    }
    pub fn typeahead_at(&self, text: &str, now_ms: u64) {
        self.apply(|c| {
            c.typeahead(text, now_ms);
            None
        });
    }
    pub fn typeahead(&self, text: &str) {
        self.typeahead_at(
            text,
            self.epoch.elapsed().as_millis().min(u64::MAX as u128) as u64,
        );
    }
    pub fn selected(&self) -> Option<K> {
        self.inner.borrow().controller.state().selected.clone()
    }
    pub fn options(&self) -> Vec<SelectOption<K>> {
        self.inner.borrow().source.clone()
    }
    pub fn set_selected(&self, selected: Option<K>) {
        self.apply(|c| {
            c.set_selected(selected);
            None
        });
    }
    pub fn query(&self, query: &str) {
        {
            let mut inner = self.inner.borrow_mut();
            inner.query = query.to_owned();
            let filtered = filter(&inner.source, query);
            inner
                .controller
                .reconcile(filtered)
                .expect("source keys already validated");
        }
        self.refresh();
    }
    pub fn reconcile(&self, options: Vec<SelectOption<K>>) -> Result<(), ModelError> {
        // Validate all source keys, including rows currently filtered out.
        SelectController::try_new(options.clone(), None, Ownership::Local)?;
        let removed = {
            let mut inner = self.inner.borrow_mut();
            let removed = inner
                .controller
                .state()
                .selected
                .as_ref()
                .is_some_and(|key| !options.iter().any(|row| &row.key == key));
            if removed && inner.controller.ownership() == Ownership::Local {
                inner.controller.set_selected(None);
            }
            let filtered = filter(&options, &inner.query);
            inner.controller.reconcile(filtered)?;
            inner.source_index = source_index(&options);
            inner.source = options;
            removed
        };
        self.refresh();
        if removed {
            self.emit_intent(SelectionIntent::Clear);
        }
        Ok(())
    }
}
fn source_index<K: Clone + Eq + Hash>(source: &[SelectOption<K>]) -> HashMap<K, i32> {
    source
        .iter()
        .enumerate()
        .map(|(index, row)| {
            (
                row.key.clone(),
                i32::try_from(index).expect("Slint models are bounded by i32 row tokens"),
            )
        })
        .collect()
}
fn filter<K: Clone>(source: &[SelectOption<K>], query: &str) -> Vec<SelectOption<K>> {
    let query = query.to_lowercase();
    source
        .iter()
        .filter(|row| {
            query.is_empty()
                || row.label.to_lowercase().contains(&query)
                || row.description.to_lowercase().contains(&query)
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn filtering_preserves_value_and_invalidates_gestures() {
        let c = SelectBinding::new(
            vec![SelectOption::new(9, "Nine"), SelectOption::new(3, "Three")],
            Some(9),
            Ownership::Local,
            |_| {},
        );
        c.open();
        let old = c.snapshot();
        c.query("Three");
        assert_eq!(c.selected(), Some(9));
        assert_eq!(c.snapshot().selected, Some(0));
        assert_eq!(c.snapshot().rows[0].token, 1);
        assert_eq!(
            c.snapshot().selection_availability,
            SelectionAvailability::Available
        );
        c.activate(0, old.generation);
        assert!(c.snapshot().open);
        assert_eq!(c.selected(), Some(9));
        c.query("");
        let rows = c.options();
        c.reconcile(rows.into_iter().rev().collect()).unwrap();
        assert_eq!(c.selected(), Some(9));
        assert_eq!(c.snapshot().selected, Some(1));
        c.reconcile(vec![SelectOption::new(3, "Three")]).unwrap();
        assert_eq!(c.selected(), None);
    }
    #[test]
    fn filtered_activation_uses_stable_source_token() {
        let c = SelectBinding::new(
            vec![SelectOption::new(9, "Nine"), SelectOption::new(3, "Three")],
            Some(9),
            Ownership::Local,
            |_| {},
        );
        c.query("Three");
        c.open();
        let row = c.snapshot().rows[0].clone();
        assert_eq!(row.token, 1);
        assert_eq!(c.snapshot().active_index, 0);
        c.activate(row.token, row.generation);
        assert_eq!(c.selected(), Some(3));
    }
    #[test]
    fn controlled_removal_reports_unavailable_until_acknowledged() {
        let c = SelectBinding::new(
            vec![SelectOption::new(9, "Nine")],
            Some(9),
            Ownership::Controlled,
            |_| {},
        );
        let calls = Rc::new(RefCell::new(Vec::new()));
        let captured = calls.clone();
        c.on_intent(move |intent| captured.borrow_mut().push(intent));
        c.reconcile(vec![]).unwrap();
        assert_eq!(c.selected(), Some(9));
        assert_eq!(
            c.snapshot().selection_availability,
            SelectionAvailability::Unavailable
        );
        assert_eq!(*calls.borrow(), vec![SelectionIntent::Clear]);
        c.set_selected(None);
        assert_eq!(
            c.snapshot().selection_availability,
            SelectionAvailability::Absent
        );
    }
    #[test]
    fn intent_handler_can_replace_itself_without_retaining_borrows() {
        let c = SelectBinding::new(
            vec![SelectOption::new(1, "One")],
            None,
            Ownership::Local,
            |_| {},
        );
        let inner = Rc::downgrade(&c.inner);
        let publisher = Rc::downgrade(&c.publish);
        let intent = Rc::downgrade(&c.intent);
        let epoch = c.epoch;
        c.on_intent(move |_| {
            let binding = SelectBinding {
                inner: inner.upgrade().unwrap(),
                publish: publisher.upgrade().unwrap(),
                intent: intent.upgrade().unwrap(),
                epoch,
            };
            binding.on_intent(|_| {});
        });
        c.open();
        c.commit();
        assert_eq!(c.selected(), Some(1));
    }
    #[test]
    fn model_storage_is_reused_during_navigation_and_callbacks_can_read_state() {
        let c = SelectBinding::new(
            (0..10_000)
                .map(|i| SelectOption::new(i, format!("Item {i}")))
                .collect(),
            None,
            Ownership::Local,
            |_| {},
        );
        c.open();
        let rows = c.snapshot().rows;
        c.navigate(NavigationStep::Last);
        assert!(Rc::ptr_eq(&rows, &c.snapshot().rows));
        let weak = Rc::downgrade(&c.inner);
        c.on_intent(move |_| {
            assert!(!weak.upgrade().unwrap().borrow().controller.state().open);
        });
        c.commit();
        assert_eq!(c.selected(), Some(9999));
    }
}
