use super::*;

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
