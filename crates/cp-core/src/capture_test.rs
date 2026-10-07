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
                Captured::Kept(Item::plain("late, but it got here"))
            }
        },
    );
    assert_eq!(got, Captured::Kept(Item::plain("late, but it got here")));
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
    let item = Item::plain("hello");
    assert_eq!(Captured::Kept(item.clone()).kept(), Some(item));
    assert_eq!(Captured::Nothing.kept(), None);
    assert_eq!(Captured::TooSlow.kept(), None);
    assert_eq!(Captured::Superseded.kept(), None);
    assert_eq!(Captured::Refused(Refusal::Declined("x")).kept(), None);
}

#[test]
fn a_clipboard_held_by_someone_else_is_insisted_on_like_a_slow_one() {
    let mut tries = 0;
    let got = insisting(
        quick(),
        || 7,
        || {
            tries += 1;
            if tries < 3 {
                Captured::Busy
            } else {
                Captured::Nothing
            }
        },
    );
    assert_eq!(got, Captured::Nothing);
    assert_eq!(tries, 3, "Busy is retried, not taken as an answer");
}

#[test]
fn a_read_that_never_comes_back_stays_a_timeout_and_is_not_swallowed() {
    let got = insisting_afresh(
        quick(),
        std::time::Duration::from_millis(5),
        std::time::Duration::from_millis(20),
        || 7,
        || {
            crate::reading::begin(|| {
                std::thread::sleep(std::time::Duration::from_secs(30));
                Captured::Nothing
            })
        },
    );
    assert_eq!(
        got,
        Captured::TooSlow,
        "a read still running is a timeout the engine must log"
    );
}

#[test]
fn a_busy_clipboard_becomes_superseded_only_when_a_newer_copy_arrived() {
    let moved = std::sync::atomic::AtomicI64::new(1);
    let got = insisting_afresh(
        quick(),
        std::time::Duration::from_secs(5),
        std::time::Duration::ZERO,
        || moved.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        || crate::reading::begin(|| Captured::Busy),
    );
    assert_eq!(got, Captured::Superseded);

    let still = insisting_afresh(
        quick(),
        std::time::Duration::from_secs(5),
        std::time::Duration::ZERO,
        || 7,
        || crate::reading::begin(|| Captured::Busy),
    );
    assert_eq!(still, Captured::Busy, "nothing newer, so it stays busy");
}

fn slow_by(pause: u64, answer: Captured) -> Pending<Captured> {
    crate::reading::begin(move || {
        std::thread::sleep(std::time::Duration::from_millis(pause));
        answer
    })
}

fn after_slow(count: impl Fn() -> i64, pause: u64, answer: Captured) -> Captured {
    let mut once = Some(answer);
    insisting_afresh(
        Retry {
            attempts: 2,
            pause: std::time::Duration::from_millis(1),
        },
        std::time::Duration::from_millis(5),
        std::time::Duration::from_secs(5),
        count,
        move || slow_by(pause, once.take().unwrap_or(Captured::Nothing)),
    )
}

#[test]
fn a_read_that_answers_after_the_retries_is_still_kept() {
    let got = after_slow(|| 7, 80, Captured::Kept(Item::plain("came from the phone")));
    assert_eq!(got, Captured::Kept(Item::plain("came from the phone")));
}

#[test]
fn a_late_answer_is_dropped_if_a_newer_copy_arrived_meanwhile() {
    let start = std::time::Instant::now();
    let got = after_slow(
        || i64::from(start.elapsed() > std::time::Duration::from_millis(40)),
        200,
        Captured::Kept(Item::plain("stale")),
    );
    assert_eq!(got, Captured::Superseded);
}

#[test]
fn each_way_the_system_blocks_reading_has_its_own_words() {
    assert_ne!(Unreadable::Asks.said(), Unreadable::Denied.said());
    assert!(Unreadable::Denied.said().contains("does not let"));
}
