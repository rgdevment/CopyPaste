use windows::Win32::Foundation::{CloseHandle, HWND, MAX_PATH};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::WindowsAndMessaging::{GetClassNameW, GetWindowThreadProcessId};

pub fn process_of(window: HWND) -> Option<u32> {
    let mut pid = 0u32;

    let thread = unsafe { GetWindowThreadProcessId(window, Some(&mut pid)) };
    (thread != 0 && pid != 0).then_some(pid)
}

pub fn name_of(pid: u32) -> Option<String> {
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
    let mut buffer = [0u16; MAX_PATH as usize];
    let mut written = buffer.len() as u32;

    let queried = unsafe {
        QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_FORMAT(0),
            windows::core::PWSTR(buffer.as_mut_ptr()),
            &mut written,
        )
    };

    let _ = unsafe { CloseHandle(process) };
    queried.ok()?;
    let path = String::from_utf16_lossy(&buffer[..written as usize]);
    Some(stem_of(&path))
}

pub fn class_of(window: HWND) -> Option<String> {
    let mut buffer = [0u16; 256];
    let written = unsafe { GetClassNameW(window, &mut buffer) };
    (written > 0).then(|| String::from_utf16_lossy(&buffer[..written as usize]))
}

pub fn described(handle: isize) -> Option<(String, String)> {
    if handle == 0 {
        return None;
    }
    let window = HWND(handle as *mut std::ffi::c_void);
    let named = name_of(process_of(window)?)?;
    Some((named, class_of(window).unwrap_or_default()))
}

pub fn in_front() -> Option<String> {
    let window = crate::frontmost::foreground()?;
    if crate::frontmost::process_of_is_ours(window) {
        return None;
    }
    let named = name_of(process_of(window)?)?;
    (!named.is_empty()).then_some(named)
}

fn stem_of(path: &str) -> String {
    let file = path.rsplit(['\\', '/']).next().unwrap_or(path);
    file.rsplit_once('.')
        .map_or(file, |(stem, _)| stem)
        .to_owned()
}

#[cfg(test)]
#[path = "source_test.rs"]
mod tests;
