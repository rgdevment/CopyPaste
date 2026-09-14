use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, DispatchMessageW, ES_MULTILINE, MSG, PM_REMOVE, PeekMessageW,
    SW_SHOW, SendMessageW, SetForegroundWindow, ShowWindow, TranslateMessage, WINDOW_EX_STYLE,
    WM_GETTEXT, WM_GETTEXTLENGTH, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};
use windows::core::{PCWSTR, w};

pub struct EditWindow {
    window: HWND,
}

impl EditWindow {
    pub fn open(title: &str) -> Option<Self> {
        let wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();

        let window = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("EDIT"),
                PCWSTR(wide.as_ptr()),
                WS_OVERLAPPEDWINDOW
                    | WS_VISIBLE
                    | windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(ES_MULTILINE as u32),
                100,
                100,
                420,
                220,
                None,
                None,
                None,
                None,
            )
        }
        .ok()?;

        let _ = unsafe { ShowWindow(window, SW_SHOW) };

        let _ = unsafe { SetForegroundWindow(window) };
        Some(Self { window })
    }

    pub fn window(&self) -> HWND {
        self.window
    }

    pub fn pump(&self, how_long: std::time::Duration) {
        let until = std::time::Instant::now() + how_long;
        while std::time::Instant::now() < until {
            let mut message = MSG::default();

            while unsafe { PeekMessageW(&mut message, None, 0, 0, PM_REMOVE) }.as_bool() {
                let _ = unsafe { TranslateMessage(&message) };

                unsafe { DispatchMessageW(&message) };
            }
            std::thread::sleep(std::time::Duration::from_millis(4));
        }
    }

    pub fn text(&self) -> String {
        let length = unsafe { SendMessageW(self.window, WM_GETTEXTLENGTH, None, None) }.0;
        let Ok(length) = usize::try_from(length) else {
            return String::new();
        };
        if length == 0 {
            return String::new();
        }
        let mut buffer = vec![0u16; length + 1];

        let read = unsafe {
            SendMessageW(
                self.window,
                WM_GETTEXT,
                Some(WPARAM(buffer.len())),
                Some(LPARAM(buffer.as_mut_ptr() as isize)),
            )
        }
        .0;
        let read = usize::try_from(read).unwrap_or(0).min(buffer.len());
        String::from_utf16_lossy(&buffer[..read])
    }
}

impl Drop for EditWindow {
    fn drop(&mut self) {
        let _ = unsafe { DestroyWindow(self.window) };
    }
}
