use super::*;
use std::sync::Arc;
use std::time::Instant;

const LONG: Duration = Duration::from_secs(30);

#[test]
fn resting_lasts_the_whole_time_when_nobody_calls() {
    let rest = Rest::new();
    let started = Instant::now();
    assert!(rest.rest(Duration::from_millis(80)));
    assert!(started.elapsed() >= Duration::from_millis(70));
}

#[test]
fn a_wake_ends_the_rest_at_once_and_is_used_up() {
    let rest = Arc::new(Rest::new());
    let other = rest.clone();
    let waker = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        other.wake();
    });
    let started = Instant::now();
    assert!(rest.rest(LONG));
    assert!(started.elapsed() < Duration::from_secs(10));
    let _ = waker.join();
    let again = Instant::now();
    assert!(rest.rest(Duration::from_millis(60)));
    assert!(again.elapsed() >= Duration::from_millis(50));
}

#[test]
fn a_wake_that_came_first_is_not_lost() {
    let rest = Rest::new();
    rest.wake();
    let started = Instant::now();
    assert!(rest.rest(LONG));
    assert!(started.elapsed() < Duration::from_secs(10));
}

#[test]
fn closing_ends_the_rest_and_every_one_after_it() {
    let rest = Arc::new(Rest::new());
    let other = rest.clone();
    let closer = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        other.close();
    });
    let started = Instant::now();
    assert!(!rest.rest(LONG));
    assert!(started.elapsed() < Duration::from_secs(10));
    let _ = closer.join();
    assert!(rest.closed());
    assert!(!rest.rest(LONG));
}
