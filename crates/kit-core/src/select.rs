use crate::{DismissReason, NavigationStep};
use std::collections::HashSet;
use std::hash::Hash;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectOption<K> {
    pub key: K,
    pub label: String,
    pub description: String,
    pub disabled: bool,
}

impl<K> SelectOption<K> {
    pub fn new(key: K, label: impl Into<String>) -> Self {
        Self {
            key,
            label: label.into(),
            description: String::new(),
            disabled: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectState<K> {
    pub selected: Option<K>,
    pub active: Option<K>,
    pub open: bool,
}

/// Local commits update the snapshot immediately. Controlled commits only emit
/// an intent; the application acknowledges them through `set_selected`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ownership {
    Local,
    Controlled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemovalPolicy {
    Clear,
    PreserveUnavailable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectionIntent<K> {
    Commit(K),
    Clear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowToken {
    pub generation: u64,
    pub index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelError {
    DuplicateKey,
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateKey => f.write_str("select option keys must be unique"),
        }
    }
}
impl std::error::Error for ModelError {}

/// Unicode lowercase prefix matching, accent-sensitive and without canonical
/// normalization. Text input/IME composition must be handled by the host.
#[derive(Debug)]
pub struct SelectController<K> {
    options: Vec<SelectOption<K>>,
    state: SelectState<K>,
    ownership: Ownership,
    removal_policy: RemovalPolicy,
    generation: u64,
    pressed: Option<RowToken>,
    typeahead: String,
    last_typeahead_ms: Option<u64>,
    typeahead_timeout_ms: u64,
}

impl<K: Clone + Eq + Hash> SelectController<K> {
    pub fn new(options: Vec<SelectOption<K>>, selected: Option<K>, ownership: Ownership) -> Self {
        Self::try_new(options, selected, ownership).expect("option keys must be unique")
    }

    pub fn try_new(
        options: Vec<SelectOption<K>>,
        selected: Option<K>,
        ownership: Ownership,
    ) -> Result<Self, ModelError> {
        Self::validate(&options)?;
        let mut result = Self {
            options,
            state: SelectState {
                selected,
                active: None,
                open: false,
            },
            ownership,
            removal_policy: RemovalPolicy::Clear,
            generation: 1,
            pressed: None,
            typeahead: String::new(),
            last_typeahead_ms: None,
            typeahead_timeout_ms: 700,
        };
        result.reconcile_selection();
        Ok(result)
    }

    fn validate(options: &[SelectOption<K>]) -> Result<(), ModelError> {
        let mut keys = HashSet::with_capacity(options.len());
        if options.iter().any(|row| !keys.insert(&row.key)) {
            return Err(ModelError::DuplicateKey);
        }
        Ok(())
    }

    pub fn state(&self) -> &SelectState<K> {
        &self.state
    }
    pub fn options(&self) -> &[SelectOption<K>] {
        &self.options
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn ownership(&self) -> Ownership {
        self.ownership
    }
    pub fn set_removal_policy(&mut self, policy: RemovalPolicy) {
        self.removal_policy = policy;
        self.reconcile_selection();
    }
    pub fn set_typeahead_timeout_ms(&mut self, timeout: u64) {
        self.typeahead_timeout_ms = timeout;
    }

    /// Acknowledges application state, including in controlled mode.
    pub fn set_selected(&mut self, selected: Option<K>) {
        self.state.selected = selected;
        self.reconcile_selection();
    }

    fn reconcile_selection(&mut self) {
        if self.removal_policy == RemovalPolicy::Clear
            && self
                .state
                .selected
                .as_ref()
                .is_some_and(|key| !self.options.iter().any(|o| &o.key == key))
        {
            self.state.selected = None;
        }
    }

    /// Validates the complete replacement before touching state. A model change
    /// invalidates every outstanding row gesture, even if its index still exists.
    pub fn reconcile(&mut self, options: Vec<SelectOption<K>>) -> Result<(), ModelError> {
        Self::validate(&options)?;
        self.options = options;
        self.generation = self
            .generation
            .checked_add(1)
            .expect("row generation exhausted");
        self.pressed = None;
        self.reconcile_selection();
        if self.state.open
            && self
                .state
                .active
                .as_ref()
                .is_none_or(|key| !self.eligible(key))
        {
            self.state.active = self
                .state
                .selected
                .as_ref()
                .filter(|key| self.eligible(key))
                .cloned()
                .or_else(|| self.first_key());
        }
        self.reset_typeahead();
        Ok(())
    }

    pub fn open(&mut self) {
        if self.state.open {
            return;
        }
        self.state.open = true;
        self.state.active = self
            .state
            .selected
            .as_ref()
            .filter(|key| self.eligible(key))
            .cloned()
            .or_else(|| self.first_key());
        self.reset_typeahead();
    }

    pub fn dismiss(&mut self, _reason: DismissReason) {
        self.state.open = false;
        self.state.active = None;
        self.pressed = None;
        self.reset_typeahead();
    }

    fn eligible(&self, key: &K) -> bool {
        self.options.iter().any(|o| &o.key == key && !o.disabled)
    }
    fn first_key(&self) -> Option<K> {
        self.options
            .iter()
            .find(|o| !o.disabled)
            .map(|o| o.key.clone())
    }

    /// Navigation clamps at edges. Page movement counts eligible options, so
    /// disabled runs cannot strand the active item.
    pub fn navigate(&mut self, step: NavigationStep, page_size: usize) {
        if !self.state.open {
            return;
        }
        let available: Vec<usize> = self
            .options
            .iter()
            .enumerate()
            .filter_map(|(i, o)| (!o.disabled).then_some(i))
            .collect();
        if available.is_empty() {
            self.state.active = None;
            return;
        }
        let current = self
            .state
            .active
            .as_ref()
            .and_then(|key| available.iter().position(|&i| &self.options[i].key == key));
        let last = available.len() - 1;
        let next = match step {
            NavigationStep::First => 0,
            NavigationStep::Last => last,
            NavigationStep::Next => current.map_or(0, |i| i.saturating_add(1).min(last)),
            NavigationStep::Previous => current.map_or(last, |i| i.saturating_sub(1)),
            NavigationStep::PageNext => {
                current.map_or(0, |i| i.saturating_add(page_size.max(1)).min(last))
            }
            NavigationStep::PagePrevious => {
                current.map_or(last, |i| i.saturating_sub(page_size.max(1)))
            }
        };
        self.state.active = Some(self.options[available[next]].key.clone());
    }

    pub fn activate(&mut self, key: &K) -> Option<SelectionIntent<K>> {
        if !self.state.open || !self.eligible(key) {
            return None;
        }
        let key = key.clone();
        if self.ownership == Ownership::Local {
            self.state.selected = Some(key.clone());
        }
        self.dismiss(DismissReason::Programmatic);
        Some(SelectionIntent::Commit(key))
    }

    pub fn commit(&mut self) -> Option<SelectionIntent<K>> {
        let key = self.state.active.clone()?;
        self.activate(&key)
    }
    pub fn clear(&mut self) -> SelectionIntent<K> {
        if self.ownership == Ownership::Local {
            self.state.selected = None;
        }
        self.dismiss(DismissReason::Programmatic);
        SelectionIntent::Clear
    }

    pub fn row_token(&self, index: usize) -> Option<RowToken> {
        (index < self.options.len()).then_some(RowToken {
            generation: self.generation,
            index,
        })
    }
    pub fn key_for_token(&self, token: RowToken) -> Option<&K> {
        (token.generation == self.generation)
            .then(|| self.options.get(token.index))
            .flatten()
            .map(|o| &o.key)
    }
    pub fn activate_token(&mut self, token: RowToken) -> Option<SelectionIntent<K>> {
        let key = self.key_for_token(token)?.clone();
        self.activate(&key)
    }
    pub fn press_row(&mut self, token: RowToken) -> bool {
        self.pressed = None;
        if self.state.open
            && self
                .key_for_token(token)
                .is_some_and(|key| self.eligible(key))
        {
            self.pressed = Some(token);
            true
        } else {
            false
        }
    }
    pub fn cancel_press(&mut self) {
        self.pressed = None;
    }
    pub fn release_row(&mut self, token: RowToken) -> Option<SelectionIntent<K>> {
        let pressed = self.pressed.take()?;
        if pressed != token {
            return None;
        }
        self.activate_token(token)
    }

    fn reset_typeahead(&mut self) {
        self.typeahead.clear();
        self.last_typeahead_ms = None;
    }

    /// Inject monotonic milliseconds. A backwards clock starts a fresh search.
    /// A repeated single character cycles matching labels, wrapping at the end.
    pub fn typeahead(&mut self, text: &str, now_ms: u64) {
        if !self.state.open || text.is_empty() || text.chars().any(char::is_control) {
            return;
        }
        let text = text.to_lowercase();
        let expired = self
            .last_typeahead_ms
            .is_none_or(|last| now_ms < last || now_ms - last >= self.typeahead_timeout_ms);
        if expired {
            self.typeahead.clear();
        }
        let repeat = !expired && self.typeahead.chars().count() == 1 && self.typeahead == text;
        if !repeat {
            self.typeahead.push_str(&text);
        }
        self.last_typeahead_ms = Some(now_ms);
        let len = self.options.len();
        if len == 0 {
            return;
        }
        let current = self
            .state
            .active
            .as_ref()
            .and_then(|key| self.options.iter().position(|o| &o.key == key));
        let start = if repeat || expired {
            current.map_or(0, |i| (i + 1) % len)
        } else {
            current.unwrap_or(0)
        };
        for offset in 0..len {
            let row = &self.options[(start + offset) % len];
            if !row.disabled && row.label.to_lowercase().starts_with(&self.typeahead) {
                self.state.active = Some(row.key.clone());
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn controller(ownership: Ownership) -> SelectController<u32> {
        let mut options = vec![
            SelectOption::new(1, "Apple"),
            SelectOption::new(2, "Apricot"),
            SelectOption::new(3, "Banana"),
        ];
        options[1].disabled = true;
        SelectController::new(options, Some(1), ownership)
    }
    #[test]
    fn selection_is_committed_explicitly_and_controlled_state_is_owned_by_host() {
        for mode in [Ownership::Local, Ownership::Controlled] {
            let mut c = controller(mode);
            c.open();
            c.navigate(NavigationStep::Next, 10);
            assert_eq!(c.state().active, Some(3));
            assert_eq!(c.state().selected, Some(1));
            assert_eq!(c.commit(), Some(SelectionIntent::Commit(3)));
            assert_eq!(
                c.state().selected,
                Some(if mode == Ownership::Local { 3 } else { 1 })
            );
            assert!(c.commit().is_none());
            assert!(!c.state().open);
        }
    }
    #[test]
    fn generation_prevents_recycled_row_activation() {
        let mut c = controller(Ownership::Local);
        c.open();
        let token = c.row_token(0).unwrap();
        assert!(c.press_row(token));
        let mut options = c.options().to_vec();
        options.reverse();
        c.reconcile(options).unwrap();
        assert!(c.release_row(token).is_none());
        assert!(c.activate_token(token).is_none());
        assert_eq!(c.state().selected, Some(1));
    }
    #[test]
    fn reorder_and_removal_preserve_identity() {
        let mut c = controller(Ownership::Local);
        c.open();
        let mut options = c.options().to_vec();
        options.reverse();
        c.reconcile(options).unwrap();
        assert_eq!(c.state().selected, Some(1));
        assert_eq!(c.state().active, Some(1));
        c.set_removal_policy(RemovalPolicy::PreserveUnavailable);
        c.reconcile(vec![]).unwrap();
        assert_eq!(c.state().selected, Some(1));
        assert_eq!(c.state().active, None);
        c.set_removal_policy(RemovalPolicy::Clear);
        assert_eq!(c.state().selected, None);
    }
    #[test]
    fn open_empty_model_can_receive_eligible_options_later() {
        let mut c = SelectController::new(vec![], None, Ownership::Local);
        c.open();
        c.reconcile(vec![SelectOption::new(1, "Loaded")]).unwrap();
        assert_eq!(c.state().active, Some(1));
        assert_eq!(c.state().selected, None);
        let mut disabled = SelectOption::new(2, "Unavailable");
        disabled.disabled = true;
        c.reconcile(vec![disabled]).unwrap();
        assert_eq!(c.state().active, None);
        c.reconcile(vec![SelectOption::new(3, "Reloaded")]).unwrap();
        assert_eq!(c.state().active, Some(3));
    }

    #[test]
    fn timed_unicode_typeahead_cycles_and_resets() {
        let mut c = SelectController::new(
            vec![
                SelectOption::new(1, "Érable"),
                SelectOption::new(2, "Été"),
                SelectOption::new(3, "Étoile"),
            ],
            None,
            Ownership::Local,
        );
        c.open();
        c.typeahead("É", 0);
        assert_eq!(c.state().active, Some(2));
        c.typeahead("é", 100);
        assert_eq!(c.state().active, Some(3));
        c.typeahead("é", 200);
        assert_eq!(c.state().active, Some(1));
        c.typeahead("é", 1000);
        c.typeahead("t", 1100);
        c.typeahead("o", 1200);
        assert_eq!(c.state().active, Some(3));
        assert_eq!(c.state().selected, None);
    }
    #[test]
    fn ten_thousand_rows_and_many_mutations_keep_invariants() {
        let mut c = SelectController::new(
            (0..10_000)
                .map(|i| {
                    let mut row = SelectOption::new(i, format!("Option {i:05}"));
                    row.disabled = i % 7 == 0;
                    row
                })
                .collect(),
            None,
            Ownership::Local,
        );
        c.open();
        c.navigate(NavigationStep::Last, 25);
        assert_eq!(c.state().active, Some(9999));
        c.navigate(NavigationStep::PagePrevious, 25);
        assert!(c.state().active.unwrap() < 9999);
        c.typeahead("Option 05000", 0);
        assert_eq!(c.state().active, Some(5000));
        let mut seed = 17u64;
        for i in 0..1200 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let steps = [
                NavigationStep::Next,
                NavigationStep::Previous,
                NavigationStep::First,
                NavigationStep::Last,
                NavigationStep::PageNext,
                NavigationStep::PagePrevious,
            ];
            c.navigate(steps[(seed % 6) as usize], 17);
            assert!(c.state().active.as_ref().is_none_or(|key| c.eligible(key)));
            if i % 97 == 0 {
                let selected = c.state().selected;
                let mut rows = c.options().to_vec();
                rows.reverse();
                c.reconcile(rows).unwrap();
                assert_eq!(c.state().selected, selected);
            }
        }
    }
    #[test]
    fn empty_disabled_duplicate_and_canceled_press_are_safe() {
        let mut c = controller(Ownership::Local);
        c.open();
        let disabled = c.row_token(1).unwrap();
        assert!(!c.press_row(disabled));
        assert!(c.activate_token(disabled).is_none());
        let token = c.row_token(0).unwrap();
        c.press_row(token);
        c.cancel_press();
        assert!(c.release_row(token).is_none());
        let generation = c.generation();
        assert_eq!(
            c.reconcile(vec![SelectOption::new(9, "a"), SelectOption::new(9, "b")]),
            Err(ModelError::DuplicateKey)
        );
        assert_eq!(generation, c.generation());
        c.reconcile(vec![]).unwrap();
        for step in [
            NavigationStep::Next,
            NavigationStep::Last,
            NavigationStep::PagePrevious,
        ] {
            c.navigate(step, 0);
            assert_eq!(c.state().active, None);
        }
        assert!(c.commit().is_none());
    }
}
