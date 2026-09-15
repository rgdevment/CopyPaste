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

pub fn anything_within<T: Send + 'static>(
    patience: Duration,
    work: impl FnOnce() -> T + Send + 'static,
) -> Option<T> {
    let (tell, hear) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tell.send(work());
    });
    hear.recv_timeout(patience).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_answers_in_time_is_delivered() {
        let seen = within(PATIENCE, || Some(vec![1, 2, 3]));
        assert_eq!(seen, Reading::Delivered(vec![1, 2, 3]));
        assert_eq!(seen.bytes(), Some(vec![1, 2, 3]));
    }

    #[test]
    fn what_answers_nothing_is_empty_not_slow() {
        let seen = within(PATIENCE, || None);
        assert_eq!(seen, Reading::Empty);
        assert!(!seen.is_too_slow());
        assert_eq!(seen.bytes(), None);
    }

    #[test]
    fn what_does_not_answer_in_time_is_abandoned() {
        let seen = within(Duration::from_millis(20), || {
            std::thread::sleep(Duration::from_millis(400));
            Some(vec![9])
        });
        assert_eq!(seen, Reading::TooSlow);
        assert!(seen.is_too_slow());
        assert_eq!(seen.bytes(), None);
    }

    #[test]
    fn abandoning_one_read_does_not_poison_the_next() {
        let abandoned = within(Duration::from_millis(10), || {
            std::thread::sleep(Duration::from_millis(300));
            Some(vec![0])
        });
        assert!(abandoned.is_too_slow());
        assert_eq!(
            within(PATIENCE, || Some(vec![7])),
            Reading::Delivered(vec![7])
        );
    }

    #[test]
    fn the_patience_sits_between_the_two_measured_worlds() {
        let good = Duration::from_millis(2);
        let hung = Duration::from_millis(30_000);
        assert!(good < PATIENCE, "lo que entrega responde en 1,5 ms");
        assert!(PATIENCE < hung, "lo que cuelga tarda 30 s en rendirse");
    }

    #[test]
    fn a_read_that_panics_is_abandoned_like_any_other() {
        let seen = within(Duration::from_millis(50), || panic!("el proveedor murió"));
        assert_eq!(seen, Reading::TooSlow);
    }
}
