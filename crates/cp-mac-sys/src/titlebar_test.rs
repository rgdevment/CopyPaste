use super::*;
use objc2::MainThreadOnly;
use objc2_app_kit::{NSBackingStoreType, NSWindowStyleMask};
use objc2_foundation::{NSPoint, NSRect, NSSize};

#[test]
fn off_the_main_thread_it_refuses_before_it_looks_at_the_window() {
    if MainThreadMarker::new().is_some() {
        return;
    }
    assert!(!without_zoom(NonNull::dangling()));
}

#[test]
fn a_titled_window_keeps_close_and_minimise_but_loses_zoom() {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
            NSWindow::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 120.0)),
            NSWindowStyleMask::Titled
                | NSWindowStyleMask::Closable
                | NSWindowStyleMask::Miniaturizable,
            NSBackingStoreType::Buffered,
            false,
        )
    };
    assert!(without_zoom(NonNull::from(&*window).cast()));
    let zoom = window
        .standardWindowButton(NSWindowButton::ZoomButton)
        .expect("a titled window has a zoom button");
    assert!(zoom.isHidden());
    let close = window
        .standardWindowButton(NSWindowButton::CloseButton)
        .expect("a titled window has a close button");
    assert!(!close.isHidden());
}
