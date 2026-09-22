use cp_core::dib;
use cp_core::item::{Item, Payload, SYNTHETIC_IMAGE, SYNTHETIC_JPEG, SYNTHETIC_TEXT};
use cp_win_sys::clipboard::Clipboard;
use cp_win_sys::formats::{CF_DIBV5, CF_HDROP, CF_UNICODETEXT, id_of};
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

pub fn to_clipboard(clipboard: &Clipboard, item: &Item) -> Restored {
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
    if id == SYNTHETIC_IMAGE {
        return image_and_bitmap(PNG, bytes, dib::from_png(bytes));
    }
    if id == SYNTHETIC_JPEG {
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
        both.push((CF_DIBV5, raw));
    }
    both
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
    fn a_rendered_jpeg_goes_back_as_jfif_and_a_bitmap() {
        let jpeg = jpeg_bytes();
        let written = writable(SYNTHETIC_JPEG, &jpeg);
        assert_eq!(written.len(), 2, "el JPEG y el clasico");
        let (jfif, _) = written
            .iter()
            .find(|(_, bytes)| bytes == &jpeg)
            .expect("el JPEG viaja intacto");
        assert_eq!(cp_win_sys::formats::name_of(*jfif), JFIF);
        assert!(written.iter().any(|(id, _)| *id == CF_DIBV5));
    }

    #[test]
    fn a_jpeg_that_does_not_decode_still_goes_back_as_itself() {
        let written = writable(SYNTHETIC_JPEG, b"esto no es un jpeg");
        assert_eq!(written.len(), 1, "sin bitmap, pero el JPEG no se pierde");
        assert!(written.iter().all(|(id, _)| *id != CF_DIBV5));
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
    fn virtual_files_never_go_back_as_themselves_but_as_files_on_disk() {
        use crate::virtual_files::{DESCRIPTOR, contents_id, descriptor_of};
        let item = Item {
            kind: Some(cp_core::kind::Kind::File),
            formats: vec![
                inline(DESCRIPTOR, &descriptor_of(&[("nota.txt", Some(4), false)])),
                Format {
                    id: "FileContents".into(),
                    payload: Payload::Announced { size: None },
                },
                inline(&contents_id(0), b"hola"),
            ],
        };
        let (owned, delivered) = pasted_files(&item);
        assert_eq!(
            delivered, 3,
            "los tres formatos virtuales salen por el drop"
        );
        assert_eq!(owned.len(), 2, "CF_HDROP y el efecto");
        assert_eq!(owned[0].0, CF_HDROP);
        let paths = crate::drop::paths_in(&owned[0].1);
        assert_eq!(paths.len(), 1);
        assert!(paths[0].ends_with("nota.txt"), "{}", paths[0]);
        assert_eq!(std::fs::read(&paths[0]).expect("en disco"), b"hola");
        assert_eq!(owned[1].1, COPY.to_le_bytes(), "pegar copia, no mueve");
        let folder = std::path::Path::new(&paths[0]).parent().expect("carpeta");
        std::fs::remove_dir_all(folder).ok();
    }

    #[test]
    fn an_item_without_virtual_files_pastes_no_files() {
        assert_eq!(pasted_files(&Item::plain("hola")), (Vec::new(), 0));
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
        assert!(item.formats.iter().all(|one| payload_of(one).is_none()));
    }

    #[test]
    fn pasting_as_plain_text_is_a_rendered_form_written_like_any_item() {
        use cp_core::paste_as::{Form, render};
        let item = Item {
            kind: Some(cp_core::kind::Kind::Text),
            formats: vec![
                inline("HTML Format", b"<b>hola</b>"),
                inline("CF_UNICODETEXT", &utf16_of("hola")),
            ],
        };
        let plain = render(Form::PlainText, &crate::content::content_of(&item, None))
            .expect("hay texto")
            .into_item();
        let written: Vec<(u32, Vec<u8>)> = plain
            .formats
            .iter()
            .filter_map(|one| payload_of(one).map(|bytes| writable(&one.id, bytes)))
            .flatten()
            .collect();
        assert_eq!(written, vec![(CF_UNICODETEXT, utf16_of("hola"))]);
        assert_eq!(item.formats.len(), 2, "el ítem guardado no cambia");
        let only_image = Item {
            kind: Some(cp_core::kind::Kind::Image),
            formats: vec![inline(PNG, &[1, 2, 3])],
        };
        assert_eq!(
            render(
                Form::PlainText,
                &crate::content::content_of(&only_image, None)
            ),
            None,
            "sin texto plano no hay forma plana que ofrecer"
        );
    }

    fn jpeg_bytes() -> Vec<u8> {
        let mut out = Vec::new();
        image::load_from_memory(&image_bytes())
            .expect("png")
            .to_rgb8()
            .write_to(
                &mut std::io::Cursor::new(&mut out),
                image::ImageFormat::Jpeg,
            )
            .expect("jpeg");
        out
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
