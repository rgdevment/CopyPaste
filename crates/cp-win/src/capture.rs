use cp_core::dib;
use cp_core::formats::{Family, Refusal, Take};
use cp_core::item::{Format, Item, Payload, SYNTHETIC_IMAGE};
use cp_core::kind::{self, Kind};
use cp_win_sys::clipboard::Clipboard;
use cp_win_sys::formats::name_of;
use cp_win_sys::reading;
use cp_win_sys::writing::text_of;

use crate::formats::CATALOG;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Captured {
    Kept(Item),
    Refused(Refusal),
    Nothing,
    TooSlow,
}

pub const PATIENCE: std::time::Duration = std::time::Duration::from_millis(400);

const _: () = assert!(PATIENCE.as_millis() > cp_win_sys::reading::PATIENCE.as_millis());
const _: () = assert!(PATIENCE.as_millis() < 30_000);

pub fn capture_within(patience: std::time::Duration) -> Captured {
    reading::anything_within(patience, || match Clipboard::open() {
        Some(clipboard) => capture(&clipboard),
        None => Captured::Nothing,
    })
    .unwrap_or(Captured::TooSlow)
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

    let family = CATALOG.classify(&offered);
    let mut formats: Vec<Format> = Vec::new();
    for (id, name) in ids.iter().zip(&names) {
        if formats.iter().any(|kept| kept.id == *name) {
            continue;
        }
        let payload = payload_for(clipboard, *id, name, &offered);
        formats.push(Format {
            id: name.clone(),
            payload,
        });
    }
    if let Some(png) = transcoded_image(clipboard, &ids, &names, &offered) {
        formats.push(png);
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

fn payload_for(clipboard: &Clipboard, id: u32, name: &str, offered: &[&str]) -> Payload {
    if CATALOG.decide(name) != Take::Payload {
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
            None => Kind::File,
        }),
    }
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

pub fn paths_in(drop: &[u8]) -> Vec<String> {
    let Some(offset) = drop
        .get(..4)
        .map(|four| u32::from_le_bytes([four[0], four[1], four[2], four[3]]) as usize)
    else {
        return Vec::new();
    };
    let wide = drop.get(16..20).is_some_and(|flag| flag[0] != 0);
    let Some(names) = drop.get(offset..) else {
        return Vec::new();
    };
    if !wide {
        return names
            .split(|byte| *byte == 0)
            .take_while(|part| !part.is_empty())
            .map(|part| String::from_utf8_lossy(part).into_owned())
            .collect();
    }
    let units: Vec<u16> = names
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes(*pair))
        .collect();
    units
        .split(|unit| *unit == 0)
        .take_while(|part| !part.is_empty())
        .map(String::from_utf16_lossy)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drop_files(paths: &[&str], wide: bool) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&20u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&u32::from(wide).to_le_bytes());
        for path in paths {
            if wide {
                for unit in path.encode_utf16() {
                    out.extend_from_slice(&unit.to_le_bytes());
                }
                out.extend_from_slice(&0u16.to_le_bytes());
            } else {
                out.extend_from_slice(path.as_bytes());
                out.push(0);
            }
        }
        if wide {
            out.extend_from_slice(&0u16.to_le_bytes());
        } else {
            out.push(0);
        }
        out
    }

    #[test]
    fn the_whole_capture_has_a_ceiling_well_under_the_thirty_seconds() {
        assert!(PATIENCE < std::time::Duration::from_secs(1));
        assert!(PATIENCE > cp_win_sys::reading::PATIENCE);
    }

    #[test]
    fn a_capture_that_does_not_finish_in_time_is_abandoned() {
        let seen = reading::anything_within(std::time::Duration::from_millis(20), || {
            std::thread::sleep(std::time::Duration::from_secs(30));
            Captured::Nothing
        });
        assert_eq!(seen, None, "el hilo se abandona y no se espera");
    }

    #[test]
    fn every_copied_path_is_read_not_just_the_first() {
        let paths = [r"C:\uno.txt", r"C:\dos.txt", r"C:\una carpeta"];
        let seen = paths_in(&drop_files(&paths, true));
        assert_eq!(seen, paths);
    }

    #[test]
    fn a_legacy_ansi_drop_is_read_too() {
        let paths = [r"C:\uno.txt", r"C:\dos.txt"];
        let seen = paths_in(&drop_files(&paths, false));
        assert_eq!(seen, paths);
    }

    #[test]
    fn a_single_path_comes_back_alone() {
        assert_eq!(
            paths_in(&drop_files(&[r"C:\solo.png"], true)),
            [r"C:\solo.png"]
        );
    }

    #[test]
    fn nonsense_is_not_a_drop() {
        assert!(paths_in(&[]).is_empty());
        assert!(paths_in(&[0, 0, 0]).is_empty());
        assert!(paths_in(&u32::MAX.to_le_bytes()).is_empty());
    }

    #[test]
    fn a_path_with_accents_and_spaces_survives() {
        let paths = [r"C:\Mis Documentos\informe ñ.pdf"];
        assert_eq!(paths_in(&drop_files(&paths, true)), paths);
    }

    #[test]
    fn the_class_of_a_drop_comes_from_its_first_path() {
        let formats = vec![Format {
            id: "CF_HDROP".into(),
            payload: Payload::Inline(drop_files(&[r"C:\video.mkv"], true)),
        }];
        assert_eq!(refine(Some(Family::Files), &formats), Some(Kind::Video));
    }

    #[test]
    fn a_folder_is_told_apart_by_its_trailing_separator() {
        let formats = vec![Format {
            id: "CF_HDROP".into(),
            payload: Payload::Inline(drop_files(&[r"C:\Documentos\\"], true)),
        }];
        assert_eq!(refine(Some(Family::Files), &formats), Some(Kind::Folder));
    }

    #[test]
    fn text_is_refined_by_what_it_says() {
        let formats = vec![Format {
            id: "CF_UNICODETEXT".into(),
            payload: Payload::Inline(cp_win_sys::writing::utf16_of("alguien@ejemplo.test")),
        }];
        assert_eq!(refine(Some(Family::Text), &formats), Some(Kind::Email));
    }

    #[test]
    fn text_that_could_not_be_read_is_still_text() {
        let formats = vec![Format {
            id: "CF_UNICODETEXT".into(),
            payload: Payload::Absent,
        }];
        assert_eq!(refine(Some(Family::Text), &formats), Some(Kind::Text));
    }

    #[test]
    fn an_image_needs_no_refining() {
        assert_eq!(refine(Some(Family::Image), &[]), Some(Kind::Image));
    }

    #[test]
    fn nothing_offered_has_no_class() {
        assert_eq!(refine(None, &[]), None);
    }

    #[test]
    fn a_drop_with_no_paths_is_still_a_file() {
        let formats = vec![Format {
            id: "CF_HDROP".into(),
            payload: Payload::Absent,
        }];
        assert_eq!(refine(Some(Family::Files), &formats), Some(Kind::File));
    }
}
