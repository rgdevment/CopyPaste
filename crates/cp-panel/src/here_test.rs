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
    assert_eq!(paste_into(0, || {}), Sent::Nobody);
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
