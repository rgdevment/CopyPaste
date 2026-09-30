use super::*;

fn mac() -> Watcher {
    Watcher::new(Cadence::OnePerCopy)
}

#[test]
fn the_counter_at_its_limits_does_not_overflow() {
    let mut watcher = mac();
    watcher.tick(i64::MIN);
    let seen = watcher.tick(i64::MAX);
    assert!(matches!(seen, Seen::Fresh { .. }));

    let mut other = mac();
    other.tick(i64::MAX);
    assert_eq!(other.tick(i64::MAX), Seen::Nothing);
    assert!(matches!(
        other.tick(i64::MIN),
        Seen::Fresh { skipped: Some(0) }
    ));
}

#[test]
fn repeated_giant_jumps_do_not_wrap_the_counter_of_losses() {
    let mut watcher = mac();
    watcher.tick(0);
    for step in 1..=4 {
        watcher.tick(i64::MAX / 4 * step);
    }
    assert!(
        watcher.missed() > Some(0),
        "something was lost, and it is known"
    );
}

#[test]
fn a_negative_counter_is_handled_like_any_other() {
    let mut watcher = mac();
    watcher.tick(-100);
    assert_eq!(watcher.tick(-99), Seen::Fresh { skipped: Some(0) });
    assert_eq!(watcher.tick(-95), Seen::Fresh { skipped: Some(3) });
}

#[test]
fn the_first_look_only_sets_the_mark() {
    let mut watcher = mac();
    assert_eq!(watcher.tick(42), Seen::Nothing);
    assert_eq!(watcher.missed(), Some(0));
}

#[test]
fn a_still_counter_is_not_an_event() {
    let mut watcher = mac();
    watcher.tick(7);
    assert_eq!(watcher.tick(7), Seen::Nothing);
}

#[test]
fn one_step_is_one_copy_with_nothing_lost() {
    let mut watcher = mac();
    watcher.tick(7);
    assert_eq!(watcher.tick(8), Seen::Fresh { skipped: Some(0) });
    assert_eq!(watcher.missed(), Some(0));
}

#[test]
fn a_burst_is_counted_not_ignored() {
    let mut watcher = mac();
    watcher.tick(10);
    assert_eq!(watcher.tick(14), Seen::Fresh { skipped: Some(3) });
    assert_eq!(
        watcher.missed(),
        Some(3),
        "three copies happened and were lost"
    );
}

#[test]
fn our_own_write_is_discarded_exactly() {
    let mut watcher = mac();
    watcher.tick(20);
    watcher.wrote(21);
    assert_eq!(watcher.tick(21), Seen::Ours);
    assert_eq!(watcher.missed(), Some(0));
}

#[test]
fn a_real_copy_right_after_ours_is_not_swallowed() {
    let mut watcher = mac();
    watcher.tick(20);
    watcher.wrote(21);
    assert_eq!(watcher.tick(21), Seen::Ours);
    assert_eq!(
        watcher.tick(22),
        Seen::Fresh { skipped: Some(0) },
        "the blind window in 2.x used to swallow this one"
    );
}

#[test]
fn a_counter_that_goes_backwards_is_a_new_session_not_a_burst() {
    let mut watcher = mac();
    watcher.tick(5000);
    assert_eq!(watcher.tick(3), Seen::Fresh { skipped: Some(0) });
    assert_eq!(watcher.missed(), Some(0));
}
