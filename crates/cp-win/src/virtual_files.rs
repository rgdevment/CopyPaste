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
mod tests {
    use super::*;
    use cp_core::kind::Kind;

    fn inline(id: &str, bytes: &[u8]) -> Format {
        Format {
            id: id.into(),
            payload: Payload::Inline(bytes.to_vec()),
        }
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("cp-virtual-{}-{name}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        dir
    }

    #[test]
    fn a_descriptor_names_its_files_with_their_sizes_and_folders() {
        let described = described_in(&descriptor_of(&[
            ("informe.pdf", Some(1234), false),
            ("adjuntos", None, true),
            ("adjuntos\\foto ñ.jpg", Some(5 << 32 | 7), false),
        ]));
        assert_eq!(
            described,
            vec![
                Described {
                    name: "informe.pdf".into(),
                    size: Some(1234),
                    is_dir: false
                },
                Described {
                    name: "adjuntos".into(),
                    size: None,
                    is_dir: true
                },
                Described {
                    name: "adjuntos\\foto ñ.jpg".into(),
                    size: Some(5 << 32 | 7),
                    is_dir: false
                },
            ]
        );
    }

    #[test]
    fn the_name_runs_to_the_last_of_its_two_hundred_and_sixty_units() {
        let longest = "n".repeat(NAME_UNITS - 1);
        let described = described_in(&descriptor_of(&[(&longest, None, false)]));
        assert_eq!(described[0].name, longest);
        let accented = "ñ".repeat(200);
        let described = described_in(&descriptor_of(&[(&accented, None, false)]));
        assert_eq!(described[0].name, accented);
    }

    #[test]
    fn a_folder_needs_both_the_flag_and_the_attribute() {
        let mut only_flag = descriptor_of(&[("x", None, true)]);
        only_flag[4 + ATTRIBUTES_AT..4 + ATTRIBUTES_AT + 4].copy_from_slice(&0x20u32.to_le_bytes());
        assert!(
            !described_in(&only_flag)[0].is_dir,
            "FD_ATTRIBUTES con un archivo corriente"
        );
        let mut only_attribute = descriptor_of(&[("x", Some(1), false)]);
        only_attribute[4 + ATTRIBUTES_AT..4 + ATTRIBUTES_AT + 4]
            .copy_from_slice(&FILE_ATTRIBUTE_DIRECTORY.to_le_bytes());
        assert!(
            !described_in(&only_attribute)[0].is_dir,
            "sin FD_ATTRIBUTES el atributo no vale"
        );
        let mut both_bits = descriptor_of(&[("x", None, true)]);
        both_bits[4 + ATTRIBUTES_AT..4 + ATTRIBUTES_AT + 4]
            .copy_from_slice(&(FILE_ATTRIBUTE_DIRECTORY | 0x20).to_le_bytes());
        assert!(described_in(&both_bits)[0].is_dir);
        let unsized_dir = described_in(&descriptor_of(&[("x", None, true)]));
        assert_eq!(unsized_dir[0].size, None);
    }

    #[test]
    fn the_size_is_read_from_its_own_two_words() {
        let mut swapped = descriptor_of(&[("x", Some(7), false)]);
        swapped[4 + SIZE_HIGH_AT..4 + SIZE_HIGH_AT + 4].copy_from_slice(&3u32.to_le_bytes());
        swapped[4 + SIZE_LOW_AT..4 + SIZE_LOW_AT + 4].copy_from_slice(&0u32.to_le_bytes());
        assert_eq!(described_in(&swapped)[0].size, Some(3 << 32));
        assert_eq!(
            described_in(&descriptor_of(&[("x", Some(u32::MAX as u64 + 2), false)]))[0].size,
            Some(u32::MAX as u64 + 2)
        );
    }

    #[test]
    fn a_truncated_or_lying_descriptor_yields_what_it_really_holds() {
        assert!(described_in(&[]).is_empty());
        assert!(described_in(&[0, 0, 0]).is_empty());
        let mut lying = descriptor_of(&[("a.txt", None, false)]);
        lying[..4].copy_from_slice(&9u32.to_le_bytes());
        assert_eq!(described_in(&lying).len(), 1, "dice nueve, trae uno");
        let cut = &descriptor_of(&[("a.txt", None, false)])[..300];
        assert!(described_in(cut).is_empty());
    }

    #[test]
    fn the_contents_ids_count_from_the_descriptor() {
        assert_eq!(contents_id(0), "FileContents#0");
        assert_eq!(index_of("FileContents#12"), Some(12));
        assert_eq!(index_of("FileContents"), None);
        assert_eq!(index_of("FileContents#x"), None);
        assert!(is_virtual(DESCRIPTOR));
        assert!(is_virtual(CONTENTS));
        assert!(is_virtual("FileContents#3"));
        assert!(!is_virtual("CF_HDROP"));
    }

    #[test]
    fn virtual_files_are_offered_only_when_there_is_no_real_drop() {
        assert!(offered_without_a_drop(&[
            DESCRIPTOR,
            CONTENTS,
            "RenPrivateItem"
        ]));
        assert!(
            !offered_without_a_drop(&[DESCRIPTOR, CONTENTS, "CF_HDROP"]),
            "el Explorador ofrece ambos: los de disco mandan"
        );
        assert!(!offered_without_a_drop(&[DESCRIPTOR]));
        assert!(!offered_without_a_drop(&[CONTENTS]));
    }

    #[test]
    fn a_name_from_the_descriptor_never_escapes_its_folder() {
        assert_eq!(
            relative_path_of("informe.pdf"),
            Some(PathBuf::from("informe.pdf"))
        );
        assert_eq!(
            relative_path_of("carpeta\\sub/nota.txt"),
            Some(PathBuf::from("carpeta").join("sub").join("nota.txt"))
        );
        for bad in [
            "",
            "..\\x.txt",
            "a\\..\\b",
            ".",
            "C:\\x.txt",
            "\\\\srv\\share",
            "con<sola>.txt",
            "trailing. ",
            "tab\tname",
        ] {
            assert_eq!(relative_path_of(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn materializing_writes_each_file_under_the_item_and_hands_back_the_top_paths() {
        let root = scratch("write");
        let item = Item {
            kind: Some(Kind::File),
            formats: vec![
                inline(
                    DESCRIPTOR,
                    &descriptor_of(&[
                        ("informe.pdf", Some(4), false),
                        ("adjuntos", None, true),
                        ("adjuntos\\nota.txt", Some(3), false),
                    ]),
                ),
                inline(&contents_id(0), b"%PDF"),
                inline(&contents_id(2), b"hola"),
            ],
        };
        let out = materialize(&item, &root).expect("se escribió");
        assert!(out.complete);
        assert_eq!(
            out.paths.len(),
            2,
            "informe.pdf y adjuntos: {:?}",
            out.paths
        );
        assert_eq!(std::fs::read(&out.paths[0]).expect("pdf"), b"%PDF");
        assert!(out.paths[1].is_dir());
        assert_eq!(
            std::fs::read(out.paths[1].join("nota.txt")).expect("nota"),
            b"hola"
        );
        assert!(
            out.paths[0].starts_with(&root),
            "{:?} fuera de {root:?}",
            out.paths[0]
        );
        let again = materialize(&item, &root).expect("pegar dos veces");
        assert_eq!(again.paths, out.paths, "la misma huella, la misma carpeta");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn what_never_arrived_or_cannot_be_named_leaves_the_paste_incomplete() {
        let root = scratch("incomplete");
        let item = Item {
            kind: Some(Kind::File),
            formats: vec![
                inline(
                    DESCRIPTOR,
                    &descriptor_of(&[
                        ("grande.iso", Some(1 << 40), false),
                        ("..\\fuera.txt", Some(1), false),
                        ("ok.txt", Some(2), false),
                    ]),
                ),
                Format {
                    id: contents_id(0),
                    payload: Payload::TooBig { size: 1 << 40 },
                },
                inline(&contents_id(1), b"x"),
                inline(&contents_id(2), b"ok"),
            ],
        };
        let out = materialize(&item, &root).expect("algo se escribió");
        assert!(!out.complete);
        assert_eq!(out.paths.len(), 1);
        assert!(out.paths[0].ends_with("ok.txt"));
        assert!(
            !root.join("fuera.txt").exists(),
            "nada se escapa de la carpeta"
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn an_item_without_virtual_files_has_nothing_to_materialize() {
        let root = scratch("none");
        assert_eq!(materialize(&Item::plain("hola"), &root), None);
        let empty = Item {
            kind: None,
            formats: vec![inline(DESCRIPTOR, &descriptor_of(&[]))],
        };
        assert_eq!(materialize(&empty, &root), None);
        let announced = Item {
            kind: None,
            formats: vec![Format {
                id: DESCRIPTOR.into(),
                payload: Payload::Announced { size: Some(596) },
            }],
        };
        assert_eq!(materialize(&announced, &root), None);
        assert!(!root.exists(), "sin archivos no se crea carpeta");
    }
}
