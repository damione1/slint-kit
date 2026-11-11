// SPDX-License-Identifier: GPL-3.0-or-later
//! Notification lifetime with an injected monotonic clock, independent of a UI runtime.
use std::{collections::VecDeque, num::NonZeroUsize, time::Duration};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ToastId(u64);
impl ToastId {
    pub fn get(self) -> u64 {
        self.0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastLifetime {
    Persistent,
    Timed(Duration),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastPause {
    Pointer,
    Focus,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastQueueError {
    IdentityExhausted,
}
#[derive(Debug)]
pub struct ToastRecord<T> {
    pub id: ToastId,
    pub value: T,
    remaining: Option<Duration>,
    pointer: bool,
    focus: bool,
}
#[derive(Debug)]
pub struct ToastQueue<T> {
    entries: VecDeque<ToastRecord<T>>,
    capacity: NonZeroUsize,
    next: u64,
    clock: Duration,
}
impl<T> ToastQueue<T> {
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            entries: VecDeque::new(),
            capacity,
            next: 1,
            clock: Duration::ZERO,
        }
    }
    pub fn entries(&self) -> impl Iterator<Item = &ToastRecord<T>> {
        self.entries.iter()
    }
    /// Time until the earliest unpaused timed record expires at `now`.
    ///
    /// Uses the same injected monotonic clock as `advance`. This does not
    /// mutate the queue: an overdue record returns zero until advanced.
    /// Empty, persistent-only and fully paused queues return `None`.
    pub fn next_timeout(&self, now: Duration) -> Option<Duration> {
        let elapsed = now.saturating_sub(self.clock);
        self.entries
            .iter()
            .filter(|entry| !entry.pointer && !entry.focus)
            .filter_map(|entry| entry.remaining)
            .map(|remaining| remaining.saturating_sub(elapsed))
            .min()
    }
    pub fn push(
        &mut self,
        value: T,
        lifetime: ToastLifetime,
        now: Duration,
    ) -> Result<ToastId, ToastQueueError> {
        self.advance(now);
        let next = self
            .next
            .checked_add(1)
            .ok_or(ToastQueueError::IdentityExhausted)?;
        let id = ToastId(self.next);
        self.next = next;
        if self.entries.len() == self.capacity.get() {
            self.entries.pop_front();
        }
        self.entries.push_back(ToastRecord {
            id,
            value,
            remaining: match lifetime {
                ToastLifetime::Persistent => None,
                ToastLifetime::Timed(duration) => Some(duration),
            },
            pointer: false,
            focus: false,
        });
        Ok(id)
    }
    pub fn remove(&mut self, id: ToastId) -> Option<T> {
        let index = self.entries.iter().position(|e| e.id == id)?;
        self.entries.remove(index).map(|entry| entry.value)
    }
    pub fn set_paused(
        &mut self,
        id: ToastId,
        source: ToastPause,
        paused: bool,
        now: Duration,
    ) -> bool {
        self.advance(now);
        let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) else {
            return false;
        };
        match source {
            ToastPause::Pointer => entry.pointer = paused,
            ToastPause::Focus => entry.focus = paused,
        };
        true
    }
    pub fn advance(&mut self, now: Duration) -> Vec<ToastId> {
        let now = now.max(self.clock);
        let delta = now - self.clock;
        self.clock = now;
        let mut expired = Vec::new();
        self.entries.retain_mut(|entry| {
            if !entry.pointer
                && !entry.focus
                && let Some(remaining) = &mut entry.remaining
            {
                *remaining = remaining.saturating_sub(delta);
            }
            let retain = entry.remaining != Some(Duration::ZERO);
            if !retain {
                expired.push(entry.id);
            }
            retain
        });
        expired
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn t(seconds: u64) -> Duration {
        Duration::from_secs(seconds)
    }
    #[test]
    fn independent_hover_and_focus_pause_and_resume_remaining_time() {
        let mut queue = ToastQueue::new(NonZeroUsize::new(3).unwrap());
        let id = queue
            .push("Saved", ToastLifetime::Timed(t(5)), t(0))
            .unwrap();
        queue.set_paused(id, ToastPause::Pointer, true, t(2));
        queue.set_paused(id, ToastPause::Focus, true, t(8));
        queue.set_paused(id, ToastPause::Pointer, false, t(10));
        assert!(queue.advance(t(20)).is_empty());
        queue.set_paused(id, ToastPause::Focus, false, t(20));
        assert!(queue.advance(t(22)).is_empty());
        assert_eq!(queue.advance(t(23)), [id]);
    }
    #[test]
    fn stale_dismiss_and_backward_clock_cannot_affect_new_notifications() {
        let mut queue = ToastQueue::new(NonZeroUsize::new(1).unwrap());
        let old = queue.push(1, ToastLifetime::Persistent, t(10)).unwrap();
        let current = queue.push(2, ToastLifetime::Timed(t(5)), t(10)).unwrap();
        assert_eq!(queue.remove(old), None);
        queue.advance(t(2));
        assert!(queue.advance(t(14)).is_empty());
        assert_eq!(queue.advance(t(15)), [current]);
    }
    #[test]
    fn next_timeout_tracks_the_earliest_running_record_and_both_pause_sources() {
        let mut queue = ToastQueue::new(NonZeroUsize::new(3).unwrap());
        assert_eq!(queue.next_timeout(t(0)), None);
        queue
            .push("Persistent", ToastLifetime::Persistent, t(0))
            .unwrap();
        assert_eq!(queue.next_timeout(t(0)), None);
        let slow = queue
            .push("Slow", ToastLifetime::Timed(t(8)), t(0))
            .unwrap();
        let fast = queue
            .push("Fast", ToastLifetime::Timed(t(5)), t(0))
            .unwrap();
        assert_eq!(queue.next_timeout(t(2)), Some(t(3)));
        queue.set_paused(fast, ToastPause::Pointer, true, t(2));
        assert_eq!(queue.next_timeout(t(2)), Some(t(6)));
        queue.set_paused(fast, ToastPause::Focus, true, t(4));
        queue.set_paused(slow, ToastPause::Pointer, true, t(4));
        assert_eq!(queue.next_timeout(t(4)), None);
        queue.set_paused(fast, ToastPause::Pointer, false, t(10));
        assert_eq!(
            queue.next_timeout(t(10)),
            None,
            "focus still pauses the faster toast"
        );
        queue.set_paused(fast, ToastPause::Focus, false, t(12));
        assert_eq!(queue.next_timeout(t(12)), Some(t(3)));
        assert_eq!(queue.advance(t(15)), [fast]);
        assert_eq!(queue.next_timeout(t(15)), None);
        queue.set_paused(slow, ToastPause::Pointer, false, t(16));
        assert_eq!(queue.next_timeout(t(16)), Some(t(4)));
        queue.remove(slow);
        assert_eq!(queue.next_timeout(t(16)), None);
    }

    #[test]
    fn next_timeout_is_nonmutating_and_handles_overdue_and_backwards_reads() {
        let mut queue = ToastQueue::new(NonZeroUsize::new(1).unwrap());
        let id = queue
            .push("Timed", ToastLifetime::Timed(t(9)), t(10))
            .unwrap();
        assert_eq!(queue.next_timeout(t(1)), Some(t(9)));
        assert_eq!(queue.next_timeout(t(12)), Some(t(7)));
        assert_eq!(queue.next_timeout(t(19)), Some(Duration::ZERO));
        assert_eq!(queue.entries().count(), 1, "only advance expires records");
        assert_eq!(queue.advance(t(19)), [id]);
        assert_eq!(queue.next_timeout(t(19)), None);
    }
}
