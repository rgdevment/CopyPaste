use crate::resting::Rest;
use crate::watch::{Cadence, Seen, Watcher};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub const EVERY: Duration = Duration::from_millis(60);
pub const NAP: Duration = Duration::from_millis(10);

const _: () = assert!(EVERY.as_millis() >= 16);
const _: () = assert!(EVERY.as_millis() <= 250);
const _: () = assert!(NAP.as_millis() <= EVERY.as_millis());

pub type Counting = Arc<dyn Fn() -> Option<i64> + Send + Sync>;
pub type Busy = Arc<dyn Fn() -> bool + Send + Sync>;

pub struct Watching {
    stop: Arc<Rest>,
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
        mut on_fresh: impl FnMut() + Send + 'static,
    ) -> Self {
        let stop = Arc::new(Rest::new());
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
                    on_fresh();
                }
                mine.rest(period);
            }
        });
        Self {
            stop,
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

impl Drop for Watching {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

#[cfg(test)]
#[path = "watching_test.rs"]
mod tests;
