// SPDX-License-Identifier: GPL-3.0-or-later
//! Which rows of an application sidebar are on screen. The model is a flat list
//! in reading order: section headings and items, where an item may name another
//! item as its parent. The rows shown follow from the set of expanded parents
//! and from whether the sidebar is collapsed to its icon rail.

use std::collections::HashSet;

/// Stable identity of a sidebar row, never a visual index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SidebarKey(i32);

impl SidebarKey {
    /// Keys are non-negative; a negative token is not an identity.
    pub fn new(value: i32) -> Option<Self> {
        (value >= 0).then_some(Self(value))
    }

    pub fn value(self) -> i32 {
        self.0
    }
}

/// What a sidebar row is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SidebarNodeKind {
    /// A heading that groups the items below it. It is never interactive.
    Section,
    /// A destination, or a parent of nested destinations.
    Item,
}

/// One row of the flat model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SidebarNode {
    pub key: Option<SidebarKey>,
    pub kind: SidebarNodeKind,
    /// The item this row is nested under. An unknown parent, a section as
    /// parent or a parent cycle leaves the row at the top level.
    pub parent: Option<SidebarKey>,
}

/// A row that is on screen, by index into the model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisibleSidebarRow {
    pub index: usize,
    pub depth: usize,
    pub expandable: bool,
    pub expanded: bool,
}

fn parent_index(nodes: &[SidebarNode], index: usize) -> Option<usize> {
    let node = nodes[index];
    let parent = node.parent?;
    match node.kind {
        SidebarNodeKind::Section => None,
        SidebarNodeKind::Item => nodes.iter().position(|candidate| {
            candidate.key == Some(parent) && candidate.kind == SidebarNodeKind::Item
        }),
    }
}

/// The ancestors of `index`, nearest first, or `None` when the chain loops.
fn ancestors(nodes: &[SidebarNode], index: usize) -> Option<Vec<usize>> {
    let mut chain = Vec::new();
    let mut current = index;
    while let Some(parent) = parent_index(nodes, current) {
        if parent == index || chain.contains(&parent) {
            return None;
        }
        chain.push(parent);
        current = parent;
    }
    Some(chain)
}

fn has_children(nodes: &[SidebarNode], index: usize) -> bool {
    nodes.iter().enumerate().any(|(child, _)| {
        child != index
            && parent_index(nodes, child) == Some(index)
            && ancestors(nodes, child).is_some()
    })
}

/// The rows to show, in model order.
///
/// Expanded, a row is shown when every ancestor is expanded. Collapsed to the
/// icon rail, only top-level items are shown: sections and nested items have
/// no place in a rail, and no parent counts as expanded.
pub fn visible_rows(
    nodes: &[SidebarNode],
    expanded: &HashSet<SidebarKey>,
    rail: bool,
) -> Vec<VisibleSidebarRow> {
    (0..nodes.len())
        .filter_map(|index| {
            let node = nodes[index];
            // A looping parent chain is broken by treating the row as top level.
            let chain = ancestors(nodes, index).unwrap_or_default();
            let depth = chain.len();
            match (node.kind, rail) {
                (SidebarNodeKind::Section, true) => return None,
                (SidebarNodeKind::Section, false) => {
                    return Some(VisibleSidebarRow {
                        index,
                        depth: 0,
                        expandable: false,
                        expanded: false,
                    });
                }
                (SidebarNodeKind::Item, true) if depth > 0 => return None,
                (SidebarNodeKind::Item, true | false) => {}
            }
            let open = |at: usize| nodes[at].key.is_some_and(|key| expanded.contains(&key));
            if !rail && !chain.iter().all(|ancestor| open(*ancestor)) {
                return None;
            }
            let expandable = has_children(nodes, index);
            Some(VisibleSidebarRow {
                index,
                depth,
                expandable,
                expanded: expandable && !rail && open(index),
            })
        })
        .collect()
}

