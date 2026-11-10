use crate::{DismissReason, FocusHandle, FocusScopeId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OverlayId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OverlayPolicy {
    pub escape: bool,
    pub outside_press: bool,
    pub system_back: bool,
    pub trigger_toggle: bool,
    pub block_while_busy: bool,
}
impl Default for OverlayPolicy {
    fn default() -> Self {
        Self {
            escape: true,
            outside_press: true,
            system_back: true,
            trigger_toggle: true,
            block_while_busy: true,
        }
    }
}
impl OverlayPolicy {
    pub fn allows(self, reason: DismissReason, busy: bool) -> bool {
        match reason {
            DismissReason::Programmatic | DismissReason::HostClosed => true,
            DismissReason::Escape => self.escape && !(busy && self.block_while_busy),
            DismissReason::OutsidePress => self.outside_press && !(busy && self.block_while_busy),
            DismissReason::SystemBack => self.system_back && !(busy && self.block_while_busy),
            DismissReason::TriggerToggle => self.trigger_toggle && !(busy && self.block_while_busy),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlaySpec {
    pub id: OverlayId,
    pub parent: Option<OverlayId>,
    pub modal: bool,
    pub focus_scope: Option<FocusScopeId>,
    pub restore_focus: Option<FocusHandle>,
    pub policy: OverlayPolicy,
}
impl OverlaySpec {
    pub fn new(id: OverlayId, parent: Option<OverlayId>, modal: bool) -> Self {
        Self {
            id,
            parent,
            modal,
            focus_scope: None,
            restore_focus: None,
            policy: OverlayPolicy::default(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CloseResult {
    Empty,
    Blocked,
    /// Descendants first, requested overlay last. Restore focus only from the
    /// final entry, preventing competing intermediate restoration requests.
    Closed(Vec<OverlaySpec>),
}
impl CloseResult {
    pub fn restore_focus(&self) -> Option<FocusHandle> {
        match self {
            Self::Closed(entries) => entries.last().and_then(|e| e.restore_focus),
            Self::Empty | Self::Blocked => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayError {
    DuplicateId,
    MissingParent,
}
impl std::fmt::Display for OverlayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::DuplicateId => "overlay ID is already registered",
            Self::MissingParent => "overlay parent is not registered",
        })
    }
}
impl std::error::Error for OverlayError {}

#[derive(Debug)]
struct Entry {
    spec: OverlaySpec,
    busy: bool,
}
/// A stack belongs to one window. Each input event is offered exactly once to
/// its topmost overlay; blocked dismissal never falls through to a parent.
///
/// Entries stay ordered from bottom (oldest registration, parents before
/// descendants) to top (most recently pushed layer). Structural identity
/// (`id`, `parent`, `modal`, `focus_scope`, `restore_focus`) is fixed when
/// pushed; in-place mutations only update runtime dismissal state through
/// [`Self::set_busy`] and [`Self::set_policy`].
#[derive(Debug, Default)]
pub struct OverlayStack {
    entries: Vec<Entry>,
}
impl OverlayStack {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn get(&self, id: OverlayId) -> Option<&OverlaySpec> {
        self.entries
            .iter()
            .find(|e| e.spec.id == id)
            .map(|e| &e.spec)
    }
    /// Iterates open overlays from bottom (oldest parent) to top (most recent layer).
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &OverlaySpec> + ExactSizeIterator {
        self.entries.iter().map(|e| &e.spec)
    }
    pub fn topmost(&self) -> Option<&OverlaySpec> {
        self.entries.last().map(|e| &e.spec)
    }
    /// Highest open overlay with `modal: true`, ignoring non-modal popovers above it.
    pub fn topmost_modal(&self) -> Option<&OverlaySpec> {
        self.entries
            .iter()
            .rev()
            .find(|e| e.spec.modal)
            .map(|e| &e.spec)
    }
    pub fn contains(&self, id: OverlayId) -> bool {
        self.get(id).is_some()
    }
    pub fn push(&mut self, spec: OverlaySpec) -> Result<(), OverlayError> {
        if self.contains(spec.id) {
            return Err(OverlayError::DuplicateId);
        }
        if spec.parent.is_some_and(|parent| !self.contains(parent)) {
            return Err(OverlayError::MissingParent);
        }
        self.entries.push(Entry { spec, busy: false });
        Ok(())
    }
    pub fn set_busy(&mut self, id: OverlayId, busy: bool) -> bool {
        if let Some(e) = self.entries.iter_mut().find(|e| e.spec.id == id) {
            e.busy = busy;
            true
        } else {
            false
        }
    }
    pub fn is_busy(&self, id: OverlayId) -> bool {
        self.entries
            .iter()
            .find(|e| e.spec.id == id)
            .is_some_and(|e| e.busy)
    }
    pub fn set_policy(&mut self, id: OverlayId, policy: OverlayPolicy) -> bool {
        if let Some(e) = self.entries.iter_mut().find(|e| e.spec.id == id) {
            e.spec.policy = policy;
            true
        } else {
            false
        }
    }
    pub fn request_close(&mut self, reason: DismissReason) -> CloseResult {
        let Some(id) = self.topmost().map(|e| e.id) else {
            return CloseResult::Empty;
        };
        self.close(id, reason)
    }
    pub fn close(&mut self, id: OverlayId, reason: DismissReason) -> CloseResult {
        if !self.contains(id) {
            return CloseResult::Empty;
        }
        // Parent registration precedes its descendants, making a single pass
        // sufficient to collect and validate the entire closing subtree before
        // mutating the stack.
        let mut ids = vec![id];
        for entry in &self.entries {
            let in_subtree = if entry.spec.id == id {
                true
            } else if entry.spec.parent.is_some_and(|p| ids.contains(&p)) {
                ids.push(entry.spec.id);
                true
            } else {
                false
            };
            if in_subtree && !entry.spec.policy.allows(reason, entry.busy) {
                return CloseResult::Blocked;
            }
        }
        let mut closed = vec![];
        let mut index = self.entries.len();
        while index > 0 {
            index -= 1;
            if ids.contains(&self.entries[index].spec.id) {
                closed.push(self.entries.remove(index).spec);
            }
        }
        CloseResult::Closed(closed)
    }
    pub fn host_closed(&mut self) -> CloseResult {
        if self.entries.is_empty() {
            return CloseResult::Empty;
        }
        CloseResult::Closed(self.entries.drain(..).rev().map(|e| e.spec).collect())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_escape_closes_one_level_and_restores_once() {
        let mut stack = OverlayStack::new();
        let mut parent = OverlaySpec::new(OverlayId(1), None, true);
        parent.restore_focus = Some(FocusHandle {
            id: crate::FocusId(1),
            generation: 1,
        });
        let mut child = OverlaySpec::new(OverlayId(2), Some(parent.id), false);
        child.restore_focus = Some(FocusHandle {
            id: crate::FocusId(2),
            generation: 2,
        });
        stack.push(parent.clone()).unwrap();
        stack.push(child.clone()).unwrap();
        let result = stack.request_close(DismissReason::Escape);
        assert_eq!(result, CloseResult::Closed(vec![child.clone()]));
        assert_eq!(result.restore_focus(), child.restore_focus);
        assert_eq!(stack.topmost(), Some(&parent));
        assert_eq!(
            stack.request_close(DismissReason::Escape),
            CloseResult::Closed(vec![parent])
        );
        assert_eq!(
            stack.request_close(DismissReason::Escape),
            CloseResult::Empty
        );
    }
    #[test]
    fn async_busy_and_reason_policy_consume_event_without_fallthrough() {
        let mut stack = OverlayStack::new();
        stack
            .push(OverlaySpec::new(OverlayId(1), None, true))
            .unwrap();
        let mut child = OverlaySpec::new(OverlayId(2), Some(OverlayId(1)), false);
        child.policy.outside_press = false;
        stack.push(child).unwrap();
        assert_eq!(
            stack.request_close(DismissReason::OutsidePress),
            CloseResult::Blocked
        );
        assert_eq!(stack.len(), 2);
        stack.set_busy(OverlayId(2), true);
        assert_eq!(
            stack.request_close(DismissReason::Escape),
            CloseResult::Blocked
        );
        assert!(matches!(
            stack.request_close(DismissReason::Programmatic),
            CloseResult::Closed(_)
        ));
        assert_eq!(stack.len(), 1);
    }
    #[test]
    fn programmatic_parent_close_cleans_descendants_and_is_idempotent() {
        let mut stack = OverlayStack::new();
        for i in 1..=3 {
            stack
                .push(OverlaySpec::new(
                    OverlayId(i),
                    if i == 1 { None } else { Some(OverlayId(i - 1)) },
                    true,
                ))
                .unwrap();
            stack.set_busy(OverlayId(i), true);
        }
        let CloseResult::Closed(closed) = stack.close(OverlayId(1), DismissReason::Programmatic)
        else {
            panic!("expected close");
        };
        assert_eq!(closed.iter().map(|e| e.id.0).collect::<Vec<_>>(), [3, 2, 1]);
        assert!(stack.is_empty());
        assert_eq!(
            stack.close(OverlayId(1), DismissReason::Programmatic),
            CloseResult::Empty
        );
        for _ in 0..100 {
            stack
                .push(OverlaySpec::new(OverlayId(1), None, true))
                .unwrap();
            stack.host_closed();
            assert!(stack.is_empty());
        }
    }
    #[test]
    fn parent_close_is_blocked_atomically_when_descendant_is_busy_or_forbids_reason() {
        let mut stack = OverlayStack::new();
        let mut root = OverlaySpec::new(OverlayId(1), None, true);
        root.restore_focus = Some(FocusHandle {
            id: crate::FocusId(10),
            generation: 1,
        });
        let mut child = OverlaySpec::new(OverlayId(2), Some(root.id), false);
        child.restore_focus = Some(FocusHandle {
            id: crate::FocusId(20),
            generation: 2,
        });
        let mut grandchild = OverlaySpec::new(OverlayId(3), Some(child.id), true);
        grandchild.restore_focus = Some(FocusHandle {
            id: crate::FocusId(30),
            generation: 3,
        });
        grandchild.policy.outside_press = false;
        let mut sibling = OverlaySpec::new(OverlayId(4), None, false);
        sibling.policy.escape = false;
        stack.push(root.clone()).unwrap();
        stack.push(child).unwrap();
        stack.push(grandchild).unwrap();
        stack.push(sibling).unwrap();

        assert_eq!(
            stack.close(OverlayId(1), DismissReason::OutsidePress),
            CloseResult::Blocked
        );
        assert_eq!(
            stack.iter().map(|s| s.id.0).collect::<Vec<_>>(),
            [1, 2, 3, 4],
            "blocked descendant prevents partial parent/child removal"
        );

        assert!(!stack.set_policy(OverlayId(99), OverlayPolicy::default()));
        assert!(stack.set_policy(OverlayId(3), OverlayPolicy::default()));
        stack.set_busy(OverlayId(3), true);
        assert_eq!(
            stack.close(OverlayId(1), DismissReason::Escape),
            CloseResult::Blocked
        );
        assert_eq!(
            stack.close(OverlayId(2), DismissReason::Escape),
            CloseResult::Blocked
        );
        assert_eq!(
            stack.iter().map(|s| s.id.0).collect::<Vec<_>>(),
            [1, 2, 3, 4]
        );
        assert!(stack.is_busy(OverlayId(3)));

        // Unrelated sibling branch is both busy and policy-blocked for Escape,
        // yet closing root (1) succeeds once its own subtree allows Escape.
        stack.set_busy(OverlayId(3), false);
        stack.set_busy(OverlayId(4), true);
        let result = stack.close(OverlayId(1), DismissReason::Escape);
        let CloseResult::Closed(closed) = &result else {
            panic!("expected root subtree to close");
        };
        assert_eq!(closed.iter().map(|s| s.id.0).collect::<Vec<_>>(), [3, 2, 1]);
        assert_eq!(result.restore_focus(), root.restore_focus);
        assert_eq!(stack.iter().map(|s| s.id.0).collect::<Vec<_>>(), [4]);
        assert!(stack.is_busy(OverlayId(4)));
    }
    #[test]
    fn queries_get_iter_topmost_modal_and_policy_mutation_preserve_order() {
        let mut stack = OverlayStack::new();
        assert_eq!(stack.topmost_modal(), None);
        let mut dialog = OverlaySpec::new(OverlayId(1), None, true);
        dialog.focus_scope = Some(FocusScopeId(1));
        let popover = OverlaySpec::new(OverlayId(2), Some(dialog.id), false);
        let mut nested_modal = OverlaySpec::new(OverlayId(3), Some(popover.id), true);
        nested_modal.focus_scope = Some(FocusScopeId(2));
        let tooltip = OverlaySpec::new(OverlayId(4), Some(nested_modal.id), false);
        stack.push(dialog.clone()).unwrap();
        stack.push(popover.clone()).unwrap();
        stack.push(nested_modal.clone()).unwrap();
        stack.push(tooltip.clone()).unwrap();

        assert_eq!(stack.get(OverlayId(2)), Some(&popover));
        assert_eq!(stack.get(OverlayId(99)), None);
        assert_eq!(stack.iter().len(), 4);
        assert_eq!(
            stack.iter().map(|s| s.id.0).collect::<Vec<_>>(),
            [1, 2, 3, 4]
        );
        assert_eq!(
            stack.iter().rev().map(|s| s.id.0).collect::<Vec<_>>(),
            [4, 3, 2, 1]
        );
        assert_eq!(stack.topmost(), Some(&tooltip));
        assert_eq!(stack.topmost_modal(), Some(&nested_modal));

        let CloseResult::Closed(closed) = stack.close(OverlayId(3), DismissReason::Escape) else {
            panic!("expected nested modal subtree to close");
        };
        assert_eq!(closed.iter().map(|s| s.id.0).collect::<Vec<_>>(), [4, 3]);
        assert_eq!(stack.topmost(), Some(&popover));
        assert_eq!(stack.topmost_modal(), Some(&dialog));
        assert_eq!(
            stack.topmost_modal().and_then(|s| s.focus_scope),
            Some(FocusScopeId(1))
        );

        let locked = OverlayPolicy {
            escape: false,
            ..OverlayPolicy::default()
        };
        assert!(stack.set_policy(OverlayId(1), locked));
        assert_eq!(stack.get(OverlayId(1)).unwrap().policy, locked);
        assert_eq!(stack.iter().map(|s| s.id.0).collect::<Vec<_>>(), [1, 2]);
        assert_eq!(
            stack.close(OverlayId(1), DismissReason::Escape),
            CloseResult::Blocked
        );
        assert!(matches!(
            stack.close(OverlayId(1), DismissReason::Programmatic),
            CloseResult::Closed(_)
        ));
        assert_eq!(stack.topmost_modal(), None);
    }
}
