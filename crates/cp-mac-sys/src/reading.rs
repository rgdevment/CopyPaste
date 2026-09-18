use std::time::Duration;

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
        assert_eq!(
            anything_within(Duration::from_millis(400), || vec![1, 2, 3]),
            Some(vec![1, 2, 3])
        );
    }

    #[test]
    fn what_does_not_answer_in_time_is_abandoned() {
        let seen = anything_within(Duration::from_millis(20), || {
            std::thread::sleep(Duration::from_millis(400));
            vec![9]
        });
        assert_eq!(seen, None);
    }

    #[test]
    fn abandoning_one_read_does_not_poison_the_next() {
        let abandoned = anything_within(Duration::from_millis(10), || {
            std::thread::sleep(Duration::from_millis(300));
            0
        });
        assert_eq!(abandoned, None);
        assert_eq!(anything_within(Duration::from_millis(400), || 7), Some(7));
    }

    #[test]
    fn a_read_that_panics_is_abandoned_like_any_other() {
        let seen = anything_within(Duration::from_millis(50), || -> u8 {
            panic!("el proveedor murió")
        });
        assert_eq!(seen, None);
    }
}
