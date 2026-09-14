use std::ffi::c_void;

type CFStringRef = *const c_void;
type CFDataRef = *const c_void;
type CFArrayRef = *const c_void;
type TISInputSourceRef = *const c_void;

unsafe extern "C" {
    static kTISPropertyUnicodeKeyLayoutData: CFStringRef;
    fn TISCopyCurrentKeyboardLayoutInputSource() -> TISInputSourceRef;
    fn TISGetInputSourceProperty(source: TISInputSourceRef, key: CFStringRef) -> CFDataRef;
    fn CFDataGetBytePtr(data: CFDataRef) -> *const u8;
    fn CFRelease(item: *const c_void);
    fn CFRetain(item: *const c_void) -> *const c_void;
    static kTISPropertyInputSourceID: CFStringRef;
    fn TISCreateInputSourceList(properties: *const c_void, include_all: bool) -> CFArrayRef;
    fn CFArrayGetCount(array: CFArrayRef) -> isize;
    fn CFArrayGetValueAtIndex(array: CFArrayRef, index: isize) -> *const c_void;
    fn CFStringGetCString(string: CFStringRef, buffer: *mut u8, size: isize, encoding: u32)
    -> bool;
    fn LMGetKbdType() -> u8;
    fn UCKeyTranslate(
        layout: *const u8,
        virtual_key: u16,
        key_action: u16,
        modifier_state: u32,
        keyboard_type: u32,
        options: u32,
        dead_keys: *mut u32,
        max_len: usize,
        actual_len: *mut usize,
        out: *mut u16,
    ) -> i32;
}

const ACTION_DOWN: u16 = 0;
const NO_DEAD_KEYS: u32 = 1;
const COMMAND_DOWN: u32 = 1;
const HIGHEST_KEYCODE: u16 = 127;

struct Layout {
    source: TISInputSourceRef,
    bytes: *const u8,
}

pub const DVORAK: &str = "com.apple.keylayout.Dvorak";
pub const DVORAK_COMMAND_QWERTY: &str = "com.apple.keylayout.DVORAK-QWERTYCMD";
pub const AZERTY: &str = "com.apple.keylayout.French";
pub const QWERTZ: &str = "com.apple.keylayout.German";
pub const SPANISH_ISO: &str = "com.apple.keylayout.Spanish-ISO";
pub const COLEMAK: &str = "com.apple.keylayout.Colemak";
pub const ABC: &str = "com.apple.keylayout.ABC";

const UTF8: u32 = 0x0800_0100;

fn source_id(source: TISInputSourceRef) -> Option<String> {
    let value = unsafe { TISGetInputSourceProperty(source, kTISPropertyInputSourceID) };
    if value.is_null() {
        return None;
    }
    let mut buffer = [0u8; 256];

    let ok = unsafe { CFStringGetCString(value, buffer.as_mut_ptr(), buffer.len() as isize, UTF8) };
    if !ok {
        return None;
    }
    let end = buffer.iter().position(|byte| *byte == 0)?;
    String::from_utf8(buffer[..end].to_vec()).ok()
}

pub fn installed_layouts() -> Vec<String> {
    let list = unsafe { TISCreateInputSourceList(std::ptr::null(), true) };
    if list.is_null() {
        return Vec::new();
    }

    let count = unsafe { CFArrayGetCount(list) };
    let mut found = Vec::new();
    for index in 0..count {
        let source = unsafe { CFArrayGetValueAtIndex(list, index) };
        if let Some(id) = source_id(source) {
            found.push(id);
        }
    }

    unsafe { CFRelease(list) };
    found
}

impl Layout {
    fn named(wanted: &str) -> Option<Self> {
        let list = unsafe { TISCreateInputSourceList(std::ptr::null(), true) };
        if list.is_null() {
            return None;
        }

        let count = unsafe { CFArrayGetCount(list) };
        let mut chosen = None;
        for index in 0..count {
            let source = unsafe { CFArrayGetValueAtIndex(list, index) };
            if source_id(source).as_deref() == Some(wanted) {
                chosen = Some(source);
                break;
            }
        }
        let result = chosen.and_then(|source| {
            let data =
                unsafe { TISGetInputSourceProperty(source, kTISPropertyUnicodeKeyLayoutData) };
            if data.is_null() {
                return None;
            }

            let bytes = unsafe { CFDataGetBytePtr(data) };
            (!bytes.is_null()).then_some((source, bytes))
        });
        match result {
            Some((source, bytes)) => {
                unsafe { CFRetain(source) };

                unsafe { CFRelease(list) };
                Some(Self { source, bytes })
            }
            None => {
                unsafe { CFRelease(list) };
                None
            }
        }
    }

    fn current() -> Option<Self> {
        let source = unsafe { TISCopyCurrentKeyboardLayoutInputSource() };
        if source.is_null() {
            return None;
        }

        let data = unsafe { TISGetInputSourceProperty(source, kTISPropertyUnicodeKeyLayoutData) };
        if data.is_null() {
            unsafe { CFRelease(source) };
            return None;
        }

        let bytes = unsafe { CFDataGetBytePtr(data) };
        if bytes.is_null() {
            unsafe { CFRelease(source) };
            return None;
        }
        Some(Self { source, bytes })
    }

    fn character_for(&self, keycode: u16, modifiers: u32) -> Option<char> {
        let mut dead_keys: u32 = 0;
        let mut produced: usize = 0;
        let mut buffer = [0u16; 4];

        let status = unsafe {
            UCKeyTranslate(
                self.bytes,
                keycode,
                ACTION_DOWN,
                modifiers,
                u32::from(keyboard_type()),
                NO_DEAD_KEYS,
                &mut dead_keys,
                buffer.len(),
                &mut produced,
                buffer.as_mut_ptr(),
            )
        };
        if status != 0 || produced == 0 {
            return None;
        }
        char::decode_utf16(buffer[..produced].iter().copied())
            .next()?
            .ok()
    }
}

impl Drop for Layout {
    fn drop(&mut self) {
        unsafe { CFRelease(self.source) };
    }
}

fn keyboard_type() -> u8 {
    unsafe { LMGetKbdType() }
}

pub fn keycode_with_command(wanted: char) -> Option<u16> {
    let layout = Layout::current()?;
    (0..=HIGHEST_KEYCODE).find(|code| {
        layout
            .character_for(*code, COMMAND_DOWN)
            .is_some_and(|got| got.eq_ignore_ascii_case(&wanted))
    })
}

pub fn keycode_with_command_in(layout: &str, wanted: char) -> Option<u16> {
    let layout = Layout::named(layout)?;
    (0..=HIGHEST_KEYCODE).find(|code| {
        layout
            .character_for(*code, COMMAND_DOWN)
            .is_some_and(|got| got.eq_ignore_ascii_case(&wanted))
    })
}

pub const QWERTY_V: u16 = 0x09;
