use super::*;

#[test]
fn the_first_restart_is_granted_without_waiting() {
    assert_eq!(asked_again(0, None), Verdict::Light { tries: 1 });
}

#[test]
fn a_second_press_right_away_waits_instead_of_starting_another_panel() {
    assert_eq!(
        asked_again(1, Some(Duration::ZERO)),
        Verdict::Wait {
            left: waits_after(1)
        }
    );
    assert!(
        matches!(
            asked_again(1, Some(Duration::from_millis(60))),
            Verdict::Wait { .. }
        ),
        "the storm was seven restarts in three minutes"
    );
}

#[test]
fn a_wait_says_only_what_is_left_of_the_turn() {
    let gap = Duration::from_millis(100);
    let Verdict::Wait { left } = asked_again(2, Some(gap)) else {
        panic!("a restart 100 ms after another has to wait");
    };
    assert_eq!(left, waits_after(2) - gap);
    assert_eq!(
        asked_again(2, Some(gap + left)),
        Verdict::Light { tries: 3 },
        "once the wait is served the restart is granted"
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
fn past_the_cap_nothing_is_started_until_the_count_is_forgotten() {
    let gap = Duration::from_secs(30);
    let Verdict::Enough { left } = asked_again(AT_MOST, Some(gap)) else {
        panic!("past the cap nothing is started");
    };
    assert_eq!(
        asked_again(AT_MOST, Some(gap + left)),
        Verdict::Light { tries: 1 },
        "after the told wait the panel is tried again from the start"
    );
    assert!(matches!(
        asked_again(AT_MOST + 3, None),
        Verdict::Enough { .. }
    ));
}

#[test]
fn a_quiet_minute_forgives_the_whole_count() {
    assert_eq!(
        asked_again(AT_MOST, Some(FORGETS_AFTER + Duration::from_secs(1))),
        Verdict::Light { tries: 1 },
        "a panel that failed an hour ago says nothing about this press"
    );
}

#[test]
fn a_panel_that_fell_is_told_from_one_that_was_asked_to_end() {
    assert!(fell(None, Some(6)), "an abort is a fall");
    assert!(fell(None, Some(9)), "a kill nobody here sent is a fall");
    assert!(
        fell(Some(101), None),
        "a panic that could not abort is a fall"
    );
    assert!(
        !fell(Some(1), None),
        "a panel that says it cannot start would only fail again"
    );
    assert!(
        fell(None, None),
        "an end nobody can read is taken as a fall"
    );
    assert!(!fell(Some(0), None), "a clean end was asked for");
    assert!(
        !fell(None, Some(15)),
        "the system ends its programs this way when the session closes"
    );
}
