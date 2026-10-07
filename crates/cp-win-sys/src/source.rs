use windows::Wdk::System::Threading::{NtQueryInformationProcess, ProcessCommandLineInformation};
use windows::Win32::Foundation::{CloseHandle, HWND, MAX_PATH, UNICODE_STRING};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
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

pub fn hosts_a_pseudoconsole(handle: isize) -> bool {
    if handle == 0 {
        return false;
    }
    let window = HWND(handle as *mut std::ffi::c_void);
    process_of(window).is_some_and(process_hosts_a_pseudoconsole)
}

pub fn process_hosts_a_pseudoconsole(parent: u32) -> bool {
    children_of(parent).into_iter().any(|(child, exe)| {
        is_console_host(&exe) && command_line_of(child).is_some_and(|line| is_headless(&line))
    })
}

pub fn is_console_host(exe: &str) -> bool {
    let exe = exe.to_ascii_lowercase();
    exe == "conhost.exe" || exe == "openconsole.exe"
}

pub fn is_headless(command_line: &str) -> bool {
    command_line
        .split_whitespace()
        .any(|word| word == "--headless")
}

fn children_of(parent: u32) -> Vec<(u32, String)> {
    every_process()
        .into_iter()
        .filter(|(_, above, _)| *above == parent)
        .map(|(pid, _, name)| (pid, name))
        .collect()
}

pub fn parent_of(pid: u32) -> Option<u32> {
    parent_in(&every_process(), pid)
}

fn parent_in(processes: &[(u32, u32, String)], pid: u32) -> Option<u32> {
    processes
        .iter()
        .find(|(one, above, _)| *one == pid && *above != 0)
        .map(|(_, above, _)| *above)
}

fn every_process() -> Vec<(u32, u32, String)> {
    let Ok(snapshot) = (unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }) else {
        return Vec::new();
    };
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut found = Vec::new();
    let mut more = unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok();
    while more {
        let end = entry
            .szExeFile
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(entry.szExeFile.len());
        found.push((
            entry.th32ProcessID,
            entry.th32ParentProcessID,
            String::from_utf16_lossy(&entry.szExeFile[..end]),
        ));
        more = unsafe { Process32NextW(snapshot, &mut entry) }.is_ok();
    }
    let _ = unsafe { CloseHandle(snapshot) };
    found
}

fn command_line_of(pid: u32) -> Option<String> {
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
    let mut needed = 0u32;
    let _ = unsafe {
        NtQueryInformationProcess(
            process,
            ProcessCommandLineInformation,
            std::ptr::null_mut(),
            0,
            &mut needed,
        )
    };
    let mut line = None;
    if needed as usize >= size_of::<UNICODE_STRING>() {
        let mut buffer = vec![0u64; (needed as usize).div_ceil(8)];
        let status = unsafe {
            NtQueryInformationProcess(
                process,
                ProcessCommandLineInformation,
                buffer.as_mut_ptr().cast(),
                needed,
                &mut needed,
            )
        };
        if status.is_ok() {
            let text = unsafe { &*buffer.as_ptr().cast::<UNICODE_STRING>() };
            if !text.Buffer.is_null() {
                let units = unsafe {
                    std::slice::from_raw_parts(text.Buffer.0, usize::from(text.Length) / 2)
                };
                line = Some(String::from_utf16_lossy(units));
            }
        }
    }
    let _ = unsafe { CloseHandle(process) };
    line
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
