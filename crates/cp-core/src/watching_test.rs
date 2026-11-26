use super::*;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, AtomicUsize, Ordering};
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

struct Looked {
    count: Arc<AtomicI64>,
    reads: Arc<AtomicUsize>,
}

impl Looked {
    fn from(start: i64) -> Self {
        Self {
            count: Arc::new(AtomicI64::new(start)),
            reads: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn counting(&self) -> Counting {
        let count = self.count.clone();
        let reads = self.reads.clone();
        Arc::new(move || {
            reads.fetch_add(1, Ordering::Relaxed);
            Some(count.load(Ordering::Relaxed))
        })
    }

    fn once_seen(&self) {
        assert!(
            waited_for(&self.reads, 1) >= 1,
            "the watcher never took its first look"
        );
    }

    fn set(&self, value: i64) {
        self.count.store(value, Ordering::Relaxed);
    }

    fn set_and_seen(&self, value: i64) {
        self.set(value);
        self.looked_again();
    }

    fn looked_again(&self) {
        let before = self.reads.load(Ordering::Relaxed);
        assert!(
            waited_for(&self.reads, before + 2) >= before + 2,
            "the watcher never looked again"
        );
    }
}

fn counted(seen: &Arc<AtomicUsize>) -> impl FnMut() + Send + 'static {
    let mine = seen.clone();
    move || {
        mine.fetch_add(1, Ordering::Relaxed);
    }
}

fn waited_for(seen: &Arc<AtomicUsize>, want: usize) -> usize {
    eventually(|| seen.load(Ordering::Relaxed) >= want);
    seen.load(Ordering::Relaxed)
}

#[test]
fn a_counter_that_moves_wakes_whoever_is_watching() {
    let count = Looked::from(10);
    let seen = Arc::new(AtomicUsize::new(0));
    let watching = Watching::every(
        Cadence::Opaque,
        QUICK,
        NAP,
        count.counting(),
        never_busy(),
        counted(&seen),
    );
    count.once_seen();
    count.set(11);
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
    let count = Looked::from(100);
    let seen = Arc::new(AtomicUsize::new(0));
    let watching = Watching::every(
        Cadence::Opaque,
        QUICK,
        NAP,
        count.counting(),
        never_busy(),
        counted(&seen),
    );
    count.once_seen();
    assert!(watching.writing());
    count.set(103);
    assert!(watching.ours());
    std::thread::sleep(QUICK * 20);
    assert_eq!(seen.load(Ordering::Relaxed), 0, "the whole stretch is ours");
    count.set(200);
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

#[test]
fn a_copy_that_lands_while_a_write_is_given_up_still_wakes_the_watcher() {
    let count = Looked::from(30);
    let seen = Arc::new(AtomicUsize::new(0));
    let watching = Watching::every(
        Cadence::Opaque,
        QUICK,
        NAP,
        count.counting(),
        never_busy(),
        counted(&seen),
    );
    count.once_seen();
    assert!(watching.writing());
    count.set(31);
    std::thread::sleep(QUICK * 4);
    assert!(watching.abandoned());
    assert!(
        waited_for(&seen, 1) >= 1,
        "the copy that beat the write is kept"
    );
    assert!(watching.close());
}

const SHORT_SETTLING: Settling = Settling {
    quiet: Duration::from_millis(30),
    ceiling: Duration::from_millis(300),
    step: Duration::from_millis(5),
};

struct Stopped {
    at: Arc<AtomicU64>,
}

impl Stopped {
    fn new() -> Self {
        Self {
            at: Arc::new(AtomicU64::new(0)),
        }
    }

    fn clock(&self) -> Clock {
        let at = self.at.clone();
        Arc::new(move || Duration::from_millis(at.load(Ordering::Relaxed)))
    }

    fn pass(&self, by: Duration) {
        let millis = u64::try_from(by.as_millis()).expect("a short pass");
        self.at.fetch_add(millis, Ordering::Relaxed);
    }
}

fn late_on(clock: Clock) -> Settle {
    Settle {
        late: Arc::new(|| true),
        settling: SHORT_SETTLING,
        clock,
    }
}

fn eventually(what: impl Fn() -> bool) -> bool {
    let until = Instant::now() + Duration::from_secs(5);
    while Instant::now() < until && !what() {
        std::thread::sleep(QUICK);
    }
    what()
}

fn settling_now(watching: &Watching) {
    assert!(
        eventually(|| !watching.caught_up_within(Duration::ZERO)),
        "the watcher never began to settle"
    );
}

fn settling_watch(
    count: Counting,
    busy: Busy,
    settle: Settle,
    seen: &Arc<AtomicUsize>,
) -> Watching {
    Watching::settling(
        Cadence::Opaque,
        QUICK,
        NAP,
        count,
        busy,
        settle,
        counted(seen),
    )
}

#[test]
fn a_late_change_is_read_once_even_if_the_count_moves_again_while_it_settles() {
    let count = Looked::from(10);
    let time = Stopped::new();
    let seen = Arc::new(AtomicUsize::new(0));
    let watching = settling_watch(count.counting(), never_busy(), late_on(time.clock()), &seen);
    count.once_seen();
    count.set(11);
    settling_now(&watching);
    count.set_and_seen(12);
    assert_eq!(
        seen.load(Ordering::Relaxed),
        0,
        "nothing is read before it settles"
    );
    time.pass(SHORT_SETTLING.quiet);
    assert!(
        watching.caught_up_within(Duration::from_secs(5)),
        "a quiet count ends the settle"
    );
    assert_eq!(seen.load(Ordering::Relaxed), 1);
    count.looked_again();
    assert!(
        watching.caught_up_within(Duration::ZERO),
        "the second move is not a second read"
    );
    assert_eq!(seen.load(Ordering::Relaxed), 1);
    assert!(watching.close());
}

#[test]
fn a_change_that_is_not_late_is_read_at_once() {
    let count = Looked::from(10);
    let time = Stopped::new();
    let seen = Arc::new(AtomicUsize::new(0));
    let watching = settling_watch(
        count.counting(),
        never_busy(),
        Settle {
            late: Arc::new(|| false),
            settling: SHORT_SETTLING,
            clock: time.clock(),
        },
        &seen,
    );
    count.once_seen();
    count.set(11);
    assert_eq!(waited_for(&seen, 1), 1);
    assert!(watching.close());
}

#[test]
fn a_write_of_ours_during_the_settle_skips_the_read() {
    let count = Looked::from(10);
    let time = Stopped::new();
    let seen = Arc::new(AtomicUsize::new(0));
    let writing = Arc::new(AtomicBool::new(false));
    let begins = writing.clone();
    let watching = settling_watch(
        count.counting(),
        {
            let ours = writing.clone();
            Arc::new(move || ours.load(Ordering::Relaxed))
        },
        Settle {
            late: Arc::new(move || {
                begins.store(true, Ordering::Relaxed);
                true
            }),
            settling: SHORT_SETTLING,
            clock: time.clock(),
        },
        &seen,
    );
    count.once_seen();
    count.set(11);
    assert!(
        eventually(|| writing.load(Ordering::Relaxed)),
        "the watcher never began to settle"
    );
    assert!(
        watching.caught_up_within(Duration::from_secs(5)),
        "our write cuts the settle short"
    );
    assert_eq!(seen.load(Ordering::Relaxed), 0);
    assert!(watching.close());
}

#[test]
fn closing_in_the_middle_of_a_settle_is_prompt() {
    let moving = Arc::new(AtomicI64::new(0));
    let time = Stopped::new();
    let seen = Arc::new(AtomicUsize::new(0));
    let ticking = moving.clone();
    let watching = settling_watch(
        Arc::new(move || Some(ticking.fetch_add(1, Ordering::Relaxed))),
        never_busy(),
        late_on(time.clock()),
        &seen,
    );
    settling_now(&watching);
    let started = Instant::now();
    assert!(watching.close());
    assert!(started.elapsed() < Duration::from_secs(2));
    assert_eq!(seen.load(Ordering::Relaxed), 0);
}

#[test]
fn a_write_of_ours_that_lands_while_it_settles_is_not_read_as_theirs() {
    let count = Looked::from(10);
    let time = Stopped::new();
    let seen = Arc::new(AtomicUsize::new(0));
    let watching = settling_watch(count.counting(), never_busy(), late_on(time.clock()), &seen);
    count.once_seen();
    count.set(11);
    settling_now(&watching);
    assert!(watching.writing());
    count.set(12);
    assert!(watching.ours());
    assert!(
        watching.caught_up_within(Duration::from_secs(5)),
        "our write cuts the settle short"
    );
    assert_eq!(
        seen.load(Ordering::Relaxed),
        0,
        "our own write is not a copy"
    );
    assert!(watching.close());
}

#[test]
fn whoever_asks_is_held_until_the_copy_being_read_is_kept() {
    let count = Looked::from(10);
    let entered = Arc::new(AtomicUsize::new(0));
    let finished = Arc::new(AtomicUsize::new(0));
    let (enters, finishes) = (entered.clone(), finished.clone());
    let watching = Watching::settling(
        Cadence::Opaque,
        QUICK,
        NAP,
        count.counting(),
        never_busy(),
        late_on(steady()),
        move || {
            enters.fetch_add(1, Ordering::Relaxed);
            std::thread::sleep(Duration::from_millis(100));
            finishes.fetch_add(1, Ordering::Relaxed);
        },
    );
    assert!(
        watching.caught_up_within(Duration::ZERO),
        "nothing is pending"
    );
    count.once_seen();
    count.set(11);
    assert_eq!(waited_for(&entered, 1), 1);
    assert!(watching.caught_up_within(Duration::from_secs(5)));
    assert_eq!(
        finished.load(Ordering::Relaxed),
        1,
        "the wait ends only once the copy is kept"
    );
    assert!(watching.close());
}
