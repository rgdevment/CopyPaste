use windows::Win32::Foundation::{HGLOBAL, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EnumClipboardFormats, GetClipboardData, GetClipboardOwner,
    GetClipboardSequenceNumber, OpenClipboard,
};
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};

const BACKOFF_MS: &[u64] = &[0, 1, 2, 5, 10, 20, 50, 100, 200, 400];

pub struct Clipboard {
    _private: (),
}

impl Clipboard {
    pub fn open() -> Option<Self> {
        for wait in BACKOFF_MS {
            // SAFETY: a null window makes the calling task the owner.
            if unsafe { OpenClipboard(Some(HWND::default())) }.is_ok() {
                return Some(Self { _private: () });
            }
            std::thread::sleep(std::time::Duration::from_millis(*wait));
        }
        None
    }

    pub fn offered(&self) -> Vec<u32> {
        let mut found = Vec::new();
        let mut id = 0u32;
        loop {
            // SAFETY: the clipboard is open for as long as `self` lives.
            id = unsafe { EnumClipboardFormats(id) };
            if id == 0 {
                break;
            }
            found.push(id);
        }
        found
    }

    pub fn owner(&self) -> Option<HWND> {
        // SAFETY: returns a borrowed handle that is not released here.
        let owner = unsafe { GetClipboardOwner() }.ok()?;
        (!owner.is_invalid()).then_some(owner)
    }

    pub fn bytes(&self, id: u32) -> Option<Vec<u8>> {
        // SAFETY: returns a borrowed handle owned by whoever filled the clipboard.
        let handle = unsafe { GetClipboardData(id) }.ok()?;
        if handle.is_invalid() {
            return None;
        }
        let global = HGLOBAL(handle.0);
        // SAFETY: the handle came from the clipboard and is still owned by it.
        let size = unsafe { GlobalSize(global) };
        if size == 0 {
            return None;
        }
        // SAFETY: the handle is valid and the lock is released below.
        let address = unsafe { GlobalLock(global) };
        if address.is_null() {
            return None;
        }
        // SAFETY: the system reports `size` readable bytes at `address`.
        let bytes = unsafe { std::slice::from_raw_parts(address.cast::<u8>(), size) }.to_vec();
        // SAFETY: matches the lock above.
        let _ = unsafe { GlobalUnlock(global) };
        Some(bytes)
    }

    pub fn size_of(&self, id: u32) -> Option<usize> {
        // SAFETY: returns a borrowed handle owned by the clipboard.
        let handle = unsafe { GetClipboardData(id) }.ok()?;
        if handle.is_invalid() {
            return None;
        }
        // SAFETY: the handle came from the clipboard and is still owned by it.
        let size = unsafe { GlobalSize(HGLOBAL(handle.0)) };
        (size > 0).then_some(size)
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        // SAFETY: pairs with the successful open that produced this value.
        let _ = unsafe { CloseClipboard() };
    }
}

pub fn sequence() -> Option<i64> {
    // SAFETY: the call takes no arguments and returns a plain integer.
    let raw = unsafe { GetClipboardSequenceNumber() };
    (raw != 0).then_some(i64::from(raw))
}

pub fn register(name: &str) -> Option<u32> {
    let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: the string is null terminated and outlives the call.
    let id = unsafe {
        windows::Win32::System::DataExchange::RegisterClipboardFormatW(windows::core::PCWSTR(
            wide.as_ptr(),
        ))
    };
    (id != 0).then_some(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_backoff_covers_most_of_a_second_without_a_busy_loop() {
        assert_eq!(BACKOFF_MS.first(), Some(&0), "el primer intento no espera");
        let total: u64 = BACKOFF_MS.iter().sum();
        assert!(
            (700..=900).contains(&total),
            "la cobertura total es de {total} ms"
        );
        assert!(
            BACKOFF_MS.windows(2).all(|pair| pair[0] <= pair[1]),
            "el retroceso no puede acortarse"
        );
    }
}
