use super::*;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
use std::time::Instant;

const QUICK: Duration = Duration::from_millis(5);
const NAP: Duration = Duration::from_millis(2);

fn never_busy() -> Busy {
    Arc::new(|| false)
}

fn at(count: &Arc<AtomicI64>) -> Counting {
    let mine = count.clone();
    Arc::new(move || Some(mine.load(Ordering::Relaxed)))
}

fn counted(seen: &Arc<AtomicUsize>) -> impl FnMut() + Send + 'static {
    let mine = seen.clone();
    move || {
        mine.fetch_add(1, Ordering::Relaxed);
    }
}

fn waited_for(seen: &Arc<AtomicUsize>, want: usize) -> usize {
    let until = Instant::now() + Duration::from_secs(5);
    while Instant::now() < until && seen.load(Ordering::Relaxed) < want {
        std::thread::sleep(QUICK);
    }
    seen.load(Ordering::Relaxed)
}

#[test]
fn a_counter_that_moves_wakes_whoever_is_watching() {
    let count = Arc::new(AtomicI64::new(10));
    let seen = Arc::new(AtomicUsize::new(0));
    let watching = Watching::every(
        Cadence::Opaque,
        QUICK,
        NAP,
        at(&count),
        never_busy(),
        counted(&seen),
    );
    std::thread::sleep(QUICK * 4);
    count.store(11, Ordering::Relaxed);
    assert!(waited_for(&seen, 1) >= 1, "a change has to be noticed");
    assert!(watching.close());
}

#[test]
fn a_counter_that_does_not_move_wakes_nobody() {
    let count = Arc::new(AtomicI64::new(7));
    let seen = Arc::new(AtomicUsize::new(0));
    let watching = Watching::every(
        Cadence::Opaque,
        QUICK,
        NAP,
        at(&count),
        never_busy(),
        counted(&seen),
    );
    std::thread::sleep(QUICK * 20);
    assert_eq!(seen.load(Ordering::Relaxed), 0);
    assert!(watching.close());
}

#[test]
fn while_the_reader_says_it_is_busy_nothing_is_looked_at() {
    let count = Arc::new(AtomicI64::new(1));
    let seen = Arc::new(AtomicUsize::new(0));
    let watching = Watching::every(
        Cadence::Opaque,
        QUICK,
        NAP,
        at(&count),
        Arc::new(|| true),
        counted(&seen),
    );
    count.store(99, Ordering::Relaxed);
    std::thread::sleep(QUICK * 20);
    assert_eq!(
        seen.load(Ordering::Relaxed),
        0,
        "our own write must not look like a copy"
    );
    assert!(watching.close());
}

#[test]
fn a_counter_nobody_can_read_is_not_an_event_either() {
    let seen = Arc::new(AtomicUsize::new(0));
    let watching = Watching::every(
        Cadence::Opaque,
        QUICK,
        NAP,
        Arc::new(|| None),
        never_busy(),
        counted(&seen),
    );
    std::thread::sleep(QUICK * 20);
    assert_eq!(seen.load(Ordering::Relaxed), 0);
    assert!(!watching.writing(), "there is no number to mark");
    assert!(!watching.ours());
    assert!(watching.close());
}

#[test]
fn a_write_of_ours_is_not_a_copy_of_theirs() {
    let count = Arc::new(AtomicI64::new(100));
    let seen = Arc::new(AtomicUsize::new(0));
    let watching = Watching::every(
        Cadence::Opaque,
        QUICK,
        NAP,
        at(&count),
        never_busy(),
        counted(&seen),
    );
    std::thread::sleep(QUICK * 4);
    assert!(watching.writing());
    count.store(103, Ordering::Relaxed);
    assert!(watching.ours());
    std::thread::sleep(QUICK * 20);
    assert_eq!(seen.load(Ordering::Relaxed), 0, "the whole stretch is ours");
    count.store(200, Ordering::Relaxed);
    assert!(waited_for(&seen, 1) >= 1, "and the next copy is theirs");
    assert!(watching.close());
}

#[test]
fn closing_twice_is_no_error_and_dropping_is_closing() {
    let count = Arc::new(AtomicI64::new(1));
    let watching = Watching::every(Cadence::Opaque, QUICK, NAP, at(&count), never_busy(), || {});
    assert!(watching.close());
    assert!(watching.close());
    drop(watching);
}

#[test]
fn a_long_period_does_not_make_closing_take_that_long() {
    let count = Arc::new(AtomicI64::new(1));
    let watching = Watching::every(
        Cadence::Opaque,
        Duration::from_secs(3_600),
        NAP,
        at(&count),
        never_busy(),
        || {},
    );
    let started = Instant::now();
    drop(watching);
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "closing the application must not wait for the next poll"
    );
}

#[test]
fn the_cadence_decides_whether_losses_can_be_told() {
    let count = Arc::new(AtomicI64::new(1));
    let opaque = Watching::every(Cadence::Opaque, QUICK, NAP, at(&count), never_busy(), || {});
    assert_eq!(opaque.missed(), None);
    let told = Watching::every(
        Cadence::OnePerCopy,
        QUICK,
        NAP,
        at(&count),
        never_busy(),
        || {},
    );
    assert_eq!(told.missed(), Some(0));
}

#[test]
fn the_period_keeps_up_with_a_person_without_spinning() {
    assert!(
        EVERY <= Duration::from_millis(250),
        "a copy must not take long to be noticed"
    );
    assert!(
        EVERY >= Duration::from_millis(16),
        "nor should it poll faster than the screen"
    );
    assert!(
        NAP <= EVERY,
        "the nap cannot outlast the period it shortens"
    );
}

#[test]
fn the_counter_is_read_once_per_period_and_not_more() {
    let reads = Arc::new(AtomicUsize::new(0));
    let mine = reads.clone();
    let watching = Watching::every(
        Cadence::Opaque,
        Duration::from_millis(60),
        NAP,
        Arc::new(move || {
            mine.fetch_add(1, Ordering::Relaxed);
            Some(1)
        }),
        never_busy(),
        || {},
    );
    std::thread::sleep(Duration::from_millis(600));
    assert!(watching.close());
    let read = reads.load(Ordering::Relaxed);
    assert!((2..=11).contains(&read), "{read} reads in 600 ms at 60 ms");
}

#[test]
fn closing_in_the_middle_of_a_long_period_is_prompt() {
    let count = Arc::new(AtomicI64::new(1));
    let watching = Watching::every(
        Cadence::Opaque,
        Duration::from_secs(3_600),
        NAP,
        at(&count),
        never_busy(),
        || {},
    );
    std::thread::sleep(Duration::from_millis(50));
    let started = Instant::now();
    assert!(watching.close());
    assert!(started.elapsed() < Duration::from_millis(400));
}
