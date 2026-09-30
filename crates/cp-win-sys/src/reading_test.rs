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
fn a_late_answer_is_picked_up_by_a_later_wait_on_the_same_read() {
    let (release, gate) = std::sync::mpsc::channel::<()>();
    let pending = begin(move || {
        let _ = gate.recv();
        42
    });
    assert_eq!(pending.wait(Duration::from_millis(20)), None, "not yet");
    release.send(()).expect("the source answers");
    assert_eq!(
        pending.wait(Duration::from_secs(5)),
        Some(42),
        "sin leer dos veces"
    );
    assert_eq!(
        pending.wait(Duration::from_millis(20)),
        None,
        "and once handed over, there is no more"
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
    let seen = within(Duration::from_millis(50), || panic!("the provider died"));
    assert_eq!(seen, Reading::TooSlow);
}

#[test]
fn waiting_tells_a_read_still_running_from_one_whose_answer_is_spent() {
    let (release, gate) = std::sync::mpsc::channel::<()>();
    let pending = begin(move || {
        let _ = gate.recv();
        42
    });
    assert!(
        matches!(
            pending.waited(Duration::from_millis(20)),
            Waited::StillRunning
        ),
        "still working"
    );
    release.send(()).expect("the source answers");
    assert!(matches!(
        pending.waited(Duration::from_secs(5)),
        Waited::Answered(42)
    ));
    assert!(
        matches!(pending.waited(Duration::from_secs(5)), Waited::Gone),
        "a spent channel is not an expired deadline"
    );
}

#[test]
fn a_read_that_panics_is_gone_not_still_running() {
    let pending = begin(|| -> u8 { panic!("the provider died") });
    assert!(matches!(
        pending.waited(Duration::from_secs(5)),
        Waited::Gone
    ));
}
