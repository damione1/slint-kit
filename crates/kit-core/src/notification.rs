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
/// Why a previously queued toast left the queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastCloseReason {
    Expired,
    Dismissed,
    Action,
    Evicted,
    Cleared,
}
/// An owned terminal record, including the original typed value.
#[derive(Debug, PartialEq, Eq)]
pub struct ToastClosed<T> {
    pub id: ToastId,
    pub value: T,
    pub reason: ToastCloseReason,
}
/// A queue operation and every closure it caused, in queue order.
#[must_use = "handle the closed records as well as the operation result"]
#[derive(Debug, PartialEq, Eq)]
pub struct ToastChange<T, R> {
    pub result: R,
    pub closed: Vec<ToastClosed<T>>,
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
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
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
    ) -> Result<ToastChange<T, ToastId>, ToastQueueError> {
        let next = self
            .next
            .checked_add(1)
            .ok_or(ToastQueueError::IdentityExhausted)?;
        let mut closed = self.advance(now);
        let id = ToastId(self.next);
        self.next = next;
        if self.entries.len() == self.capacity.get() {
            let entry = self
                .entries
                .pop_front()
                .expect("a full queue has a first entry");
            closed.push(ToastClosed {
                id: entry.id,
                value: entry.value,
                reason: ToastCloseReason::Evicted,
            });
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
        Ok(ToastChange { result: id, closed })
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
    ) -> ToastChange<T, bool> {
        let closed = self.advance(now);
        let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) else {
            return ToastChange {
                result: false,
                closed,
            };
        };
        match source {
            ToastPause::Pointer => entry.pointer = paused,
            ToastPause::Focus => entry.focus = paused,
        };
        ToastChange {
            result: true,
            closed,
        }
    }
    /// Update a live identity in place. Keep its pause sources and restart its lifetime.
    /// Overdue records expire before the lookup and cannot be revived.
    pub fn update(
        &mut self,
        id: ToastId,
        lifetime: ToastLifetime,
        now: Duration,
        update: impl FnOnce(&mut T),
    ) -> ToastChange<T, bool> {
        let closed = self.advance(now);
        let Some(entry) = self.entries.iter_mut().find(|entry| entry.id == id) else {
            return ToastChange {
                result: false,
                closed,
            };
        };
        update(&mut entry.value);
        entry.remaining = match lifetime {
            ToastLifetime::Persistent => None,
            ToastLifetime::Timed(duration) => Some(duration),
        };
        ToastChange {
            result: true,
            closed,
        }
    }
    /// Expire overdue records, then close all remaining live records.
    pub fn clear(&mut self, now: Duration) -> Vec<ToastClosed<T>> {
        let mut closed = self.advance(now);
        closed.extend(self.entries.drain(..).map(|entry| ToastClosed {
            id: entry.id,
            value: entry.value,
            reason: ToastCloseReason::Cleared,
        }));
        closed
    }
    pub fn advance(&mut self, now: Duration) -> Vec<ToastClosed<T>> {
        let now = now.max(self.clock);
        let delta = now - self.clock;
        self.clock = now;
        let mut closed = Vec::new();
        let mut retained = VecDeque::with_capacity(self.entries.len());
        while let Some(mut entry) = self.entries.pop_front() {
            if !entry.pointer
                && !entry.focus
                && let Some(remaining) = &mut entry.remaining
            {
                *remaining = remaining.saturating_sub(delta);
            }
            if entry.remaining == Some(Duration::ZERO) {
                closed.push(ToastClosed {
                    id: entry.id,
                    value: entry.value,
                    reason: ToastCloseReason::Expired,
                });
            } else {
                retained.push_back(entry);
            }
        }
        self.entries = retained;
        closed
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn ids<T>(closed: Vec<ToastClosed<T>>) -> Vec<ToastId> {
        closed.into_iter().map(|record| record.id).collect()
    }
    fn t(seconds: u64) -> Duration {
        Duration::from_secs(seconds)
    }
    #[test]
    fn independent_hover_and_focus_pause_and_resume_remaining_time() {
        let mut queue = ToastQueue::new(NonZeroUsize::new(3).unwrap());
        let id = queue
            .push("Saved", ToastLifetime::Timed(t(5)), t(0))
            .unwrap()
            .result;
        assert!(queue.set_paused(id, ToastPause::Pointer, true, t(2)).result);
        assert!(queue.set_paused(id, ToastPause::Focus, true, t(8)).result);
        assert!(
            queue
                .set_paused(id, ToastPause::Pointer, false, t(10))
                .result
        );
        assert!(queue.advance(t(20)).is_empty());
        assert!(queue.set_paused(id, ToastPause::Focus, false, t(20)).result);
        assert!(queue.advance(t(22)).is_empty());
        assert_eq!(ids(queue.advance(t(23))), [id]);
    }
    #[test]
    fn stale_dismiss_and_backward_clock_cannot_affect_new_notifications() {
        let mut queue = ToastQueue::new(NonZeroUsize::new(1).unwrap());
        let old = queue
            .push(1, ToastLifetime::Persistent, t(10))
            .unwrap()
            .result;
        let current = queue
            .push(2, ToastLifetime::Timed(t(5)), t(10))
            .unwrap()
            .result;
        assert_eq!(queue.remove(old), None);
        assert!(queue.advance(t(2)).is_empty());
        assert!(queue.advance(t(14)).is_empty());
        assert_eq!(ids(queue.advance(t(15))), [current]);
    }
    #[test]
    fn next_timeout_tracks_the_earliest_running_record_and_both_pause_sources() {
        let mut queue = ToastQueue::new(NonZeroUsize::new(3).unwrap());
        assert_eq!(queue.next_timeout(t(0)), None);
        assert!(
            queue
                .push("Persistent", ToastLifetime::Persistent, t(0))
                .unwrap()
                .closed
                .is_empty()
        );
        assert_eq!(queue.next_timeout(t(0)), None);
        let slow = queue
            .push("Slow", ToastLifetime::Timed(t(8)), t(0))
            .unwrap()
            .result;
        let fast = queue
            .push("Fast", ToastLifetime::Timed(t(5)), t(0))
            .unwrap()
            .result;
        assert_eq!(queue.next_timeout(t(2)), Some(t(3)));
        assert!(
            queue
                .set_paused(fast, ToastPause::Pointer, true, t(2))
                .result
        );
        assert_eq!(queue.next_timeout(t(2)), Some(t(6)));
        assert!(queue.set_paused(fast, ToastPause::Focus, true, t(4)).result);
        assert!(
            queue
                .set_paused(slow, ToastPause::Pointer, true, t(4))
                .result
        );
        assert_eq!(queue.next_timeout(t(4)), None);
        assert!(
            queue
                .set_paused(fast, ToastPause::Pointer, false, t(10))
                .result
        );
        assert_eq!(
            queue.next_timeout(t(10)),
            None,
            "focus still pauses the faster toast"
        );
        assert!(
            queue
                .set_paused(fast, ToastPause::Focus, false, t(12))
                .result
        );
        assert_eq!(queue.next_timeout(t(12)), Some(t(3)));
        assert_eq!(ids(queue.advance(t(15))), [fast]);
        assert_eq!(queue.next_timeout(t(15)), None);
        assert!(
            queue
                .set_paused(slow, ToastPause::Pointer, false, t(16))
                .result
        );
        assert_eq!(queue.next_timeout(t(16)), Some(t(4)));
        queue.remove(slow);
        assert_eq!(queue.next_timeout(t(16)), None);
    }

    #[test]
    fn next_timeout_is_nonmutating_and_handles_overdue_and_backwards_reads() {
        let mut queue = ToastQueue::new(NonZeroUsize::new(1).unwrap());
        let id = queue
            .push("Timed", ToastLifetime::Timed(t(9)), t(10))
            .unwrap()
            .result;
        assert_eq!(queue.next_timeout(t(1)), Some(t(9)));
        assert_eq!(queue.next_timeout(t(12)), Some(t(7)));
        assert_eq!(queue.next_timeout(t(19)), Some(Duration::ZERO));
        assert_eq!(queue.entries().count(), 1, "only advance expires records");
        assert_eq!(ids(queue.advance(t(19))), [id]);
        assert_eq!(queue.next_timeout(t(19)), None);
    }
    #[test]
    fn closure_records_own_payloads_across_expiry_eviction_and_clear() {
        #[derive(Debug, PartialEq, Eq)]
        struct Payload(&'static str);
        let mut queue = ToastQueue::new(NonZeroUsize::new(2).unwrap());
        let first = queue
            .push(Payload("expired"), ToastLifetime::Timed(t(1)), t(0))
            .unwrap();
        assert!(first.closed.is_empty());
        let persistent = queue
            .push(Payload("evicted"), ToastLifetime::Persistent, t(0))
            .unwrap();
        assert!(persistent.closed.is_empty());
        let next = queue
            .push(Payload("kept"), ToastLifetime::Persistent, t(2))
            .unwrap();
        assert_eq!(
            next.closed,
            [ToastClosed {
                id: first.result,
                value: Payload("expired"),
                reason: ToastCloseReason::Expired
            }]
        );
        let last = queue
            .push(Payload("last"), ToastLifetime::Persistent, t(2))
            .unwrap();
        assert_eq!(
            last.closed,
            [ToastClosed {
                id: persistent.result,
                value: Payload("evicted"),
                reason: ToastCloseReason::Evicted
            }]
        );
        assert_eq!(queue.len(), 2);
        assert_eq!(
            queue.clear(t(2)),
            [
                ToastClosed {
                    id: next.result,
                    value: Payload("kept"),
                    reason: ToastCloseReason::Cleared
                },
                ToastClosed {
                    id: last.result,
                    value: Payload("last"),
                    reason: ToastCloseReason::Cleared
                },
            ]
        );
        assert!(queue.is_empty());
        assert!(queue.clear(t(3)).is_empty());
    }
    #[test]
    fn updates_keep_identity_and_both_pauses_and_never_revive_expired_records() {
        let mut queue = ToastQueue::new(NonZeroUsize::new(2).unwrap());
        let expired = queue
            .push(String::from("old"), ToastLifetime::Timed(t(1)), t(0))
            .unwrap()
            .result;
        let kept = queue
            .push(String::from("loading"), ToastLifetime::Persistent, t(0))
            .unwrap()
            .result;
        let pause = queue.set_paused(kept, ToastPause::Pointer, true, t(2));
        assert!(pause.result);
        assert_eq!(
            pause.closed,
            [ToastClosed {
                id: expired,
                value: String::from("old"),
                reason: ToastCloseReason::Expired
            }]
        );
        assert!(queue.set_paused(kept, ToastPause::Focus, true, t(2)).result);
        let updated = queue.update(kept, ToastLifetime::Timed(t(5)), t(3), |value| {
            *value = String::from("saved")
        });
        assert!(updated.result && updated.closed.is_empty());
        assert_eq!(queue.entries().next().unwrap().id, kept);
        assert_eq!(queue.next_timeout(t(30)), None);
        assert!(
            queue
                .set_paused(kept, ToastPause::Pointer, false, t(30))
                .result
        );
        assert_eq!(queue.next_timeout(t(35)), None, "focus remains paused");
        assert!(
            queue
                .set_paused(kept, ToastPause::Focus, false, t(40))
                .result
        );
        assert_eq!(queue.next_timeout(t(40)), Some(t(5)));
        let failed = queue.update(kept, ToastLifetime::Persistent, t(45), |_| {
            panic!("an expired payload cannot be edited")
        });
        assert!(!failed.result);
        assert_eq!(
            failed.closed,
            [ToastClosed {
                id: kept,
                value: String::from("saved"),
                reason: ToastCloseReason::Expired
            }]
        );
        assert!(queue.is_empty());
        assert!(
            !queue
                .update(kept, ToastLifetime::Persistent, t(46), |_| panic!(
                    "no revival"
                ))
                .result
        );
    }
}
