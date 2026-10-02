use windows::Win32::Foundation::{HGLOBAL, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EnumClipboardFormats, GetClipboardData, GetClipboardOwner,
    GetClipboardSequenceNumber, OpenClipboard,
};
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};

const BACKOFF_MS: &[u64] = &[0, 1, 2, 5, 10, 20, 50, 100, 200, 400];
const CLEARING_MS: &[u64] = &[0, 1, 2, 5, 10, 20, 50, 100, 200, 400, 400, 400];

pub const STUCK_AFTER: std::time::Duration = std::time::Duration::from_secs(5);

const QUICK_TURNS: usize = 4;

const _: () = assert!(QUICK_TURNS < CLEARING_MS.len());

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
    assert!(clearing > backoff);
};

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, TryLockError};

static SESSION: Mutex<()> = Mutex::new(());

fn session() -> Option<MutexGuard<'static, ()>> {
    match SESSION.try_lock() {
        Ok(held) => Some(held),
        Err(TryLockError::Poisoned(held)) => Some(held.into_inner()),
        Err(TryLockError::WouldBlock) => None,
    }
}

static READERS: AtomicUsize = AtomicUsize::new(0);
static WRITES_COMING: AtomicUsize = AtomicUsize::new(0);
static OLDEST_READ: AtomicU64 = AtomicU64::new(0);

fn now_ms() -> u64 {
    std::time::UNIX_EPOCH.elapsed().map_or(0, |gone| {
        u64::try_from(gone.as_millis()).unwrap_or(u64::MAX)
    })
}

pub struct Reading {
    _private: (),
}

pub fn reading() -> Reading {
    if READERS.fetch_add(1, Ordering::SeqCst) == 0 {
        OLDEST_READ.store(now_ms(), Ordering::SeqCst);
    }
    Reading { _private: () }
}

impl Drop for Reading {
    fn drop(&mut self) {
        if READERS.fetch_sub(1, Ordering::SeqCst) == 1 {
            OLDEST_READ.store(0, Ordering::SeqCst);
        }
    }
}

fn stuck_for(readers: usize, since: u64, now: u64) -> Option<std::time::Duration> {
    if readers == 0 || since == 0 || now < since {
        return None;
    }
    let gone = std::time::Duration::from_millis(now - since);
    (gone >= STUCK_AFTER).then_some(gone)
}

pub fn read_stuck_for() -> Option<std::time::Duration> {
    stuck_for(
        READERS.load(Ordering::SeqCst),
        OLDEST_READ.load(Ordering::SeqCst),
        now_ms(),
    )
}

struct Coming {
    _private: (),
}

pub fn writing_now() -> bool {
    WRITES_COMING.load(Ordering::SeqCst) > 0
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

pub struct Clipboard<'a> {
    _session: MutexGuard<'static, ()>,
    _counted: Option<Reading>,
    _borrowed: Option<&'a Reading>,
    opened_for: For,
    _coming: Option<Coming>,
}

impl Clipboard<'static> {
    pub fn open() -> Option<Self> {
        for wait in BACKOFF_MS {
            if WRITES_COMING.load(Ordering::SeqCst) == 0 {
                let counted = reading();
                if WRITES_COMING.load(Ordering::SeqCst) == 0
                    && let Some(held) = session()
                    && unsafe { OpenClipboard(Some(HWND::default())) }.is_ok()
                {
                    return Some(Self {
                        _session: held,
                        _counted: Some(counted),
                        _borrowed: None,
                        opened_for: For::Reading,
                        _coming: None,
                    });
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(*wait));
        }
        None
    }

    pub fn to_write() -> Option<Self> {
        let coming = coming();
        for (turn, wait) in CLEARING_MS.iter().enumerate() {
            std::thread::sleep(std::time::Duration::from_millis(*wait));
            if READERS.load(Ordering::SeqCst) == 0
                && let Some(held) = session()
                && unsafe { OpenClipboard(Some(HWND::default())) }.is_ok()
            {
                return Some(Self {
                    _session: held,
                    _counted: None,
                    _borrowed: None,
                    opened_for: For::Writing,
                    _coming: Some(coming),
                });
            }
            if turn >= QUICK_TURNS && read_stuck_for().is_some() {
                return None;
            }
        }
        None
    }
}

impl<'a> Clipboard<'a> {
    pub fn within(counted: &'a Reading) -> Option<Self> {
        for wait in BACKOFF_MS {
            if WRITES_COMING.load(Ordering::SeqCst) == 0
                && let Some(held) = session()
                && unsafe { OpenClipboard(Some(HWND::default())) }.is_ok()
            {
                return Some(Self {
                    _session: held,
                    _counted: None,
                    _borrowed: Some(counted),
                    opened_for: For::Reading,
                    _coming: None,
                });
            }
            std::thread::sleep(std::time::Duration::from_millis(*wait));
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

impl Drop for Clipboard<'_> {
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
