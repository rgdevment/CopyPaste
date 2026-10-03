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
