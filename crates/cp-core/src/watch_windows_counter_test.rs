use super::*;

fn windows() -> Watcher {
    Watcher::new(Cadence::Opaque)
}

#[test]
fn one_ordinary_copy_is_not_a_burst_of_four() {
    let mut watcher = windows();
    watcher.tick(51);
    assert_eq!(watcher.tick(56), Seen::Fresh { skipped: None });
    assert_eq!(watcher.missed(), None, "it is not known, and it says so");

    let mut as_if_mac = Watcher::new(Cadence::OnePerCopy);
    as_if_mac.tick(51);
    assert_eq!(
        as_if_mac.tick(56),
        Seen::Fresh { skipped: Some(4) },
        "the wrong cadence invents four lost copies"
    );
}

#[test]
fn every_measured_jump_is_a_single_copy() {
    for jump in [5, 9, 11, 12] {
        let mut watcher = windows();
        watcher.tick(1000);
        assert_eq!(
            watcher.tick(1000 + jump),
            Seen::Fresh { skipped: None },
            "a jump of {jump} is still one copy"
        );
    }
}

#[test]
fn a_counter_that_jumps_backwards_still_reports_each_poll() {
    let mut watcher = windows();
    watcher.tick(1000);
    assert_eq!(watcher.tick(0), Seen::Fresh { skipped: None });
    assert_eq!(watcher.tick(1005), Seen::Fresh { skipped: None });
    assert_eq!(watcher.missed(), None);
}

#[test]
fn our_own_write_is_still_discarded_exactly() {
    let mut watcher = windows();
    watcher.tick(100);
    watcher.wrote(112);
    assert_eq!(watcher.tick(112), Seen::Ours);
    assert_eq!(watcher.tick(124), Seen::Fresh { skipped: None });
}

#[test]
fn a_still_counter_is_still_not_an_event() {
    let mut watcher = windows();
    watcher.tick(7);
    assert_eq!(watcher.tick(7), Seen::Nothing);
}

#[test]
fn the_watcher_says_which_counter_it_is_reading() {
    assert_eq!(windows().cadence(), Cadence::Opaque);
    assert_eq!(
        Watcher::new(Cadence::OnePerCopy).cadence(),
        Cadence::OnePerCopy
    );
}
