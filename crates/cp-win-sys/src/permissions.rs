use crate::clipboard;
use crate::frontmost;

pub const MEDIUM: u32 = 0x2000;
pub const HIGH: u32 = 0x3000;

const _: () = assert!(MEDIUM < HIGH);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Readiness {
    pub reaches_window_station: bool,
    pub integrity: Option<u32>,
}

impl Readiness {
    pub fn probe() -> Self {
        Self {
            reaches_window_station: clipboard::sequence().is_some(),
            integrity: frontmost::integrity_of(std::process::id()),
        }
    }

    pub fn can_watch(&self) -> bool {
        self.reaches_window_station
    }

    pub fn can_paste_into(&self, target: u32) -> bool {
        self.integrity.is_some_and(|ours| ours >= target)
    }

    pub fn is_elevated(&self) -> bool {
        self.integrity.is_some_and(|ours| ours >= HIGH)
    }
}

#[cfg(test)]
#[path = "permissions_test.rs"]
mod tests;
