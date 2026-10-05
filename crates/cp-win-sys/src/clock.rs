use windows::Win32::Foundation::{FILETIME, SYSTEMTIME};
use windows::Win32::System::Time::{
    FileTimeToSystemTime, SystemTimeToFileTime, SystemTimeToTzSpecificLocalTime,
};

const UNIX_EPOCH_TICKS: i64 = 116_444_736_000_000_000;
const TICKS_PER_MILLI: i64 = 10_000;
const TICKS_PER_SECOND: i64 = 10_000_000;

fn ticks_of(time: FILETIME) -> i64 {
    (i64::from(time.dwHighDateTime) << 32) | i64::from(time.dwLowDateTime)
}

pub fn utc_offset_at(millis: i64) -> i64 {
    let ticks = millis
        .saturating_mul(TICKS_PER_MILLI)
        .saturating_add(UNIX_EPOCH_TICKS);
    if ticks < 0 {
        return 0;
    }
    let universal = FILETIME {
        dwLowDateTime: (ticks & 0xFFFF_FFFF) as u32,
        dwHighDateTime: (ticks >> 32) as u32,
    };
    let mut utc = SYSTEMTIME::default();
    if unsafe { FileTimeToSystemTime(&universal, &mut utc) }.is_err() {
        return 0;
    }
    let mut local = SYSTEMTIME::default();
    if unsafe { SystemTimeToTzSpecificLocalTime(None, &utc, &mut local) }.is_err() {
        return 0;
    }
    let mut local_ticks = FILETIME::default();
    if unsafe { SystemTimeToFileTime(&local, &mut local_ticks) }.is_err() {
        return 0;
    }
    (ticks_of(local_ticks) - ticks) / TICKS_PER_SECOND
}

#[cfg(test)]
#[path = "clock_test.rs"]
mod tests;
