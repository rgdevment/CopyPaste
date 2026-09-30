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
    match anything_within(patience, read) {
        Some(Some(bytes)) => Reading::Delivered(bytes),
        Some(None) => Reading::Empty,
        None => Reading::TooSlow,
    }
}

pub struct Pending<T> {
    hear: std::sync::mpsc::Receiver<T>,
}

pub enum Waited<T> {
    Answered(T),
    StillRunning,
    Gone,
}

impl<T> Pending<T> {
    pub fn wait(&self, patience: Duration) -> Option<T> {
        match self.waited(patience) {
            Waited::Answered(what) => Some(what),
            Waited::StillRunning | Waited::Gone => None,
        }
    }

    pub fn waited(&self, patience: Duration) -> Waited<T> {
        match self.hear.recv_timeout(patience) {
            Ok(what) => Waited::Answered(what),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Waited::StillRunning,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Waited::Gone,
        }
    }
}

pub fn begin<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> Pending<T> {
    let (tell, hear) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tell.send(work());
    });
    Pending { hear }
}

pub fn anything_within<T: Send + 'static>(
    patience: Duration,
    work: impl FnOnce() -> T + Send + 'static,
) -> Option<T> {
    begin(work).wait(patience)
}

#[cfg(test)]
#[path = "reading_test.rs"]
mod tests;
