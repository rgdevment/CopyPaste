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

#[test]
fn the_whole_stretch_of_our_write_is_ours() {
    let mut watcher = windows();
    watcher.tick(32259);
    watcher.writing(32259);
    assert_eq!(watcher.tick(32260), Seen::Ours, "EmptyClipboard moved it");
    assert_eq!(watcher.tick(32261), Seen::Ours, "SetClipboardData moved it");
    watcher.wrote(32264);
    assert_eq!(
        watcher.tick(32264),
        Seen::Ours,
        "the three synthesized ones"
    );
    assert_eq!(watcher.tick(32267), Seen::Fresh { skipped: None });
}

#[test]
fn a_poll_inside_the_closed_stretch_does_not_shut_it_early() {
    let mut watcher = windows();
    watcher.tick(32259);
    watcher.writing(32259);
    watcher.wrote(32264);
    assert_eq!(watcher.tick(32262), Seen::Ours);
    assert_eq!(watcher.tick(32264), Seen::Ours, "the end is still ours");
    assert_eq!(watcher.tick(32267), Seen::Fresh { skipped: None });
}

#[test]
fn a_write_that_never_landed_does_not_deafen_the_watcher() {
    let mut watcher = windows();
    watcher.tick(100);
    watcher.writing(100);
    watcher.wrote(100);
    assert_eq!(
        watcher.tick(103),
        Seen::Fresh { skipped: None },
        "the clipboard was never taken, so what follows is theirs"
    );
}

#[test]
fn a_counter_below_the_open_stretch_is_a_new_session() {
    let mut watcher = windows();
    watcher.tick(100);
    watcher.writing(100);
    assert_eq!(watcher.tick(60), Seen::Fresh { skipped: None });
    assert_eq!(watcher.tick(61), Seen::Fresh { skipped: None });
}
