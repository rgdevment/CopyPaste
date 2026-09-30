use crate::formats::Refusal;
use crate::item::Item;
use crate::reading::{Pending, Waited};
use crate::watch::{Retried, Retry, insist};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Captured {
    Kept(Item),
    Refused(Refusal),
    Nothing,
    TooSlow,
    Busy,
    Superseded,
}

impl Captured {
    pub fn kept(self) -> Option<Item> {
        match self {
            Captured::Kept(item) => Some(item),
            _ => None,
        }
    }
}

pub fn insisting(
    retry: Retry,
    count: impl Fn() -> i64,
    mut once: impl FnMut() -> Captured,
) -> Captured {
    let attempt = || match once() {
        Captured::TooSlow | Captured::Busy => None,
        other => Some(other),
    };
    match insist(retry, count, attempt, std::thread::sleep) {
        Retried::Done(captured) => captured,
        Retried::Superseded => Captured::Superseded,
        Retried::Exhausted => Captured::TooSlow,
    }
}

pub fn insisting_afresh(
    retry: Retry,
    patience: std::time::Duration,
    count: impl Fn() -> i64,
    mut afresh: impl FnMut() -> Pending<Captured>,
) -> Captured {
    let started = count();
    let mut pending = afresh();
    let mut left = retry.attempts;
    let mut last = Captured::TooSlow;
    let got = insisting(retry, &count, || {
        left = left.saturating_sub(1);
        match pending.waited(patience) {
            Waited::StillRunning => {
                last = Captured::TooSlow;
                Captured::TooSlow
            }
            Waited::Answered(Captured::TooSlow | Captured::Busy) | Waited::Gone => {
                if left > 0 && count() == started {
                    pending = afresh();
                }
                last = Captured::Busy;
                Captured::Busy
            }
            Waited::Answered(other) => other,
        }
    });
    let got = match got {
        Captured::TooSlow => last,
        other => other,
    };
    match got {
        Captured::Nothing | Captured::Busy if count() != started => Captured::Superseded,
        other => other,
    }
}

#[cfg(test)]
#[path = "capture_test.rs"]
mod tests;
