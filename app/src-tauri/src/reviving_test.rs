use super::*;

#[test]
fn the_first_restart_is_granted_without_waiting() {
    assert_eq!(asked_again(0, None), Verdict::Light { tries: 1 });
}

#[test]
fn a_second_press_right_away_waits_instead_of_starting_another_panel() {
    assert_eq!(asked_again(1, Some(Duration::ZERO)), Verdict::Wait);
    assert_eq!(
        asked_again(1, Some(Duration::from_millis(60))),
        Verdict::Wait,
        "the storm was seven restarts in three minutes"
    );
}

#[test]
fn the_wait_grows_with_every_attempt() {
    let waits: Vec<Duration> = (0..6).map(waits_after).collect();
    for pair in waits.windows(2) {
        assert!(pair[1] >= pair[0], "{waits:?}");
    }
    assert_eq!(waits_after(0), FIRST_WAIT);
    assert!(waits_after(4) >= FIRST_WAIT * 16);
    assert_eq!(
        waits_after(9),
        waits_after(4),
        "the wait stops growing so it never becomes a lifetime"
    );
}

#[test]
fn once_the_wait_is_over_the_next_restart_is_granted() {
    assert_eq!(
        asked_again(2, Some(waits_after(2))),
        Verdict::Light { tries: 3 }
    );
}

#[test]
fn past_the_cap_nothing_is_started_however_long_the_gap() {
    assert_eq!(
        asked_again(AT_MOST, Some(Duration::from_secs(30))),
        Verdict::Enough
    );
    assert_eq!(asked_again(AT_MOST + 3, None), Verdict::Enough);
}

#[test]
fn a_quiet_minute_forgives_the_whole_count() {
    assert_eq!(
        asked_again(AT_MOST, Some(FORGETS_AFTER + Duration::from_secs(1))),
        Verdict::Light { tries: 1 },
        "a panel that failed an hour ago says nothing about this press"
    );
}
