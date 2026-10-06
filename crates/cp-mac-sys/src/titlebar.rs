use objc2_app_kit::{NSWindow, NSWindowButton};
use objc2_foundation::MainThreadMarker;
use std::ffi::c_void;
use std::ptr::NonNull;

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

#[cfg(test)]
#[path = "titlebar_test.rs"]
mod tests;
