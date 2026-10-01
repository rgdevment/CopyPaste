use cp_core::watch::{Cadence, Seen, Watcher};
use cp_mac_sys::pasteboard;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub const EVERY: Duration = Duration::from_millis(60);
const NAP: Duration = Duration::from_millis(10);

const _: () = assert!(EVERY.as_millis() >= 16);
const _: () = assert!(EVERY.as_millis() <= 250);
const _: () = assert!(NAP.as_millis() <= EVERY.as_millis());

pub struct Watching {
    stop: Arc<AtomicBool>,
    watcher: Arc<Mutex<Watcher>>,
    thread: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl Watching {
    pub fn every(period: Duration, mut on_fresh: impl FnMut() + Send + 'static) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let watcher = Arc::new(Mutex::new(Watcher::new(Cadence::OnePerCopy)));
        let mine = stop.clone();
        let theirs = watcher.clone();
        let thread = std::thread::spawn(move || {
            while !mine.load(Ordering::Relaxed) {
                let count = pasteboard::change_count_from_any_thread();
                if let Ok(mut watcher) = theirs.lock()
                    && let Seen::Fresh { .. } = watcher.tick(count)
                {
                    drop(watcher);
                    on_fresh();
                }
                let until = std::time::Instant::now() + period;
                while !mine.load(Ordering::Relaxed) && std::time::Instant::now() < until {
                    std::thread::sleep(period.min(NAP));
                }
            }
        });
        Self {
            stop,
            watcher,
            thread: Mutex::new(Some(thread)),
        }
    }

    pub fn start(on_fresh: impl FnMut() + Send + 'static) -> Self {
        Self::every(EVERY, on_fresh)
    }

    pub fn close(&self) -> bool {
        self.stop.store(true, Ordering::Relaxed);
        let Ok(mut held) = self.thread.lock() else {
            return false;
        };
        match held.take() {
            Some(thread) => cp_core::closing::join_soon(thread),
            None => true,
        }
    }

    pub fn writing(&self) -> bool {
        let Ok(mut watcher) = self.watcher.lock() else {
            return false;
        };
        watcher.writing(pasteboard::change_count_from_any_thread());
        true
    }

    pub fn ours(&self) -> bool {
        let Ok(mut watcher) = self.watcher.lock() else {
            return false;
        };
        watcher.wrote(pasteboard::change_count_from_any_thread());
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
