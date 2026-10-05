use objc2_foundation::NSTimeZone;

pub fn utc_offset_seconds() -> i64 {
    NSTimeZone::localTimeZone().secondsFromGMT() as i64
}

#[cfg(test)]
#[path = "clock_test.rs"]
mod tests;
