use super::*;

#[test]
fn this_process_has_an_integrity_level() {
    assert!(integrity_of(std::process::id()).is_some());
}

#[test]
fn a_process_that_does_not_exist_has_no_level() {
    assert_eq!(integrity_of(0), None);
}

#[test]
fn nothing_ahead_is_no_target() {
    assert_eq!(target_at(0), None);
}

#[test]
fn a_handle_that_is_no_longer_a_window_is_no_target() {
    assert_eq!(target_at(isize::MAX), None);
}

#[test]
fn what_is_ahead_can_be_picked_up_again_by_its_handle() {
    let handle = ahead();
    if handle == 0 {
        return;
    }
    let target = target_at(handle).expect("la ventana de delante sigue viva");
    assert_eq!(target.window.0 as isize, handle);
}

#[test]
fn a_window_of_our_own_is_never_what_is_ahead() {
    let handle = ahead();
    if handle == 0 {
        return;
    }
    let window = target_at(handle).expect("viva").window;
    assert_ne!(process_of(window), Some(std::process::id()));
}

#[test]
fn we_are_never_out_of_our_own_reach() {
    let Some(window) = foreground() else {
        return;
    };
    if process_of(window) == Some(std::process::id()) {
        assert!(!out_of_reach(window));
    }
}

#[test]
fn an_invalid_window_is_not_alive() {
    assert!(!is_alive(HWND::default()));
}

#[test]
fn attaching_to_our_own_thread_is_refused() {
    let ours = unsafe { windows::Win32::System::Threading::GetCurrentThreadId() };
    assert!(Attached::to(ours).is_none());
    assert!(Attached::to(0).is_none());
}

#[test]
fn neither_the_panel_nor_its_host_is_ever_where_a_paste_goes() {
    assert!(is_one_of(Some(10), &[10, 20]));
    assert!(
        is_one_of(Some(20), &[10, 20]),
        "the settings window of the host is ours too"
    );
    assert!(!is_one_of(Some(30), &[10, 20]));
    assert!(!is_one_of(None, &[10, 20]));
}

#[test]
fn a_window_that_is_gone_is_never_pulled_forward() {
    assert!(!pull_forward(HWND(std::ptr::null_mut())));
}

#[test]
fn without_activating_sets_and_clears_only_its_own_flag() {
    let flag = WS_EX_NOACTIVATE.0 as isize;
    let other = 0x0000_0008_isize;
    assert_eq!(without_activating(other, true), other | flag);
    assert_eq!(without_activating(other | flag, false), other);
    assert_eq!(without_activating(other | flag, true), other | flag);
    assert_eq!(without_activating(other, false), other);
    assert_eq!(without_activating(0, true), flag);
}

fn ex_style(window: isize) -> isize {
    unsafe { GetWindowLongPtrW(HWND(window as *mut _), GWL_EXSTYLE) }
}

fn noactivate() -> isize {
    WS_EX_NOACTIVATE.0 as isize
}

#[test]
fn a_window_that_must_not_activate_on_click_gets_the_flag() {
    let edit = crate::window::EditWindow::open("no activate").expect("a window to restyle");
    let window = edit.window().0 as isize;
    clicks_activate(window, false);
    assert_ne!(ex_style(window) & noactivate(), 0);
}

#[test]
fn a_window_that_activates_on_click_loses_the_flag() {
    let edit = crate::window::EditWindow::open("activate").expect("a window to restyle");
    let window = edit.window().0 as isize;
    clicks_activate(window, false);
    clicks_activate(window, true);
    assert_eq!(ex_style(window) & noactivate(), 0);
}

#[test]
fn asking_for_the_same_click_behavior_twice_changes_nothing() {
    let edit = crate::window::EditWindow::open("twice").expect("a window to restyle");
    let window = edit.window().0 as isize;
    clicks_activate(window, false);
    let once = ex_style(window);
    clicks_activate(window, false);
    assert_eq!(ex_style(window), once);
    clicks_activate(window, true);
    let off = ex_style(window);
    clicks_activate(window, true);
    assert_eq!(ex_style(window), off);
}

#[test]
fn the_other_extended_styles_survive_the_click_behavior() {
    let edit = crate::window::EditWindow::open("others").expect("a window to restyle");
    let window = edit.window().0 as isize;
    clicks_activate(window, true);
    let before = ex_style(window);
    clicks_activate(window, false);
    assert_eq!(ex_style(window) & !noactivate(), before & !noactivate());
    clicks_activate(window, true);
    assert_eq!(ex_style(window), before);
}
