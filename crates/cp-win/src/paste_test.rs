use super::*;

#[test]
fn the_clipboard_is_written_before_anything_touches_the_focus() {
    assert_eq!(phases().first(), Some(&Phase::WriteClipboard));
    assert_eq!(phases().last(), Some(&Phase::Send));
}

#[test]
fn the_order_is_the_one_the_core_fixes() {
    assert_eq!(phases(), ORDER);
}

#[test]
fn an_elevated_target_degrades_without_retrying() {
    let mut attempt = Attempt::default();
    assert_eq!(attempt.on_failure(Failure::TargetElevated), Next::Degrade);
}

#[test]
fn the_waits_are_shorter_than_the_paste_they_guard() {
    assert!(SETTLE <= MODIFIERS_GO);
    assert!(MODIFIERS_GO < Duration::from_millis(500));
    assert!(u128::from(TARGET_ANSWERS_MS) < MODIFIERS_GO.as_millis() * 2);
}

#[test]
fn a_target_that_is_us_is_on_target() {
    let Some(window) = frontmost::foreground() else {
        return;
    };
    let target = Target {
        window,
        focus: None,
        thread: 0,
    };
    assert_eq!(focus_of(&target), Focus::OnTarget);
}

#[test]
fn a_window_that_is_not_in_front_is_elsewhere() {
    let Some(front) = frontmost::foreground() else {
        return;
    };
    let other = Target {
        window: windows::Win32::Foundation::HWND(std::ptr::dangling_mut()),
        focus: None,
        thread: 0,
    };
    assert_ne!(other.window, front);
    assert_eq!(focus_of(&other), Focus::Elsewhere);
}

#[test]
fn a_window_that_is_gone_is_told_once_the_panel_has_hidden() {
    let mut hidden = false;
    let gone = Target {
        window: HWND::default(),
        focus: None,
        thread: 0,
    };
    assert_eq!(
        paste_into(&gone, || hidden = true),
        Outcome::Degraded(Failure::TargetGone)
    );
    assert!(hidden);
}

#[test]
fn a_target_that_lost_the_front_before_the_keystroke_is_never_typed_into() {
    assert_eq!(hold_of(Focus::OnTarget), Hold::Go);
    assert_eq!(
        hold_of(Focus::Elsewhere),
        Hold::Stop,
        "another window took the front meanwhile: it stays on the clipboard"
    );
    assert_eq!(
        hold_of(Focus::Unknown),
        Hold::Wait,
        "nobody in front is Windows switching, and it is waited out"
    );
}

#[test]
fn the_keys_wait_until_the_target_has_put_its_focus_back() {
    assert_eq!(
        focus_still_landing(Duration::ZERO),
        Some(FOCUS_LANDS),
        "Electron apps such as Claude drop a Ctrl+V sent the moment they come forward"
    );
    assert_eq!(
        focus_still_landing(Duration::from_millis(100)),
        Some(FOCUS_LANDS - Duration::from_millis(100))
    );
    assert_eq!(focus_still_landing(FOCUS_LANDS), None);
    assert_eq!(focus_still_landing(Duration::from_secs(1)), None);
}
