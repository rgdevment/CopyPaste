use super::*;

#[test]
fn what_the_platform_promises_about_file_thumbnails_is_what_it_does() {
    if !THUMBNAILS_FILES {
        assert!(
            thumb_of_file(Path::new("/"), 128).is_none(),
            "si no dibuja miniaturas de archivos, no puede devolver una"
        );
    }
}

#[test]
fn the_folders_the_panel_writes_into_hang_from_the_same_root() {
    let Some(root) = data_dir() else {
        return;
    };
    assert!(thumbs_dir().expect("miniaturas").starts_with(&root));
}

#[test]
fn asking_the_system_for_its_theme_answers_the_same_twice() {
    assert_eq!(system_is_light(), system_is_light());
}

#[test]
fn nobody_in_front_reads_as_zero_and_pasting_there_reaches_nobody() {
    let said = std::rc::Rc::new(std::cell::Cell::new(None));
    let heard = said.clone();
    paste_into(0, || {}, move |sent| heard.set(Some(sent)));
    assert_eq!(said.take(), Some(Sent::Nobody));
}

#[test]
fn nobody_in_front_never_hides_the_panel() {
    let mut hidden = false;
    paste_into(0, || hidden = true, |_| {});
    assert!(!hidden);
}

#[cfg(target_os = "macos")]
#[test]
fn a_target_that_cannot_be_named_reaches_nobody() {
    let said = std::rc::Rc::new(std::cell::Cell::new(None));
    let heard = said.clone();
    paste_into(-1, || {}, move |sent| heard.set(Some(sent)));
    assert_eq!(said.take(), Some(Sent::Nobody));
}

#[test]
fn a_handle_from_another_system_is_left_as_it_is_when_the_panel_goes() {
    use raw_window_handle::XlibWindowHandle;
    ground(RawWindowHandle::Xlib(XlibWindowHandle::new(1)));
}

#[cfg(target_os = "macos")]
#[test]
fn what_the_paste_ends_in_is_what_the_panel_is_told() {
    use cp_mac::paste::Outcome;
    assert_eq!(
        platform::sent_of(Outcome::Sent {
            took: std::time::Duration::ZERO,
            via: cp_core::paste::Route::Keystroke,
        }),
        Sent::Done
    );
    assert_eq!(
        platform::sent_of(Outcome::Degraded(Failure::ForegroundTimeout)),
        Sent::Degraded(Failure::ForegroundTimeout)
    );
}

#[cfg(target_os = "macos")]
#[test]
fn a_target_that_already_quit_reaches_nobody() {
    let said = std::rc::Rc::new(std::cell::Cell::new(None));
    let heard = said.clone();
    paste_into(i32::MAX as isize, || {}, move |sent| heard.set(Some(sent)));
    assert_eq!(said.take(), Some(Sent::Nobody));
}

#[test]
fn a_handle_from_a_system_this_one_is_not_is_no_window_to_drag_from() {
    use raw_window_handle::XlibWindowHandle;
    assert_eq!(
        drag_out(
            RawWindowHandle::Xlib(XlibWindowHandle::new(1)),
            &[Path::new("/etc/hosts")]
        ),
        Dragged::Elsewhere
    );
}

#[cfg(target_os = "macos")]
#[test]
fn nothing_to_drag_never_starts_a_drag() {
    use raw_window_handle::AppKitWindowHandle;
    let handle = RawWindowHandle::AppKit(AppKitWindowHandle::new(std::ptr::NonNull::dangling()));
    assert_ne!(drag_out(handle, &[]), Dragged::Started);
}

#[cfg(target_os = "windows")]
#[test]
fn nothing_to_drag_never_starts_a_drag() {
    use raw_window_handle::Win32WindowHandle;
    let hwnd = std::num::NonZeroIsize::new(1).expect("one is not zero");
    let handle = RawWindowHandle::Win32(Win32WindowHandle::new(hwnd));
    assert_ne!(drag_out(handle, &[]), Dragged::Started);
}

