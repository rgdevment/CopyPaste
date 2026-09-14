use std::ffi::c_void;

type CFStringRef = *const c_void;

// SAFETY: CoreFoundation signatures as declared in its header.
unsafe extern "C" {
    static kCFRunLoopDefaultMode: CFStringRef;
    fn CFRunLoopRunInMode(
        mode: CFStringRef,
        seconds: f64,
        return_after_source_handled: bool,
    ) -> i32;
}

pub fn pump(seconds: f64) {
    // SAFETY: the mode is the system constant and the call only yields the thread to the run loop for the given time.
    unsafe { CFRunLoopRunInMode(kCFRunLoopDefaultMode, seconds, false) };
}
