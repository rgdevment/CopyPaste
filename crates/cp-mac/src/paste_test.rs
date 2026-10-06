use super::*;
use std::time::{Duration, Instant};

fn ready(can_post: bool, accessibility: bool, secure_input: bool) -> Readiness {
    Readiness {
        can_post,
        accessibility,
        secure_input,
    }
}

#[test]
fn a_keystroke_is_the_way_when_nothing_is_in_the_way() {
    assert_eq!(route_of(&ready(true, true, false)), Some(Route::Keystroke));
}

#[test]
fn without_posting_events_the_menu_still_serves() {
    assert_eq!(route_of(&ready(false, true, false)), Some(Route::Menu));
}

#[test]
fn a_session_wide_flag_does_not_decide_the_route_in_advance() {
    assert_eq!(
        route_of(&ready(true, false, true)),
        Some(Route::Keystroke),
        "any app holding secure input would otherwise stop every paste before it is tried"
    );
}

#[test]
fn protected_input_sends_us_round_by_the_menu() {
    assert_eq!(
        around_protected_input(Route::Keystroke, true),
        Some(Route::Menu),
        "a posted keystroke is dropped while the system protects what is typed, a menu press is not"
    );
    assert_eq!(
        around_protected_input(Route::Menu, true),
        Some(Route::Menu),
        "the menu was already the way in"
    );
}

#[test]
fn protected_input_without_the_menu_is_no_way_at_all() {
    assert_eq!(
        around_protected_input(Route::Keystroke, false),
        None,
        "claiming the keystroke here would report a paste that never arrived"
    );
}

#[test]
fn neither_way_is_no_way() {
    assert_eq!(route_of(&ready(false, false, false)), None);
}

fn somewhere() -> Destination {
    Destination {
        pid: 4242,
        bundle_id: None,
    }
}

fn fresh(route: Route) -> (Pasting, Instant) {
    let now = Instant::now();
    (Pasting::new(route, somewhere(), now), now)
}

#[test]
fn a_target_already_in_front_with_no_keys_held_is_pasted_at_once() {
    let (mut pasting, now) = fresh(Route::Keystroke);
    assert_eq!(pasting.next(true, Focus::OnTarget, false, now), Step::Send);
}

#[test]
fn a_target_that_is_gone_stops_the_paste_before_anything_else() {
    let (mut pasting, now) = fresh(Route::Keystroke);
    assert_eq!(
        pasting.next(false, Focus::OnTarget, false, now),
        Step::Stop(Failure::TargetGone)
    );
}

#[test]
fn a_target_that_dies_while_being_raised_is_reported_as_gone() {
    let (mut pasting, now) = fresh(Route::Keystroke);
    assert_eq!(
        pasting.next(true, Focus::Elsewhere, false, now),
        Step::Raise
    );
    assert_eq!(
        pasting.next(false, Focus::Elsewhere, false, now),
        Step::Stop(Failure::TargetGone)
    );
}

#[test]
fn a_target_behind_another_app_is_raised_first() {
    let (mut pasting, now) = fresh(Route::Keystroke);
    assert_eq!(
        pasting.next(true, Focus::Elsewhere, false, now),
        Step::Raise
    );
}

#[test]
fn not_knowing_who_is_in_front_is_treated_as_not_being_there() {
    let (mut pasting, now) = fresh(Route::Keystroke);
    assert_eq!(pasting.next(true, Focus::Unknown, false, now), Step::Raise);
}

#[test]
fn once_the_target_comes_forward_the_paste_goes_out() {
    let (mut pasting, now) = fresh(Route::Keystroke);
    assert_eq!(
        pasting.next(true, Focus::Elsewhere, false, now),
        Step::Raise
    );
    assert_eq!(
        pasting.next(true, Focus::Elsewhere, false, now),
        Step::Raise
    );
    assert_eq!(pasting.next(true, Focus::OnTarget, false, now), Step::Send);
}

#[test]
fn a_target_that_never_comes_forward_gives_up_after_every_retry() {
    let (mut pasting, now) = fresh(Route::Keystroke);
    let mut raised = 0;
    loop {
        match pasting.next(true, Focus::Elsewhere, false, now) {
            Step::Raise => raised += 1,
            Step::Stop(why) => {
                assert_eq!(why, Failure::ForegroundTimeout);
                break;
            }
            other => panic!("it should only raise or stop, and it said {other:?}"),
        }
        assert!(raised <= 64, "it never gave up");
    }
    assert_eq!(raised, usize::from(cp_core::paste::RACE_RETRIES) + 1);
}

#[test]
fn the_first_raise_is_free_and_does_not_spend_a_retry() {
    let (mut once, now) = fresh(Route::Keystroke);
    once.next(true, Focus::Elsewhere, false, now);
    let (mut never, _) = fresh(Route::Keystroke);
    let mut left_once = 0;
    while once.next(true, Focus::Elsewhere, false, now) == Step::Raise {
        left_once += 1;
    }
    let mut left_never = 0;
    while never.next(true, Focus::Elsewhere, false, now) == Step::Raise {
        left_never += 1;
    }
    assert_eq!(left_never, left_once + 1);
}

