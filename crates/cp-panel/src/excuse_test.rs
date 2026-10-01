use super::*;

const EVERY: [Failure; 7] = [
    Failure::NotForeground,
    Failure::ForegroundTimeout,
    Failure::NoKeyboardFocus,
    Failure::TargetGone,
    Failure::SendDenied,
    Failure::TargetElevated,
    Failure::InputProtected,
];

#[test]
fn every_way_a_paste_fails_says_something_in_both_tongues() {
    for failure in EVERY {
        for english in [false, true] {
            let said = why_not(failure, english);
            assert!(!said.is_empty(), "{failure:?} says nothing");
            assert!(
                said.len() > 20,
                "{failure:?} says too little to act on: {said}"
            );
        }
    }
}

#[test]
fn each_tongue_is_its_own() {
    for failure in EVERY {
        assert_ne!(
            why_not(failure, false),
            why_not(failure, true),
            "{failure:?} was left in one tongue"
        );
    }
}

#[test]
fn a_window_that_went_away_is_not_told_like_one_that_refused() {
    assert_ne!(
        why_not(Failure::TargetGone, false),
        why_not(Failure::TargetElevated, false),
        "two causes a person would act on differently cannot share one sentence"
    );
    assert_ne!(
        why_not(Failure::NoKeyboardFocus, false),
        why_not(Failure::SendDenied, false)
    );
}

#[test]
fn the_reason_comes_first_because_the_tail_is_what_gets_cut() {
    for failure in EVERY {
        for english in [false, true] {
            let said = why_not(failure, english);
            let (reason, rest) = said.split_once(';').unwrap_or((said, ""));
            assert!(
                !rest.trim().is_empty(),
                "{failure:?} never says the copy survived: {said}"
            );
            assert!(
                reason.len() <= 52,
                "{failure:?} spends {} characters on the reason, and the footer elides well before that",
                reason.len()
            );
        }
    }
}

#[test]
fn what_is_said_always_ends_by_saying_the_copy_is_safe() {
    for failure in EVERY {
        for english in [false, true] {
            let said = why_not(failure, english);
            let safe = said.ends_with("sigue copiado") || said.ends_with("still copied");
            assert!(
                safe,
                "{failure:?} alarms without saying the copy survived: {said}"
            );
        }
    }
}

#[test]
fn the_two_ways_of_losing_the_front_are_told_the_same() {
    assert_eq!(
        why_not(Failure::NotForeground, false),
        why_not(Failure::ForegroundTimeout, false),
        "the difference between them is ours, not the person's"
    );
}
