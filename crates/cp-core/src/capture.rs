use crate::formats::Refusal;
use crate::item::Item;
use crate::watch::{Retried, Retry, insist};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Captured {
    Kept(Item),
    Refused(Refusal),
    Nothing,
    TooSlow,
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
        Captured::TooSlow => None,
        other => Some(other),
    };
    match insist(retry, count, attempt, std::thread::sleep) {
        Retried::Done(captured) => captured,
        Retried::Superseded => Captured::Superseded,
        Retried::Exhausted => Captured::TooSlow,
    }
}

#[cfg(test)]
#[path = "capture_test.rs"]
mod tests;
