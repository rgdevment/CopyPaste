use std::ffi::c_void;

type CFStringRef = *const c_void;

unsafe extern "C" {
    static kCFRunLoopDefaultMode: CFStringRef;
    fn CFRunLoopRunInMode(
        mode: CFStringRef,
        seconds: f64,
        return_after_source_handled: bool,
    ) -> i32;
}

pub fn pump(seconds: f64) {
    unsafe { CFRunLoopRunInMode(kCFRunLoopDefaultMode, seconds, false) };
}
