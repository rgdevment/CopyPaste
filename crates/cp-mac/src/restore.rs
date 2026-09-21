use cp_core::item::{Item, Payload, SYNTHETIC_IMAGE, SYNTHETIC_TEXT};
use cp_mac_sys::pasteboard::Pasteboard;

fn type_of(id: &str) -> &str {
    match id {
        SYNTHETIC_TEXT => PLAIN_TEXT,
        SYNTHETIC_IMAGE => PNG,
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

fn plain_text_of(item: &Item) -> Option<&[u8]> {
    writable_of(item)
        .into_iter()
        .find_map(|(kind, bytes)| (kind == PLAIN_TEXT).then_some(bytes))
}

pub fn to_pasteboard_as_plain_text(pb: &Pasteboard, item: &Item) -> Restored {
    let Some(text) = plain_text_of(item) else {
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
const PNG: &str = "public.png";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Restored {
    Written { formats: usize, incomplete: bool },
    NothingToWrite,
    Failed,
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
    fn a_synthetic_text_goes_back_as_the_type_the_system_reads() {
        let item = Item {
            kind: None,
            formats: vec![inline(SYNTHETIC_TEXT, b"hola")],
        };
        assert_eq!(writable_of(&item), vec![(PLAIN_TEXT, &b"hola"[..])]);
        assert_eq!(plain_text_of(&item), Some(&b"hola"[..]));
    }

    #[test]
    fn a_synthetic_image_goes_back_as_a_png() {
        let item = Item {
            kind: None,
            formats: vec![inline(SYNTHETIC_IMAGE, &[137, 80, 78, 71])],
        };
        assert_eq!(writable_of(&item), vec![(PNG, &[137u8, 80, 78, 71][..])]);
    }

    #[test]
    fn a_captured_type_goes_back_under_its_own_name() {
        let item = Item {
            kind: None,
            formats: vec![
                inline("public.rtf", b"{\\rtf1 hola}"),
                inline(PLAIN_TEXT, b"hola"),
                Format {
                    id: "public.tiff".into(),
                    payload: Payload::Announced { size: None },
                },
            ],
        };
        let written = writable_of(&item);
        assert_eq!(written.len(), 2, "lo anunciado sin bytes no se escribe");
        assert_eq!(written[0].0, "public.rtf");
        assert_eq!(plain_text_of(&item), Some(&b"hola"[..]));
    }

    #[test]
    fn nothing_readable_is_nothing_to_write() {
        let item = Item {
            kind: None,
            formats: vec![Format {
                id: PLAIN_TEXT.into(),
                payload: Payload::Absent,
            }],
        };
        assert!(writable_of(&item).is_empty());
        assert_eq!(plain_text_of(&item), None);
    }
}
