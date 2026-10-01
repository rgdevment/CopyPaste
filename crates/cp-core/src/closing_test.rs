use super::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[test]
fn a_thread_that_already_ended_is_joined_without_waiting() {
    let thread = std::thread::spawn(|| {});
    let started = Instant::now();
    assert!(join_within(thread, Duration::from_secs(5)));
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[test]
fn a_thread_that_will_not_end_is_left_behind_instead_of_joined() {
    let stop = Arc::new(AtomicBool::new(false));
    let mine = stop.clone();
    let thread = std::thread::spawn(move || {
        while !mine.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(2));
        }
    });
    assert!(
        !join_within(thread, Duration::from_millis(40)),
        "joining it would be waiting for ever"
    );
    stop.store(true, Ordering::Relaxed);
}

#[test]
fn a_thread_that_panicked_still_counts_as_ended() {
    let before = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let thread = std::thread::spawn(|| panic!("on purpose"));
    while !thread.is_finished() {
        std::thread::sleep(Duration::from_millis(2));
    }
    let joined = join_within(thread, Duration::from_millis(40));
    std::panic::set_hook(before);
    assert!(joined, "it ended, however badly");
}

#[test]
fn the_patience_is_long_enough_for_a_thread_that_is_only_busy() {
    let thread = std::thread::spawn(|| std::thread::sleep(Duration::from_millis(60)));
    assert!(join_within(thread, PATIENCE));
}
