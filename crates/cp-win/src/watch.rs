pub fn sequence(raw: u32) -> Option<i64> {
    (raw != 0).then_some(i64::from(raw))
}

#[cfg(test)]
#[path = "watch_test.rs"]
mod tests;
