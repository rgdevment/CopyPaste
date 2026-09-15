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
mod tests {
    use super::*;

    #[test]
    fn the_period_keeps_up_with_a_person_without_spinning() {
        assert!(
            EVERY <= Duration::from_millis(250),
            "una copia no puede tardar en verse"
        );
        assert!(
            EVERY >= Duration::from_millis(16),
            "ni sondear mas rapido que la pantalla"
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
            "cerrar la aplicacion no puede esperar al siguiente sondeo"
        );
    }
}
