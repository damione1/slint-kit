use std::collections::{BTreeMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FocusId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FocusScopeId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FocusHandle {
    pub id: FocusId,
    pub generation: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusRequest {
    pub target: FocusHandle,
    pub serial: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusDirection {
    Forward,
    Backward,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusError {
    MissingScope,
    ScopeCycle,
    RootScope,
    ScopeInUse,
}

impl std::fmt::Display for FocusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::MissingScope => "focus scope is not registered",
            Self::ScopeCycle => "focus scope parent would create a cycle",
            Self::RootScope => "the root focus scope cannot be replaced or removed",
            Self::ScopeInUse => "focus scope still contains targets or child scopes",
        })
    }
}
impl std::error::Error for FocusError {}

#[derive(Debug)]
struct Target {
    handle: FocusHandle,
    scope: FocusScopeId,
    order: i64,
    eligible: bool,
}

/// Window-local logical focus registry. Handles identify a particular mounting,
/// not merely a numeric ID. Hosts must project requests to real focus targets.
#[derive(Debug)]
pub struct FocusRegistry {
    scopes: BTreeMap<FocusScopeId, Option<FocusScopeId>>,
    targets: BTreeMap<FocusId, Target>,
    generation: u64,
    request_serial: u64,
    traps: Vec<FocusScopeId>,
}

