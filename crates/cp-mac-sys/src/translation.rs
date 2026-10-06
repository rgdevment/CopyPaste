use std::ffi::{c_char, c_int, c_void};

const TRANSLATED: &std::ffi::CStr = c"sysctl.proc_translated";

unsafe extern "C" {
    fn sysctlbyname(
        name: *const c_char,
        old: *mut c_void,
        old_len: *mut usize,
        new: *mut c_void,
        new_len: usize,
    ) -> c_int;
}

fn is_translated(status: i32, value: i32) -> bool {
    status == 0 && value == 1
}

pub fn translated() -> bool {
    let mut value: i32 = 0;
    let mut len = size_of::<i32>();
    let status = unsafe {
        sysctlbyname(
            TRANSLATED.as_ptr(),
            (&raw mut value).cast(),
            &raw mut len,
            std::ptr::null_mut(),
            0,
        )
    };
    is_translated(status, value)
}

#[cfg(test)]
#[path = "translation_test.rs"]
mod tests;
