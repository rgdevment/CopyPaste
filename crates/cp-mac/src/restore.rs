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

fn one_item_per_file<'a>(
    writable: &[(&'a str, &'a [u8])],
) -> Option<Vec<Vec<(&'a str, &'a [u8])>>> {
    let joined = writable.iter().find(|(uti, _)| *uti == FILE_URL)?.1;
    let urls: Vec<&[u8]> = joined
        .split(|byte| *byte == b'\n')
        .map(|url| {
            let from = url
                .iter()
                .take_while(|byte| byte.is_ascii_whitespace())
                .count();
            let kept = url.len()
                - url
                    .iter()
                    .rev()
                    .take_while(|byte| byte.is_ascii_whitespace())
                    .count();
            &url[from..kept]
        })
        .filter(|url| !url.is_empty())
        .collect();
    if urls.len() < 2 {
        return None;
    }
    let others: Vec<(&str, &[u8])> = writable
        .iter()
        .filter(|(uti, _)| *uti != FILE_URL)
        .copied()
        .collect();
    Some(
        urls.into_iter()
            .enumerate()
            .map(|(at, url)| {
                let mut entries = vec![(FILE_URL, url)];
                if at == 0 {
                    entries.extend(others.iter().copied());
                }
                entries
            })
            .collect(),
    )
}

pub fn to_pasteboard(pb: &Pasteboard, item: &Item) -> Restored {
    let writable = writable_of(item);

    if writable.is_empty() {
        return Restored::NothingToWrite;
    }

    if let Some(per_file) = one_item_per_file(&writable) {
        return if pb.write_items_data(&per_file) {
            Restored::Written {
                formats: writable.len(),
                incomplete: writable.len() != item.formats.len(),
            }
        } else {
            Restored::Failed
        };
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
const FILE_URL: &str = "public.file-url";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Restored {
    Written { formats: usize, incomplete: bool },
    NothingToWrite,
    Failed,
}

#[cfg(test)]
#[path = "restore_test.rs"]
mod tests;
