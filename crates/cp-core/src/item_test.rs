use super::*;

#[test]
fn small_things_stay_in_the_row() {
    let small = vec![0u8; 1024];
    assert!(matches!(Payload::stored(small), Payload::Inline(_)));
}

#[test]
fn the_boundary_belongs_to_the_row() {
    assert!(matches!(
        Payload::stored(vec![0u8; INLINE_UP_TO]),
        Payload::Inline(_)
    ));
    assert!(matches!(
        Payload::stored(vec![0u8; INLINE_UP_TO + 1]),
        Payload::Blob(_)
    ));
}

#[test]
fn the_three_placements_have_exact_boundaries() {
    assert_eq!(placement(0), Placement::Row);
    assert_eq!(placement(INLINE_UP_TO), Placement::Row);
    assert_eq!(placement(INLINE_UP_TO + 1), Placement::Blob);
    assert_eq!(placement(BLOB_UP_TO), Placement::Blob);
    assert_eq!(placement(BLOB_UP_TO + 1), Placement::Refused);
    assert_eq!(placement(usize::MAX), Placement::Refused);
}

#[test]
fn what_does_not_fit_is_recorded_not_dropped() {
    let huge = Payload::TooBig {
        size: BLOB_UP_TO + 1,
    };
    assert_eq!(huge.size(), Some(BLOB_UP_TO + 1), "the size is kept");
}

#[test]
fn an_announced_type_without_a_size_is_still_a_format() {
    let announced = Format {
        id: "com.apple.icns".into(),
        payload: Payload::Announced { size: None },
    };
    let item = Item {
        kind: None,
        formats: vec![announced],
    };
    assert!(item.format("com.apple.icns").is_some());
    assert_eq!(item.stored_bytes(), 0, "announced is not stored");
}

#[test]
fn two_items_that_look_alike_but_differ_have_different_identities() {
    let one = Item {
        kind: None,
        formats: vec![Format {
            id: "public.png".into(),
            payload: Payload::Inline(vec![1, 2, 3]),
        }],
    };
    let other = Item {
        kind: None,
        formats: vec![Format {
            id: "public.png".into(),
            payload: Payload::Inline(vec![1, 2, 4]),
        }],
    };
    assert_ne!(
        one.fingerprint(),
        other.fingerprint(),
        "two different captures with the same empty preview"
    );
}

#[test]
fn the_identity_does_not_depend_on_the_order_of_the_formats() {
    let a = Format {
        id: "public.rtf".into(),
        payload: Payload::Inline(vec![9]),
    };
    let b = Format {
        id: "public.utf8-plain-text".into(),
        payload: Payload::Inline(vec![8]),
    };
    let one = Item {
        kind: None,
        formats: vec![a.clone(), b.clone()],
    };
    let other = Item {
        kind: None,
        formats: vec![b, a],
    };
    assert_eq!(one.fingerprint(), other.fingerprint());
}

#[test]
fn two_copies_of_the_same_google_document_are_one_item() {
    let docs = |guid: &str| {
        Item {
        kind: Some(crate::kind::Kind::Text),
        formats: vec![
            Format {
                id: "public.html".into(),
                payload: Payload::Inline(
                    format!("<b style=\"font-weight:normal;\" id=\"docs-internal-guid-{guid}\"><span>hi</span></b>")
                        .into_bytes(),
                ),
            },
            Format {
                id: "public.utf8-plain-text".into(),
                payload: Payload::Inline(b"hi".to_vec()),
            },
        ],
    }
    };
    let first = docs("4a1e6b2f-7fff-1d3e-8c5a-2b3c4d5e6f70");
    let second = docs("0c9d8e7f-7fff-aaaa-bbbb-000000000001");
    assert_ne!(first, second, "the bytes really do differ");
    assert_eq!(first.fingerprint(), second.fingerprint());
    assert_ne!(
        first.fingerprint(),
        Item::plain("hi").fingerprint(),
        "plain text copied from where it was pasted is still a different item"
    );
    let other_words = docs("4a1e6b2f-7fff-1d3e-8c5a-2b3c4d5e6f70");
    let mut changed = other_words.clone();
    changed.formats[1].payload = Payload::Inline(b"bye".to_vec());
    assert_ne!(other_words.fingerprint(), changed.fingerprint());
}