#[test]
fn held_modifiers_are_waited_for_in_short_looks() {
    let (mut pasting, now) = fresh(Route::Keystroke);
    assert_eq!(
        pasting.next(true, Focus::OnTarget, true, now),
        Step::Wait(KEYS_LOOKED_AT)
    );
    let later = now + KEYS_LOOKED_AT;
    assert_eq!(
        pasting.next(true, Focus::OnTarget, true, later),
        Step::Wait(KEYS_LOOKED_AT)
    );
}

#[test]
fn letting_go_of_the_modifiers_sends_the_paste() {
    let (mut pasting, now) = fresh(Route::Keystroke);
    pasting.next(true, Focus::OnTarget, true, now);
    assert_eq!(
        pasting.next(true, Focus::OnTarget, false, now + KEYS_LOOKED_AT),
        Step::Send
    );
}

#[test]
fn modifiers_held_too_long_do_not_hold_the_paste_forever() {
    let (mut pasting, now) = fresh(Route::Keystroke);
    pasting.next(true, Focus::OnTarget, true, now);
    assert_eq!(
        pasting.next(true, Focus::OnTarget, true, now + KEYS_LET_GO),
        Step::Send
    );
}

#[test]
fn the_wait_for_the_keys_counts_from_when_the_target_came_forward() {
    let (mut pasting, now) = fresh(Route::Keystroke);
    pasting.next(true, Focus::Elsewhere, true, now);
    let forward = now + KEYS_LET_GO * 2;
    assert_eq!(
        pasting.next(true, Focus::OnTarget, true, forward),
        Step::Wait(KEYS_LOOKED_AT)
    );
}

#[test]
fn a_paste_through_the_menu_ignores_held_keys_only_after_the_same_wait() {
    let (mut pasting, now) = fresh(Route::Menu);
    assert_eq!(
        pasting.next(true, Focus::OnTarget, true, now),
        Step::Wait(KEYS_LOOKED_AT)
    );
}

#[test]
fn without_protected_input_the_chosen_way_stands() {
    let (pasting, _) = fresh(Route::Keystroke);
    assert_eq!(
        pasting.route_now(&ready(true, false, false)),
        Some(Route::Keystroke)
    );
    let (menu, _) = fresh(Route::Menu);
    assert_eq!(
        menu.route_now(&ready(false, true, false)),
        Some(Route::Menu)
    );
}

#[test]
fn protected_input_turns_a_keystroke_into_the_menu_when_it_can() {
    let (pasting, _) = fresh(Route::Keystroke);
    assert_eq!(
        pasting.route_now(&ready(true, true, true)),
        Some(Route::Menu)
    );
}

#[test]
fn protected_input_with_no_menu_leaves_no_way_to_paste() {
    let (pasting, _) = fresh(Route::Keystroke);
    assert_eq!(pasting.route_now(&ready(true, false, true)), None);
}

#[test]
fn protected_input_leaves_the_menu_alone() {
    let (pasting, _) = fresh(Route::Menu);
    assert_eq!(
        pasting.route_now(&ready(false, true, true)),
        Some(Route::Menu)
    );
}

#[test]
fn the_app_in_front_is_told_apart_from_the_target() {
    assert_eq!(focus_from(Some(7), 7), Focus::OnTarget);
    assert_eq!(focus_from(Some(8), 7), Focus::Elsewhere);
    assert_eq!(focus_from(None, 7), Focus::Unknown);
}

#[test]
fn looking_at_the_keys_is_much_shorter_than_waiting_for_them() {
    assert!(KEYS_LOOKED_AT * 10 <= KEYS_LET_GO);
}

#[test]
fn the_longest_paste_still_ends_within_a_second() {
    let raises = u32::from(cp_core::paste::RACE_RETRIES) + 1;
    assert!(SETTLE * raises + KEYS_LET_GO < Duration::from_secs(1));
}

#[test]
fn no_way_to_paste_is_refused_before_the_panel_is_hidden() {
    let Some(paster) = Paster::new() else {
        return;
    };
    let mut hidden = false;
    let said = paster.start_via(None, somewhere(), || hidden = true);
    assert_eq!(said.err(), Some(Outcome::Degraded(Failure::SendDenied)));
    assert!(!hidden, "the panel stays up so the refusal can be told");
}

#[test]
fn a_way_to_paste_hides_the_panel_before_anything_is_sent() {
    let Some(paster) = Paster::new() else {
        return;
    };
    let mut hidden = false;
    let said = paster.start_via(Some(Route::Keystroke), somewhere(), || hidden = true);
    assert!(said.is_ok());
    assert!(hidden);
}

#[test]
fn a_target_that_does_not_exist_is_given_up_on_without_raising_anything() {
    let Some(paster) = Paster::new() else {
        return;
    };
    let gone = Destination {
        pid: i32::MAX,
        bundle_id: None,
    };
    let mut pasting = paster
        .start_via(Some(Route::Keystroke), gone, || {})
        .expect("a way to paste was given");
    assert_eq!(
        paster.advance(&mut pasting),
        Advance::Done(Outcome::Degraded(Failure::TargetGone))
    );
}
