use std::ffi::c_void;

type CGEventSourceRef = *const c_void;
type CGEventRef = *const c_void;

unsafe extern "C" {
    fn CGEventSourceCreate(state_id: i32) -> CGEventSourceRef;
    fn CGEventSourceFlagsState(state_id: i32) -> u64;
    fn CGEventSourceSetLocalEventsFilterDuringSuppressionState(
        source: CGEventSourceRef,
        filter: u32,
        state: i32,
    );
    fn CGEventCreateKeyboardEvent(
        source: CGEventSourceRef,
        keycode: u16,
        key_down: bool,
    ) -> CGEventRef;
    fn CGEventSetFlags(event: CGEventRef, flags: u64);
    fn CGEventPost(tap: u32, event: CGEventRef);
    fn CFRelease(item: *const c_void);
}

const COMBINED_SESSION: i32 = 0;
const HID_TAP: u32 = 0;
const MASK_COMMAND: u64 = 0x0010_0000;
const LEFT_COMMAND: u64 = 0x0000_0008;
const PERMIT_ALL: u32 = 3;
const SUPPRESSION_INTERVAL: i32 = 0;

pub fn physical_modifiers() -> u64 {
    unsafe { CGEventSourceFlagsState(COMBINED_SESSION) }
}

pub fn modifiers_still_held() -> bool {
    const ANY: u64 = 0x000e_0000;
    physical_modifiers() & ANY != 0
}

pub struct Keystroke {
    source: CGEventSourceRef,
}

impl Keystroke {
    pub fn new() -> Option<Self> {
        let source = unsafe { CGEventSourceCreate(COMBINED_SESSION) };
        if source.is_null() {
            return None;
        }

        unsafe {
            CGEventSourceSetLocalEventsFilterDuringSuppressionState(
                source,
                PERMIT_ALL,
                SUPPRESSION_INTERVAL,
            )
        };
        Some(Self { source })
    }

    pub fn command(&self, keycode: u16) -> bool {
        let flags = MASK_COMMAND | LEFT_COMMAND;
        let (Some(down), Some(up)) = (
            self.event(keycode, true, flags),
            self.event(keycode, false, flags),
        ) else {
            return false;
        };

        unsafe { CGEventPost(HID_TAP, down) };

        unsafe { CFRelease(down) };

        std::thread::sleep(std::time::Duration::from_millis(9));

        unsafe { CGEventPost(HID_TAP, up) };

        unsafe { CFRelease(up) };
        true
    }

    fn event(&self, keycode: u16, down: bool, flags: u64) -> Option<CGEventRef> {
        let event = unsafe { CGEventCreateKeyboardEvent(self.source, keycode, down) };
        if event.is_null() {
            return None;
        }

        unsafe { CGEventSetFlags(event, flags) };
        Some(event)
    }
}

impl Drop for Keystroke {
    fn drop(&mut self) {
        unsafe { CFRelease(self.source) };
    }
}
