use cp_core::item::{Format, Item, Payload};
use std::path::{Path, PathBuf};

pub const DESCRIPTOR: &str = "FileGroupDescriptorW";
pub const CONTENTS: &str = "FileContents";
const CONTENTS_PREFIX: &str = "FileContents#";

const ENTRY: usize = 592;
const ATTRIBUTES_AT: usize = 36;
const SIZE_HIGH_AT: usize = 64;
const SIZE_LOW_AT: usize = 68;
const NAME_AT: usize = 72;
const NAME_UNITS: usize = 260;
const FD_ATTRIBUTES: u32 = 0x04;
const FD_FILESIZE: u32 = 0x40;
const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;
const FORBIDDEN: &str = "<>:\"|?*";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Described {
    pub name: String,
    pub size: Option<u64>,
    pub is_dir: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Materialized {
    pub paths: Vec<PathBuf>,
    pub complete: bool,
}

pub fn offered_without_a_drop(offered: &[&str]) -> bool {
    offered.contains(&DESCRIPTOR) && offered.contains(&CONTENTS) && !offered.contains(&"CF_HDROP")
}

pub fn described_in(descriptor: &[u8]) -> Vec<Described> {
    let Some(count) = u32_at(descriptor, 0) else {
        return Vec::new();
    };
    (0..count as usize)
        .map_while(|index| {
            let at = 4 + index * ENTRY;
            descriptor.get(at..at + ENTRY).and_then(entry)
        })
        .collect()
}

fn entry(bytes: &[u8]) -> Option<Described> {
    let flags = u32_at(bytes, 0)?;
    let attributes = u32_at(bytes, ATTRIBUTES_AT)?;
    let units: Vec<u16> = bytes
        .get(NAME_AT..NAME_AT + NAME_UNITS * 2)?
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes(*pair))
        .take_while(|unit| *unit != 0)
        .collect();
    let size = if flags & FD_FILESIZE != 0 {
        let high = u64::from(u32_at(bytes, SIZE_HIGH_AT)?);
        let low = u64::from(u32_at(bytes, SIZE_LOW_AT)?);
        Some((high << 32) + low)
    } else {
        None
    };
    let described = flags & FD_ATTRIBUTES != 0;
    Some(Described {
        name: String::from_utf16_lossy(&units),
        size,
        is_dir: described && attributes & FILE_ATTRIBUTE_DIRECTORY != 0,
    })
}

fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    let four: [u8; 4] = bytes.get(at..at + 4)?.try_into().ok()?;
    Some(u32::from_le_bytes(four))
}

pub fn contents_id(index: usize) -> String {
    format!("{CONTENTS_PREFIX}{index}")
}

pub fn index_of(id: &str) -> Option<usize> {
    id.strip_prefix(CONTENTS_PREFIX)?.parse().ok()
}

pub fn is_virtual(id: &str) -> bool {
    id == DESCRIPTOR || id == CONTENTS || index_of(id).is_some()
}

pub fn relative_path_of(name: &str) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for part in name.split(['\\', '/']) {
        let unsafe_part = part.is_empty()
            || part.ends_with([' ', '.'])
            || part
                .chars()
                .any(|c| c.is_control() || FORBIDDEN.contains(c));
        if unsafe_part {
            return None;
        }
        out.push(part);
    }
    (out.components().next().is_some()).then_some(out)
}

pub fn materialize(item: &Item, root: &Path) -> Option<Materialized> {
    let described = described_in(bytes_of(item.format(DESCRIPTOR)?)?);
    if described.is_empty() {
        return None;
    }
    let dir = root.join(format!("{:016x}", item.fingerprint()));
    let mut paths: Vec<PathBuf> = Vec::new();
    let mut complete = true;
    for (index, one) in described.iter().enumerate() {
        let Some(relative) = relative_path_of(&one.name) else {
            complete = false;
            continue;
        };
        let target = dir.join(&relative);
        let written = if one.is_dir {
            std::fs::create_dir_all(&target).is_ok()
        } else {
            item.format(&contents_id(index))
                .and_then(bytes_of)
                .is_some_and(|bytes| write_under(&target, bytes))
        };
        if !written {
            complete = false;
            continue;
        }
        if let Some(first) = relative.components().next() {
            let top = dir.join(first);
            if !paths.contains(&top) {
                paths.push(top);
            }
        }
    }
    (!paths.is_empty()).then_some(Materialized { paths, complete })
}

fn write_under(target: &Path, bytes: &[u8]) -> bool {
    target
        .parent()
        .is_some_and(|parent| std::fs::create_dir_all(parent).is_ok())
        && std::fs::write(target, bytes).is_ok()
}

fn bytes_of(format: &Format) -> Option<&[u8]> {
    match &format.payload {
        Payload::Inline(bytes) | Payload::Blob(bytes) => Some(bytes.as_slice()),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) fn descriptor_of(entries: &[(&str, Option<u64>, bool)]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    for (name, size, is_dir) in entries {
        let mut one = vec![0u8; ENTRY];
        let mut flags = 0u32;
        if let Some(size) = size {
            flags |= FD_FILESIZE;
            one[SIZE_HIGH_AT..SIZE_HIGH_AT + 4]
                .copy_from_slice(&((size >> 32) as u32).to_le_bytes());
            one[SIZE_LOW_AT..SIZE_LOW_AT + 4].copy_from_slice(&(*size as u32).to_le_bytes());
        }
        if *is_dir {
            flags |= FD_ATTRIBUTES;
            one[ATTRIBUTES_AT..ATTRIBUTES_AT + 4]
                .copy_from_slice(&FILE_ATTRIBUTE_DIRECTORY.to_le_bytes());
        }
        one[..4].copy_from_slice(&flags.to_le_bytes());
        for (at, unit) in name.encode_utf16().take(NAME_UNITS - 1).enumerate() {
            one[NAME_AT + at * 2..NAME_AT + at * 2 + 2].copy_from_slice(&unit.to_le_bytes());
        }
        out.extend_from_slice(&one);
    }
    out
}

#[cfg(test)]
#[path = "virtual_files_test.rs"]
mod tests;
