pub use cp_core::capture::Captured;
use cp_core::capture::insisting_afresh;
use cp_core::dib;
use cp_core::formats::{Family, Refusal, Take};
use cp_core::item::{BLOB_UP_TO, Format, Item, Payload, SYNTHETIC_IMAGE};
use cp_core::kind::{self, Kind};
use cp_core::reading;
use cp_win_sys::clipboard::{self, Clipboard};
use cp_win_sys::formats::name_of;
use cp_win_sys::writing::text_of;

pub use crate::drop::paths_in;
use crate::formats::CATALOG;
use crate::virtual_files::{self, CONTENTS, DESCRIPTOR};

pub const PATIENCE: std::time::Duration = std::time::Duration::from_millis(400);

const _: () = assert!(PATIENCE.as_millis() > cp_win_sys::reading::PATIENCE.as_millis());
const _: () = assert!(PATIENCE.as_millis() < 30_000);

pub fn capture_within(patience: std::time::Duration) -> Captured {
    let counted = clipboard::reading();
    reading::anything_within(patience, move || capture_counted(&counted))
        .unwrap_or(Captured::TooSlow)
}

pub fn capture_insisting(patience: std::time::Duration, retry: cp_core::watch::Retry) -> Captured {
    insisting_afresh(retry, patience, sequence_now, begin_counted)
}

fn begin_counted() -> reading::Pending<Captured> {
    let counted = clipboard::reading();
    reading::begin(move || capture_counted(&counted))
}

pub fn capture_now() -> Captured {
    let counted = clipboard::reading();
    capture_counted(&counted)
}

fn capture_counted(counted: &clipboard::Reading) -> Captured {
    let captured = match Clipboard::within(counted) {
        Some(clipboard) => capture(&clipboard),
        None => Captured::Busy,
    };
    let Captured::Kept(item) = captured else {
        return captured;
    };
    if !awaits_virtual_contents(&item) {
        return Captured::Kept(item);
    }
    let started = sequence_now();
    let item = with_virtual_contents(item);
    if sequence_now() != started {
        return Captured::Superseded;
    }
    Captured::Kept(item)
}

fn sequence_now() -> i64 {
    clipboard::sequence().unwrap_or(i64::MIN)
}

pub fn capture(clipboard: &Clipboard) -> Captured {
    let ids = clipboard.offered();
    if ids.is_empty() {
        return Captured::Nothing;
    }
    let names: Vec<String> = ids.iter().map(|id| name_of(*id)).collect();
    let offered: Vec<&str> = names.iter().map(String::as_str).collect();

    if let Some(refusal) = CATALOG.refusal(&offered) {
        return Captured::Refused(refusal);
    }
    if let Some(refusal) = asked_not_to_be_kept(clipboard, &ids, &names) {
        return Captured::Refused(refusal);
    }

    let virtual_only = virtual_files::offered_without_a_drop(&offered);
    let family = CATALOG
        .classify(&offered)
        .or_else(|| virtual_only.then_some(Family::Files));
    let mut formats: Vec<Format> = Vec::new();
    for (id, name) in ids.iter().zip(&names) {
        if formats.iter().any(|kept| kept.id == *name) {
            continue;
        }
        let payload = payload_for(clipboard, family, *id, name, &offered);
        formats.push(Format {
            id: name.clone(),
            payload,
        });
    }
    if let Some(png) = transcoded_image(clipboard, &ids, &names, &offered) {
        formats.push(png);
    }
    if virtual_only {
        announce_virtual_files(clipboard, &ids, &names, &mut formats);
    }

    Captured::Kept(Item {
        kind: refine(family, &formats),
        formats,
    })
}

fn asked_not_to_be_kept(clipboard: &Clipboard, ids: &[u32], names: &[String]) -> Option<Refusal> {
    ids.iter().zip(names).find_map(|(id, name)| {
        if !CATALOG.denied_when_zero.contains(&name.as_str()) {
            return None;
        }
        let value = read(clipboard, *id).unwrap_or_default();
        CATALOG.declines(name, &value)
    })
}

fn payload_for(
    clipboard: &Clipboard,
    family: Option<Family>,
    id: u32,
    name: &str,
    offered: &[&str],
) -> Payload {
    if CATALOG.decide_in(family, name) != Take::Payload {
        return Payload::Announced { size: None };
    }
    if CATALOG.costlier_twin(name, offered) {
        return Payload::Announced {
            size: clipboard.size_of(id),
        };
    }
    match read(clipboard, id) {
        Some(bytes) => Payload::stored(bytes),
        None => Payload::Absent,
    }
}

