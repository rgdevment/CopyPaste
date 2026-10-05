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
    assert!(written.iter().any(|(id, _)| *id == CF_DIB));
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
    assert!(written.iter().any(|(id, _)| *id == CF_DIB));
}

#[test]
fn a_jpeg_that_does_not_decode_still_goes_back_as_itself() {
    let written = writable(SYNTHETIC_JPEG, b"this is not a jpeg");
    assert_eq!(written.len(), 1, "no bitmap, but the JPEG is not lost");
    assert!(written.iter().all(|(id, _)| *id != CF_DIB));
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

#[test]
fn an_image_captured_on_windows_goes_back_as_a_bitmap_too() {
    let png = image_bytes();
    let written = writable(PNG, &png);
    assert!(
        written.iter().any(|(id, _)| *id == CF_DIB),
        "every app that wants a bitmap gets one, not only the private PNG atom"
    );
    assert!(
        written.iter().any(|(_, bytes)| bytes == &png),
        "the PNG travels intact"
    );
}

#[test]
fn the_bitmap_we_publish_has_the_header_its_name_promises() {
    let png = image_bytes();
    let written = writable(PNG, &png);
    let (_, raw) = written
        .iter()
        .find(|(id, _)| *id == CF_DIB)
        .expect("a bitmap travels with the png");
    let declared = u32::from_le_bytes(raw[..4].try_into().expect("a header starts the bitmap"));
    assert!(
        declared == 40 || declared == 108,
        "CF_DIB means a 40 or 108 byte header; {declared} would be a different format"
    );
}

#[test]
fn every_image_the_catalog_keeps_comes_back_as_a_bitmap_too() {
    let png = image_bytes();
    let jpeg = jpeg_bytes();
    for id in crate::formats::CATALOG.images_by_preference {
        let bytes = match *id {
            "JFIF" => &jpeg,
            _ => &png,
        };
        let written = writable(id, bytes);
        assert!(
            written.iter().any(|(kept, _)| *kept == CF_DIB)
                || written
                    .iter()
                    .any(|(kept, _)| *kept == id_of(id).unwrap_or_default()),
            "{id} is captured and comes back with nothing an app can read"
        );
    }
}

fn an_image() -> Ready {
    ready_for(&Item {
        kind: Some(cp_core::kind::Kind::Image),
        formats: vec![inline(SYNTHETIC_IMAGE, &image_bytes())],
    })
}

#[test]
fn a_browser_is_handed_the_image_and_a_file_it_can_name() {
    let mut ready = an_image();
    let before = ready.wanted();
    ready.offer_files(&[r"C:\Temp\CopyPaste\dragged\a3f1\logo final.png"]);
    assert!(
        ready.offers(CF_HDROP),
        "the file is what gives the upload its own name"
    );
    assert!(
        ready.offers(CF_DIB),
        "the image stays for whatever reads the bitmap"
    );
    let effect = id_of(DROP_EFFECT).expect("the system names its drop effect");
    assert!(
        ready.offers(effect),
        "a paste of a file says it is a copy, not a move"
    );
    assert_eq!(ready.wanted(), before + 2);
}

#[test]
fn offering_the_files_twice_leaves_one_list_of_them() {
    let mut ready = an_image();
    ready.offer_files(&[r"C:\a.png"]);
    ready.offer_files(&[r"C:\b.png"]);
    let lists = ready.owned.iter().filter(|(id, _)| *id == CF_HDROP).count();
    assert_eq!(lists, 1);
    assert!(
        ready
            .owned
            .iter()
            .any(|(id, bytes)| *id == CF_HDROP && *bytes == drop_of(&[r"C:\b.png"]))
    );
}

#[test]
fn a_terminal_is_handed_the_path_as_text_beside_the_image() {
    let mut ready = an_image();
    ready.offer_text(r#""C:\Temp\logo final.png""#);
    assert!(
        ready.owned.iter().any(|(id, bytes)| *id == CF_UNICODETEXT
            && *bytes == utf16_of(r#""C:\Temp\logo final.png""#))
    );
    assert!(ready.offers(CF_DIB));
    assert!(
        !ready.offers(CF_HDROP),
        "a terminal reads text first, and a file list would not help"
    );
}

#[test]
fn nothing_offered_changes_nothing() {
    let mut ready = an_image();
    let before = ready.wanted();
    ready.offer_files::<&str>(&[]);
    ready.offer_text("");
    assert_eq!(ready.wanted(), before);
    assert!(!ready.offers(CF_HDROP));
    assert!(!ready.offers(CF_UNICODETEXT));
}
