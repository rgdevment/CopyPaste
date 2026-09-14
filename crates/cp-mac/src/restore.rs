use cp_core::item::{Item, Payload};
use cp_mac_sys::pasteboard::Pasteboard;

pub fn to_pasteboard(pb: &Pasteboard, item: &Item) -> Restored {
    let writable: Vec<(&str, &[u8])> = item
        .formats
        .iter()
        .filter_map(|format| match &format.payload {
            Payload::Inline(bytes) | Payload::Blob(bytes) => {
                Some((format.id.as_str(), bytes.as_slice()))
            }
            _ => None,
        })
        .collect();

    if writable.is_empty() {
        return Restored::NothingToWrite;
    }

    let entries: Vec<(&str, &[u8])> = writable.clone();
    if pb.write_all(&entries) {
        Restored::Written {
            formats: writable.len(),
            incomplete: writable.len() != item.formats.len(),
        }
    } else {
        Restored::Failed
    }
}

pub fn to_pasteboard_as_plain_text(pb: &Pasteboard, item: &Item) -> Restored {
    let Some(text) = item
        .formats
        .iter()
        .find_map(|format| match &format.payload {
            Payload::Inline(bytes) | Payload::Blob(bytes) if format.id == PLAIN_TEXT => {
                Some(bytes.as_slice())
            }
            _ => None,
        })
    else {
        return Restored::NothingToWrite;
    };

    if pb.write_all(&[(PLAIN_TEXT, text)]) {
        Restored::Written {
            formats: 1,
            incomplete: false,
        }
    } else {
        Restored::Failed
    }
}

const PLAIN_TEXT: &str = "public.utf8-plain-text";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Restored {
    Written { formats: usize, incomplete: bool },
    NothingToWrite,
    Failed,
}
