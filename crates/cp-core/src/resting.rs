use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

#[derive(Default)]
struct State {
    woken: bool,
    closed: bool,
}

#[derive(Default)]
pub struct Rest {
    state: Mutex<State>,
    signal: Condvar,
}

impl Rest {
    pub fn new() -> Self {
        Self::default()
    }

    fn held(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn wake(&self) {
        self.held().woken = true;
        self.signal.notify_all();
    }

    pub fn close(&self) {
        self.held().closed = true;
        self.signal.notify_all();
    }

    pub fn closed(&self) -> bool {
        self.held().closed
    }

    pub fn rest(&self, most: Duration) -> bool {
        let held = self.held();
        let (mut held, _) = self
            .signal
            .wait_timeout_while(held, most, |state| !state.woken && !state.closed)
            .unwrap_or_else(PoisonError::into_inner);
        held.woken = false;
        !held.closed
    }
}

#[cfg(test)]
#[path = "resting_test.rs"]
mod tests;
