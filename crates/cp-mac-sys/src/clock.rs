use objc2_foundation::{NSDate, NSTimeZone};

pub fn utc_offset_at(millis: i64) -> i64 {
    let date = NSDate::dateWithTimeIntervalSince1970(millis as f64 / 1_000.0);
    NSTimeZone::localTimeZone().secondsFromGMTForDate(&date) as i64
}

#[cfg(test)]
#[path = "clock_test.rs"]
mod tests;
