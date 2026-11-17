// SPDX-License-Identifier: GPL-3.0-or-later
//! Generation guard for asynchronous work: only the latest request may deliver.

/// Proof that a request began on one [`Fence`]. A ticket is current until the
/// fence begins another request or is invalidated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FenceTicket {
    owner: u64,
    generation: u64,
}

/// Admits the result of the latest request only. Starting a request makes every
/// earlier ticket stale, so a slow first response cannot overwrite a newer one.
#[derive(Debug)]
pub struct Fence {
    owner: u64,
    generation: u64,
}

impl Default for Fence {
    fn default() -> Self {
        Self {
            owner: crate::field::next_owner(),
            generation: 0,
        }
    }
}

impl Fence {
    /// Start a request. Every earlier ticket becomes stale.
    pub fn begin(&mut self) -> FenceTicket {
        self.advance();
        FenceTicket {
            owner: self.owner,
            generation: self.generation,
        }
    }

    /// Make every outstanding ticket stale without starting a request.
    pub fn invalidate(&mut self) {
        self.advance();
    }

    /// Whether `ticket` came from this fence and no request began since.
    pub fn is_current(&self, ticket: FenceTicket) -> bool {
        ticket.owner == self.owner && ticket.generation == self.generation
    }

    fn advance(&mut self) {
        self.generation = self
            .generation
            .checked_add(1)
            .expect("fence generation exhausted");
    }
}

/// The state of a value that arrives asynchronously.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Loading<T, E> {
    /// A request is in flight.
    Loading,
    /// The latest request succeeded.
    Ready(T),
    /// The latest request failed.
    Failed(E),
}

impl<T, E> From<Result<T, E>> for Loading<T, E> {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(value) => Self::Ready(value),
            Err(error) => Self::Failed(error),
        }
    }
}

impl<T, E> Loading<T, E> {
    /// Whether a request is in flight.
    pub fn is_loading(&self) -> bool {
        match self {
            Self::Loading => true,
            Self::Ready(_) | Self::Failed(_) => false,
        }
    }

    /// The loaded value, if the latest request succeeded.
    pub fn ready(&self) -> Option<&T> {
        match self {
            Self::Ready(value) => Some(value),
            Self::Loading | Self::Failed(_) => None,
        }
    }

    /// The error, if the latest request failed.
    pub fn failed(&self) -> Option<&E> {
        match self {
            Self::Failed(error) => Some(error),
            Self::Loading | Self::Ready(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_latest_ticket_is_current() {
        let mut fence = Fence::default();
        let first = fence.begin();
        assert!(fence.is_current(first));
        let second = fence.begin();
        assert!(!fence.is_current(first));
        assert!(fence.is_current(second));
        fence.invalidate();
        assert!(!fence.is_current(second));
    }

    #[test]
    fn a_ticket_from_another_fence_is_never_current() {
        let mut one = Fence::default();
        let mut other = Fence::default();
        let ticket = one.begin();
        other.begin();
        assert!(!other.is_current(ticket));
    }

    #[test]
    fn loading_follows_a_result() {
        let ready: Loading<i32, &str> = Ok(3).into();
        assert_eq!(ready.ready(), Some(&3));
        assert!(!ready.is_loading());
        let failed: Loading<i32, &str> = Err("down").into();
        assert_eq!(failed.failed(), Some(&"down"));
        assert!(Loading::<i32, &str>::Loading.is_loading());
    }
}