fn read(clipboard: &Clipboard, id: u32) -> Option<Vec<u8>> {
    clipboard.bytes(id)
}

fn transcoded_image(
    clipboard: &Clipboard,
    ids: &[u32],
    names: &[String],
    offered: &[&str],
) -> Option<Format> {
    let chosen = CATALOG.preferred_image(offered)?;
    if !chosen.starts_with("CF_DIB") {
        return None;
    }
    let id = ids
        .iter()
        .zip(names)
        .find(|(_, name)| name.as_str() == chosen)
        .map(|(id, _)| *id)?;
    let raw = read(clipboard, id)?;
    let png = dib::to_png(&raw)?;
    Some(Format {
        id: SYNTHETIC_IMAGE.into(),
        payload: Payload::stored(png),
    })
}

fn announce_virtual_files(
    clipboard: &Clipboard,
    ids: &[u32],
    names: &[String],
    formats: &mut Vec<Format>,
) {
    let descriptor_id = ids
        .iter()
        .zip(names)
        .find(|(_, name)| name.as_str() == DESCRIPTOR)
        .map(|(id, _)| *id);
    let Some(descriptor) = descriptor_id.and_then(|id| read(clipboard, id)) else {
        return;
    };
    let described = virtual_files::described_in(&descriptor);
    if described.is_empty() {
        return;
    }
    if let Some(slot) = formats.iter_mut().find(|one| one.id == DESCRIPTOR) {
        slot.payload = Payload::stored(descriptor);
    }
    for (index, one) in described.iter().enumerate() {
        if one.is_dir {
            continue;
        }
        formats.push(Format {
            id: virtual_files::contents_id(index),
            payload: match one.size {
                Some(size) if size > BLOB_UP_TO as u64 => Payload::TooBig {
                    size: usize::try_from(size).unwrap_or(usize::MAX),
                },
                _ => Payload::Absent,
            },
        });
    }
}

fn awaits_virtual_contents(item: &Item) -> bool {
    item.formats.iter().any(|one| {
        matches!(one.payload, Payload::Absent) && virtual_files::index_of(&one.id).is_some()
    })
}

fn with_virtual_contents(mut item: Item) -> Item {
    let Some(count) = bytes_of(&item.formats, DESCRIPTOR)
        .map(|descriptor| virtual_files::described_in(descriptor).len())
    else {
        return item;
    };
    let Some(contents_id) = cp_win_sys::formats::id_of(CONTENTS) else {
        return item;
    };
    let delivered = cp_win_sys::ole::indexed_contents(contents_id, count, BLOB_UP_TO);
    for format in &mut item.formats {
        if !matches!(format.payload, Payload::Absent) {
            continue;
        }
        if let Some(bytes) = virtual_files::index_of(&format.id)
            .and_then(|index| delivered.get(index))
            .and_then(Clone::clone)
        {
            format.payload = Payload::stored(bytes);
        }
    }
    item
}

fn refine(family: Option<Family>, formats: &[Format]) -> Option<Kind> {
    match family? {
        Family::Image => Some(Kind::Image),
        Family::Text => {
            let text = bytes_of(formats, "CF_UNICODETEXT").and_then(text_of);
            Some(text.as_deref().map_or(Kind::Text, kind::classify_text))
        }
        Family::Files => Some(match first_path(formats) {
            Some(path) => {
                let name = path.rsplit(['\\', '/']).next().unwrap_or(&path).to_owned();
                kind::classify_file(&name, path.ends_with('\\'))
            }
            None => first_described(formats)
                .map_or(Kind::File, |one| kind::classify_file(&one.name, one.is_dir)),
        }),
    }
}

fn first_described(formats: &[Format]) -> Option<virtual_files::Described> {
    let descriptor = bytes_of(formats, DESCRIPTOR)?;
    virtual_files::described_in(descriptor).into_iter().next()
}

fn bytes_of<'a>(formats: &'a [Format], id: &str) -> Option<&'a [u8]> {
    formats
        .iter()
        .find(|one| one.id == id)
        .and_then(|one| match &one.payload {
            Payload::Inline(bytes) | Payload::Blob(bytes) => Some(bytes.as_slice()),
            _ => None,
        })
}

fn first_path(formats: &[Format]) -> Option<String> {
    let drop = bytes_of(formats, "CF_HDROP")?;
    paths_in(drop).into_iter().next()
}

#[cfg(test)]
#[path = "capture_test.rs"]
mod tests;
