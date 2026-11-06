// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use crate::ui;
use kit_core::{ModelError, NavigationStep, Ownership, SelectController, SelectOption};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    hash::Hash,
    rc::Rc,
    sync::atomic::{AtomicI32, Ordering},
};
static NEXT_TOKEN: AtomicI32 = AtomicI32::new(1);
static NEXT_GENERATION: AtomicI32 = AtomicI32::new(1);
fn unique(counter: &AtomicI32) -> i32 {
    let mut value = counter.load(Ordering::Relaxed);
    loop {
        let next = value
            .checked_add(1)
            .expect("model identity space exhausted");
        match counter.compare_exchange_weak(value, next, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(previous) => return previous,
            Err(actual) => value = actual,
        }
    }
}

/// Public row identity and availability. Tokens are assigned by KeyedModel,
/// never inferred from a row's current position.
pub trait KeyedRow: Clone + 'static {
    fn stamp(&mut self, token: i32, generation: i32);
    fn enabled(&self) -> bool;
    fn label(&self) -> String;
}
pub trait SelectableRow: KeyedRow {
    fn select(&mut self, selected: bool);
}
struct Entry<K, R> {
    key: K,
    token: i32,
    row: R,
}
struct State<K, R> {
    entries: Vec<Entry<K, R>>,
    generation: i32,
    active: Option<K>,
}
type ModelPublisher = Rc<dyn Fn()>;
/// A host-owned collection with stable typed keys, token fencing and a
/// virtualizable Slint model. Clones share one collection, not a window.
pub struct KeyedModel<K, R> {
    state: Rc<RefCell<State<K, R>>>,
    rows: Rc<VecModel<R>>,
    publish: Rc<RefCell<Vec<ModelPublisher>>>,
}
impl<K, R> Clone for KeyedModel<K, R> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            rows: self.rows.clone(),
            publish: self.publish.clone(),
        }
    }
}
impl<K: Clone + Eq + Hash + 'static, R: KeyedRow> KeyedModel<K, R> {
    pub fn new(entries: impl IntoIterator<Item = (K, R)>) -> Result<Self, ModelError> {
        let model = Self {
            state: Rc::new(RefCell::new(State {
                entries: vec![],
                generation: unique(&NEXT_GENERATION),
                active: None,
            })),
            rows: Rc::new(VecModel::default()),
            publish: Rc::new(RefCell::new(Vec::new())),
        };
        model.replace(entries)?;
        Ok(model)
    }
    pub fn model(&self) -> ModelRc<R> {
        self.rows.clone().into()
    }
    pub fn generation(&self) -> i32 {
        self.state.borrow().generation
    }
    pub fn active_index(&self) -> i32 {
        let state = self.state.borrow();
        state
            .active
            .as_ref()
            .and_then(|key| state.entries.iter().position(|entry| &entry.key == key))
            .map_or(-1, |index| {
                i32::try_from(index).expect("model index fits i32")
            })
    }
    pub fn active_row(&self) -> Option<R> {
        let state = self.state.borrow();
        state
            .entries
            .iter()
            .find(|entry| state.active.as_ref() == Some(&entry.key))
            .map(|entry| entry.row.clone())
    }
    /// Publish the current row to an editor through a weak window reference.
    /// Multiple views may observe a shared collection independently.
    pub fn bind_active<W: ComponentHandle + 'static>(&self, window: &W, setter: fn(&W, Option<R>)) {
        let window = window.as_weak();
        let state = Rc::downgrade(&self.state);
        self.publish.borrow_mut().push(Rc::new(move || {
            let (Some(window), Some(state)) = (window.upgrade(), state.upgrade()) else {
                return;
            };
            let state = state.borrow();
            let row = state
                .entries
                .iter()
                .find(|entry| state.active.as_ref() == Some(&entry.key))
                .map(|entry| entry.row.clone());
            drop(state);
            setter(&window, row);
        }));
        self.refresh();
    }
    /// Update only the still-active identity from the current projection.
    pub fn update_active(
        &self,
        token: i32,
        generation: i32,
        update: impl FnOnce(&mut R),
    ) -> Option<K> {
        let key = self.accept(token, generation)?;
        if self.state.borrow().active.as_ref() != Some(&key) {
            return None;
        }
        self.update(token, generation, update)
    }
    pub fn accept(&self, token: i32, generation: i32) -> Option<K> {
        let state = self.state.borrow();
        if generation != state.generation {
            return None;
        }
        state
            .entries
            .iter()
            .find(|entry| entry.token == token && entry.row.enabled())
            .map(|entry| entry.key.clone())
    }
    pub fn key(&self, token: i32) -> Option<K> {
        self.state
            .borrow()
            .entries
            .iter()
            .find(|entry| entry.token == token && entry.row.enabled())
            .map(|entry| entry.key.clone())
    }
    pub fn replace(&self, entries: impl IntoIterator<Item = (K, R)>) -> Result<(), ModelError> {
        let entries = entries.into_iter().collect::<Vec<_>>();
        let mut keys = HashSet::new();
        if entries.iter().any(|(key, _)| !keys.insert(key)) {
            return Err(ModelError::DuplicateKey);
        }
        let mut state = self.state.borrow_mut();
        let tokens = state
            .entries
            .iter()
            .map(|entry| (entry.key.clone(), entry.token))
            .collect::<HashMap<_, _>>();
        state.generation = unique(&NEXT_GENERATION);
        let generation = state.generation;
        state.entries = entries
            .into_iter()
            .map(|(key, mut row)| {
                let token = tokens
                    .get(&key)
                    .copied()
                    .unwrap_or_else(|| unique(&NEXT_TOKEN));
                row.stamp(token, generation);
                Entry { key, token, row }
            })
            .collect();
        if !state
            .entries
            .iter()
            .any(|entry| state.active.as_ref() == Some(&entry.key) && entry.row.enabled())
        {
            state.active = None;
        }
        drop(state);
        self.refresh();
        Ok(())
    }
    fn refresh(&self) {
        let rows: Vec<_> = self
            .state
            .borrow()
            .entries
            .iter()
            .map(|entry| entry.row.clone())
            .collect();
        self.rows.set_vec(rows);
        let callbacks = self.publish.borrow().clone();
        for callback in callbacks {
            callback();
        }
    }
    /// Reorder rows without changing their stable keys, selection or active key.
    /// Every reorder advances the generation, invalidating old pointer gestures.
    pub fn sort_by(&self, mut compare: impl FnMut(&K, &K) -> std::cmp::Ordering) {
        let mut entries = self
            .state
            .borrow()
            .entries
            .iter()
            .map(|entry| (entry.key.clone(), entry.row.clone()))
            .collect::<Vec<_>>();
        entries.sort_by(|(left, _), (right, _)| compare(left, right));
        self.replace(entries)
            .expect("sorting preserves unique keys");
    }
    /// A sortable header action that alternates ascending/descending order.
    /// The application supplies its domain comparator and receives the display
    /// direction; identity, generation and row publication stay in the model.
    pub fn sorting_by(
        &self,
        compare: impl Fn(&K, &K) -> std::cmp::Ordering + 'static,
        changed: impl Fn(ui::DataSortDirection) + 'static,
    ) -> impl Fn() + 'static {
        let model = self.clone();
        let ascending = std::cell::Cell::new(false);
        move || {
            ascending.set(!ascending.get());
            model.sort_by(|left, right| {
                if ascending.get() {
                    compare(left, right)
                } else {
                    compare(right, left)
                }
            });
            changed(if ascending.get() {
                ui::DataSortDirection::Ascending
            } else {
                ui::DataSortDirection::Descending
            });
        }
    }
    pub fn insert(&self, key: K, row: R) -> Result<(), ModelError> {
        let mut entries = self
            .state
            .borrow()
            .entries
            .iter()
            .map(|entry| (entry.key.clone(), entry.row.clone()))
            .collect::<Vec<_>>();
        entries.push((key, row));
        self.replace(entries)
    }
    pub fn update(&self, token: i32, generation: i32, update: impl FnOnce(&mut R)) -> Option<K> {
        let key = self.accept(token, generation)?;
        let mut entries = self
            .state
            .borrow()
            .entries
            .iter()
            .map(|entry| (entry.key.clone(), entry.row.clone()))
            .collect::<Vec<_>>();
        update(
            &mut entries
                .iter_mut()
                .find(|(candidate, _)| candidate == &key)
                .expect("accepted key exists")
                .1,
        );
        self.replace(entries)
            .expect("updating a row preserves unique keys");
        Some(key)
    }
    pub fn remove(&self, token: i32) -> Option<K> {
        let key = self.key(token)?;
        let entries = self
            .state
            .borrow()
            .entries
            .iter()
            .filter(|entry| entry.key != key)
            .map(|entry| (entry.key.clone(), entry.row.clone()))
            .collect::<Vec<_>>();
        self.replace(entries)
            .expect("removal preserves unique keys");
        Some(key)
    }
    pub fn remove_fenced(&self, token: i32, generation: i32) -> Option<K> {
        self.accept(token, generation)?;
        self.remove(token)
    }
    pub fn navigate(&self, step: NavigationStep) {
        let mut state = self.state.borrow_mut();
        let options = state
            .entries
            .iter()
            .map(|entry| SelectOption {
                key: entry.key.clone(),
                label: entry.row.label(),
                description: String::new(),
                disabled: !entry.row.enabled(),
            })
            .collect();
        let mut controller = SelectController::new(options, state.active.clone(), Ownership::Local);
        controller.open();
        let step = if state.active.is_none() {
            match step {
                NavigationStep::Next => NavigationStep::First,
                NavigationStep::Previous => NavigationStep::Last,
                NavigationStep::First
                | NavigationStep::Last
                | NavigationStep::PageNext
                | NavigationStep::PagePrevious => step,
            }
        } else {
            step
        };
        controller.navigate(step, 10);
        state.active = controller.state().active.clone();
        drop(state);
        self.refresh();
    }
    pub fn navigation(&self) -> impl Fn(ui::NavigationStep) + 'static {
        let model = self.clone();
        move |step| model.navigate(step.into())
    }
    /// Connect the collection projection through explicit setters. No naming
    /// convention or strong reference to the application window is required.
    pub fn bind<W: ComponentHandle + 'static>(
        &self,
        window: &W,
        rows: fn(&W, ModelRc<R>),
        generation: fn(&W, i32),
        active: fn(&W, i32),
    ) {
        let weak = window.as_weak();
        let state = Rc::downgrade(&self.state);
        let model = Rc::downgrade(&self.rows);
        self.publish.borrow_mut().push(Rc::new(move || {
            let (Some(window), Some(state), Some(model)) =
                (weak.upgrade(), state.upgrade(), model.upgrade())
            else {
                return;
            };
            let state = state.borrow();
            rows(&window, model.into());
            generation(&window, state.generation);
            active(
                &window,
                state
                    .active
                    .as_ref()
                    .and_then(|key| state.entries.iter().position(|entry| &entry.key == key))
                    .map_or(-1, |index| {
                        i32::try_from(index).expect("model index fits i32")
                    }),
            );
        }));
        self.refresh();
    }
}
impl<K: Clone + Eq + Hash + 'static, R: SelectableRow> KeyedModel<K, R> {
    pub fn select(&self, token: i32, generation: i32) -> Option<K> {
        let key = self.accept(token, generation)?;
        let mut state = self.state.borrow_mut();
        state.active = Some(key.clone());
        for entry in &mut state.entries {
            entry.row.select(entry.key == key);
        }
        drop(state);
        self.refresh();
        Some(key)
    }
    pub fn activation(&self, handler: impl Fn(K) + 'static) -> impl Fn(i32, i32) + 'static {
        let model = self.clone();
        move |token, generation| {
            if let Some(key) = model.select(token, generation) {
                handler(key);
            }
        }
    }
}
impl KeyedRow for ui::TableRow {
    fn stamp(&mut self, token: i32, _: i32) {
        self.token = token;
    }
    fn enabled(&self) -> bool {
        !self.disabled
    }
    fn label(&self) -> String {
        self.cells
            .row_data(0)
            .map_or_else(String::new, |label| label.to_string())
    }
}
impl SelectableRow for ui::TableRow {
    fn select(&mut self, selected: bool) {
        self.selected = selected;
    }
}
impl KeyedRow for ui::ContentItem {
    fn stamp(&mut self, token: i32, _: i32) {
        self.token = token;
    }
    fn enabled(&self) -> bool {
        !self.disabled
    }
    fn label(&self) -> String {
        self.title.to_string()
    }
}
impl SelectableRow for ui::ContentItem {
    fn select(&mut self, selected: bool) {
        self.selected = selected;
    }
}
impl KeyedRow for ui::TextLine {
    fn stamp(&mut self, token: i32, _: i32) {
        self.token = token;
    }
    fn enabled(&self) -> bool {
        true
    }
    fn label(&self) -> String {
        self.text.to_string()
    }
}
impl SelectableRow for ui::TextLine {
    fn select(&mut self, selected: bool) {
        self.selected = selected;
    }
}
impl KeyedRow for ui::ChipRow {
    fn stamp(&mut self, token: i32, _: i32) {
        self.token = token;
    }
    fn enabled(&self) -> bool {
        self.removable
    }
    fn label(&self) -> String {
        self.label.to_string()
    }
}
impl KeyedRow for ui::FileViewEntry {
    fn stamp(&mut self, token: i32, generation: i32) {
        self.token = token;
        self.generation = generation;
    }
    fn enabled(&self) -> bool {
        self.removable
    }
    fn label(&self) -> String {
        self.name.to_string()
    }
}

impl KeyedRow for ui::NotificationRow {
    fn stamp(&mut self, token: i32, generation: i32) {
        self.item.token = token;
        self.item.generation = generation;
    }
    fn enabled(&self) -> bool {
        !self.item.disabled
    }
    fn label(&self) -> String {
        self.item.label.to_string()
    }
}
impl KeyedRow for ui::ToastEntry {
    fn stamp(&mut self, token: i32, _: i32) {
        self.token = token;
    }
    fn enabled(&self) -> bool {
        true
    }
    fn label(&self) -> String {
        self.title.to_string()
    }
}
