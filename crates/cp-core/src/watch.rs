#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seen {
    Nothing,
    Ours,
    Fresh { skipped: Option<u64> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cadence {
    OnePerCopy,
    Opaque,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Retry {
    pub attempts: u8,
    pub pause: std::time::Duration,
}

pub const RETRY: Retry = Retry {
    attempts: 5,
    pause: std::time::Duration::from_millis(150),
};

const _: () = assert!(RETRY.attempts >= 2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Retried<T> {
    Done(T),
    Superseded,
    Exhausted,
}

pub fn insist<T>(
    retry: Retry,
    count: impl Fn() -> i64,
    mut attempt: impl FnMut() -> Option<T>,
    pause: impl Fn(std::time::Duration),
) -> Retried<T> {
    let started_at = count();
    for done in 0..retry.attempts.max(1) {
        if done > 0 {
            pause(retry.pause);
            if count() != started_at {
                return Retried::Superseded;
            }
        }
        if let Some(got) = attempt() {
            return Retried::Done(got);
        }
    }
    Retried::Exhausted
}

#[derive(Debug)]
pub struct Watcher {
    last: Option<i64>,
    ours: Option<i64>,
    missed: u64,
    cadence: Cadence,
}

impl Watcher {
    pub fn new(cadence: Cadence) -> Self {
        Self {
            last: None,
            ours: None,
            missed: 0,
            cadence,
        }
    }

    pub fn wrote(&mut self, count: i64) {
        self.ours = Some(count);
    }

    pub fn tick(&mut self, count: i64) -> Seen {
        let Some(last) = self.last else {
            self.last = Some(count);
            return Seen::Nothing;
        };
        if count == last {
            return Seen::Nothing;
        }
        self.last = Some(count);
        let ours = self.ours.take();
        if ours == Some(count) {
            return Seen::Ours;
        }
        let skipped = match self.cadence {
            Cadence::OnePerCopy => {
                let skipped = count.saturating_sub(last).saturating_sub(1).max(0) as u64;
                self.missed = self.missed.saturating_add(skipped);
                Some(skipped)
            }
            Cadence::Opaque => None,
        };
        Seen::Fresh { skipped }
    }

    pub fn missed(&self) -> Option<u64> {
        match self.cadence {
            Cadence::OnePerCopy => Some(self.missed),
            Cadence::Opaque => None,
        }
    }

    pub fn cadence(&self) -> Cadence {
        self.cadence
    }
}

#[cfg(test)]
#[path = "watch_insisting.rs"]
mod insisting;

#[cfg(test)]
#[path = "watch_test.rs"]
mod tests;

#[cfg(test)]
#[path = "watch_windows_counter.rs"]
mod windows_counter;

#[cfg(test)]
#[path = "watch_properties.rs"]
mod properties;
