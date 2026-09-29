use super::*;
use std::cell::Cell;
use std::time::Duration;

fn quick() -> Retry {
    Retry {
        attempts: 4,
        pause: Duration::from_millis(7),
    }
}

#[test]
fn a_source_that_answers_the_first_time_is_not_asked_twice() {
    let asked = Cell::new(0);
    let paused = Cell::new(0);
    let got = insist(
        quick(),
        || 1,
        || {
            asked.set(asked.get() + 1);
            Some("hi")
        },
        |_| paused.set(paused.get() + 1),
    );
    assert_eq!(got, Retried::Done("hi"));
    assert_eq!((asked.get(), paused.get()), (1, 0));
}

#[test]
fn a_source_that_takes_a_while_is_asked_again_after_a_pause() {
    let asked = Cell::new(0);
    let pauses = std::cell::RefCell::new(Vec::new());
    let got = insist(
        quick(),
        || 1,
        || {
            asked.set(asked.get() + 1);
            (asked.get() == 3).then_some("at last")
        },
        |how_long| pauses.borrow_mut().push(how_long),
    );
    assert_eq!(got, Retried::Done("at last"));
    assert_eq!(asked.get(), 3);
    assert_eq!(
        *pauses.borrow(),
        vec![Duration::from_millis(7); 2],
        "a pause between every two attempts, none before the first"
    );
}

#[test]
fn a_source_that_never_answers_is_given_up_on_after_the_attempts() {
    let asked = Cell::new(0);
    let got: Retried<()> = insist(
        quick(),
        || 1,
        || {
            asked.set(asked.get() + 1);
            None
        },
        |_| {},
    );
    assert_eq!(got, Retried::Exhausted);
    assert_eq!(asked.get(), 4, "exactly the policy's attempts");
}

#[test]
fn a_newer_copy_while_waiting_wins_and_the_old_one_is_dropped() {
    let asked = Cell::new(0);
    let counter = Cell::new(10);
    let got: Retried<()> = insist(
        quick(),
        || counter.get(),
        || {
            asked.set(asked.get() + 1);
            None
        },
        |_| counter.set(11),
    );
    assert_eq!(got, Retried::Superseded);
    assert_eq!(
        asked.get(),
        1,
        "whatever gets copied afterward, the watcher will see; it isn't read twice"
    );
}

#[test]
fn one_attempt_is_one_ask_and_no_pause() {
    let asked = Cell::new(0);
    let paused = Cell::new(0);
    let policy = Retry {
        attempts: 1,
        pause: Duration::from_millis(7),
    };
    let got: Retried<()> = insist(
        policy,
        || 1,
        || {
            asked.set(asked.get() + 1);
            None
        },
        |_| paused.set(paused.get() + 1),
    );
    assert_eq!(got, Retried::Exhausted);
    assert_eq!((asked.get(), paused.get()), (1, 0));
}

#[test]
fn zero_attempts_still_asks_once() {
    let asked = Cell::new(0);
    let policy = Retry {
        attempts: 0,
        pause: Duration::ZERO,
    };
    let got = insist(
        policy,
        || 1,
        || {
            asked.set(asked.get() + 1);
            Some(())
        },
        |_| {},
    );
    assert_eq!(got, Retried::Done(()));
    assert_eq!(asked.get(), 1);
}

#[test]
fn the_shipped_policy_waits_less_than_a_person_notices() {
    let worst = RETRY.pause.as_millis() * u128::from(RETRY.attempts - 1);
    assert!(worst <= 1_000, "{worst} ms of piled-up pauses is too much");
}
