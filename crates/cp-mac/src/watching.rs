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
    thread: Option<std::thread::JoinHandle<()>>,
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
            thread: Some(thread),
        }
    }

    pub fn start(on_fresh: impl FnMut() + Send + 'static) -> Self {
        Self::every(EVERY, on_fresh)
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
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn stopping_is_what_drop_does_and_it_waits_for_the_thread() {
        let watching = Watching::every(Duration::from_millis(5), || {});
        assert!(watching.thread.is_some());
        drop(watching);
    }

    #[test]
    fn a_long_period_does_not_make_stopping_take_that_long() {
        let watching = Watching::every(Duration::from_secs(3600), || {});
        let started = std::time::Instant::now();
        drop(watching);
        assert!(
            started.elapsed() < Duration::from_secs(1),
            "closing the application must not wait for the next poll"
        );
    }

    #[test]
    fn the_pasteboard_counts_one_per_copy_so_a_miss_can_be_told() {
        let watching = Watching::every(Duration::from_secs(3600), || {});
        assert_eq!(watching.missed(), Some(0));
    }
}
