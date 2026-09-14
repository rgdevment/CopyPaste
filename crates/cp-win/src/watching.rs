use cp_core::watch::{Cadence, Seen, Watcher};
use cp_win_sys::clipboard;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub const EVERY: Duration = Duration::from_millis(60);

const _: () = assert!(EVERY.as_millis() >= 16);
const _: () = assert!(EVERY.as_millis() <= 250);

pub struct Watching {
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Watching {
    pub fn every(period: Duration, mut on_fresh: impl FnMut() + Send + 'static) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let mine = stop.clone();
        let thread = std::thread::spawn(move || {
            let mut watcher = Watcher::new(Cadence::Opaque);
            while !mine.load(Ordering::Relaxed) {
                if let Some(count) = clipboard::sequence()
                    && let Seen::Fresh { .. } = watcher.tick(count)
                {
                    on_fresh();
                }
                std::thread::sleep(period);
            }
        });
        Self {
            stop,
            thread: Some(thread),
        }
    }

    pub fn start(on_fresh: impl FnMut() + Send + 'static) -> Self {
        Self::every(EVERY, on_fresh)
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
}
