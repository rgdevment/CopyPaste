use cp_core::watch::Cadence;
use cp_core::watching::{NAP, Watching};
use cp_mac_sys::pasteboard;
use std::sync::Arc;
use std::time::Duration;

pub fn every(period: Duration, on_fresh: impl FnMut() + Send + 'static) -> Watching {
    Watching::every(
        Cadence::OnePerCopy,
        period,
        NAP,
        Arc::new(|| Some(pasteboard::change_count_from_any_thread())),
        Arc::new(|| false),
        on_fresh,
    )
}

#[cfg(test)]
#[path = "watching_test.rs"]
mod tests;
