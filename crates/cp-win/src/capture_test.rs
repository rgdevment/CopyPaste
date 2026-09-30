use super::*;
use crate::drop::drop_of;
use crate::virtual_files::descriptor_of;

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
    assert_eq!(seen, None, "the thread is abandoned and not waited for");
}

#[test]
fn the_class_of_a_drop_comes_from_its_first_path() {
    let formats = vec![Format {
        id: "CF_HDROP".into(),
        payload: Payload::Inline(drop_of(&[r"C:\video.mkv"])),
    }];
    assert_eq!(refine(Some(Family::Files), &formats), Some(Kind::Video));
}

#[test]
fn a_folder_is_told_apart_by_its_trailing_separator() {
    let formats = vec![Format {
        id: "CF_HDROP".into(),
        payload: Payload::Inline(drop_of(&[r"C:\Documents\\"])),
    }];
    assert_eq!(refine(Some(Family::Files), &formats), Some(Kind::Folder));
}

#[test]
fn text_is_refined_by_what_it_says() {
    let formats = vec![Format {
        id: "CF_UNICODETEXT".into(),
        payload: Payload::Inline(cp_win_sys::writing::utf16_of("someone@example.test")),
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
fn a_virtual_file_is_classed_by_the_name_its_descriptor_gives() {
    let formats = vec![Format {
        id: DESCRIPTOR.into(),
        payload: Payload::Inline(descriptor_of(&[("capture.png", Some(9), false)])),
    }];
    assert_eq!(refine(Some(Family::Files), &formats), Some(Kind::Image));
    let folder = vec![Format {
        id: DESCRIPTOR.into(),
        payload: Payload::Inline(descriptor_of(&[("attachments", None, true)])),
    }];
    assert_eq!(refine(Some(Family::Files), &folder), Some(Kind::Folder));
    let real_drop_wins = vec![
        Format {
            id: "CF_HDROP".into(),
            payload: Payload::Inline(drop_of(&[r"C:\video.mkv"])),
        },
        folder[0].clone(),
    ];
    assert_eq!(
        refine(Some(Family::Files), &real_drop_wins),
        Some(Kind::Video)
    );
}

#[test]
fn only_a_virtual_item_with_something_absent_waits_for_the_ole_read() {
    let waiting = Item {
        kind: Some(Kind::File),
        formats: vec![
            Format {
                id: DESCRIPTOR.into(),
                payload: Payload::Inline(descriptor_of(&[("a.txt", Some(1), false)])),
            },
            Format {
                id: virtual_files::contents_id(0),
                payload: Payload::Absent,
            },
        ],
    };
    assert!(awaits_virtual_contents(&waiting));
    let too_big = Item {
        kind: Some(Kind::File),
        formats: vec![Format {
            id: virtual_files::contents_id(0),
            payload: Payload::TooBig { size: 1 << 40 },
        }],
    };
    assert!(
        !awaits_virtual_contents(&too_big),
        "what does not fit is not asked for"
    );
    let plain_absent = Item {
        kind: Some(Kind::Text),
        formats: vec![Format {
            id: "CF_UNICODETEXT".into(),
            payload: Payload::Absent,
        }],
    };
    assert!(!awaits_virtual_contents(&plain_absent));
    assert!(!awaits_virtual_contents(&Item::plain("hello")));
}

#[test]
fn a_drop_with_no_paths_is_still_a_file() {
    let formats = vec![Format {
        id: "CF_HDROP".into(),
        payload: Payload::Absent,
    }];
    assert_eq!(refine(Some(Family::Files), &formats), Some(Kind::File));
}

fn a_dib_of_one_colour() -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&2i32.to_le_bytes());
    out.extend_from_slice(&2i32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&32u16.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&[0u8; 16]);
    out.extend_from_slice(&[0x20, 0x60, 0xA0, 0xFF].repeat(4));
    out
}

#[test]
fn a_picture_on_the_clipboard_crosses_as_a_png_and_not_as_its_raw_bitmap() {
    let raw = a_dib_of_one_colour();
    {
        let clipboard = Clipboard::open().expect("the clipboard opens");
        clipboard.replace(&[(cp_win_sys::formats::CF_DIB, &raw)]);
    }
    let clipboard = Clipboard::open().expect("the clipboard opens");
    let item = capture(&clipboard).kept().expect("a picture was captured");
    assert_eq!(item.kind, Some(Kind::Image));
    let picture = item
        .formats
        .iter()
        .find(|one| one.id == SYNTHETIC_IMAGE)
        .expect("the picture is carried under its own name");
    let bytes = match &picture.payload {
        Payload::Inline(bytes) => bytes.clone(),
        Payload::Blob(bytes) => bytes.clone(),
        other => panic!("the picture arrived as {other:?}"),
    };
    assert_eq!(
        &bytes[..8],
        b"\x89PNG\r\n\x1a\n",
        "a bitmap off the clipboard is transcoded, because nothing else reads a raw DIB"
    );
}

#[test]
fn a_read_that_was_given_up_cannot_have_the_clipboard_emptied_under_it() {
    let big: Vec<u8> = "abcdefghij"
        .repeat(200_000)
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();
    let mut refused = 0;
    for _ in 0..60 {
        let _ = capture_within(std::time::Duration::from_nanos(1));
        match Clipboard::open() {
            Some(clipboard) => {
                clipboard.replace(&[(cp_win_sys::formats::CF_UNICODETEXT, &big)]);
            }
            None => refused += 1,
        }
    }
    assert!(
        refused < 60,
        "every single attempt was refused, so nothing was exercised"
    );
}
