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
