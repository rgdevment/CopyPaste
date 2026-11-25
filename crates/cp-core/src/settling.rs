use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settling {
    pub quiet: Duration,
    pub ceiling: Duration,
    pub step: Duration,
}

pub const SETTLING: Settling = Settling {
    quiet: Duration::from_millis(300),
    ceiling: Duration::from_millis(1_000),
    step: Duration::from_millis(20),
};

const _: () = assert!(SETTLING.step.as_millis() < SETTLING.quiet.as_millis());
const _: () = assert!(SETTLING.quiet.as_millis() < SETTLING.ceiling.as_millis());
const _: () = assert!(SETTLING.ceiling.as_millis() <= 1_000);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Settled {
    At(Option<i64>),
    Cut,
}

pub fn settle(
    settling: Settling,
    count: impl Fn() -> Option<i64>,
    cut: impl Fn() -> bool,
    elapsed: impl Fn() -> Duration,
    pause: impl Fn(Duration),
) -> Settled {
    let mut last = count();
    let mut quiet_since = elapsed();
    loop {
        if cut() {
            return Settled::Cut;
        }
        let now = elapsed();
        if now.saturating_sub(quiet_since) >= settling.quiet || now >= settling.ceiling {
            return Settled::At(last);
        }
        pause(settling.step);
        let seen = count();
        if seen.is_some() && seen != last {
            last = seen;
            quiet_since = elapsed();
        }
    }
}

#[cfg(test)]
#[path = "settling_test.rs"]
mod tests;
