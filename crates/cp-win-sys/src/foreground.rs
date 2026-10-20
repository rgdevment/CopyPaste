use std::cell::RefCell;
use std::collections::HashMap;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::WindowsAndMessaging::{
    EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
};

type Heard = std::rc::Rc<dyn Fn(isize)>;

thread_local! {
    static HOOKED: RefCell<HashMap<isize, Heard>> = RefCell::new(HashMap::new());
}

pub struct Listening(HWINEVENTHOOK);

impl Drop for Listening {
    fn drop(&mut self) {
        let _ = unsafe { UnhookWinEvent(self.0) };
        HOOKED.with(|hooked| hooked.borrow_mut().remove(&key_of(self.0)));
    }
}

pub fn listen(heard: impl Fn(isize) + 'static) -> Option<Listening> {
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
        return None;
    }
    HOOKED.with(|hooked| {
        hooked
            .borrow_mut()
            .insert(key_of(hook), std::rc::Rc::new(heard))
    });
    Some(Listening(hook))
}

fn key_of(hook: HWINEVENTHOOK) -> isize {
    hook.0 as isize
}

fn told(hook: HWINEVENTHOOK, window: HWND) {
    let heard = HOOKED.with(|hooked| hooked.borrow().get(&key_of(hook)).cloned());
    if let Some(heard) = heard {
        heard(window.0 as isize);
    }
}

unsafe extern "system" fn changed(
    hook: HWINEVENTHOOK,
    _event: u32,
    window: HWND,
    _object: i32,
    _child: i32,
    _thread: u32,
    _time: u32,
) {
    told(hook, window);
}

#[cfg(test)]
#[path = "foreground_test.rs"]
mod tests;
