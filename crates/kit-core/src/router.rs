// SPDX-License-Identifier: GPL-3.0-or-later
//! The route stack behind screen navigation.
//!
//! `R` is the application's own route type. A plain enum names the screens;
//! an enum whose variants carry data (`Detail { id }`) carries the route
//! parameters with the screen. The stack never inspects `R`.

/// A non-empty stack of routes: the current screen on top of the screens
/// that "back" returns to.
///
/// The root route is part of the type, so the stack cannot be empty and
/// `current` cannot fail. `pop` stops at the root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteStack<R> {
    /// Screens behind the current one, oldest first.
    history: Vec<R>,
    current: R,
}

impl<R> RouteStack<R> {
    /// A stack holding only `root`.
    pub fn new(root: R) -> Self {
        Self {
            history: Vec::new(),
            current: root,
        }
    }

    /// The screen being shown.
    pub fn current(&self) -> &R {
        &self.current
    }

    /// The screens behind the current one, oldest first.
    pub fn history(&self) -> &[R] {
        &self.history
    }

    /// Number of screens, including the current one. Always at least 1.
    pub fn depth(&self) -> usize {
        self.history.len() + 1
    }

    /// Whether `pop` has a screen to return to.
    pub fn can_go_back(&self) -> bool {
        !self.history.is_empty()
    }

    /// Show `route` on top of the current screen.
    pub fn push(&mut self, route: R) {
        let previous = std::mem::replace(&mut self.current, route);
        self.history.push(previous);
    }

    /// Return to the previous screen. Does nothing at the root and returns
    /// whether the stack changed.
    pub fn pop(&mut self) -> bool {
        match self.history.pop() {
            Some(previous) => {
                self.current = previous;
                true
            }
            None => false,
        }
    }

    /// Show `route` instead of the current screen; "back" still returns to
    /// the screen behind it.
    pub fn replace(&mut self, route: R) {
        self.current = route;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop_and_replace_move_the_current_route() {
        let mut stack = RouteStack::new("list");
        assert_eq!((stack.depth(), stack.can_go_back()), (1, false));
        stack.push("detail");
        stack.push("edit");
        assert_eq!(*stack.current(), "edit");
        assert_eq!(stack.history(), ["list", "detail"]);
        stack.replace("done");
        assert_eq!((*stack.current(), stack.depth()), ("done", 3));
        assert!(stack.pop());
        assert_eq!(*stack.current(), "detail");
        assert!(stack.pop());
        assert!(!stack.pop(), "the root stays");
        assert_eq!(*stack.current(), "list");
    }
}
