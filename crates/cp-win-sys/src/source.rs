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

fn stem_of(path: &str) -> String {
    let file = path.rsplit(['\\', '/']).next().unwrap_or(path);
    file.rsplit_once('.')
        .map_or(file, |(stem, _)| stem)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_name_is_what_a_person_recognises() {
        assert_eq!(
            stem_of(r"C:\Program Files\Google\Chrome\chrome.exe"),
            "chrome"
        );
        assert_eq!(stem_of(r"C:\Windows\explorer.exe"), "explorer");
        assert_eq!(stem_of("WINWORD.EXE"), "WINWORD");
    }

    #[test]
    fn a_name_without_a_folder_or_an_extension_still_works() {
        assert_eq!(stem_of("notepad"), "notepad");
        assert_eq!(stem_of(""), "");
    }

    #[test]
    fn a_folder_with_a_dot_does_not_eat_the_name() {
        assert_eq!(stem_of(r"C:\apps\v1.2\editor.exe"), "editor");
        assert_eq!(stem_of(r"C:\apps\v1.2\editor"), "editor");
    }

    #[test]
    fn both_separators_are_understood() {
        assert_eq!(stem_of("C:/Windows/System32/cmd.exe"), "cmd");
    }

    #[test]
    fn this_very_process_can_be_named() {
        let mine = std::process::id();
        assert!(name_of(mine).is_some());
    }
}
