use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{EmptyClipboard, SetClipboardData};
use windows::Win32::System::Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock};

use crate::clipboard::Clipboard;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Written {
    Placed { formats: usize },
    Refused,
}

impl Clipboard<'_> {
    pub fn replace(&self, entries: &[(u32, &[u8])]) -> Written {
        if entries.is_empty() {
            return Written::Refused;
        }
        let Some(ready) = reserved(entries) else {
            return Written::Refused;
        };

        if !self.opened_to_write() {
            release(&ready);
            return Written::Refused;
        }
        if unsafe { EmptyClipboard() }.is_err() {
            release(&ready);
            return Written::Refused;
        }
        let mut placed = 0;
        for (id, block) in ready {
            match unsafe { SetClipboardData(id, Some(HANDLE(block.0))) } {
                Ok(_) => placed += 1,

                Err(_) => unsafe {
                    let _ = GlobalFree(Some(block));
                },
            }
        }
        if placed == 0 {
            Written::Refused
        } else {
            Written::Placed { formats: placed }
        }
    }
}

fn reserved(entries: &[(u32, &[u8])]) -> Option<Vec<(u32, HGLOBAL)>> {
    let mut ready = Vec::with_capacity(entries.len());
    for (id, bytes) in entries {
        if let Some(block) = block_of(bytes) {
            ready.push((*id, block));
        }
    }
    (!ready.is_empty()).then_some(ready)
}

fn release(blocks: &[(u32, HGLOBAL)]) {
    for (_, block) in blocks {
        let _ = unsafe { GlobalFree(Some(*block)) };
    }
}

fn block_of(bytes: &[u8]) -> Option<HGLOBAL> {
    let block = unsafe { GlobalAlloc(GMEM_MOVEABLE, bytes.len()) }.ok()?;

    let address = unsafe { GlobalLock(block) };
    if address.is_null() {
        let _ = unsafe { GlobalFree(Some(block)) };
        return None;
    }

    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), address.cast::<u8>(), bytes.len()) };

    let _ = unsafe { GlobalUnlock(block) };
    Some(block)
}

pub fn utf16_of(text: &str) -> Vec<u8> {
    let mut units: Vec<u16> = text.encode_utf16().collect();
    units.push(0);
    units.iter().flat_map(|unit| unit.to_le_bytes()).collect()
}

pub fn text_of(bytes: &[u8]) -> Option<String> {
    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes(*pair))
        .take_while(|unit| *unit != 0)
        .collect();
    String::from_utf16(&units).ok()
}

#[cfg(test)]
#[path = "writing_test.rs"]
mod tests;
