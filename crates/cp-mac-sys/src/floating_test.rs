use super::*;
use objc2_app_kit::NSBackingStoreType;
use objc2_foundation::{NSPoint, NSRect, NSSize};

#[test]
fn off_the_main_thread_it_refuses_before_it_looks_at_the_view() {
    if MainThreadMarker::new().is_some() {
        return;
    }
    assert!(!keys_without_activating(NonNull::dangling()));
}

#[test]
fn a_plain_window_becomes_a_panel_that_never_activates_the_app() {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
            NSWindow::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(40.0, 40.0)),
            NSWindowStyleMask::Borderless,
            NSBackingStoreType::Buffered,
            false,
        )
    };
    let view = window
        .contentView()
        .expect("a window always has a content view");
    keys_without_activating(NonNull::from(&*view).cast());
    assert!(window.isKindOfClass(Floating::class()));
    assert!(
        window
            .styleMask()
            .contains(NSWindowStyleMask::NonactivatingPanel)
    );
    assert!(!window.hidesOnDeactivate());
    window.orderOut(None);
}
