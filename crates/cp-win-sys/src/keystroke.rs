use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, MAP_VIRTUAL_KEY_TYPE,
    MapVirtualKeyW, SendInput, VIRTUAL_KEY, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};

pub const OURS: usize = 0x0C0B_9A57;

const VK_V: VIRTUAL_KEY = VIRTUAL_KEY(b'V' as u16);

pub fn paste_batch() -> Vec<INPUT> {
    let mut batch = Vec::with_capacity(9);
    for held in [VK_MENU, VK_SHIFT, VK_LWIN, VK_RWIN, VK_CONTROL] {
        batch.push(key(held, true));
    }
    batch.push(key(VK_CONTROL, false));
    batch.push(key(VK_V, false));
    batch.push(key(VK_V, true));
    batch.push(key(VK_CONTROL, true));
    batch
}

fn key(code: VIRTUAL_KEY, up: bool) -> INPUT {
    let mut flags = KEYBD_EVENT_FLAGS(0);
    if up {
        flags |= KEYEVENTF_KEYUP;
    }

    let scan = unsafe { MapVirtualKeyW(u32::from(code.0), MAP_VIRTUAL_KEY_TYPE(0)) } as u16;
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: windows::Win32::UI::Input::KeyboardAndMouse::KEYBDINPUT {
                wVk: code,
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: OURS,
            },
        },
    }
}

pub fn send(batch: &[INPUT]) -> bool {
    let sent = unsafe { SendInput(batch, std::mem::size_of::<INPUT>() as i32) };
    sent as usize == batch.len()
}

pub fn modifiers_still_held() -> bool {
    [VK_CONTROL, VK_MENU, VK_SHIFT, VK_LWIN, VK_RWIN]
        .into_iter()
        .any(pressed)
}

fn pressed(code: VIRTUAL_KEY) -> bool {
    let state =
        unsafe { windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(i32::from(code.0)) };
    state as u16 & 0x8000 != 0
}

#[cfg(test)]
#[path = "keystroke_test.rs"]
mod tests;
