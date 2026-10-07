use std::sync::atomic::{AtomicIsize, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Leaving {
    Back,
    Away,
}

pub fn aimed_at(in_front: isize, before: isize, showing: bool) -> isize {
    if in_front == 0 && showing {
        before
    } else {
        in_front
    }
}

pub fn handed_back(
    ahead: &AtomicIsize,
    leaving: Leaving,
    ours_in_front: impl FnOnce() -> bool,
) -> Option<isize> {
    let was = ahead.swap(0, Ordering::Relaxed);
    (leaving == Leaving::Back && was != 0 && ours_in_front()).then_some(was)
}

#[cfg(test)]
#[path = "aside_test.rs"]
mod tests;
