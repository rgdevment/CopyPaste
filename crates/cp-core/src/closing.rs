use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub const PATIENCE: Duration = Duration::from_millis(1_500);
const NAP: Duration = Duration::from_millis(5);

const _: () = assert!(PATIENCE.as_millis() >= 500);

pub fn join_within(thread: JoinHandle<()>, patience: Duration) -> bool {
    let until = Instant::now() + patience;
    while !thread.is_finished() {
        if Instant::now() >= until {
            return false;
        }
        std::thread::sleep(NAP.min(patience));
    }
    let _ = thread.join();
    true
}

pub fn join_soon(thread: JoinHandle<()>) -> bool {
    join_within(thread, PATIENCE)
}

#[cfg(test)]
#[path = "closing_test.rs"]
mod tests;
