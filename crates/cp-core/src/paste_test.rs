use super::*;

#[test]
fn the_keystroke_is_the_route_whenever_it_is_allowed() {
    assert_eq!(route_for(true, false), Some(Route::Keystroke));
    assert_eq!(route_for(true, true), Some(Route::Keystroke));
}

#[test]
fn the_menu_is_the_route_only_when_the_keystroke_is_denied() {
    assert_eq!(route_for(false, true), Some(Route::Menu));
    assert_eq!(route_for(false, false), None);
}

#[test]
fn a_slow_app_gets_the_dozen_tries_the_2x_needed() {
    let mut attempt = Attempt::default();
    let mut tries = 1;
    while attempt.on_failure(Failure::NotForeground) == Next::Retry {
        tries += 1;
    }
    assert_eq!(
        tries, 12,
        "Office and Electron take hundreds of ms to come forward"
    );
}

#[test]
fn an_elevated_target_is_never_retried() {
    let mut attempt = Attempt::default();
    assert_eq!(attempt.on_failure(Failure::TargetElevated), Next::Degrade);
    assert_eq!(attempt.on_failure(Failure::TargetElevated), Next::Degrade);
    assert!(Failure::TargetElevated.is_permanent());
}

#[test]
fn what_a_second_try_could_fix_is_not_permanent() {
    for failure in [
        Failure::NotForeground,
        Failure::ForegroundTimeout,
        Failure::NoKeyboardFocus,
    ] {
        assert!(!failure.is_permanent(), "{failure:?}");
        assert_eq!(Attempt::default().on_failure(failure), Next::Retry);
    }
}

#[test]
fn a_target_that_is_gone_is_not_worth_retrying_either() {
    assert_eq!(
        Attempt::default().on_failure(Failure::TargetGone),
        Next::Degrade
    );
}

#[test]
fn the_clipboard_is_written_before_anything_touches_focus() {
    let write = ORDER.iter().position(|p| *p == Phase::WriteClipboard);
    let hide = ORDER.iter().position(|p| *p == Phase::HidePanel);
    let forward = ORDER.iter().position(|p| *p == Phase::BringTargetForward);
    assert_eq!(write, Some(0));
    assert!(write < hide && write < forward);
}

#[test]
fn focus_is_verified_before_sending_and_never_after() {
    let verify = ORDER
        .iter()
        .position(|p| *p == Phase::VerifyKeyboardFocus)
        .unwrap();
    let send = ORDER.iter().position(|p| *p == Phase::Send).unwrap();
    assert!(verify < send);
    assert_eq!(send, ORDER.len() - 1);
}

#[test]
fn what_is_unknown_gets_pasted() {
    assert!(!aborts(Focus::Unknown));
    assert!(!aborts(Focus::OnTarget));
    assert!(aborts(Focus::Elsewhere));
}

#[test]
fn a_successful_send_is_never_retried() {
    let mut attempt = Attempt::default();
    attempt.sending();
    assert_eq!(
        attempt.on_failure(Failure::ForegroundTimeout),
        Next::Degrade,
        "a double paste is worse than none"
    );
}

#[test]
fn races_are_retried_and_then_given_up() {
    let mut attempt = Attempt::default();
    for _ in 0..RACE_RETRIES {
        assert_eq!(attempt.on_failure(Failure::ForegroundTimeout), Next::Retry);
    }
    assert_eq!(
        attempt.on_failure(Failure::NoKeyboardFocus),
        Next::Degrade,
        "the budget is shared between the two races"
    );
}

#[test]
fn a_denied_send_is_reported_not_retried() {
    let mut attempt = Attempt::default();
    assert_eq!(
        attempt.on_failure(Failure::SendDenied),
        Next::Degrade,
        "it's UIPI or TCC: retrying changes nothing"
    );
}

#[test]
fn secure_input_warns_but_never_aborts() {
    assert!(!aborts_on(Warning::SecureInputActive));
}
