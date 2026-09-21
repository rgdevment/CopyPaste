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
mod tests {
    use super::*;

    fn quick() -> Retry {
        Retry {
            attempts: 3,
            pause: std::time::Duration::from_millis(1),
        }
    }

    #[test]
    fn a_slow_source_that_answers_on_the_second_try_is_kept() {
        let mut tries = 0;
        let got = insisting(
            quick(),
            || 7,
            || {
                tries += 1;
                if tries < 2 {
                    Captured::TooSlow
                } else {
                    Captured::Kept(Item::plain("tarde pero llega"))
                }
            },
        );
        assert_eq!(got, Captured::Kept(Item::plain("tarde pero llega")));
    }

    #[test]
    fn a_source_that_stays_slow_is_too_slow_in_the_end() {
        let mut tries = 0;
        let got = insisting(
            quick(),
            || 7,
            || {
                tries += 1;
                Captured::TooSlow
            },
        );
        assert_eq!(got, Captured::TooSlow);
        assert_eq!(tries, 3);
    }

    #[test]
    fn an_empty_clipboard_is_an_answer_not_a_delay() {
        let mut tries = 0;
        let got = insisting(
            quick(),
            || 7,
            || {
                tries += 1;
                Captured::Nothing
            },
        );
        assert_eq!(got, Captured::Nothing);
        assert_eq!(tries, 1);
    }

    #[test]
    fn a_refusal_is_final_on_the_first_try() {
        let mut tries = 0;
        let got = insisting(
            quick(),
            || 7,
            || {
                tries += 1;
                Captured::Refused(Refusal::Marked("org.nspasteboard.ConcealedType"))
            },
        );
        assert_eq!(
            got,
            Captured::Refused(Refusal::Marked("org.nspasteboard.ConcealedType"))
        );
        assert_eq!(tries, 1);
    }

    #[test]
    fn a_copy_made_while_insisting_supersedes_the_slow_one() {
        let count = std::cell::Cell::new(7);
        let got = insisting(
            quick(),
            || count.get(),
            || {
                count.set(8);
                Captured::TooSlow
            },
        );
        assert_eq!(got, Captured::Superseded);
    }

    #[test]
    fn only_what_was_kept_is_an_item() {
        let item = Item::plain("hola");
        assert_eq!(Captured::Kept(item.clone()).kept(), Some(item));
        assert_eq!(Captured::Nothing.kept(), None);
        assert_eq!(Captured::TooSlow.kept(), None);
        assert_eq!(Captured::Superseded.kept(), None);
        assert_eq!(Captured::Refused(Refusal::Declined("x")).kept(), None);
    }
}
