use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, WPARAM};
use windows::Win32::Security::{
    GetTokenInformation, TOKEN_MANDATORY_LABEL, TOKEN_QUERY, TokenIntegrityLevel,
};
use windows::Win32::System::Threading::{
    AttachThreadInput, OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetFocus, SetFocus};
use windows::Win32::UI::WindowsAndMessaging::{
    AllowSetForegroundWindow, GUITHREADINFO, GetForegroundWindow, GetGUIThreadInfo, IsWindow,
    SMTO_ABORTIFHUNG, SMTO_BLOCK, SendMessageTimeoutW, SetForegroundWindow, WM_NULL,
};

use crate::source::process_of;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
    pub window: HWND,
    pub focus: Option<HWND>,
    pub thread: u32,
}

pub fn foreground() -> Option<HWND> {
    let window = unsafe { GetForegroundWindow() };
    (!window.is_invalid()).then_some(window)
}

pub fn capture_target() -> Option<Target> {
    let window = foreground()?;

    let thread =
        unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(window, None) };
    Some(Target {
        window,
        focus: inner_focus(thread),
        thread,
    })
}

pub fn let_it_come_forward(pid: u32) -> bool {
    unsafe { AllowSetForegroundWindow(pid) }.is_ok()
}

pub fn ahead() -> isize {
    foreground().map_or(0, |window| window.0 as isize)
}

pub fn target_at(handle: isize) -> Option<Target> {
    if handle == 0 {
        return None;
    }
    let window = HWND(handle as *mut std::ffi::c_void);
    is_alive(window).then(|| target_for(window))
}

pub fn target_for(window: HWND) -> Target {
    let thread =
        unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(window, None) };
    Target {
        window,
        focus: inner_focus(thread),
        thread,
    }
}

fn inner_focus(thread: u32) -> Option<HWND> {
    let mut info = GUITHREADINFO {
        cbSize: u32::try_from(std::mem::size_of::<GUITHREADINFO>()).ok()?,
        ..Default::default()
    };

    unsafe { GetGUIThreadInfo(thread, &mut info) }.ok()?;
    (!info.hwndFocus.is_invalid()).then_some(info.hwndFocus)
}

pub fn is_alive(window: HWND) -> bool {
    unsafe { IsWindow(Some(window)) }.as_bool()
}

pub fn bring_forward(window: HWND) -> bool {
    unsafe { SetForegroundWindow(window) }.as_bool()
}

pub fn answers(window: HWND, patience_ms: u32) -> bool {
    let mut ignored = 0usize;

    let replied = unsafe {
        SendMessageTimeoutW(
            window,
            WM_NULL,
            WPARAM(0),
            LPARAM(0),
            SMTO_ABORTIFHUNG | SMTO_BLOCK,
            patience_ms,
            Some(&mut ignored),
        )
    };
    replied.0 != 0
}

pub struct Attached {
    ours: u32,
    theirs: u32,
}

impl Attached {
    pub fn to(thread: u32) -> Option<Self> {
        let ours = unsafe { windows::Win32::System::Threading::GetCurrentThreadId() };
        if thread == 0 || thread == ours {
            return None;
        }

        unsafe { AttachThreadInput(ours, thread, true) }
            .as_bool()
            .then_some(Self {
                ours,
                theirs: thread,
            })
    }

    pub fn focus_on(&self, window: HWND) -> bool {
        let _ = unsafe { SetFocus(Some(window)) };
        self.focused() == Some(window)
    }

    pub fn focused(&self) -> Option<HWND> {
        let window = unsafe { GetFocus() };
        (!window.is_invalid()).then_some(window)
    }
}

impl Drop for Attached {
    fn drop(&mut self) {
        let _ = unsafe { AttachThreadInput(self.ours, self.theirs, false) };
    }
}

pub fn integrity_of(pid: u32) -> Option<u32> {
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
    let mut token = windows::Win32::Foundation::HANDLE::default();

    let opened = unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut token) };

    let _ = unsafe { CloseHandle(process) };
    opened.ok()?;
    let level = level_of(token);

    let _ = unsafe { CloseHandle(token) };
    level
}

fn level_of(token: windows::Win32::Foundation::HANDLE) -> Option<u32> {
    let mut size = 0u32;

    let _ = unsafe { GetTokenInformation(token, TokenIntegrityLevel, None, 0, &mut size) };
    if size == 0 {
        return None;
    }
    let mut buffer = vec![0u8; size as usize];

    unsafe {
        GetTokenInformation(
            token,
            TokenIntegrityLevel,
            Some(buffer.as_mut_ptr().cast()),
            size,
            &mut size,
        )
    }
    .ok()?;
    let label = buffer.as_ptr().cast::<TOKEN_MANDATORY_LABEL>();

    let sid = unsafe { (*label).Label.Sid };

    let count = unsafe { windows::Win32::Security::GetSidSubAuthorityCount(sid) };
    if count.is_null() {
        return None;
    }

    let last = unsafe { u32::from(*count) }.checked_sub(1)?;

    let authority = unsafe { windows::Win32::Security::GetSidSubAuthority(sid, last) };

    Some(unsafe { *authority })
}

pub fn out_of_reach(target: HWND) -> bool {
    let Some(theirs) = process_of(target).and_then(integrity_of) else {
        return false;
    };
    let Some(ours) = integrity_of(std::process::id()) else {
        return false;
    };
    theirs > ours
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn this_process_has_an_integrity_level() {
        assert!(integrity_of(std::process::id()).is_some());
    }

    #[test]
    fn a_process_that_does_not_exist_has_no_level() {
        assert_eq!(integrity_of(0), None);
    }

    #[test]
    fn nothing_ahead_is_no_target() {
        assert_eq!(target_at(0), None);
    }

    #[test]
    fn a_handle_that_is_no_longer_a_window_is_no_target() {
        assert_eq!(target_at(isize::MAX), None);
    }

    #[test]
    fn what_is_ahead_can_be_picked_up_again_by_its_handle() {
        let handle = ahead();
        if handle == 0 {
            return;
        }
        let target = target_at(handle).expect("la ventana de delante sigue viva");
        assert_eq!(target.window.0 as isize, handle);
    }

    #[test]
    fn we_are_never_out_of_our_own_reach() {
        let Some(window) = foreground() else {
            return;
        };
        if process_of(window) == Some(std::process::id()) {
            assert!(!out_of_reach(window));
        }
    }

    #[test]
    fn an_invalid_window_is_not_alive() {
        assert!(!is_alive(HWND::default()));
    }

    #[test]
    fn attaching_to_our_own_thread_is_refused() {
        let ours = unsafe { windows::Win32::System::Threading::GetCurrentThreadId() };
        assert!(Attached::to(ours).is_none());
        assert!(Attached::to(0).is_none());
    }
}
