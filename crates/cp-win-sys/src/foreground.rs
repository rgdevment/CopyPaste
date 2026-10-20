use std::cell::RefCell;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::WindowsAndMessaging::{
    EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
};

type Heard = Box<dyn Fn(isize)>;

thread_local! {
    static HEARD: RefCell<Option<Heard>> = const { RefCell::new(None) };
}

pub struct Listening(HWINEVENTHOOK);

impl Drop for Listening {
    fn drop(&mut self) {
        let _ = unsafe { UnhookWinEvent(self.0) };
        HEARD.with(|heard| heard.borrow_mut().take());
    }
}

pub fn listen(heard: impl Fn(isize) + 'static) -> Option<Listening> {
    HEARD.with(|held| *held.borrow_mut() = Some(Box::new(heard)));
    let hook = unsafe {
        SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(changed),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        )
    };
    if hook.is_invalid() {
        HEARD.with(|held| held.borrow_mut().take());
        return None;
    }
    Some(Listening(hook))
}

fn told(window: HWND) {
    HEARD.with(|held| {
        if let Some(heard) = held.borrow().as_ref() {
            heard(window.0 as isize);
        }
    });
}

unsafe extern "system" fn changed(
    _hook: HWINEVENTHOOK,
    _event: u32,
    window: HWND,
    _object: i32,
    _child: i32,
    _thread: u32,
    _time: u32,
) {
    told(window);
}

#[cfg(test)]
#[path = "foreground_test.rs"]
mod tests;
