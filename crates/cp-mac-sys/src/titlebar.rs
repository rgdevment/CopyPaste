use objc2::MainThreadOnly;
use objc2_app_kit::{NSBackingStoreType, NSWindow, NSWindowButton, NSWindowStyleMask};
use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize};
use std::ffi::c_void;
use std::ptr::NonNull;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Buttons {
    pub zoom_shown: bool,
    pub close_shown: bool,
    pub minimise_shown: bool,
}

pub fn without_zoom(ns_window: NonNull<c_void>) -> bool {
    if MainThreadMarker::new().is_none() {
        return false;
    }
    let window: &NSWindow = unsafe { ns_window.cast::<NSWindow>().as_ref() };
    let Some(zoom) = window.standardWindowButton(NSWindowButton::ZoomButton) else {
        return false;
    };
    zoom.setHidden(true);
    true
}

pub fn a_titled_window_would_lose_zoom() -> Option<Buttons> {
    let mtm = MainThreadMarker::new()?;
    let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
            NSWindow::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(200.0, 120.0)),
            NSWindowStyleMask::Titled
                | NSWindowStyleMask::Closable
                | NSWindowStyleMask::Miniaturizable
                | NSWindowStyleMask::Resizable,
            NSBackingStoreType::Buffered,
            false,
        )
    };
    unsafe { window.setReleasedWhenClosed(false) };
    if !without_zoom(NonNull::from(&*window).cast()) {
        return None;
    }
    let shown = |which| {
        window
            .standardWindowButton(which)
            .is_some_and(|button| !button.isHidden())
    };
    let buttons = Buttons {
        zoom_shown: shown(NSWindowButton::ZoomButton),
        close_shown: shown(NSWindowButton::CloseButton),
        minimise_shown: shown(NSWindowButton::MiniaturizeButton),
    };
    window.close();
    Some(buttons)
}

#[cfg(test)]
#[path = "titlebar_test.rs"]
mod tests;
