use super::*;

fn nowhere() -> NonNull<c_void> {
    NonNull::dangling()
}

#[test]
fn off_the_main_thread_it_refuses_before_it_looks_at_the_view() {
    if MainThreadMarker::new().is_some() {
        return;
    }
    let one = Path::new("/tmp/cp-drag-elsewhere.txt");
    assert_eq!(from_view(nowhere(), &[one]), Dragged::Elsewhere);
}

#[test]
fn nothing_to_drag_is_answered_before_the_view_is_read() {
    if MainThreadMarker::new().is_none() {
        return;
    }
    assert_eq!(from_view(nowhere(), &[]), Dragged::Nothing);
}

#[test]
fn a_file_becomes_something_a_drag_can_carry_with_its_icon_on_it() {
    let at = NSPoint::new(0.0, 0.0);
    let item = item_for(Path::new("/etc/hosts"), at).expect("a file that is always there");
    let frame = item.draggingFrame();
    assert_eq!(frame.size.width, ICON_SIDE);
    assert_eq!(frame.size.height, ICON_SIDE);
}

#[test]
fn a_path_macos_cannot_spell_is_nothing_to_drag() {
    use std::os::unix::ffi::OsStrExt;
    let bad = std::ffi::OsStr::from_bytes(&[0xff, 0xfe]);
    assert!(
        item_for(Path::new(bad), NSPoint::new(0.0, 0.0)).is_none(),
        "a path that is not text cannot become an NSURL, and half a drag is worse than none"
    );
}

#[test]
fn off_the_main_thread_there_is_no_drag_source_to_ask() {
    if MainThreadMarker::new().is_some() {
        return;
    }
    assert!(
        !source_answers(),
        "AppKit is not to be asked from another thread, and silence is not a yes"
    );
}