/// The expanded set after asking `key` to open or close. Only an item with
/// nested items can expand; keys no longer in the model are dropped.
pub fn toggled_expansion(
    nodes: &[SidebarNode],
    expanded: &HashSet<SidebarKey>,
    key: SidebarKey,
    open: bool,
) -> HashSet<SidebarKey> {
    let known = |candidate: &SidebarKey| nodes.iter().any(|node| node.key == Some(*candidate));
    let mut next: HashSet<SidebarKey> = expanded.iter().copied().filter(known).collect();
    let expandable = nodes
        .iter()
        .position(|node| node.key == Some(key) && node.kind == SidebarNodeKind::Item)
        .is_some_and(|index| has_children(nodes, index));
    if !expandable {
        return next;
    }
    if open {
        next.insert(key);
    } else {
        next.remove(&key);
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(value: i32) -> Option<SidebarKey> {
        SidebarKey::new(value)
    }
    fn section(value: i32) -> SidebarNode {
        SidebarNode {
            key: key(value),
            kind: SidebarNodeKind::Section,
            parent: None,
        }
    }
    fn item(value: i32, parent: Option<i32>) -> SidebarNode {
        SidebarNode {
            key: key(value),
            kind: SidebarNodeKind::Item,
            parent: parent.and_then(SidebarKey::new),
        }
    }
    fn indexes(rows: &[VisibleSidebarRow]) -> Vec<usize> {
        rows.iter().map(|row| row.index).collect()
    }

    #[test]
    fn nested_rows_follow_their_parent_expansion() {
        let nodes = [section(0), item(1, None), item(2, Some(1)), item(3, None)];
        let closed = visible_rows(&nodes, &HashSet::new(), false);
        assert_eq!(indexes(&closed), [0, 1, 3]);
        assert!(closed[1].expandable && !closed[1].expanded);
        let open = visible_rows(&nodes, &HashSet::from([SidebarKey(1)]), false);
        assert_eq!(indexes(&open), [0, 1, 2, 3]);
        assert_eq!(open[2].depth, 1);
        assert!(open[1].expanded);
    }

    #[test]
    fn a_deep_row_needs_every_ancestor_open() {
        let nodes = [item(1, None), item(2, Some(1)), item(3, Some(2))];
        let inner_only = visible_rows(&nodes, &HashSet::from([SidebarKey(2)]), false);
        assert_eq!(indexes(&inner_only), [0]);
        let both = visible_rows(
            &nodes,
            &HashSet::from([SidebarKey(1), SidebarKey(2)]),
            false,
        );
        assert_eq!(indexes(&both), [0, 1, 2]);
        assert_eq!(both[2].depth, 2);
    }

    #[test]
    fn the_rail_keeps_top_level_items_only() {
        let nodes = [section(0), item(1, None), item(2, Some(1)), item(3, None)];
        let rail = visible_rows(&nodes, &HashSet::from([SidebarKey(1)]), true);
        assert_eq!(indexes(&rail), [1, 3]);
        assert!(rail[0].expandable && !rail[0].expanded);
    }

    #[test]
    fn an_unusable_parent_leaves_the_row_at_the_top_level() {
        let nodes = [
            section(0),
            item(1, Some(0)),
            item(2, Some(9)),
            item(3, Some(4)),
            item(4, Some(3)),
        ];
        let rows = visible_rows(&nodes, &HashSet::new(), false);
        assert_eq!(indexes(&rows), [0, 1, 2, 3, 4]);
        assert!(rows.iter().all(|row| row.depth == 0 && !row.expandable));
    }

    #[test]
    fn only_a_parent_can_expand_and_stale_keys_are_dropped() {
        let nodes = [item(1, None), item(2, Some(1)), item(3, None)];
        let stale = HashSet::from([SidebarKey(7)]);
        let opened = toggled_expansion(&nodes, &stale, SidebarKey(1), true);
        assert_eq!(opened, HashSet::from([SidebarKey(1)]));
        assert_eq!(
            toggled_expansion(&nodes, &opened, SidebarKey(3), true),
            opened
        );
        assert!(toggled_expansion(&nodes, &opened, SidebarKey(1), false).is_empty());
    }
}
