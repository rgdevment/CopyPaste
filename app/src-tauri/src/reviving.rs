use std::time::Duration;

pub const AT_MOST: u32 = 5;
pub const FIRST_WAIT: Duration = Duration::from_millis(250);
pub const FORGETS_AFTER: Duration = Duration::from_secs(60);

const ASKED_TO_END: i32 = 15;
const COULD_NOT_START: i32 = 1;

const _: () = assert!(AT_MOST >= 2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Light { tries: u32 },
    Wait { left: Duration },
    Enough { left: Duration },
}

pub fn asked_again(tries: u32, since: Option<Duration>) -> Verdict {
    if forgotten(since) {
        return Verdict::Light { tries: 1 };
    }
    if tries >= AT_MOST {
        return Verdict::Enough {
            left: FORGETS_AFTER.saturating_sub(since.unwrap_or_default()),
        };
    }
    if let Some(gap) = since
        && gap < waits_after(tries)
    {
        return Verdict::Wait {
            left: waits_after(tries) - gap,
        };
    }
    Verdict::Light { tries: tries + 1 }
}

fn forgotten(since: Option<Duration>) -> bool {
    since.is_some_and(|gap| gap >= FORGETS_AFTER)
}

pub fn waits_after(tries: u32) -> Duration {
    FIRST_WAIT * 2_u32.pow(tries.min(4))
}

pub fn fell(code: Option<i32>, signal: Option<i32>) -> bool {
    match (code, signal) {
        (_, Some(ASKED_TO_END)) => false,
        (_, Some(_)) => true,
        (Some(code), None) => code != 0 && code != COULD_NOT_START,
        (None, None) => true,
    }
}

#[cfg(test)]
#[path = "reviving_test.rs"]
mod tests;
