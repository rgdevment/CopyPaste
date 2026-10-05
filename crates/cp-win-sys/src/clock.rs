use windows::Win32::Foundation::FILETIME;
use windows::Win32::System::SystemInformation::{GetLocalTime, GetSystemTime};
use windows::Win32::System::Time::SystemTimeToFileTime;

const TICKS_PER_MINUTE: f64 = 600_000_000.0;

pub fn utc_offset_seconds() -> i64 {
    let local = unsafe { GetLocalTime() };
    let utc = unsafe { GetSystemTime() };
    let mut local_ticks = FILETIME::default();
    let mut utc_ticks = FILETIME::default();
    if unsafe { SystemTimeToFileTime(&local, &mut local_ticks) }.is_err()
        || unsafe { SystemTimeToFileTime(&utc, &mut utc_ticks) }.is_err()
    {
        return 0;
    }
    let ticks =
        |time: FILETIME| (i64::from(time.dwHighDateTime) << 32) | i64::from(time.dwLowDateTime);
    let minutes =
        ((ticks(local_ticks) - ticks(utc_ticks)) as f64 / TICKS_PER_MINUTE).round() as i64;
    minutes * 60
}

#[cfg(test)]
#[path = "clock_test.rs"]
mod tests;
