use cp_core::dib;
use cp_core::item::{Item, Payload, SYNTHETIC_IMAGE, SYNTHETIC_TEXT};
use cp_win_sys::clipboard::Clipboard;
use cp_win_sys::formats::{CF_DIBV5, CF_UNICODETEXT, id_of};
use cp_win_sys::writing::{Written, utf16_of};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Restored {
    Written { formats: usize, incomplete: bool },
    NothingToWrite,
    Failed,
}

pub fn to_clipboard(clipboard: &Clipboard, item: &Item) -> Restored {
    let mut owned: Vec<(u32, Vec<u8>)> = Vec::new();
    let mut returned = 0;
    for format in &item.formats {
        let Some(bytes) = payload_of(format) else {
            continue;
        };
        let ids = writable(&format.id, bytes);
        if !ids.is_empty() {
            returned += 1;
        }
        for (id, bytes) in ids {
            if !owned.iter().any(|(kept, _)| *kept == id) {
                owned.push((id, bytes));
            }
        }
    }
    if owned.is_empty() {
        return Restored::NothingToWrite;
    }
    let entries: Vec<(u32, &[u8])> = owned
        .iter()
        .map(|(id, bytes)| (*id, bytes.as_slice()))
        .collect();
    match clipboard.replace(&entries) {
        Written::Placed { formats } => Restored::Written {
            formats,
            incomplete: returned != item.formats.len(),
        },
        Written::Refused => Restored::Failed,
    }
}

pub fn to_clipboard_as_plain_text(clipboard: &Clipboard, item: &Item) -> Restored {
    let Some(text) = plain_text_of(item) else {
        return Restored::NothingToWrite;
    };
    match clipboard.replace(&[(CF_UNICODETEXT, &text)]) {
        Written::Placed { .. } => Restored::Written {
            formats: 1,
            incomplete: false,
        },
        Written::Refused => Restored::Failed,
    }
}

fn payload_of(format: &cp_core::item::Format) -> Option<&[u8]> {
    match &format.payload {
        Payload::Inline(bytes) | Payload::Blob(bytes) => Some(bytes.as_slice()),
        _ => None,
    }
}

fn writable(id: &str, bytes: &[u8]) -> Vec<(u32, Vec<u8>)> {
    if id == SYNTHETIC_TEXT {
        return match std::str::from_utf8(bytes) {
            Ok(text) => vec![(CF_UNICODETEXT, utf16_of(text))],
            Err(_) => Vec::new(),
        };
    }
    if id == SYNTHETIC_IMAGE {
        let mut both = Vec::new();
        if let Some(png) = id_of("PNG") {
            both.push((png, bytes.to_vec()));
        }
        if let Some(raw) = dib::from_png(bytes) {
            both.push((CF_DIBV5, raw));
        }
        return both;
    }
    id_of(id).map_or_else(Vec::new, |id| vec![(id, bytes.to_vec())])
}

fn plain_text_of(item: &Item) -> Option<Vec<u8>> {
    for format in &item.formats {
        let Some(bytes) = payload_of(format) else {
            continue;
        };
        if format.id == "CF_UNICODETEXT" {
            return Some(bytes.to_vec());
        }
        if format.id == SYNTHETIC_TEXT {
            return std::str::from_utf8(bytes).ok().map(utf16_of);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use cp_core::item::Format;

    fn inline(id: &str, bytes: &[u8]) -> Format {
        Format {
            id: id.into(),
            payload: Payload::Inline(bytes.to_vec()),
        }
    }

    #[test]
    fn a_synthetic_text_goes_back_as_the_unicode_the_system_wants() {
        let written = writable(SYNTHETIC_TEXT, b"hola");
        assert_eq!(written, vec![(CF_UNICODETEXT, utf16_of("hola"))]);
    }

    #[test]
    fn a_synthetic_image_goes_back_as_both_a_png_and_a_bitmap() {
        let png = image_bytes();
        let written = writable(SYNTHETIC_IMAGE, &png);
        assert_eq!(written.len(), 2, "el moderno y el clasico");
        assert!(written.iter().any(|(id, _)| *id == CF_DIBV5));
        assert!(
            written.iter().any(|(_, bytes)| bytes == &png),
            "el PNG viaja intacto"
        );
    }

    #[test]
    fn a_name_the_system_does_not_know_is_registered_not_dropped() {
        let written = writable("Rich Text Format", b"{\\rtf1}");
        assert_eq!(written.len(), 1);
        assert!(written[0].0 >= 0xC000);
    }

    #[test]
    fn an_image_that_needs_two_ids_is_not_an_incomplete_restore() {
        let png = image_bytes();
        let ids = writable(SYNTHETIC_IMAGE, &png);
        assert_eq!(ids.len(), 2, "un formato guardado sale por dos vias");
        let item = Item {
            kind: None,
            formats: vec![Format {
                id: SYNTHETIC_IMAGE.into(),
                payload: Payload::Inline(png),
            }],
        };
        assert_eq!(item.formats.len(), 1, "y sigue siendo un solo formato");
    }

    #[test]
    fn nothing_readable_is_nothing_to_write() {
        let item = Item {
            kind: None,
            formats: vec![Format {
                id: "CF_UNICODETEXT".into(),
                payload: Payload::Announced { size: Some(10) },
            }],
        };
        assert_eq!(plain_text_of(&item), None);
    }

    #[test]
    fn the_plain_text_is_the_one_the_system_uses() {
        let item = Item {
            kind: None,
            formats: vec![
                inline("Rich Text Format", b"{\\rtf1 hola}"),
                inline("CF_UNICODETEXT", &utf16_of("hola")),
            ],
        };
        assert_eq!(plain_text_of(&item), Some(utf16_of("hola")));
    }

    #[test]
    fn a_synthetic_item_can_also_be_pasted_flat() {
        let item = Item::plain("hola");
        assert_eq!(plain_text_of(&item), Some(utf16_of("hola")));
    }

    #[test]
    fn asking_for_flat_text_does_not_touch_what_is_stored() {
        let item = Item {
            kind: None,
            formats: vec![
                inline("Rich Text Format", b"{\\rtf1 con estilos}"),
                inline("CF_UNICODETEXT", &utf16_of("con estilos")),
            ],
        };
        let before = item.clone();
        let _ = plain_text_of(&item);
        assert_eq!(item, before, "el item conserva su RTF para la proxima vez");
    }

    fn image_bytes() -> Vec<u8> {
        let mut dib = Vec::new();
        dib.extend_from_slice(&40u32.to_le_bytes());
        dib.extend_from_slice(&2i32.to_le_bytes());
        dib.extend_from_slice(&2i32.to_le_bytes());
        dib.extend_from_slice(&1u16.to_le_bytes());
        dib.extend_from_slice(&32u16.to_le_bytes());
        dib.extend_from_slice(&0u32.to_le_bytes());
        dib.extend_from_slice(&16u32.to_le_bytes());
        dib.resize(40, 0);
        dib.extend_from_slice(&[0x20, 0x60, 0xA0, 0xFF].repeat(4));
        dib::to_png(&dib).expect("png")
    }
}
