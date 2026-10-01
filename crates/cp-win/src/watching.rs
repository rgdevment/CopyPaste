use cp_core::watch::Cadence;
use cp_core::watching::{NAP, Watching};
use cp_win_sys::clipboard;
use std::sync::Arc;
use std::time::Duration;

pub fn every(period: Duration, on_fresh: impl FnMut() + Send + 'static) -> Watching {
    Watching::every(
        Cadence::Opaque,
        period,
        NAP,
        Arc::new(clipboard::sequence),
        Arc::new(clipboard::writing_now),
        on_fresh,
    )
}

#[cfg(test)]
#[path = "watching_test.rs"]
mod tests;
