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
        formats: vec![inline(SYNTHETIC_TEXT, b"hello")],
    };
    assert_eq!(writable_of(&item), vec![(PLAIN_TEXT, &b"hello"[..])]);
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
fn a_rendered_jpeg_goes_back_as_the_type_the_system_reads() {
    let item = cp_core::paste_as::Rendered::Jpeg(vec![0xFF, 0xD8]).into_item();
    assert_eq!(writable_of(&item), vec![(JPEG, &[0xFFu8, 0xD8][..])]);
}

#[test]
fn a_captured_type_goes_back_under_its_own_name() {
    let item = Item {
        kind: None,
        formats: vec![
            inline("public.rtf", b"{\\rtf1 hello}"),
            inline(PLAIN_TEXT, b"hello"),
            Format {
                id: "public.tiff".into(),
                payload: Payload::Announced { size: None },
            },
        ],
    };
    let written = writable_of(&item);
    assert_eq!(
        written.len(),
        2,
        "what is announced without bytes is not written"
    );
    assert_eq!(written[0].0, "public.rtf");
    assert_eq!(written[1].0, PLAIN_TEXT);
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
}

#[test]
fn pasting_as_plain_text_is_a_rendered_form_written_like_any_item() {
    use cp_core::paste_as::{Form, render};
    let item = Item {
        kind: Some(cp_core::kind::Kind::Text),
        formats: vec![
            inline("public.html", b"<b>hello</b>"),
            inline(PLAIN_TEXT, b"hello"),
        ],
    };
    let content = crate::content::content_of(&item, None);
    let plain = render(Form::PlainText, &content)
        .expect("there is text")
        .into_item();
    assert_eq!(writable_of(&plain), vec![(PLAIN_TEXT, &b"hello"[..])]);
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

#[test]
fn one_file_stays_a_single_item() {
    let writable = vec![(FILE_URL, &b"file:///a/one.txt"[..])];
    assert!(
        one_item_per_file(&writable).is_none(),
        "a lone file needs no splitting, and the plain write already serves it"
    );
}

#[test]
fn several_files_become_one_item_each() {
    let joined = &b"file:///a/one.txt
file:///a/two.txt
file:///a/three.txt"[..];
    let per_file = one_item_per_file(&[(FILE_URL, joined)]).expect("three files, three items");
    assert_eq!(
        per_file.len(),
        3,
        "one pasteboard item per file, not one blob"
    );
    assert_eq!(
        per_file[1],
        vec![(FILE_URL, &b"file:///a/two.txt"[..])],
        "every item carries one url and nothing else"
    );
}

#[test]
fn the_other_formats_ride_on_the_first_file() {
    let writable = vec![
        (PLAIN_TEXT, &b"one.txt two.txt"[..]),
        (
            FILE_URL,
            &b"file:///a/one.txt
file:///a/two.txt"[..],
        ),
    ];
    let per_file = one_item_per_file(&writable).expect("two files");
    assert_eq!(per_file.len(), 2);
    assert!(
        per_file[0].iter().any(|(uti, _)| *uti == PLAIN_TEXT),
        "what is not a url travels with the first file"
    );
    assert!(
        per_file[1].iter().all(|(uti, _)| *uti == FILE_URL),
        "and is not repeated on the rest"
    );
}

#[test]
fn a_format_that_is_not_text_survives_the_split() {
    let png = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0xFF, 0xFE][..];
    let writable = vec![
        (PNG, png),
        (
            FILE_URL,
            &b"file:///a/one.png
file:///a/two.png"[..],
        ),
    ];
    let per_file = one_item_per_file(&writable).expect("two files");
    assert!(
        per_file[0]
            .iter()
            .any(|(uti, bytes)| *uti == PNG && *bytes == png),
        "bytes that are not valid text are not quietly dropped on the way out"
    );
}

#[test]
fn a_windows_line_ending_does_not_travel_inside_the_url() {
    let joined = &b"file:///a/one.txt
file:///a/two.txt"[..];
    let per_file = one_item_per_file(&[(FILE_URL, joined)]).expect("two files");
    assert_eq!(
        per_file[0],
        vec![(FILE_URL, &b"file:///a/one.txt"[..])],
        "a stray carriage return would make Finder look for a file nobody named"
    );
}

#[test]
fn an_empty_line_between_urls_is_not_a_file() {
    let joined = &b"file:///a/one.txt

file:///a/two.txt
"[..];
    let per_file = one_item_per_file(&[(FILE_URL, joined)]).expect("two files");
    assert_eq!(
        per_file.len(),
        2,
        "the blank lines do not become empty items"
    );
}

#[test]
fn a_path_offered_to_a_terminal_rides_along_as_the_plain_text() {
    let item = Item {
        kind: None,
        formats: vec![inline(SYNTHETIC_IMAGE, &[137, 80, 78, 71])],
    };
    let written = offering(writable_of(&item), Some("\"/tmp/a b.png\""));
    assert_eq!(
        written,
        vec![
            (PNG, &[137u8, 80, 78, 71][..]),
            (PLAIN_TEXT, &b"\"/tmp/a b.png\""[..])
        ]
    );
}

#[test]
fn an_offered_text_replaces_the_one_the_item_had_and_nothing_offered_changes_nothing() {
    let item = Item {
        kind: None,
        formats: vec![inline(PLAIN_TEXT, b"old"), inline("public.rtf", b"{\rtf1}")],
    };
    let written = offering(writable_of(&item), Some("new"));
    assert_eq!(
        written,
        vec![("public.rtf", &b"{\rtf1}"[..]), (PLAIN_TEXT, &b"new"[..])]
    );
    assert_eq!(offering(writable_of(&item), None), writable_of(&item));
    assert_eq!(offering(writable_of(&item), Some("")), writable_of(&item));
}

#[test]
fn a_format_announced_without_bytes_is_not_counted_as_one_that_fell_short() {
    let item = Item {
        kind: None,
        formats: vec![
            inline(PLAIN_TEXT, b"hello"),
            Format {
                id: "public.tiff".into(),
                payload: Payload::Announced { size: Some(9) },
            },
            Format {
                id: "public.png".into(),
                payload: Payload::TooBig { size: 1 << 30 },
            },
        ],
    };
    let writable = writable_of(&item);
    assert_eq!(writable.len(), 1, "only what carries bytes is written");
    assert_eq!(
        restored_of(writable.len(), Some(0)),
        Restored::Written {
            formats: 1,
            wanted: 1
        }
    );
}

#[test]
fn one_refused_file_among_several_counts_as_one_and_not_as_all_of_them() {
    let joined = &b"file:///a/one.txt
file:///a/two.txt
file:///a/three.txt"[..];
    let per_file = one_item_per_file(&[(FILE_URL, joined)]).expect("three files");
    let wanted = per_file.iter().map(Vec::len).sum();
    assert_eq!(
        restored_of(wanted, Some(1)),
        Restored::Written {
            formats: 2,
            wanted: 3
        }
    );
}

#[test]
fn nothing_landing_is_a_failure_and_not_a_short_write() {
    assert_eq!(restored_of(1, Some(1)), Restored::Failed);
    assert_eq!(restored_of(3, None), Restored::Failed);
}