impl Default for FocusRegistry {
    fn default() -> Self {
        Self::new()
    }
}
impl FocusRegistry {
    pub fn new() -> Self {
        let mut scopes = BTreeMap::new();
        scopes.insert(FocusScopeId(0), None);
        Self {
            scopes,
            targets: BTreeMap::new(),
            generation: 0,
            request_serial: 0,
            traps: vec![],
        }
    }
    pub fn register_scope(
        &mut self,
        id: FocusScopeId,
        parent: FocusScopeId,
    ) -> Result<(), FocusError> {
        if id == FocusScopeId(0) {
            return Err(FocusError::RootScope);
        }
        if !self.scopes.contains_key(&parent) {
            return Err(FocusError::MissingScope);
        }
        if self.in_scope(parent, id) {
            return Err(FocusError::ScopeCycle);
        }
        self.scopes.insert(id, Some(parent));
        Ok(())
    }
    pub fn unregister_scope(&mut self, id: FocusScopeId) -> Result<(), FocusError> {
        if id == FocusScopeId(0) {
            return Err(FocusError::RootScope);
        }
        if self.targets.values().any(|t| t.scope == id)
            || self.scopes.values().any(|parent| *parent == Some(id))
        {
            return Err(FocusError::ScopeInUse);
        }
        self.scopes.remove(&id);
        self.traps.retain(|scope| *scope != id);
        Ok(())
    }
    pub fn register(
        &mut self,
        id: FocusId,
        scope: FocusScopeId,
        order: i64,
        eligible: bool,
    ) -> FocusHandle {
        self.try_register(id, scope, order, eligible)
            .expect("focus scope must be registered")
    }
    pub fn try_register(
        &mut self,
        id: FocusId,
        scope: FocusScopeId,
        order: i64,
        eligible: bool,
    ) -> Result<FocusHandle, FocusError> {
        if !self.scopes.contains_key(&scope) {
            return Err(FocusError::MissingScope);
        }
        self.generation = self
            .generation
            .checked_add(1)
            .expect("focus generation exhausted");
        let handle = FocusHandle {
            id,
            generation: self.generation,
        };
        self.targets.insert(
            id,
            Target {
                handle,
                scope,
                order,
                eligible,
            },
        );
        Ok(handle)
    }
    pub fn unregister(&mut self, handle: FocusHandle) -> bool {
        if self
            .targets
            .get(&handle.id)
            .is_some_and(|t| t.handle == handle)
        {
            self.targets.remove(&handle.id);
            true
        } else {
            false
        }
    }
    pub fn set_eligible(&mut self, handle: FocusHandle, eligible: bool) -> bool {
        if let Some(target) = self
            .targets
            .get_mut(&handle.id)
            .filter(|t| t.handle == handle)
        {
            target.eligible = eligible;
            true
        } else {
            false
        }
    }
    pub fn set_order(&mut self, handle: FocusHandle, order: i64) -> bool {
        if let Some(target) = self
            .targets
            .get_mut(&handle.id)
            .filter(|t| t.handle == handle)
        {
            target.order = order;
            true
        } else {
            false
        }
    }
    pub fn is_eligible(&self, handle: FocusHandle) -> bool {
        self.targets
            .get(&handle.id)
            .is_some_and(|t| t.handle == handle && t.eligible)
    }
    pub fn len(&self) -> usize {
        self.targets.len()
    }
    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }
    pub fn push_trap(&mut self, scope: FocusScopeId) -> Result<(), FocusError> {
        if !self.scopes.contains_key(&scope) {
            return Err(FocusError::MissingScope);
        }
        self.traps.retain(|s| *s != scope);
        self.traps.push(scope);
        Ok(())
    }
    pub fn pop_trap(&mut self, scope: FocusScopeId) {
        self.traps.retain(|s| *s != scope);
    }
    pub fn active_trap(&self) -> Option<FocusScopeId> {
        // A parent mounted after its child must not loosen the deeper trap.
        // For unrelated scopes the most recently activated trap wins.
        self.traps
            .iter()
            .copied()
            .fold(None, |active, candidate| match active {
                Some(scope) if self.in_scope(scope, candidate) => Some(scope),
                Some(_) | None => Some(candidate),
            })
    }
    fn in_scope(&self, mut child: FocusScopeId, ancestor: FocusScopeId) -> bool {
        let mut visited = HashSet::new();
        while visited.insert(child) {
            if child == ancestor {
                return true;
            }
            let Some(Some(parent)) = self.scopes.get(&child) else {
                return false;
            };
            child = *parent;
        }
        false
    }
    pub fn request(&mut self, handle: FocusHandle) -> Option<FocusRequest> {
        let target = self.targets.get(&handle.id)?;
        if target.handle != handle
            || !target.eligible
            || self
                .active_trap()
                .is_some_and(|scope| !self.in_scope(target.scope, scope))
        {
            return None;
        }
        self.request_serial = self
            .request_serial
            .checked_add(1)
            .expect("focus request serial exhausted");
        Some(FocusRequest {
            target: handle,
            serial: self.request_serial,
        })
    }
    pub fn eligible_in(&self, scope: FocusScopeId) -> Vec<FocusHandle> {
        let mut targets: Vec<_> = self
            .targets
            .values()
            .filter(|t| t.eligible && self.in_scope(t.scope, scope))
            .collect();
        targets.sort_by_key(|t| (t.order, t.handle.id));
        targets.into_iter().map(|t| t.handle).collect()
    }
    /// The innermost active trap determines the cycle, including custom targets
    /// registered in descendant scopes. An empty scope produces no fake focus.
    pub fn cycle(
        &mut self,
        scope: FocusScopeId,
        current: Option<FocusHandle>,
        direction: FocusDirection,
    ) -> Option<FocusRequest> {
        let scope = self.active_trap().unwrap_or(scope);
        let targets = self.eligible_in(scope);
        if targets.is_empty() {
            return None;
        }
        let position = current.and_then(|h| targets.iter().position(|target| *target == h));
        let index = match (position, direction) {
            (Some(0), FocusDirection::Backward) | (None, FocusDirection::Backward) => {
                targets.len() - 1
            }
            (Some(i), FocusDirection::Backward) => i - 1,
            (Some(i), FocusDirection::Forward) => (i + 1) % targets.len(),
            (None, FocusDirection::Forward) => 0,
        };
        self.request(targets[index])
    }
    /// Call after unmounting the closing trap. The host can retry when mounting
    /// completes; stale registrations never resolve to a reused numeric ID.
    pub fn restore(
        &mut self,
        preferred: &[FocusHandle],
        fallback_scope: FocusScopeId,
    ) -> Option<FocusRequest> {
        for handle in preferred {
            if let Some(request) = self.request(*handle) {
                return Some(request);
            }
        }
        let scope = self.active_trap().unwrap_or(fallback_scope);
        for handle in self.eligible_in(scope) {
            if let Some(request) = self.request(handle) {
                return Some(request);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dynamic_traps_cycle_custom_targets_and_reject_background() {
        let mut r = FocusRegistry::new();
        let root = FocusScopeId(0);
        let dialog = FocusScopeId(1);
        let popup = FocusScopeId(2);
        r.register_scope(dialog, root).unwrap();
        r.register_scope(popup, dialog).unwrap();
        let background = r.register(FocusId(1), root, 0, true);
        let a = r.register(FocusId(2), dialog, 0, true);
        let b = r.register(FocusId(3), dialog, 1, true);
        let custom = r.register(FocusId(4), popup, 0, true);
        r.push_trap(dialog).unwrap();
        assert!(r.request(background).is_none());
        assert_eq!(
            r.cycle(root, Some(a), FocusDirection::Forward)
                .unwrap()
                .target,
            custom
        );
        r.set_eligible(custom, false);
        assert_eq!(
            r.cycle(root, Some(a), FocusDirection::Forward)
                .unwrap()
                .target,
            b
        );
        assert_eq!(
            r.cycle(root, Some(a), FocusDirection::Backward)
                .unwrap()
                .target,
            b
        );
        r.push_trap(popup).unwrap();
        assert!(r.cycle(root, None, FocusDirection::Forward).is_none());
        r.set_eligible(custom, true);
        assert_eq!(
            r.cycle(root, None, FocusDirection::Forward).unwrap().target,
            custom
        );
        assert!(r.request(a).is_none());
        r.pop_trap(popup);
        assert_eq!(r.restore(&[a], dialog).unwrap().target, a);
    }
    #[test]
    fn stale_mount_cannot_unregister_or_restore_reused_id() {
        let mut r = FocusRegistry::new();
        let root = FocusScopeId(0);
        let old = r.register(FocusId(1), root, 0, true);
        r.unregister(old);
        let new = r.register(FocusId(1), root, 1, true);
        let fallback = r.register(FocusId(2), root, 0, true);
        assert!(!r.unregister(old));
        assert!(!r.set_eligible(old, false));
        assert!(r.request(old).is_none());
        assert_eq!(r.restore(&[old], root).unwrap().target, fallback);
        let one = r.request(new).unwrap();
        let two = r.request(new).unwrap();
        assert!(two.serial > one.serial);
    }
    #[test]
    fn deeper_trap_survives_parent_activation_and_dynamic_removal() {
        let mut registry = FocusRegistry::new();
        let root = FocusScopeId(0);
        let parent = FocusScopeId(1);
        let child = FocusScopeId(2);
        registry.register_scope(parent, root).unwrap();
        registry.register_scope(child, parent).unwrap();
        let parent_target = registry.register(FocusId(1), parent, 0, true);
        let old_child = registry.register(FocusId(2), child, 0, true);
        registry.push_trap(child).unwrap();
        registry.push_trap(parent).unwrap();
        assert_eq!(registry.active_trap(), Some(child));
        assert!(registry.request(parent_target).is_none());
        assert_eq!(
            registry.unregister_scope(child),
            Err(FocusError::ScopeInUse)
        );
        registry.unregister(old_child);
        assert!(
            registry
                .cycle(root, Some(old_child), FocusDirection::Forward)
                .is_none()
        );
        let new_child = registry.register(FocusId(2), child, 0, true);
        assert_eq!(
            registry.restore(&[old_child], root).unwrap().target,
            new_child
        );
        registry.set_eligible(new_child, false);
        assert!(registry.restore(&[old_child], root).is_none());
        registry.unregister(new_child);
        registry.unregister_scope(child).unwrap();
        assert_eq!(registry.active_trap(), Some(parent));
        assert_eq!(
            registry.restore(&[old_child], root).unwrap().target,
            parent_target
        );
    }

    #[test]
    fn lifecycle_does_not_leak_targets_or_scopes() {
        let mut r = FocusRegistry::new();
        for _ in 0..100 {
            let scope = FocusScopeId(1);
            r.register_scope(scope, FocusScopeId(0)).unwrap();
            let h = r.register(FocusId(1), scope, 0, true);
            r.push_trap(scope).unwrap();
            r.unregister(h);
            r.unregister_scope(scope).unwrap();
            assert!(r.is_empty());
            assert_eq!(r.active_trap(), None);
        }
        r.register_scope(FocusScopeId(1), FocusScopeId(0)).unwrap();
        r.register_scope(FocusScopeId(2), FocusScopeId(1)).unwrap();
        assert_eq!(
            r.register_scope(FocusScopeId(1), FocusScopeId(2)),
            Err(FocusError::ScopeCycle)
        );
    }
}
