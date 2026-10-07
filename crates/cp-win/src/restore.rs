use cp_core::dib;
use cp_core::item::{Item, Payload, SYNTHETIC_IMAGE, SYNTHETIC_JPEG, SYNTHETIC_TEXT};
use cp_win_sys::clipboard::Clipboard;
use cp_win_sys::formats::{CF_DIB, CF_HDROP, CF_UNICODETEXT, id_of};
use cp_win_sys::writing::{Written, utf16_of};

use crate::drop::drop_of;
use crate::transfer::COPY;
use crate::virtual_files::{self, Materialized};

const PNG: &str = "PNG";
const JFIF: &str = "JFIF";
const DROP_EFFECT: &str = "Preferred DropEffect";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Restored {
    Written { formats: usize, incomplete: bool },
    NothingToWrite,
    Failed,
}

pub struct Ready {
    owned: Vec<(u32, Vec<u8>)>,
    returned: usize,
    had: usize,
}

impl Ready {
    pub fn wanted(&self) -> usize {
        self.owned.len()
    }

    pub fn fitted(&self, placed: usize) -> (usize, usize) {
        if placed == self.owned.len() {
            (self.returned, self.had)
        } else {
            (placed, self.owned.len())
        }
    }

    pub fn offer_files<S: AsRef<str>>(&mut self, paths: &[S]) {
        if paths.is_empty() {
            return;
        }
        let effect = id_of(DROP_EFFECT);
        self.owned
            .retain(|(id, _)| *id != CF_HDROP && Some(*id) != effect);
        self.owned.push((CF_HDROP, drop_of(paths)));
        if let Some(effect) = effect {
            self.owned.push((effect, COPY.to_le_bytes().to_vec()));
        }
    }

    pub fn offer_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.owned.retain(|(id, _)| *id != CF_UNICODETEXT);
        self.owned.push((CF_UNICODETEXT, utf16_of(text)));
    }

    pub fn offers(&self, id: u32) -> bool {
        self.owned.iter().any(|(kept, _)| *kept == id)
    }
}

pub fn ready_for(item: &Item) -> Ready {
    let (mut owned, mut returned) = pasted_files(item);
    for format in &item.formats {
        if virtual_files::is_virtual(&format.id) {
            continue;
        }
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
    let had = item
        .formats
        .iter()
        .filter(|one| payload_of(one).is_some() || virtual_files::is_virtual(&one.id))
        .count();
    Ready {
        owned,
        returned,
        had,
    }
}

pub fn place(clipboard: &Clipboard, ready: &Ready) -> Restored {
    if ready.owned.is_empty() {
        return Restored::NothingToWrite;
    }
    let entries: Vec<(u32, &[u8])> = ready
        .owned
        .iter()
        .map(|(id, bytes)| (*id, bytes.as_slice()))
        .collect();
    match clipboard.replace(&entries) {
        Written::Placed { formats } => Restored::Written {
            formats,
            incomplete: ready.returned != ready.had || formats != ready.owned.len(),
        },
        Written::Refused => Restored::Failed,
    }
}

pub fn to_clipboard(clipboard: &Clipboard, item: &Item) -> Restored {
    place(clipboard, &ready_for(item))
}

fn pasted_files(item: &Item) -> (Vec<(u32, Vec<u8>)>, usize) {
    let Some(Materialized { paths, complete }) =
        virtual_files::materialize(item, &cp_win_sys::paths::pastes_dir())
    else {
        return (Vec::new(), 0);
    };
    let names: Vec<String> = paths
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();
    let mut owned = vec![(CF_HDROP, drop_of(&names))];
    if let Some(effect) = id_of(DROP_EFFECT) {
        owned.push((effect, COPY.to_le_bytes().to_vec()));
    }
    let delivered = if complete {
        item.formats
            .iter()
            .filter(|one| virtual_files::is_virtual(&one.id))
            .count()
    } else {
        0
    };
    (owned, delivered)
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
    if id == SYNTHETIC_IMAGE || id == PNG {
        return image_and_bitmap(PNG, bytes, dib::from_png(bytes));
    }
    if id == SYNTHETIC_JPEG || id == JFIF {
        return image_and_bitmap(JFIF, bytes, dib::from_jpeg(bytes));
    }
    id_of(id).map_or_else(Vec::new, |id| vec![(id, bytes.to_vec())])
}

fn image_and_bitmap(name: &str, encoded: &[u8], raw: Option<Vec<u8>>) -> Vec<(u32, Vec<u8>)> {
    let mut both = Vec::new();
    if let Some(id) = id_of(name) {
        both.push((id, encoded.to_vec()));
    }
    if let Some(raw) = raw {
        both.push((CF_DIB, raw));
    }
    both
}

#[cfg(test)]
#[path = "restore_test.rs"]
mod tests;
