use cp_core::settling::SETTLING;
use cp_core::watch::Cadence;
use cp_core::watching::{NAP, Settle, Watching, steady};
use cp_win_sys::clipboard;
use std::sync::Arc;
use std::time::Duration;

const SERVED_BY_ITS_OWNER: &str = "DataObject";

pub fn every(period: Duration, on_fresh: impl FnMut() + Send + 'static) -> Watching {
    Watching::settling(
        Cadence::Opaque,
        period,
        NAP,
        Arc::new(clipboard::sequence),
        Arc::new(clipboard::writing_now),
        Settle {
            late: Arc::new(served_late),
            settling: SETTLING,
            clock: steady(),
        },
        on_fresh,
    )
}

fn served_late() -> bool {
    clipboard::has_owner() && clipboard::holds(SERVED_BY_ITS_OWNER)
}

#[cfg(test)]
#[path = "watching_test.rs"]
mod tests;
