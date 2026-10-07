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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unreadable {
    Asks,
    Denied,
}

impl Unreadable {
    pub fn said(self) -> &'static str {
        match self {
            Unreadable::Asks => "the system asks before CopyPaste may read the clipboard",
            Unreadable::Denied => "the system does not let CopyPaste read the clipboard",
        }
    }
}

impl Captured {
    pub fn kept(self) -> Option<Item> {
        match self {
            Captured::Kept(item) => Some(item),
            _ => None,
        }
    }
}

pub const LATE: std::time::Duration = std::time::Duration::from_secs(15);

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
    late: std::time::Duration,
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
        Captured::TooSlow if matches!(last, Captured::TooSlow) => {
            answered_late(&pending, patience, late, &count, started)
        }
        Captured::TooSlow => last,
        other => other,
    };
    match got {
        Captured::Nothing | Captured::Busy if count() != started => Captured::Superseded,
        other => other,
    }
}

fn answered_late(
    pending: &Pending<Captured>,
    patience: std::time::Duration,
    late: std::time::Duration,
    count: &impl Fn() -> i64,
    started: i64,
) -> Captured {
    let deadline = std::time::Instant::now() + late;
    while count() == started && std::time::Instant::now() < deadline {
        match pending.waited(patience) {
            Waited::StillRunning => {}
            Waited::Answered(Captured::TooSlow | Captured::Busy) | Waited::Gone => {
                return Captured::TooSlow;
            }
            Waited::Answered(answer) => {
                return if count() == started {
                    answer
                } else {
                    Captured::Superseded
                };
            }
        }
    }
    match count() == started {
        true => Captured::TooSlow,
        false => Captured::Superseded,
    }
}

#[cfg(test)]
#[path = "capture_test.rs"]
mod tests;
