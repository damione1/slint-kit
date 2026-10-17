//! Stable-identity choice models, independently owned by each consumer.
use crate::CheckState;
use crate::RangeThumb;
use crate::{
    DecimalStep, DecimalValue, NavigationStep, NumericBounds, Ownership, ParseIssue, SelectOption,
    StepDirection,
};
use std::{collections::HashSet, hash::Hash};

impl CheckState {
    pub fn toggle(self) -> Self {
        match self {
            Self::Unchecked | Self::Mixed => Self::Checked,
            Self::Checked => Self::Unchecked,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MultiSelectionIntent<K> {
    Add(K),
    Remove(K),
    Clear,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChoiceFailure {
    Missing,
    Disabled,
    LimitReached,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MultiSelectionModelError {
    DuplicateKey,
    LimitExceeded,
}
#[derive(Debug)]
pub struct MultiSelectController<K> {
    options: Vec<SelectOption<K>>,
    selected: Vec<K>,
    ownership: Ownership,
    limit: Option<usize>,
    active: Option<K>,
}
impl<K: Clone + Eq + Hash> MultiSelectController<K> {
    pub fn new(
        options: Vec<SelectOption<K>>,
        selected: Vec<K>,
        ownership: Ownership,
        limit: Option<usize>,
    ) -> Result<Self, MultiSelectionModelError> {
        if limit.is_some_and(|limit| selected.len() > limit) {
            return Err(MultiSelectionModelError::LimitExceeded);
        }
        let mut keys = HashSet::new();
        if options.iter().any(|row| !keys.insert(&row.key)) {
            return Err(MultiSelectionModelError::DuplicateKey);
        }
        let mut seen = HashSet::new();
        if selected.iter().any(|key| !seen.insert(key)) {
            return Err(MultiSelectionModelError::DuplicateKey);
        }
        Ok(Self {
            options,
            selected,
            ownership,
            limit,
            active: None,
        })
    }
    pub fn selected(&self) -> &[K] {
        &self.selected
    }
    pub fn active(&self) -> Option<&K> {
        self.active.as_ref()
    }
    pub fn options(&self) -> &[SelectOption<K>] {
        &self.options
    }
    pub fn set_selected(&mut self, selected: Vec<K>) -> Result<(), MultiSelectionModelError> {
        if self.limit.is_some_and(|limit| selected.len() > limit) {
            return Err(MultiSelectionModelError::LimitExceeded);
        }
        let mut keys = HashSet::new();
        if selected.iter().any(|key| !keys.insert(key)) {
            return Err(MultiSelectionModelError::DuplicateKey);
        }
        self.selected = selected;
        Ok(())
    }
    pub fn set_options(
        &mut self,
        options: Vec<SelectOption<K>>,
    ) -> Result<(), MultiSelectionModelError> {
        let mut keys = HashSet::new();
        if options.iter().any(|row| !keys.insert(&row.key)) {
            return Err(MultiSelectionModelError::DuplicateKey);
        }
        self.options = options;
        if self.active.as_ref().is_some_and(|key| {
            !self
                .options
                .iter()
                .any(|row| &row.key == key && !row.disabled)
        }) {
            self.active = None;
        }
        Ok(())
    }
    pub fn toggle(&mut self, key: &K) -> Result<MultiSelectionIntent<K>, ChoiceFailure> {
        let row = self
            .options
            .iter()
            .find(|row| &row.key == key)
            .ok_or(ChoiceFailure::Missing)?;
        if row.disabled {
            return Err(ChoiceFailure::Disabled);
        }
        let intent = if self.selected.contains(key) {
            MultiSelectionIntent::Remove(key.clone())
        } else {
            if self.limit.is_some_and(|limit| self.selected.len() >= limit) {
                return Err(ChoiceFailure::LimitReached);
            }
            MultiSelectionIntent::Add(key.clone())
        };
        if self.ownership == Ownership::Local {
            match &intent {
                MultiSelectionIntent::Add(key) => self.selected.push(key.clone()),
                MultiSelectionIntent::Remove(key) => self.selected.retain(|value| value != key),
                MultiSelectionIntent::Clear => self.selected.clear(),
            }
        }
        Ok(intent)
    }
    pub fn clear(&mut self) -> MultiSelectionIntent<K> {
        if self.ownership == Ownership::Local {
            self.selected.clear();
        }
        MultiSelectionIntent::Clear
    }
    /// Roving navigation uses stable keys and skips unavailable rows.
    pub fn navigate(&mut self, step: NavigationStep, page: usize) -> Option<&K> {
        let enabled: Vec<&K> = self
            .options
            .iter()
            .filter(|row| !row.disabled)
            .map(|row| &row.key)
            .collect();
        if enabled.is_empty() {
            self.active = None;
            return None;
        }
        let index = self
            .active
            .as_ref()
            .and_then(|key| enabled.iter().position(|row| *row == key));
        let next = match step {
            NavigationStep::First => 0,
            NavigationStep::Last => enabled.len() - 1,
            NavigationStep::Next => index.map_or(0, |index| (index + 1).min(enabled.len() - 1)),
            NavigationStep::Previous => {
                index.map_or(enabled.len() - 1, |index| index.saturating_sub(1))
            }
            NavigationStep::PageNext => index
                .unwrap_or(0)
                .saturating_add(page.max(1))
                .min(enabled.len() - 1),
            NavigationStep::PagePrevious => index
                .unwrap_or(enabled.len() - 1)
                .saturating_sub(page.max(1)),
        };
        self.active = Some(enabled[next].clone());
        self.active.as_ref()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrderedRange<T> {
    start: T,
    end: T,
}
impl<T: Ord + Copy> OrderedRange<T> {
    pub fn new(start: T, end: T) -> Result<Self, ParseIssue> {
        if start > end {
            Err(ParseIssue::InvalidRange)
        } else {
            Ok(Self { start, end })
        }
    }
    pub fn start(self) -> T {
        self.start
    }
    pub fn end(self) -> T {
        self.end
    }
}

#[derive(Debug)]
pub struct RangeController {
    bounds: NumericBounds<DecimalValue>,
    value: OrderedRange<DecimalValue>,
    step: DecimalStep,
    drag_snapshot: Option<OrderedRange<DecimalValue>>,
}
impl RangeController {
    pub fn new(
        bounds: NumericBounds<DecimalValue>,
        value: OrderedRange<DecimalValue>,
        step: DecimalStep,
    ) -> Result<Self, ParseIssue> {
        bounds.validate(value.start)?;
        bounds.validate(value.end)?;
        Ok(Self {
            bounds,
            value,
            step,
            drag_snapshot: None,
        })
    }
    pub fn value(&self) -> OrderedRange<DecimalValue> {
        self.value
    }
    pub fn set(&mut self, thumb: RangeThumb, value: DecimalValue) -> Result<(), ParseIssue> {
        self.bounds.validate(value)?;
        self.value = match thumb {
            RangeThumb::Start => OrderedRange::new(value, self.value.end)?,
            RangeThumb::End => OrderedRange::new(self.value.start, value)?,
        };
        Ok(())
    }
    pub fn step(&mut self, thumb: RangeThumb, direction: StepDirection) -> Result<(), ParseIssue> {
        let current = match thumb {
            RangeThumb::Start => self.value.start,
            RangeThumb::End => self.value.end,
        };
        self.set(thumb, self.step.apply(current, direction, self.bounds)?)
    }
    pub fn begin_drag(&mut self) {
        self.drag_snapshot = Some(self.value);
    }
    pub fn commit_drag(&mut self) {
        self.drag_snapshot = None;
    }
    pub fn cancel_drag(&mut self) {
        if let Some(value) = self.drag_snapshot.take() {
            self.value = value;
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RatingScale {
    steps: u16,
}
impl RatingScale {
    pub fn new(steps: u16) -> Result<Self, ParseIssue> {
        if steps == 0 {
            Err(ParseIssue::InvalidRange)
        } else {
            Ok(Self { steps })
        }
    }
    pub fn score(self, step: u16) -> Result<Rating, ParseIssue> {
        if step > self.steps {
            Err(ParseIssue::AboveMaximum)
        } else {
            Ok(Rating { step, scale: self })
        }
    }
    pub fn steps(self) -> u16 {
        self.steps
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rating {
    step: u16,
    scale: RatingScale,
}
impl Rating {
    pub fn step(self) -> u16 {
        self.step
    }
    pub fn scale(self) -> RatingScale {
        self.scale
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchGeneration {
    owner: u64,
    generation: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SearchState<T, E> {
    Empty,
    Loading(SearchGeneration),
    Ready(Vec<T>),
    Failed(E),
}
#[derive(Debug)]
pub struct SearchController<T, E> {
    owner: u64,
    generation: u64,
    query: String,
    state: SearchState<T, E>,
}
impl<T, E> Default for SearchController<T, E> {
    fn default() -> Self {
        Self {
            owner: crate::field::next_owner(),
            generation: 0,
            query: String::new(),
            state: SearchState::Empty,
        }
    }
}
impl<T, E> SearchController<T, E> {
    pub fn query(&self) -> &str {
        &self.query
    }
    pub fn state(&self) -> &SearchState<T, E> {
        &self.state
    }
    pub fn search(&mut self, query: String) -> Option<SearchGeneration> {
        self.generation = self
            .generation
            .checked_add(1)
            .expect("search generation exhausted");
        self.query = query;
        if self.query.is_empty() {
            self.state = SearchState::Empty;
            return None;
        }
        let ticket = SearchGeneration {
            owner: self.owner,
            generation: self.generation,
        };
        self.state = SearchState::Loading(ticket);
        Some(ticket)
    }
    pub fn complete(&mut self, ticket: SearchGeneration, result: Result<Vec<T>, E>) -> bool {
        if !matches!(self.state,SearchState::Loading(active) if active==ticket) {
            return false;
        }
        self.state = match result {
            Ok(rows) => SearchState::Ready(rows),
            Err(error) => SearchState::Failed(error),
        };
        true
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stable_multi_choices_and_disabled_navigation() {
        let mut disabled = SelectOption::new(2, "two");
        disabled.disabled = true;
        let mut choices = MultiSelectController::new(
            vec![
                SelectOption::new(1, "one"),
                disabled,
                SelectOption::new(3, "three"),
            ],
            vec![],
            Ownership::Local,
            Some(1),
        )
        .unwrap();
        assert_eq!(choices.navigate(NavigationStep::Next, 10), Some(&1));
        assert_eq!(choices.navigate(NavigationStep::Next, 10), Some(&3));
        assert_eq!(choices.toggle(&2), Err(ChoiceFailure::Disabled));
        choices.toggle(&3).unwrap();
        assert_eq!(choices.toggle(&1), Err(ChoiceFailure::LimitReached));
        choices
            .set_options(vec![
                SelectOption::new(3, "new three"),
                SelectOption::new(1, "one"),
            ])
            .unwrap();
        assert_eq!(choices.selected(), &[3]);
    }
    #[test]
    fn acknowledged_selection_respects_limit_atomically() {
        let options = vec![SelectOption::new(1, "one"), SelectOption::new(2, "two")];
        assert!(matches!(
            MultiSelectController::new(options.clone(), vec![1, 2], Ownership::Controlled, Some(1)),
            Err(MultiSelectionModelError::LimitExceeded)
        ));
        let mut controller =
            MultiSelectController::new(options, vec![1], Ownership::Controlled, Some(1)).unwrap();
        assert_eq!(
            controller.set_selected(vec![1, 2]),
            Err(MultiSelectionModelError::LimitExceeded)
        );
        assert_eq!(controller.selected(), &[1]);
    }
    #[test]
    fn range_does_not_cross_and_cancel_restores() {
        let d = DecimalValue::from_integer;
        let mut range = RangeController::new(
            NumericBounds::new(d(0), d(10)).unwrap(),
            OrderedRange::new(d(2), d(8)).unwrap(),
            DecimalStep::new(d(1)).unwrap(),
        )
        .unwrap();
        assert!(range.set(RangeThumb::Start, d(9)).is_err());
        range.begin_drag();
        range.set(RangeThumb::Start, d(4)).unwrap();
        range.cancel_drag();
        assert_eq!(range.value().start(), d(2));
    }
    #[test]
    fn search_generation_is_per_owner() {
        let mut a = SearchController::<u8, ()>::default();
        let mut b = SearchController::<u8, ()>::default();
        let stale = a.search("a".into()).unwrap();
        let ticket = a.search("b".into()).unwrap();
        b.search("b".into());
        assert!(!a.complete(stale, Ok(vec![1])));
        assert!(!b.complete(ticket, Ok(vec![2])));
        assert!(a.complete(ticket, Ok(vec![3])));
    }
}
