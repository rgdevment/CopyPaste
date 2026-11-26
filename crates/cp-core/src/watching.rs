use crate::resting::Rest;
use crate::settling::{Settled, Settling, settle};
use crate::watch::{Cadence, Seen, Watcher};
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::Duration;

pub const EVERY: Duration = Duration::from_millis(60);
pub const NAP: Duration = Duration::from_millis(10);

const _: () = assert!(EVERY.as_millis() >= 16);
const _: () = assert!(EVERY.as_millis() <= 250);
const _: () = assert!(NAP.as_millis() <= EVERY.as_millis());

pub const CAUGHT_UP_WITHIN: Duration = Duration::from_millis(1_500);

const _: () = assert!(CAUGHT_UP_WITHIN.as_millis() > crate::settling::SETTLING.ceiling.as_millis());

pub type Counting = Arc<dyn Fn() -> Option<i64> + Send + Sync>;
pub type Busy = Arc<dyn Fn() -> bool + Send + Sync>;
pub type Late = Arc<dyn Fn() -> bool + Send + Sync>;
pub type Clock = Arc<dyn Fn() -> Duration + Send + Sync>;

pub fn steady() -> Clock {
    let origin = std::time::Instant::now();
    Arc::new(move || origin.elapsed())
}

pub struct Settle {
    pub late: Late,
    pub settling: Settling,
    pub clock: Clock,
}

impl Settle {
    pub fn never() -> Self {
        Self {
            late: Arc::new(|| false),
            settling: crate::settling::SETTLING,
            clock: steady(),
        }
    }
}

#[derive(Default)]
struct Behind {
    keeping: Mutex<bool>,
    done: Condvar,
}

impl Behind {
    fn keeping(&self, now: bool) {
        *self.keeping.lock().unwrap_or_else(PoisonError::into_inner) = now;
        self.done.notify_all();
    }

    fn caught_up_within(&self, most: Duration) -> bool {
        let held = self.keeping.lock().unwrap_or_else(PoisonError::into_inner);
        let (held, _) = self
            .done
            .wait_timeout_while(held, most, |keeping| *keeping)
            .unwrap_or_else(PoisonError::into_inner);
        !*held
    }
}

pub struct Watching {
    stop: Arc<Rest>,
    behind: Arc<Behind>,
    watcher: Arc<Mutex<Watcher>>,
    thread: Mutex<Option<std::thread::JoinHandle<()>>>,
    count: Counting,
}

impl Watching {
    pub fn every(
        cadence: Cadence,
        period: Duration,
        nap: Duration,
        count: Counting,
        busy: Busy,
        on_fresh: impl FnMut() + Send + 'static,
    ) -> Self {
        Self::settling(cadence, period, nap, count, busy, Settle::never(), on_fresh)
    }

    pub fn settling(
        cadence: Cadence,
        period: Duration,
        nap: Duration,
        count: Counting,
        busy: Busy,
        settle_first: Settle,
        mut on_fresh: impl FnMut() + Send + 'static,
    ) -> Self {
        let stop = Arc::new(Rest::new());
        let behind = Arc::new(Behind::default());
        let keeping = behind.clone();
        let watcher = Arc::new(Mutex::new(Watcher::new(cadence)));
        let mine = stop.clone();
        let theirs = watcher.clone();
        let reading = count.clone();
        let thread = std::thread::spawn(move || {
            while !mine.closed() {
                if busy() {
                    mine.rest(nap);
                    continue;
                }
                if let Some(now) = reading()
                    && let Ok(mut watcher) = theirs.lock()
                    && let Seen::Fresh { .. } = watcher.tick(now)
                {
                    drop(watcher);
                    keeping.keeping(true);
                    let cut = || {
                        busy()
                            || mine.closed()
                            || theirs.lock().map_or(true, |watcher| watcher.writing_ours())
                    };
                    if let Settled::At(at) = settled(&settle_first, &reading, &cut, &mine) {
                        if let Ok(mut watcher) = theirs.lock() {
                            watcher.settled(at);
                        }
                        on_fresh();
                    }
                    keeping.keeping(false);
                }
                mine.rest(period);
            }
        });
        Self {
            stop,
            behind,
            watcher,
            thread: Mutex::new(Some(thread)),
            count,
        }
    }

    pub fn close(&self) -> bool {
        self.stop.close();
        let Ok(mut held) = self.thread.lock() else {
            return false;
        };
        match held.take() {
            Some(thread) => crate::closing::join_within(thread, crate::closing::A_MOMENT),
            None => true,
        }
    }

    pub fn caught_up_within(&self, most: Duration) -> bool {
        self.behind.caught_up_within(most)
    }

    pub fn writing(&self) -> bool {
        let Some(count) = (self.count)() else {
            return false;
        };
        let Ok(mut watcher) = self.watcher.lock() else {
            return false;
        };
        watcher.writing(count);
        true
    }

    pub fn abandoned(&self) -> bool {
        let Ok(mut watcher) = self.watcher.lock() else {
            return false;
        };
        watcher.abandoned();
        true
    }

    pub fn ours(&self) -> bool {
        let Some(count) = (self.count)() else {
            return false;
        };
        let Ok(mut watcher) = self.watcher.lock() else {
            return false;
        };
        watcher.wrote(count);
        true
    }

    pub fn missed(&self) -> Option<u64> {
        self.watcher
            .lock()
            .ok()
            .and_then(|watcher| watcher.missed())
    }
}

fn settled(
    settle_first: &Settle,
    count: &Counting,
    cut: &impl Fn() -> bool,
    stop: &Rest,
) -> Settled {
    if !(settle_first.late)() {
        return Settled::At(None);
    }
    let clock = &settle_first.clock;
    let started = clock();
    settle(
        settle_first.settling,
        || count(),
        cut,
        || clock().saturating_sub(started),
        |pause| {
            stop.rest(pause);
        },
    )
}

impl Drop for Watching {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

#[cfg(test)]
#[path = "watching_test.rs"]
mod tests;