#[test]
fn two_copies_of_the_same_word_paragraph_are_one_item() {
    let word = |rsid: &str, base: &str, stamp: u8| Item {
        kind: Some(crate::kind::Kind::Text),
        formats: vec![
            Format {
                id: "public.rtf".into(),
                payload: Payload::Inline(
                    format!("{{\\rtf1{{\\*\\rsidtbl \\rsid{rsid}}}\\insrsid{rsid} hi}}")
                        .into_bytes(),
                ),
            },
            Format {
                id: "public.utf8-plain-text".into(),
                payload: Payload::Inline(b"hi".to_vec()),
            },
            Format {
                id: "public.html".into(),
                payload: Payload::Inline(b"<p class=MsoNormal>hi</p>".to_vec()),
            },
            Format {
                id: "com.apple.webarchive".into(),
                payload: Payload::Inline(format!("applewebdata://{base}<html>").into_bytes()),
            },
            Format {
                id: "com.apple.flat-rtfd".into(),
                payload: Payload::Inline(vec![b'r', b't', b'f', b'd', stamp]),
            },
        ],
    };
    let first = word("11014195", "BD20AAE0-5062", 1);
    let second = word("15277860", "85F1BD98-1A2D", 2);
    assert_ne!(first, second);
    assert_eq!(
        first.fingerprint(),
        second.fingerprint(),
        "the rsids, the webarchive UUID and the RTFD timestamp are not content"
    );
    let mut other_words = second.clone();
    other_words.formats[2].payload = Payload::Inline(b"<p class=MsoNormal>bye</p>".to_vec());
    assert_ne!(first.fingerprint(), other_words.fingerprint());
}

#[test]
fn a_copy_that_is_only_a_rendering_still_has_an_identity_of_its_own() {
    let rtf = |body: &str| Item {
        kind: Some(crate::kind::Kind::Text),
        formats: vec![Format {
            id: "public.rtf".into(),
            payload: Payload::Inline(body.as_bytes().to_vec()),
        }],
    };
    assert_eq!(
        rtf("{\\rtf1 a}").fingerprint(),
        rtf("{\\rtf1 a}").fingerprint()
    );
    assert_ne!(
        rtf("{\\rtf1 a}").fingerprint(),
        rtf("{\\rtf1 b}").fingerprint()
    );
    assert_ne!(
        rtf("{\\rtf1 a}").fingerprint(),
        Item {
            kind: None,
            formats: vec![]
        }
        .fingerprint()
    );
}

#[test]
fn what_was_only_announced_does_not_change_the_identity() {
    let with_note = Item {
        kind: None,
        formats: vec![
            Format {
                id: "public.utf8-plain-text".into(),
                payload: Payload::Inline(b"hi".to_vec()),
            },
            Format {
                id: "com.apple.icns".into(),
                payload: Payload::Announced { size: Some(999) },
            },
        ],
    };
    let without = Item {
        kind: None,
        formats: vec![Format {
            id: "public.utf8-plain-text".into(),
            payload: Payload::Inline(b"hi".to_vec()),
        }],
    };
    assert_eq!(with_note.fingerprint(), without.fingerprint());
}

#[test]
fn an_item_says_when_it_needs_a_blob_store() {
    let small = Item {
        kind: None,
        formats: vec![Format {
            id: "t".into(),
            payload: Payload::Inline(vec![0; 10]),
        }],
    };
    let big = Item {
        kind: None,
        formats: vec![Format {
            id: "t".into(),
            payload: Payload::Blob(vec![0; 10]),
        }],
    };
    assert!(!small.needs_blob_store());
    assert!(big.needs_blob_store());
}

#[test]
fn only_what_was_really_kept_counts_towards_the_size() {
    let item = Item {
        kind: None,
        formats: vec![
            Format {
                id: "a".into(),
                payload: Payload::Inline(vec![0u8; 100]),
            },
            Format {
                id: "b".into(),
                payload: Payload::Blob(vec![0u8; 900]),
            },
            Format {
                id: "c".into(),
                payload: Payload::TooBig { size: 999_999 },
            },
            Format {
                id: "d".into(),
                payload: Payload::Absent,
            },
        ],
    };
    assert_eq!(item.stored_bytes(), 1000);
}
