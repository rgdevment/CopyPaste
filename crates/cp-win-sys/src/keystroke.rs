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
    // SAFETY: the union holds a keyboard event because the type says so.
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
    // SAFETY: every entry is a keyboard event of the declared size.
    let sent = unsafe { SendInput(batch, std::mem::size_of::<INPUT>() as i32) };
    sent as usize == batch.len()
}

pub fn modifiers_still_held() -> bool {
    [VK_CONTROL, VK_MENU, VK_SHIFT, VK_LWIN, VK_RWIN]
        .into_iter()
        .any(pressed)
}

fn pressed(code: VIRTUAL_KEY) -> bool {
    // SAFETY: the call only reads the asynchronous state of one key.
    let state =
        unsafe { windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(i32::from(code.0)) };
    state as u16 & 0x8000 != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::KEYEVENTF_SCANCODE;

    #[test]
    fn the_batch_releases_before_it_presses() {
        let batch = paste_batch();
        assert_eq!(batch.len(), 9);
        let ups: Vec<bool> = batch
            .iter()
            // SAFETY: every entry was built as a keyboard event.
            .map(|one| unsafe { one.Anonymous.ki.dwFlags }.contains(KEYEVENTF_KEYUP))
            .collect();
        assert_eq!(ups[..5], [true; 5], "los cinco modificadores se sueltan");
        assert_eq!(ups[5..], [false, false, true, true]);
    }

    #[test]
    fn both_windows_keys_are_released_because_there_is_no_generic_one() {
        let batch = paste_batch();
        // SAFETY: every entry was built as a keyboard event.
        let codes: Vec<u16> = batch
            .iter()
            .map(|one| unsafe { one.Anonymous.ki.wVk }.0)
            .collect();
        assert!(codes.contains(&VK_LWIN.0));
        assert!(codes.contains(&VK_RWIN.0));
    }

    #[test]
    fn the_windows_key_is_released_before_the_v_is_pressed() {
        let batch = paste_batch();
        // SAFETY: every entry was built as a keyboard event.
        let codes: Vec<u16> = batch
            .iter()
            .map(|one| unsafe { one.Anonymous.ki.wVk }.0)
            .collect();
        let win = codes
            .iter()
            .position(|code| *code == VK_LWIN.0)
            .expect("win");
        let v = codes.iter().position(|code| *code == VK_V.0).expect("v");
        assert!(
            win < v,
            "con la tecla Windows pisada, la V abre el historial del sistema"
        );
    }

    #[test]
    fn every_event_carries_a_scan_code() {
        for one in paste_batch() {
            // SAFETY: the entry was built as a keyboard event.
            let scan = unsafe { one.Anonymous.ki.wScan };
            assert_ne!(scan, 0, "hay destinos que leen el scancode y no el virtual");
        }
    }

    #[test]
    fn every_event_is_marked_as_ours() {
        for one in paste_batch() {
            // SAFETY: the entry was built as a keyboard event.
            assert_eq!(unsafe { one.Anonymous.ki.dwExtraInfo }, OURS);
        }
    }

    #[test]
    fn the_control_that_presses_is_not_the_one_that_releases() {
        let batch = paste_batch();
        // SAFETY: every entry was built as a keyboard event.
        let control: Vec<bool> = batch
            .iter()
            .filter(|one| unsafe { one.Anonymous.ki.wVk } == VK_CONTROL)
            .map(|one| unsafe { one.Anonymous.ki.dwFlags }.contains(KEYEVENTF_KEYUP))
            .collect();
        assert_eq!(
            control,
            [true, false, true],
            "suelta, pulsa y vuelve a soltar"
        );
    }

    #[test]
    fn the_batch_is_not_sent_in_pieces() {
        assert_eq!(
            paste_batch().len(),
            9,
            "partirlo deja que otro inyector se intercale"
        );
    }

    #[test]
    fn nothing_is_flagged_as_a_bare_scan_code() {
        for one in paste_batch() {
            // SAFETY: the entry was built as a keyboard event.
            let flags = unsafe { one.Anonymous.ki.dwFlags };
            assert!(
                !flags.contains(KEYEVENTF_SCANCODE),
                "el codigo virtual es el que respeta la distribucion"
            );
        }
    }
}
