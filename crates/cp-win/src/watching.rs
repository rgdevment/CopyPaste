use cp_core::watch::{Cadence, Seen, Watcher};
use cp_win_sys::clipboard;
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
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Watching {
    pub fn every(period: Duration, mut on_fresh: impl FnMut() + Send + 'static) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let watcher = Arc::new(Mutex::new(Watcher::new(Cadence::Opaque)));
        let mine = stop.clone();
        let theirs = watcher.clone();
        let thread = std::thread::spawn(move || {
            while !mine.load(Ordering::Relaxed) {
                if clipboard::writing_now() {
                    std::thread::sleep(NAP);
                    continue;
                }
                if let Some(count) = clipboard::sequence()
                    && let Ok(mut watcher) = theirs.lock()
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
            thread: Some(thread),
        }
    }

    pub fn start(on_fresh: impl FnMut() + Send + 'static) -> Self {
        Self::every(EVERY, on_fresh)
    }

    pub fn writing(&self) -> bool {
        let Some(count) = clipboard::sequence() else {
            return false;
        };
        let Ok(mut watcher) = self.watcher.lock() else {
            return false;
        };
        watcher.writing(count);
        true
    }

    pub fn ours(&self) -> bool {
        let Some(count) = clipboard::sequence() else {
            return false;
        };
        let Ok(mut watcher) = self.watcher.lock() else {
            return false;
        };
        watcher.wrote(count);
        true
    }
}

impl Drop for Watching {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
#[path = "watching_test.rs"]
mod tests;
