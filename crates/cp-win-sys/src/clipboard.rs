use windows::Win32::Foundation::{HGLOBAL, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EnumClipboardFormats, GetClipboardData, GetClipboardOwner,
    GetClipboardSequenceNumber, OpenClipboard,
};
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};

const BACKOFF_MS: &[u64] = &[0, 1, 2, 5, 10, 20, 50, 100, 200, 400];
const CLEARING_MS: &[u64] = &[0, 1, 2, 5, 10, 20, 50, 100];

const _: () = assert!(CLEARING_MS[0] == 0);

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static READERS: AtomicUsize = AtomicUsize::new(0);
static A_WRITE_IS_COMING: AtomicBool = AtomicBool::new(false);

pub struct Reading {
    _stays_put: std::marker::PhantomData<*const ()>,
}

pub fn reading() -> Reading {
    READERS.fetch_add(1, Ordering::AcqRel);
    Reading {
        _stays_put: std::marker::PhantomData,
    }
}

impl Drop for Reading {
    fn drop(&mut self) {
        READERS.fetch_sub(1, Ordering::AcqRel);
    }
}

pub(crate) struct Alone {
    _stays_put: std::marker::PhantomData<*const ()>,
}

#[cfg(test)]
pub(crate) fn readers_now() -> usize {
    READERS.load(Ordering::Acquire)
}

pub(crate) fn alone(ours: usize) -> Option<Alone> {
    A_WRITE_IS_COMING.store(true, Ordering::Release);
    let held = Alone {
        _stays_put: std::marker::PhantomData,
    };
    for wait in CLEARING_MS {
        std::thread::sleep(std::time::Duration::from_millis(*wait));
        if READERS.load(Ordering::Acquire) <= ours {
            return Some(held);
        }
    }
    None
}

impl Drop for Alone {
    fn drop(&mut self) {
        A_WRITE_IS_COMING.store(false, Ordering::Release);
    }
}

pub struct Clipboard {
    _counted: Reading,
    _stays_put: std::marker::PhantomData<*const ()>,
}

impl Clipboard {
    pub fn open() -> Option<Self> {
        for wait in BACKOFF_MS {
            if !A_WRITE_IS_COMING.load(Ordering::Acquire) {
                let counted = reading();
                if unsafe { OpenClipboard(Some(HWND::default())) }.is_ok() {
                    return Some(Self {
                        _counted: counted,
                        _stays_put: std::marker::PhantomData,
                    });
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(*wait));
        }
        None
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
