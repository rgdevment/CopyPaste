use super::*;

#[test]
fn off_the_main_thread_it_refuses_before_it_looks_at_the_view() {
    if MainThreadMarker::new().is_some() {
        return;
    }
    assert!(!keys_without_activating(NonNull::dangling()));
}

#[test]
fn off_the_main_thread_nothing_is_put_back() {
    if MainThreadMarker::new().is_some() {
        return;
    }
    assert!(!grounded(NonNull::dangling()));
}

#[test]
fn off_the_main_thread_the_probe_has_nothing_to_say() {
    if MainThreadMarker::new().is_some() {
        return;
    }
    assert_eq!(a_panel_would_float(), None);
}

#[test]
fn the_panel_fits_inside_a_plain_window() {
    assert!(fits(Floating::class(), NSWindow::class()));
}

#[test]
fn a_panel_of_the_system_is_the_same_size_as_a_window() {
    assert!(fits(NSPanel::class(), NSWindow::class()));
    assert!(fits(NSWindow::class(), NSPanel::class()));
}

#[test]
fn a_class_bigger_than_the_window_is_never_swapped_in() {
    assert!(!fits(NSWindow::class(), NSObject::class()));
}

#[test]
fn a_smaller_class_fits_in_a_bigger_one() {
    assert!(fits(NSObject::class(), NSWindow::class()));
}

#[test]
fn the_panel_is_a_panel_of_the_system() {
    assert!(Floating::class().superclass() == Some(NSPanel::class()));
}

#[test]
fn nothing_was_swapped_on_a_thread_that_never_floated_a_window() {
    assert_eq!(WAS.with(Cell::get), None);
}
