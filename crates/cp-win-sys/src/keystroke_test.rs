use super::*;
use windows::Win32::UI::Input::KeyboardAndMouse::KEYEVENTF_SCANCODE;

#[test]
fn the_batch_releases_before_it_presses() {
    let batch = paste_batch();
    assert_eq!(batch.len(), 9);
    let ups: Vec<bool> = batch
        .iter()
        .map(|one| unsafe { one.Anonymous.ki.dwFlags }.contains(KEYEVENTF_KEYUP))
        .collect();
    assert_eq!(ups[..5], [true; 5], "los cinco modificadores se sueltan");
    assert_eq!(ups[5..], [false, false, true, true]);
}

#[test]
fn both_windows_keys_are_released_because_there_is_no_generic_one() {
    let batch = paste_batch();

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
        let scan = unsafe { one.Anonymous.ki.wScan };
        assert_ne!(scan, 0, "hay destinos que leen el scancode y no el virtual");
    }
}

#[test]
fn every_event_is_marked_as_ours() {
    for one in paste_batch() {
        assert_eq!(unsafe { one.Anonymous.ki.dwExtraInfo }, OURS);
    }
}

#[test]
fn the_control_that_presses_is_not_the_one_that_releases() {
    let batch = paste_batch();

    let control: Vec<bool> = batch
        .iter()
        .filter(|one| unsafe { one.Anonymous.ki.wVk } == VK_CONTROL)
        .map(|one| unsafe { one.Anonymous.ki.dwFlags }.contains(KEYEVENTF_KEYUP))
        .collect();
    assert_eq!(
        control,
        [true, false, true],
        "suelta, presiona y vuelve a soltar"
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
        let flags = unsafe { one.Anonymous.ki.dwFlags };
        assert!(
            !flags.contains(KEYEVENTF_SCANCODE),
            "el codigo virtual es el que respeta la distribucion"
        );
    }
}
