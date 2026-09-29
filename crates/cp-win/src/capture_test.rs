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
