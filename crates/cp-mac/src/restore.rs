use cp_core::item::{Item, Payload, SYNTHETIC_IMAGE, SYNTHETIC_JPEG, SYNTHETIC_TEXT};
use cp_mac_sys::pasteboard::Pasteboard;

fn type_of(id: &str) -> &str {
    match id {
        SYNTHETIC_TEXT => PLAIN_TEXT,
        SYNTHETIC_IMAGE => PNG,
        SYNTHETIC_JPEG => JPEG,
        other => other,
    }
}

fn writable_of(item: &Item) -> Vec<(&str, &[u8])> {
    item.formats
        .iter()
        .filter_map(|format| match &format.payload {
            Payload::Inline(bytes) | Payload::Blob(bytes) => {
                Some((type_of(&format.id), bytes.as_slice()))
            }
            _ => None,
        })
        .collect()
}

pub fn to_pasteboard(pb: &Pasteboard, item: &Item) -> Restored {
    let writable = writable_of(item);

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

const PLAIN_TEXT: &str = "public.utf8-plain-text";
const PNG: &str = "public.png";
const JPEG: &str = "public.jpeg";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Restored {
    Written { formats: usize, incomplete: bool },
    NothingToWrite,
    Failed,
}

#[cfg(test)]
#[path = "restore_test.rs"]
mod tests;
