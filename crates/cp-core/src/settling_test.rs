use super::*;
use std::cell::{Cell, RefCell};

const SHORT: Settling = Settling {
    quiet: Duration::from_millis(100),
    ceiling: Duration::from_millis(400),
    step: Duration::from_millis(10),
};

struct Clock {
    now: Cell<Duration>,
}

impl Clock {
    fn new() -> Self {
        Self {
            now: Cell::new(Duration::ZERO),
        }
    }

    fn elapsed(&self) -> Duration {
        self.now.get()
    }

    fn pause(&self, by: Duration) {
        self.now.set(self.now.get() + by);
    }
}

fn settled_with(
    clock: &Clock,
    count: impl Fn(Duration) -> Option<i64>,
    cut: impl Fn(Duration) -> bool,
) -> Settled {
    settle(
        SHORT,
        || count(clock.elapsed()),
        || cut(clock.elapsed()),
        || clock.elapsed(),
        |by| clock.pause(by),
    )
}

#[test]
fn the_settling_waits_longer_than_a_step_and_never_past_a_second() {
    assert!(SETTLING.step < SETTLING.quiet);
    assert!(SETTLING.quiet < SETTLING.ceiling);
    assert!(SETTLING.ceiling <= Duration::from_secs(1));
}

#[test]
fn a_count_that_never_moves_settles_after_the_quiet_window() {
    let clock = Clock::new();
    let seen = settled_with(&clock, |_| Some(7), |_| false);
    assert_eq!(seen, Settled::At(Some(7)));
    assert_eq!(clock.elapsed(), SHORT.quiet);
}

#[test]
fn a_count_that_moves_restarts_the_quiet_window_and_settles_on_the_latest() {
    let clock = Clock::new();
    let seen = settled_with(
        &clock,
        |at| Some(if at < Duration::from_millis(50) { 1 } else { 2 }),
        |_| false,
    );
    assert_eq!(seen, Settled::At(Some(2)));
    assert!(
        clock.elapsed() >= Duration::from_millis(50) + SHORT.quiet,
        "the window counts from the last move, not from the start"
    );
}

#[test]
fn a_count_that_keeps_moving_is_stopped_by_the_ceiling() {
    let clock = Clock::new();
    let seen = settled_with(&clock, |at| Some(at.as_millis() as i64), |_| false);
    let Settled::At(Some(last)) = seen else {
        panic!("a moving count still settles, got {seen:?}");
    };
    assert!(last > 0);
    assert_eq!(clock.elapsed(), SHORT.ceiling);
}

#[test]
fn a_cut_from_the_start_returns_cut_without_waiting() {
    let clock = Clock::new();
    let seen = settled_with(&clock, |_| Some(1), |_| true);
    assert_eq!(seen, Settled::Cut);
    assert_eq!(clock.elapsed(), Duration::ZERO);
}

#[test]
fn a_cut_that_arrives_mid_wait_returns_cut() {
    let clock = Clock::new();
    let seen = settled_with(&clock, |_| Some(1), |at| at >= Duration::from_millis(40));
    assert_eq!(seen, Settled::Cut);
    assert_eq!(clock.elapsed(), Duration::from_millis(40));
}

#[test]
fn a_count_that_cannot_be_read_settles_as_nothing() {
    let clock = Clock::new();
    let seen = settled_with(&clock, |_| None, |_| false);
    assert_eq!(seen, Settled::At(None));
}

#[test]
fn a_count_that_appears_late_settles_on_it() {
    let clock = Clock::new();
    let seen = settled_with(
        &clock,
        |at| (at >= Duration::from_millis(30)).then_some(5),
        |_| false,
    );
    assert_eq!(seen, Settled::At(Some(5)));
}

#[test]
fn the_wait_is_made_in_steps() {
    let clock = Clock::new();
    let pauses = RefCell::new(Vec::new());
    settle(
        SHORT,
        || Some(1),
        || false,
        || clock.elapsed(),
        |by| {
            pauses.borrow_mut().push(by);
            clock.pause(by);
        },
    );
    let pauses = pauses.into_inner();
    assert_eq!(pauses.len(), 10);
    assert!(pauses.iter().all(|by| *by == SHORT.step));
}

#[test]
fn a_count_lost_mid_wait_keeps_the_last_one_read() {
    let clock = Clock::new();
    let seen = settled_with(
        &clock,
        |at| (at < Duration::from_millis(30)).then_some(9),
        |_| false,
    );
    assert_eq!(seen, Settled::At(Some(9)));
}
