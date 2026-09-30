use std::time::Duration;

pub const PATIENCE: Duration = Duration::from_millis(100);

const _: () = assert!(PATIENCE.as_millis() > 10);
const _: () = assert!(PATIENCE.as_millis() < 30_000);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reading {
    Delivered(Vec<u8>),
    Empty,
    TooSlow,
}

impl Reading {
    pub fn bytes(self) -> Option<Vec<u8>> {
        match self {
            Reading::Delivered(bytes) => Some(bytes),
            Reading::Empty | Reading::TooSlow => None,
        }
    }

    pub fn is_too_slow(&self) -> bool {
        matches!(self, Reading::TooSlow)
    }
}

pub fn within(
    patience: Duration,
    read: impl FnOnce() -> Option<Vec<u8>> + Send + 'static,
) -> Reading {
    let (tell, hear) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tell.send(read());
    });
    match hear.recv_timeout(patience) {
        Ok(Some(bytes)) => Reading::Delivered(bytes),
        Ok(None) => Reading::Empty,
        Err(_) => Reading::TooSlow,
    }
}

#[cfg(test)]
#[path = "reading_test.rs"]
mod tests;
