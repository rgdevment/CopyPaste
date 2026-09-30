use std::time::Duration;

pub struct Pending<T> {
    hear: std::sync::mpsc::Receiver<T>,
}

impl<T> Pending<T> {
    pub fn wait(&self, patience: Duration) -> Option<T> {
        self.hear.recv_timeout(patience).ok()
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
