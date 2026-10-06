use std::time::Duration;

pub const AT_MOST: u32 = 5;
pub const FIRST_WAIT: Duration = Duration::from_millis(250);
pub const FORGETS_AFTER: Duration = Duration::from_secs(60);

const _: () = assert!(AT_MOST >= 2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Light { tries: u32 },
    Wait,
    Enough,
}

pub fn asked_again(tries: u32, since: Option<Duration>) -> Verdict {
    if forgotten(since) {
        return Verdict::Light { tries: 1 };
    }
    if tries >= AT_MOST {
        return Verdict::Enough;
    }
    if since.is_some_and(|gap| gap < waits_after(tries)) {
        return Verdict::Wait;
    }
    Verdict::Light { tries: tries + 1 }
}

fn forgotten(since: Option<Duration>) -> bool {
    since.is_some_and(|gap| gap > FORGETS_AFTER)
}

pub fn left_to_wait(tries: u32, since: Option<Duration>) -> Duration {
    match since {
        Some(gap) if !forgotten(since) => waits_after(tries).saturating_sub(gap),
        _ => Duration::ZERO,
    }
}

pub fn waits_after(tries: u32) -> Duration {
    FIRST_WAIT * 2_u32.pow(tries.min(4))
}

#[cfg(test)]
#[path = "reviving_test.rs"]
mod tests;
