use objc2::runtime::AnyObject;
use objc2::{ClassType, MainThreadOnly, define_class};
use objc2_app_kit::{NSPanel, NSResponder, NSView, NSWindow, NSWindowStyleMask};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol};
use std::ffi::c_void;
use std::ptr::NonNull;

define_class!(
    #[unsafe(super(NSPanel, NSWindow, NSResponder, NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "CopyPastePanel"]
    pub struct Floating;

    impl Floating {
        #[unsafe(method(canBecomeKeyWindow))]
        fn can_become_key(&self) -> bool {
            true
        }

        #[unsafe(method(canBecomeMainWindow))]
        fn can_become_main(&self) -> bool {
            false
        }
    }
);

pub fn keys_without_activating(ns_view: NonNull<c_void>) -> bool {
    if MainThreadMarker::new().is_none() {
        return false;
    }
    let view: &NSView = unsafe { ns_view.cast::<NSView>().as_ref() };
    let Some(window) = view.window() else {
        return false;
    };
    if !window.isKindOfClass(Floating::class()) {
        floats(&window);
    }
    window.makeKeyAndOrderFront(None);
    window.isKeyWindow()
}

fn floats(window: &NSWindow) {
    let object: &AnyObject = window.as_ref();
    unsafe { AnyObject::set_class(object, Floating::class()) };
    window.setStyleMask(window.styleMask() | NSWindowStyleMask::NonactivatingPanel);
    window.setHidesOnDeactivate(false);
}

#[cfg(test)]
#[path = "floating_test.rs"]
mod tests;
