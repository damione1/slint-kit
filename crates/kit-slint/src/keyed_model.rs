// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use crate::ui;
use kit_core::{ModelError, NavigationStep};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    hash::Hash,
    rc::Rc,
    sync::atomic::{AtomicI32, Ordering},
};
static NEXT_TOKEN: AtomicI32 = AtomicI32::new(1);
fn unique(counter: &AtomicI32) -> Result<i32, ModelError> {
    let mut current = counter.load(Ordering::Relaxed);
    loop {
        let next = current
            .checked_add(1)
            .ok_or(ModelError::IdentityExhausted)?;
        match counter.compare_exchange_weak(current, next, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(token) => return Ok(token),
            Err(observed) => current = observed,
        }
    }
}
// Slint's signed int has a finite positive interaction tag space. Each model
// advances its own wide sequence and cycles that tag deliberately; a busy
// collection cannot consume another collection's generations. Live gestures
// are cancelled on every tag change, including the wrap. Removed row tokens
// remain process-unique and are never recycled.
fn view_generation(generation: u64) -> i32 {
    let limit = u64::try_from(i32::MAX).expect("positive i32 bound fits u64");
    i32::try_from(generation.saturating_sub(1) % limit + 1)
        .expect("bounded generation tag fits i32")
}
fn next_generation(generation: u64) -> u64 {
    generation.wrapping_add(1).max(1)
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
    revision: Rc<()>,
}
// A row revision exists only together with the key it describes. Initial
// publication remains distinct from an already-published empty selection.
enum ActivePublication<K> {
    Unpublished,
    Empty,
    Record { key: K, revision: Rc<()> },
}
impl<K: Eq> ActivePublication<K> {
    fn unchanged<R>(&self, entry: Option<&Entry<K, R>>) -> bool {
        match self {
            Self::Unpublished => false,
            Self::Empty => entry.is_none(),
            Self::Record { key, revision } => entry
                .is_some_and(|entry| key == &entry.key && Rc::ptr_eq(revision, &entry.revision)),
        }
    }
}
struct State<K, R> {
    entries: Vec<Entry<K, R>>,
    generation: u64,
    active: Option<K>,
    indices: HashMap<K, usize>,
}
enum MembershipChange {
    Insert(usize),
    Remove(usize),
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
                generation: 0,
                active: None,
                indices: HashMap::new(),
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
    /// Current positive Slint interaction tag, local to this model.
    ///
    /// The tag deliberately cycles after `i32::MAX`; it is an interaction fence,
    /// not a persistent record version. Row tokens are never recycled.
    pub fn generation(&self) -> i32 {
        view_generation(self.state.borrow().generation)
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
        let previous = RefCell::new(ActivePublication::Unpublished);
        let window = window.as_weak();
        let state = Rc::downgrade(&self.state);
        self.publish.borrow_mut().push(Rc::new(move || {
            let (Some(window), Some(state)) = (window.upgrade(), state.upgrade()) else {
                return;
            };
            let state = state.borrow();
            let entry = state
                .active
                .as_ref()
                .and_then(|key| state.indices.get(key))
                .map(|index| &state.entries[*index]);
            if previous.borrow().unchanged(entry) {
                return;
            }
            *previous.borrow_mut() = match entry {
                Some(entry) => ActivePublication::Record {
                    key: entry.key.clone(),
                    revision: entry.revision.clone(),
                },
                None => ActivePublication::Empty,
            };
            let row = entry.map(|entry| entry.row.clone());
            drop(state);
            setter(&window, row);
        }));
        self.publish();
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
        if generation != view_generation(state.generation) {
            return None;
        }
        state
            .entries
            .iter()
            .find(|entry| entry.token == token && entry.row.enabled())
            .map(|entry| entry.key.clone())
    }
    /// Current position of a record in the projected model, for components
    /// that scroll a virtual list to a keyed row (`KitNotificationCenter.focus-index`).
    /// Publish it with the row's token; the position is a projection, not identity.
    pub fn index_of(&self, key: &K) -> Option<i32> {
        let state = self.state.borrow();
        let index = *state.indices.get(key)?;
        Some(i32::try_from(index).expect("model index fits i32"))
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
        self.replace_allocating(entries, || unique(&NEXT_TOKEN))
    }
    fn replace_allocating(
        &self,
        entries: impl IntoIterator<Item = (K, R)>,
        mut allocate: impl FnMut() -> Result<i32, ModelError>,
    ) -> Result<(), ModelError> {
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
        let next = next_generation(state.generation);
        let generation = view_generation(next);
        let projected = entries
            .into_iter()
            .map(|(key, mut row)| {
                let token = match tokens.get(&key).copied() {
                    Some(token) => token,
                    None => allocate()?,
                };
                row.stamp(token, generation);
                Ok(Entry {
                    key,
                    token,
                    row,
                    revision: Rc::new(()),
                })
            })
            .collect::<Result<Vec<_>, ModelError>>()?;
        state.generation = next;
        state.entries = projected;
        state.indices = state
            .entries
            .iter()
            .enumerate()
            .map(|(index, entry)| (entry.key.clone(), index))
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
        self.publish();
    }
    fn publish(&self) {
        let callbacks = self.publish.borrow().clone();
        for callback in callbacks {
            callback();
        }
    }
    /// Reorder rows without changing their stable keys, selection or active key.
    /// Every reorder advances the generation, invalidating old pointer gestures.
    pub fn sort_by(&self, mut compare: impl FnMut(&K, &K) -> std::cmp::Ordering) {
        let mut state = self.state.borrow_mut();
        state
            .entries
            .sort_by(|left, right| compare(&left.key, &right.key));
        state.generation = next_generation(state.generation);
        let generation = view_generation(state.generation);
        for entry in &mut state.entries {
            entry.row.stamp(entry.token, generation);
        }
        state.indices = state
            .entries
            .iter()
            .enumerate()
            .map(|(index, entry)| (entry.key.clone(), index))
            .collect();
        drop(state);
        self.refresh();
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
    /// A `KitDataTable.cell-committed` handler that admits a commit only
    /// when its armed generation is the current one and its row is still an
    /// enabled entry, then hands the row key, the column token and the field
    /// commit to the application, which runs the commit through that
    /// column's `FieldSpec::accept`. A sort or filter that advanced the
    /// generation while the cell was edited is rejected here, so no commit
    /// reaches another row.
    pub fn cell_commits(
        &self,
        handler: impl Fn(K, i32, ui::FieldCommit) + 'static,
    ) -> impl Fn(i32, i32, i32, ui::FieldCommit) + 'static {
        let model = self.clone();
        move |token, column, generation, commit| {
            if let Some(key) = model.accept(token, generation) {
                handler(key, column, commit);
            }
        }
    }
    pub fn insert(&self, key: K, row: R) -> Result<(), ModelError> {
        self.insert_allocating(key, row, || unique(&NEXT_TOKEN))
    }
    fn insert_allocating(
        &self,
        key: K,
        row: R,
        allocate: impl FnOnce() -> Result<i32, ModelError>,
    ) -> Result<(), ModelError> {
        let mut state = self.state.borrow_mut();
        if state.indices.contains_key(&key) {
            return Err(ModelError::DuplicateKey);
        }
        let index = state.entries.len();
        state.entries.push(Entry {
            key,
            token: allocate()?,
            row,
            revision: Rc::new(()),
        });
        drop(state);
        self.membership_changed(MembershipChange::Insert(index));
        Ok(())
    }
    fn membership_changed(&self, change: MembershipChange) {
        let mut state = self.state.borrow_mut();
        state.generation = next_generation(state.generation);
        let generation = view_generation(state.generation);
        for entry in &mut state.entries {
            entry.row.stamp(entry.token, generation);
        }
        state.indices = state
            .entries
            .iter()
            .enumerate()
            .map(|(index, entry)| (entry.key.clone(), index))
            .collect();
        let rows: Vec<_> = state
            .entries
            .iter()
            .map(|entry| entry.row.clone())
            .collect();
        drop(state);
        // Structural notifications retain every surviving repeater delegate.
        // Stamping the new generation still fences gestures on other rows.
        match change {
            MembershipChange::Insert(index) => self.rows.insert(index, rows[index].clone()),
            MembershipChange::Remove(index) => {
                self.rows.remove(index);
            }
        }
        for (index, row) in rows.into_iter().enumerate() {
            self.rows.set_row_data(index, row);
        }
        self.publish();
    }
    /// Number of projected records.
    pub fn len(&self) -> usize {
        self.state.borrow().entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Read a domain record, including a disabled one.
    pub fn get(&self, key: &K) -> Option<R> {
        let state = self.state.borrow();
        state
            .indices
            .get(key)
            .map(|index| state.entries[*index].row.clone())
    }
    /// Update a domain record without resetting delegates or invalidating other gestures.
    /// Unlike UI intents, domain access may re-enable a disabled record.
    pub fn update_by_key(&self, key: &K, update: impl FnOnce(&mut R)) -> bool {
        let mut state = self.state.borrow_mut();
        let Some(index) = state.indices.get(key).copied() else {
            return false;
        };
        let generation = view_generation(state.generation);
        let entry = &mut state.entries[index];
        update(&mut entry.row);
        entry.row.stamp(entry.token, generation);
        entry.revision = Rc::new(());
        let row = entry.row.clone();
        if !entry.row.enabled() && state.active.as_ref() == Some(key) {
            state.active = None;
        }
        drop(state);
        self.rows.set_row_data(index, row);
        self.publish();
        true
    }
    pub fn update(&self, token: i32, generation: i32, update: impl FnOnce(&mut R)) -> Option<K> {
        let key = self.accept(token, generation)?;
        self.update_by_key(&key, update);
        Some(key)
    }
    /// Remove a domain record regardless of its disabled state.
    pub fn remove_by_key(&self, key: &K) -> bool {
        let mut state = self.state.borrow_mut();
        let Some(index) = state.indices.get(key).copied() else {
            return false;
        };
        state.entries.remove(index);
        if state.active.as_ref() == Some(key) {
            state.active = None;
        }
        drop(state);
        self.membership_changed(MembershipChange::Remove(index));
        true
    }
    pub fn remove(&self, token: i32) -> Option<K> {
        let key = self.key(token)?;
        self.remove_by_key(&key);
        Some(key)
    }
    pub fn remove_fenced(&self, token: i32, generation: i32) -> Option<K> {
        self.accept(token, generation)?;
        self.remove(token)
    }
    /// Navigate with an explicit current viewport capacity for page keys.
    /// Non-page steps ignore the capacity; zero capacity advances one row.
    pub fn navigate(&self, step: NavigationStep, page_rows: usize) {
        self.navigate_page(step, page_rows);
    }
    /// Navigate by the view's current visible row count for page keys.
    pub fn navigate_page(&self, step: NavigationStep, page_rows: usize) {
        let mut state = self.state.borrow_mut();
        let current = state
            .active
            .as_ref()
            .and_then(|key| state.indices.get(key))
            .copied();
        let first = state.entries.iter().position(|entry| entry.row.enabled());
        let last = state.entries.iter().rposition(|entry| entry.row.enabled());
        let target = match step {
            NavigationStep::First => first,
            NavigationStep::Last => last,
            NavigationStep::Next => current
                .and_then(|index| {
                    state
                        .entries
                        .iter()
                        .enumerate()
                        .skip(index + 1)
                        .find(|(_, entry)| entry.row.enabled())
                        .map(|(index, _)| index)
                })
                .or(current)
                .or(first),
            NavigationStep::Previous => current
                .and_then(|index| {
                    state.entries[..index]
                        .iter()
                        .rposition(|entry| entry.row.enabled())
                })
                .or(current)
                .or(last),
            NavigationStep::PageNext => {
                let start = current.unwrap_or(0).saturating_add(page_rows.max(1));
                state
                    .entries
                    .iter()
                    .enumerate()
                    .skip(start)
                    .find(|(_, entry)| entry.row.enabled())
                    .map(|(index, _)| index)
                    .or(last)
            }
            NavigationStep::PagePrevious => {
                let start = current
                    .unwrap_or(state.entries.len())
                    .saturating_sub(page_rows.max(1));
                state
                    .entries
                    .iter()
                    .take(start + 1)
                    .rposition(|entry| entry.row.enabled())
                    .or(first)
            }
        };
        state.active = target.map(|index| state.entries[index].key.clone());
        drop(state);
        self.publish();
    }
    /// Bind navigation to the view's current page size without retaining its window.
    /// Read `page_rows` on each key so resizing, text scaling and density changes
    /// affect the next PageUp/PageDown immediately. Values below one mean one row.
    pub fn navigation<W: ComponentHandle + 'static>(
        &self,
        window: &W,
        page_rows: fn(&W) -> i32,
    ) -> impl Fn(ui::NavigationStep) + 'static {
        let model = self.clone();
        let window = window.as_weak();
        move |step| {
            if let Some(window) = window.upgrade() {
                let page = usize::try_from(page_rows(&window).max(1))
                    .expect("positive page rows fit usize");
                model.navigate(step.into(), page);
            }
        }
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
            generation(&window, view_generation(state.generation));
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
        for (index, entry) in state.entries.iter_mut().enumerate() {
            entry.row.select(entry.key == key);
            self.rows.set_row_data(index, entry.row.clone());
        }
        drop(state);
        self.publish();
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
        self.content
            .row_data(0)
            .map(|cell| cell.text.to_string())
            .or_else(|| self.cells.row_data(0).map(|label| label.to_string()))
            .unwrap_or_default()
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

impl KeyedRow for ui::StepEntry {
    fn stamp(&mut self, token: i32, generation: i32) {
        self.token = token;
        self.generation = generation;
    }
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn label(&self) -> String {
        self.label.to_string()
    }
}

impl KeyedRow for ui::ButtonGroupEntry {
    fn stamp(&mut self, token: i32, generation: i32) {
        self.token = token;
        self.generation = generation;
    }
    fn enabled(&self) -> bool {
        !self.disabled && !self.busy
    }
    fn label(&self) -> String {
        self.text.to_string()
    }
}

impl KeyedRow for ui::AvatarEntry {
    fn stamp(&mut self, token: i32, generation: i32) {
        self.token = token;
        self.generation = generation;
    }
    fn enabled(&self) -> bool {
        !self.disabled
    }
    fn label(&self) -> String {
        if self.label.is_empty() {
            self.initials.to_string()
        } else {
            self.label.to_string()
        }
    }
}

#[cfg(test)]
mod identity_tests {
    use super::*;
    #[test]
    fn exhausted_token_allocation_is_an_error_and_does_not_wrap() {
        let counter = AtomicI32::new(i32::MAX - 1);
        assert_eq!(unique(&counter), Ok(i32::MAX - 1));
        assert_eq!(unique(&counter), Err(ModelError::IdentityExhausted));
        assert_eq!(unique(&counter), Err(ModelError::IdentityExhausted));
        assert_eq!(counter.load(Ordering::Relaxed), i32::MAX);
    }
    #[test]
    fn wrapping_a_live_model_rejects_the_previous_generation() {
        let model = KeyedModel::new([(1usize, ui::TextLine::default())]).unwrap();
        model.state.borrow_mut().generation = u64::try_from(i32::MAX).unwrap() - 1;
        model.replace([(1usize, ui::TextLine::default())]).unwrap();
        let token = model.rows.row_data(0).unwrap().token;
        assert_eq!(model.generation(), i32::MAX);
        assert_eq!(model.accept(token, i32::MAX), Some(1));
        model.replace([(1usize, ui::TextLine::default())]).unwrap();
        assert_eq!(model.generation(), 1);
        assert_eq!(model.accept(token, i32::MAX), None);
        assert_eq!(model.accept(token, 1), Some(1));
    }
    #[test]
    fn exhausted_replacement_keeps_records_generation_active_identity_and_publication() {
        let model = KeyedModel::new([(
            1usize,
            ui::ContentItem {
                title: "before".into(),
                ..Default::default()
            },
        )])
        .unwrap();
        let before = model.rows.row_data(0).unwrap();
        model.select(before.token, model.generation()).unwrap();
        let generation = model.generation();
        let publications = Rc::new(std::cell::Cell::new(0));
        let observed = publications.clone();
        model
            .publish
            .borrow_mut()
            .push(Rc::new(move || observed.set(observed.get() + 1)));
        let counter = AtomicI32::new(i32::MAX - 1);
        assert_eq!(
            model.replace_allocating(
                [
                    (
                        1usize,
                        ui::ContentItem {
                            title: "replacement".into(),
                            ..Default::default()
                        }
                    ),
                    (2usize, ui::ContentItem::default()),
                    (3usize, ui::ContentItem::default()),
                ],
                || unique(&counter)
            ),
            Err(ModelError::IdentityExhausted)
        );
        assert_eq!(model.generation(), generation);
        assert_eq!(model.len(), 1);
        assert_eq!(model.active_index(), 0);
        assert_eq!(model.active_row().unwrap().title, "before");
        assert_eq!(model.accept(before.token, generation), Some(1));
        assert_eq!(model.rows.row_data(0).unwrap().title, "before");
        assert_eq!(publications.get(), 0);
        assert_eq!(
            model.insert_allocating(2usize, ui::ContentItem::default(), || unique(&counter)),
            Err(ModelError::IdentityExhausted)
        );
        assert_eq!(model.generation(), generation);
        assert_eq!(model.len(), 1);
        assert_eq!(model.active_row().unwrap().title, "before");
        assert_eq!(publications.get(), 0);
    }

    #[test]
    fn model_generation_tags_have_an_explicit_nonzero_wrap() {
        let last = u64::try_from(i32::MAX).unwrap();
        assert_eq!(view_generation(last), i32::MAX);
        assert_eq!(view_generation(last + 1), 1);
        assert_eq!(next_generation(u64::MAX), 1);
    }
}
