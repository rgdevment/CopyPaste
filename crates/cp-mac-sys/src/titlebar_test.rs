use super::*;

#[test]
fn off_the_main_thread_it_refuses_before_it_looks_at_the_window() {
    if MainThreadMarker::new().is_some() {
        return;
    }
    assert!(!without_zoom(NonNull::dangling()));
}

#[test]
fn off_the_main_thread_the_probe_has_nothing_to_say() {
    if MainThreadMarker::new().is_some() {
        return;
    }
    assert_eq!(a_titled_window_would_lose_zoom(), None);
}
