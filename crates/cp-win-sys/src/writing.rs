use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{EmptyClipboard, SetClipboardData};
use windows::Win32::System::Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock};

use crate::clipboard::Clipboard;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Written {
    Placed { formats: usize },
    Refused,
}

impl Clipboard {
    pub fn replace(&self, entries: &[(u32, &[u8])]) -> Written {
        if entries.is_empty() {
            return Written::Refused;
        }
        let Some(ready) = reserved(entries) else {
            return Written::Refused;
        };

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
        match block_of(bytes) {
            Some(block) => ready.push((*id, block)),
            None => {
                release(&ready);
                return None;
            }
        }
    }
    Some(ready)
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
mod tests {
    use super::*;

    #[test]
    fn every_block_is_reserved_before_the_clipboard_is_emptied() {
        let entries: Vec<(u32, &[u8])> = vec![(1, b"uno".as_slice()), (2, b"dos".as_slice())];
        let ready = reserved(&entries).expect("la memoria se consigue");
        assert_eq!(ready.len(), 2, "lo que se pidio, reservado y sin entregar");
        release(&ready);
    }

    #[test]
    fn nothing_to_write_reserves_nothing() {
        let ready = reserved(&[]).expect("vacio no es fallo");
        assert!(ready.is_empty());
        release(&ready);
    }

    #[test]
    fn text_survives_the_round_trip() {
        for original in ["hola", "", "acentos: ñáéíóú", "emoji: 🦀", "日本語"] {
            let bytes = utf16_of(original);
            assert_eq!(text_of(&bytes).as_deref(), Some(original), "«{original}»");
        }
    }

    #[test]
    fn the_encoding_ends_where_the_terminator_says() {
        let bytes = utf16_of("ab");
        assert_eq!(bytes.len(), 6, "dos unidades más el cero");
        assert_eq!(&bytes[4..], &[0, 0]);
    }

    #[test]
    fn what_comes_after_the_terminator_is_not_text() {
        let mut bytes = utf16_of("hola");
        bytes.extend_from_slice(&utf16_of("basura"));
        assert_eq!(text_of(&bytes).as_deref(), Some("hola"));
    }

    #[test]
    fn an_odd_number_of_bytes_does_not_panic() {
        assert_eq!(text_of(&[0x68]), Some(String::new()));
        assert_eq!(text_of(&[]), Some(String::new()));
    }

    #[test]
    fn a_lone_surrogate_is_refused_instead_of_mangled() {
        let broken = [0x00u8, 0xD8, 0x41, 0x00];
        assert_eq!(text_of(&broken), None);
    }
}
