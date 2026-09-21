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
    fn a_late_answer_is_picked_up_by_a_later_wait_on_the_same_read() {
        let (release, gate) = std::sync::mpsc::channel::<()>();
        let pending = begin(move || {
            let _ = gate.recv();
            42
        });
        assert_eq!(pending.wait(Duration::from_millis(20)), None, "aún no");
        release.send(()).expect("la fuente contesta");
        assert_eq!(
            pending.wait(Duration::from_secs(5)),
            Some(42),
            "sin leer dos veces"
        );
        assert_eq!(
            pending.wait(Duration::from_millis(20)),
            None,
            "y una vez entregada, no hay más"
        );
    }

    #[test]
    fn a_read_that_panics_is_abandoned_like_any_other() {
        let seen = anything_within(Duration::from_millis(50), || -> u8 {
            panic!("el proveedor murió")
        });
        assert_eq!(seen, None);
    }
}