#[test]
fn no_window_is_aimed_at_nowhere_in_particular() {
    assert_eq!(super::towards(0), crate::landing::Towards::Elsewhere);
}

#[test]
fn nobody_in_front_is_pasted_to_as_anywhere_else() {
    assert_eq!(towards(0), crate::landing::Towards::Elsewhere);
    assert_eq!(towards(-1), crate::landing::Towards::Elsewhere);
}

#[cfg(target_os = "macos")]
fn heard() -> (
    std::rc::Rc<std::cell::RefCell<Vec<Sent>>>,
    impl FnOnce(Sent) + 'static,
) {
    let said = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let kept = said.clone();
    (said, move |sent| kept.borrow_mut().push(sent))
}

#[cfg(target_os = "macos")]
fn steps(
    waits: u32,
    then: cp_mac::paste::Outcome,
) -> (
    std::rc::Rc<std::cell::Cell<u32>>,
    impl FnMut() -> cp_mac::paste::Advance + 'static,
) {
    let asked = std::rc::Rc::new(std::cell::Cell::new(0));
    let counted = asked.clone();
    let step = move || {
        counted.set(counted.get() + 1);
        if counted.get() <= waits {
            cp_mac::paste::Advance::After(std::time::Duration::from_millis(10))
        } else {
            cp_mac::paste::Advance::Done(then.clone())
        }
    };
    (asked, step)
}

#[cfg(target_os = "macos")]
fn sent() -> cp_mac::paste::Outcome {
    cp_mac::paste::Outcome::Sent {
        took: std::time::Duration::ZERO,
        via: cp_core::paste::Route::Keystroke,
    }
}

#[cfg(target_os = "macos")]
#[test]
fn the_paste_goes_on_between_turns_of_the_loop_until_it_is_done() {
    let (said, finished) = heard();
    let (asked, step) = steps(3, sent());
    platform::keep_pasting(step, finished, std::time::Duration::ZERO);
    for _ in 0..50 {
        if !said.borrow().is_empty() {
            break;
        }
        i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(10));
    }
    assert_eq!(*said.borrow(), vec![Sent::Done]);
    assert_eq!(asked.get(), 4);
}

#[cfg(target_os = "macos")]
#[test]
fn nothing_is_tried_before_the_loop_has_turned_once() {
    let (said, finished) = heard();
    let (asked, step) = steps(0, sent());
    platform::keep_pasting(step, finished, std::time::Duration::from_millis(30));
    assert_eq!(asked.get(), 0);
    assert!(said.borrow().is_empty());
    for _ in 0..50 {
        if !said.borrow().is_empty() {
            break;
        }
        i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(10));
    }
    assert_eq!(asked.get(), 1);
}

#[cfg(target_os = "macos")]
#[test]
fn a_paste_that_fails_tells_the_panel_why_exactly_once() {
    let (said, finished) = heard();
    let (asked, step) = steps(
        2,
        cp_mac::paste::Outcome::Degraded(Failure::ForegroundTimeout),
    );
    platform::keep_pasting(step, finished, std::time::Duration::ZERO);
    for _ in 0..80 {
        i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(10));
    }
    assert_eq!(
        *said.borrow(),
        vec![Sent::Degraded(Failure::ForegroundTimeout)]
    );
    assert_eq!(asked.get(), 3, "once it is done nothing is asked again");
}

#[test]
fn the_pointer_lands_on_a_work_area_that_holds_it() {
    let Some(pointer) = pointer() else {
        return;
    };
    let area = pointer.area;
    assert!(
        area.right > area.left && area.bottom > area.top,
        "{pointer:?}"
    );
    assert!(pointer.scale >= 1.0, "{pointer:?}");
}

#[test]
fn the_reading_permission_always_has_an_answer_or_none() {
    let _ = unreadable();
}

#[test]
fn the_panel_can_stay_awake_while_it_watches() {
    let _awake = keep_awake();
}
