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
    let written = writable(SYNTHETIC_TEXT, b"hello");
    assert_eq!(written, vec![(CF_UNICODETEXT, utf16_of("hello"))]);
}

#[test]
fn a_synthetic_image_goes_back_as_both_a_png_and_a_bitmap() {
    let png = image_bytes();
    let written = writable(SYNTHETIC_IMAGE, &png);
    assert_eq!(written.len(), 2, "the modern one and the classic one");
    assert!(written.iter().any(|(id, _)| *id == CF_DIBV5));
    assert!(
        written.iter().any(|(_, bytes)| bytes == &png),
        "the PNG travels intact"
    );
}

#[test]
fn a_rendered_jpeg_goes_back_as_jfif_and_a_bitmap() {
    let jpeg = jpeg_bytes();
    let written = writable(SYNTHETIC_JPEG, &jpeg);
    assert_eq!(written.len(), 2, "the JPEG and the classic one");
    let (jfif, _) = written
        .iter()
        .find(|(_, bytes)| bytes == &jpeg)
        .expect("the JPEG travels intact");
    assert_eq!(cp_win_sys::formats::name_of(*jfif), JFIF);
    assert!(written.iter().any(|(id, _)| *id == CF_DIBV5));
}

#[test]
fn a_jpeg_that_does_not_decode_still_goes_back_as_itself() {
    let written = writable(SYNTHETIC_JPEG, b"this is not a jpeg");
    assert_eq!(written.len(), 1, "no bitmap, but the JPEG is not lost");
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
    assert_eq!(ids.len(), 2, "one stored format goes out by two routes");
    let item = Item {
        kind: None,
        formats: vec![Format {
            id: SYNTHETIC_IMAGE.into(),
            payload: Payload::Inline(png),
        }],
    };
    assert_eq!(item.formats.len(), 1, "and it is still a single format");
}

#[test]
fn virtual_files_never_go_back_as_themselves_but_as_files_on_disk() {
    use crate::virtual_files::{DESCRIPTOR, contents_id, descriptor_of};
    let item = Item {
        kind: Some(cp_core::kind::Kind::File),
        formats: vec![
            inline(DESCRIPTOR, &descriptor_of(&[("note.txt", Some(4), false)])),
            Format {
                id: "FileContents".into(),
                payload: Payload::Announced { size: None },
            },
            inline(&contents_id(0), b"hello"),
        ],
    };
    let (owned, delivered) = pasted_files(&item);
    assert_eq!(
        delivered, 3,
        "the three virtual formats go out through the drop"
    );
    assert_eq!(owned.len(), 2, "CF_HDROP and the effect");
    assert_eq!(owned[0].0, CF_HDROP);
    let paths = crate::drop::paths_in(&owned[0].1);
    assert_eq!(paths.len(), 1);
    assert!(paths[0].ends_with("note.txt"), "{}", paths[0]);
    assert_eq!(std::fs::read(&paths[0]).expect("on disk"), b"hello");
    assert_eq!(
        owned[1].1,
        COPY.to_le_bytes(),
        "pasting copies, it does not move"
    );
    let folder = std::path::Path::new(&paths[0]).parent().expect("a folder");
    std::fs::remove_dir_all(folder).ok();
}

#[test]
fn an_item_without_virtual_files_pastes_no_files() {
    assert_eq!(pasted_files(&Item::plain("hello")), (Vec::new(), 0));
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
            inline("HTML Format", b"<b>hello</b>"),
            inline("CF_UNICODETEXT", &utf16_of("hello")),
        ],
    };
    let plain = render(Form::PlainText, &crate::content::content_of(&item, None))
        .expect("there is text")
        .into_item();
    let written: Vec<(u32, Vec<u8>)> = plain
        .formats
        .iter()
        .filter_map(|one| payload_of(one).map(|bytes| writable(&one.id, bytes)))
        .flatten()
        .collect();
    assert_eq!(written, vec![(CF_UNICODETEXT, utf16_of("hello"))]);
    assert_eq!(item.formats.len(), 2, "the stored item does not change");
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
        "with no plain text there is no plain form to offer"
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
