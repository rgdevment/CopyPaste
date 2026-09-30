use windows::Win32::Foundation::{HGLOBAL, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EnumClipboardFormats, GetClipboardData, GetClipboardOwner,
    GetClipboardSequenceNumber, OpenClipboard,
};
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};

const BACKOFF_MS: &[u64] = &[0, 1, 2, 5, 10, 20, 50, 100, 200, 400];
const CLEARING_MS: &[u64] = &[0, 1, 2, 5, 10, 20, 50, 100, 200, 400];

const _: () = assert!(CLEARING_MS[0] == 0);
const _: () = {
    let mut backoff = 0;
    let mut clearing = 0;
    let mut turn = 0;
    while turn < BACKOFF_MS.len() {
        backoff += BACKOFF_MS[turn];
        turn += 1;
    }
    turn = 0;
    while turn < CLEARING_MS.len() {
        clearing += CLEARING_MS[turn];
        turn += 1;
    }
    assert!(clearing >= backoff);
};

use std::sync::atomic::{AtomicUsize, Ordering};

static READERS: AtomicUsize = AtomicUsize::new(0);
static WRITES_COMING: AtomicUsize = AtomicUsize::new(0);

pub struct Reading {
    _private: (),
}

pub fn reading() -> Reading {
    READERS.fetch_add(1, Ordering::SeqCst);
    Reading { _private: () }
}

impl Drop for Reading {
    fn drop(&mut self) {
        READERS.fetch_sub(1, Ordering::SeqCst);
    }
}

struct Coming {
    _private: (),
}

fn coming() -> Coming {
    WRITES_COMING.fetch_add(1, Ordering::SeqCst);
    Coming { _private: () }
}

impl Drop for Coming {
    fn drop(&mut self) {
        WRITES_COMING.fetch_sub(1, Ordering::SeqCst);
    }
}

#[derive(PartialEq, Eq)]
enum For {
    Reading,
    Writing,
}

pub struct Clipboard {
    _counted: Option<Reading>,
    opened_for: For,
    _coming: Option<Coming>,
}

impl Clipboard {
    pub fn open() -> Option<Self> {
        for wait in BACKOFF_MS {
            if WRITES_COMING.load(Ordering::SeqCst) == 0 {
                let counted = reading();
                if READERS.load(Ordering::SeqCst) > 0
                    && WRITES_COMING.load(Ordering::SeqCst) == 0
                    && unsafe { OpenClipboard(Some(HWND::default())) }.is_ok()
                {
                    return Some(Self {
                        _counted: Some(counted),
                        opened_for: For::Reading,
                        _coming: None,
                    });
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(*wait));
        }
        None
    }

    pub fn within(_counted: &Reading) -> Option<Self> {
        for wait in BACKOFF_MS {
            if unsafe { OpenClipboard(Some(HWND::default())) }.is_ok() {
                return Some(Self {
                    _counted: None,
                    opened_for: For::Reading,
                    _coming: None,
                });
            }
            std::thread::sleep(std::time::Duration::from_millis(*wait));
        }
        None
    }

    pub fn to_write() -> Option<Self> {
        let coming = coming();
        for wait in CLEARING_MS {
            std::thread::sleep(std::time::Duration::from_millis(*wait));
            if READERS.load(Ordering::SeqCst) == 0
                && unsafe { OpenClipboard(Some(HWND::default())) }.is_ok()
            {
                return Some(Self {
                    _counted: None,
                    opened_for: For::Writing,
                    _coming: Some(coming),
                });
            }
        }
        None
    }

    pub(crate) fn opened_to_write(&self) -> bool {
        self.opened_for == For::Writing
    }

    pub fn offered(&self) -> Vec<u32> {
        let mut found = Vec::new();
        let mut id = 0u32;
        loop {
            id = unsafe { EnumClipboardFormats(id) };
            if id == 0 {
                break;
            }
            found.push(id);
        }
        found
    }

    pub fn owner(&self) -> Option<HWND> {
        let owner = unsafe { GetClipboardOwner() }.ok()?;
        (!owner.is_invalid()).then_some(owner)
    }

    pub fn bytes(&self, id: u32) -> Option<Vec<u8>> {
        let handle = unsafe { GetClipboardData(id) }.ok()?;
        if handle.is_invalid() {
            return None;
        }
        global_bytes(HGLOBAL(handle.0), usize::MAX)
    }

    pub fn size_of(&self, id: u32) -> Option<usize> {
        let handle = unsafe { GetClipboardData(id) }.ok()?;
        if handle.is_invalid() {
            return None;
        }

        let size = unsafe { GlobalSize(HGLOBAL(handle.0)) };
        (size > 0).then_some(size)
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        let _ = unsafe { CloseClipboard() };
    }
}

pub(crate) fn global_bytes(global: HGLOBAL, up_to: usize) -> Option<Vec<u8>> {
    let size = unsafe { GlobalSize(global) };
    if size == 0 || size > up_to {
        return None;
    }

    let address = unsafe { GlobalLock(global) };
    if address.is_null() {
        return None;
    }

    let bytes = unsafe { std::slice::from_raw_parts(address.cast::<u8>(), size) }.to_vec();

    let _ = unsafe { GlobalUnlock(global) };
    Some(bytes)
}

pub fn sequence() -> Option<i64> {
    let raw = unsafe { GetClipboardSequenceNumber() };
    (raw != 0).then_some(i64::from(raw))
}

pub fn register(name: &str) -> Option<u32> {
    let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();

    let id = unsafe {
        windows::Win32::System::DataExchange::RegisterClipboardFormatW(windows::core::PCWSTR(
            wide.as_ptr(),
        ))
    };
    (id != 0).then_some(id)
}

#[cfg(test)]
#[path = "clipboard_test.rs"]
mod tests;
