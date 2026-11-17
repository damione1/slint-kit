// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
//! `Router<R>`: typed screen navigation for Rust code and the `KitRouter`
//! component.
//!
//! The application declares its screens as one route type `R`: an enum, or an
//! enum whose variants carry the screen's parameters. Rust pushes, pops and
//! replaces routes; the router projects the current route into the window,
//! where each `KitScreen` shows itself when its route is current. The
//! projection is the one place that converts `R` to the Slint enum, with an
//! exhaustive `match`: a new route does not compile until it is handled.
use crate::{Store, Subscription, ui};
use kit_core::{RouteChange, RouteStack};

impl From<RouteChange> for ui::RouteChange {
    fn from(value: RouteChange) -> Self {
        match value {
            RouteChange::Push => Self::Push,
            RouteChange::Pop => Self::Pop,
            RouteChange::Replace => Self::Replace,
        }
    }
}

/// The route stack after a navigation, with what the navigation did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Navigation<R> {
    stack: RouteStack<R>,
    change: RouteChange,
}

impl<R> Navigation<R> {
    /// The screen to show.
    pub fn route(&self) -> &R {
        self.stack.current()
    }

    /// The last navigation. A new router reports `Replace`: nothing moved.
    pub fn change(&self) -> RouteChange {
        self.change
    }

    /// The screens behind the current one, oldest first.
    pub fn history(&self) -> &[R] {
        self.stack.history()
    }

    /// Number of screens, including the current one.
    pub fn depth(&self) -> usize {
        self.stack.depth()
    }

    /// Whether `Router::pop` has a screen to return to. Bind it to
    /// `KitRouter.can-go-back`.
    pub fn can_go_back(&self) -> bool {
        self.stack.can_go_back()
    }
}

/// Typed navigation state, shared by clones on the UI thread.
///
/// A router is a [`Store`] of the route stack: observers run synchronously
/// after each navigation and a change made by an observer is applied before
/// the pass ends.
#[derive(Clone)]
pub struct Router<R: Clone + 'static> {
    state: Store<Navigation<R>>,
}

impl<R: Clone + 'static> Router<R> {
    /// A router showing `root`, with nothing behind it.
    pub fn new(root: R) -> Self {
        Self {
            state: Store::new(Navigation {
                stack: RouteStack::new(root),
                change: RouteChange::Replace,
            }),
        }
    }

    /// The current navigation state.
    pub fn get(&self) -> Navigation<R> {
        self.state.get()
    }

    /// A copy of the current route.
    pub fn current(&self) -> R {
        self.state.with(|navigation| navigation.route().clone())
    }

    /// Show `route` on top of the current screen.
    pub fn push(&self, route: R) {
        self.state.update(|navigation| {
            navigation.stack.push(route);
            navigation.change = RouteChange::Push;
        });
    }

    /// Return to the previous screen. At the root nothing changes, no
    /// observer runs, and the result is `false`.
    pub fn pop(&self) -> bool {
        if !self.state.with(Navigation::can_go_back) {
            return false;
        }
        self.state.update(|navigation| {
            navigation.stack.pop();
            navigation.change = RouteChange::Pop;
        });
        true
    }

    /// Show `route` instead of the current screen; `pop` still returns to the
    /// screen behind it.
    pub fn replace(&self, route: R) {
        self.state.update(|navigation| {
            navigation.stack.replace(route);
            navigation.change = RouteChange::Replace;
        });
    }

    /// Call `observer` now with the current navigation, then after every
    /// navigation. This is how a router projects into a window: the closure
    /// sets `change`, the current route and `can-go-back`.
    #[must_use = "dropping the subscription stops the observer"]
    pub fn observe(
        &self,
        observer: impl Fn(&Navigation<R>) + 'static,
    ) -> Subscription<Navigation<R>> {
        self.state.observe(observer)
    }

    /// Call `observer` with the new route and what the navigation did, after
    /// every navigation. It is not called for the current route.
    #[must_use = "dropping the subscription stops the observer"]
    pub fn on_navigated(
        &self,
        observer: impl Fn(&R, RouteChange) + 'static,
    ) -> Subscription<Navigation<R>> {
        self.state
            .subscribe(move |navigation| observer(navigation.route(), navigation.change()))
    }
}
