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
