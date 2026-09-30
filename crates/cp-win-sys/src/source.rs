use windows::Win32::Foundation::{CloseHandle, HWND, MAX_PATH};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

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
