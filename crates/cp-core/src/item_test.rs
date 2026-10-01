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

#[test]
fn capitalisation_changes_the_identity_of_plain_text() {
    assert_ne!(
        Item::plain("TEXTO").fingerprint(),
        Item::plain("texto").fingerprint(),
        "the same word in another case is a different capture"
    );
}

#[test]
fn a_lone_nul_byte_does_not_panic_and_still_tells_content_apart() {
    assert_ne!(
        Item::plain("a\0b").fingerprint(),
        Item::plain("ab").fingerprint()
    );
    assert_eq!(
        Item::plain("a\0b").fingerprint(),
        Item::plain("a\0b").fingerprint()
    );
}

#[test]
fn crlf_and_lf_line_endings_are_different_identities() {
    assert_ne!(
        Item::plain("one\r\ntwo").fingerprint(),
        Item::plain("one\ntwo").fingerprint(),
        "line endings are bytes like any other; nothing normalises them here"
    );
}

#[test]
fn emoji_only_content_has_a_stable_and_distinct_identity() {
    assert_eq!(
        Item::plain("🚀🎉").fingerprint(),
        Item::plain("🚀🎉").fingerprint()
    );
    assert_ne!(
        Item::plain("🚀🎉").fingerprint(),
        Item::plain("🚀").fingerprint()
    );
}

#[test]
fn right_to_left_and_mixed_script_text_hashes_byte_for_byte() {
    let arabic = "مرحبا بالعالم";
    let hebrew = "שלום עולם";
    assert_eq!(
        Item::plain(arabic).fingerprint(),
        Item::plain(arabic).fingerprint()
    );
    assert_ne!(
        Item::plain(arabic).fingerprint(),
        Item::plain(hebrew).fingerprint()
    );
    assert_ne!(
        Item::plain("hello مرحبا").fingerprint(),
        Item::plain("مرحبا hello").fingerprint(),
        "the same two scripts in another order are different content"
    );
}

#[test]
fn the_same_letter_in_two_unicode_normal_forms_is_two_different_identities() {
    let nfc = "caf\u{00e9}";
    let nfd = "cafe\u{0301}";
    assert_ne!(
        nfc.as_bytes(),
        nfd.as_bytes(),
        "the two spellings really are different bytes"
    );
    assert_ne!(
        Item::plain(nfc).fingerprint(),
        Item::plain(nfd).fingerprint(),
        "the identity is raw bytes, with no Unicode normalisation; the same visible word \
         typed or pasted from a source that prefers the other normal form becomes a second item"
    );
}

fn rich(plain: &str, html: &str, rtf: &str) -> Item {
    Item {
        kind: Some(crate::kind::Kind::Text),
        formats: vec![
            Format {
                id: "public.utf8-plain-text".into(),
                payload: Payload::Inline(plain.as_bytes().to_vec()),
            },
            Format {
                id: "public.html".into(),
                payload: Payload::Inline(html.as_bytes().to_vec()),
            },
            Format {
                id: "public.rtf".into(),
                payload: Payload::Inline(rtf.as_bytes().to_vec()),
            },
        ],
    }
}

fn plain_only(text: &str) -> Item {
    Item {
        kind: Some(crate::kind::Kind::Text),
        formats: vec![Format {
            id: "public.utf8-plain-text".into(),
            payload: Payload::Inline(text.as_bytes().to_vec()),
        }],
    }
}

#[test]
fn the_plain_text_that_comes_out_of_a_rich_copy_has_an_identity_of_its_own() {
    let styled = rich("hello", "<b>hello</b>", r"{\rtf1 hello}");
    let replayed = plain_only("hello");
    assert_ne!(
        styled.fingerprint(),
        replayed.fingerprint(),
        "the rich capture mixes its markup into the identity, so the plain text \
         pasted out of it and copied again is a second item"
    );
    assert_eq!(
        replayed.fingerprint(),
        plain_only("hello").fingerprint(),
        "and that second item is stable, so copying it again does not pile up"
    );
}

#[test]
fn only_the_rendering_changing_leaves_the_identity_alone() {
    let one = rich("hello", "<b>hello</b>", r"{\rtf1\ansi hello}");
    let other = rich("hello", "<b>hello</b>", r"{\rtf1\mac\deff0 hello}");
    assert_ne!(one, other);
    assert_eq!(
        one.fingerprint(),
        other.fingerprint(),
        "two copies of the same paragraph that only differ in their RTF are one item"
    );
}

#[test]
fn the_markup_changing_is_a_different_item_even_with_the_same_plain_text() {
    let one = rich("hello", "<b>hello</b>", r"{\rtf1 hello}");
    let other = rich("hello", "<i>hello</i>", r"{\rtf1 hello}");
    assert_ne!(
        one.fingerprint(),
        other.fingerprint(),
        "bold and italic are not the same copy"
    );
}

#[test]
fn letter_case_is_content_and_not_a_spelling() {
    assert_ne!(
        Item::plain("TEXTO").fingerprint(),
        Item::plain("texto").fingerprint()
    );
}

#[test]
fn the_line_endings_of_a_text_are_part_of_what_was_copied() {
    assert_ne!(
        Item::plain("a\r\nb").fingerprint(),
        Item::plain("a\nb").fingerprint(),
        "the same two lines copied from a Windows editor and from a Unix one are two items"
    );
}

#[test]
fn a_byte_order_mark_is_content_like_any_other_byte() {
    assert_ne!(
        Item::plain("\u{feff}hello").fingerprint(),
        Item::plain("hello").fingerprint()
    );
}

#[test]
fn nothing_copied_and_an_empty_text_are_told_apart() {
    let nothing = Item {
        kind: None,
        formats: vec![],
    };
    assert_ne!(Item::plain("").fingerprint(), nothing.fingerprint());
    assert_eq!(Item::plain("").stored_bytes(), 0);
}

#[test]
fn control_characters_travel_whole_into_the_identity() {
    let with_nul = Item::plain("a\0b");
    assert_eq!(
        with_nul.stored_bytes(),
        3,
        "the nul is a byte like any other"
    );
    assert_ne!(with_nul.fingerprint(), Item::plain("ab").fingerprint());
    for odd in ["\u{7}", "\u{1b}[31m", "\u{200b}", "\u{202e}", "🏳️‍🌈"] {
        assert_eq!(
            Item::plain(odd).fingerprint(),
            Item::plain(odd).fingerprint()
        );
    }
}

#[test]
fn whitespace_that_looks_alike_never_collapses_into_one_item() {
    let shapes = [" ", "\n", "\t", "\u{a0}", "\u{3000}"];
    let mut seen: Vec<u64> = shapes
        .iter()
        .map(|one| Item::plain(one).fingerprint())
        .collect();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(
        seen.len(),
        shapes.len(),
        "none of them collapses into another"
    );
}

#[test]
fn a_blob_and_an_inline_payload_of_the_same_bytes_are_one_identity() {
    let bytes = vec![7u8; 32];
    let inline = Item {
        kind: None,
        formats: vec![Format {
            id: "public.png".into(),
            payload: Payload::Inline(bytes.clone()),
        }],
    };
    let blob = Item {
        kind: None,
        formats: vec![Format {
            id: "public.png".into(),
            payload: Payload::Blob(bytes),
        }],
    };
    assert_eq!(
        inline.fingerprint(),
        blob.fingerprint(),
        "where the bytes were kept is not part of what was copied"
    );
}

#[test]
fn what_is_too_big_to_keep_still_says_which_capture_it_was() {
    let refused = Item {
        kind: None,
        formats: vec![Format {
            id: "public.png".into(),
            payload: Payload::TooBig {
                size: BLOB_UP_TO + 1,
            },
        }],
    };
    assert_ne!(
        refused.fingerprint(),
        Item {
            kind: None,
            formats: vec![]
        }
        .fingerprint(),
        "sharing the hash of nothing at all made the first of them answer for every later one"
    );
    assert_eq!(refused.stored_bytes(), 0);
    assert!(
        !refused.is_comparable(),
        "and nobody can claim it is the same capture as another one nobody read either"
    );
}

#[test]
fn the_exact_size_that_still_fits_in_a_row_is_an_item_like_any_other() {
    for size in [INLINE_UP_TO - 1, INLINE_UP_TO, INLINE_UP_TO + 1] {
        let item = Item::plain(&"a".repeat(size));
        assert_eq!(item.stored_bytes(), size);
        assert!(
            !item.needs_blob_store(),
            "Item::plain always inlines, whatever placement would say about {size}"
        );
    }
    assert_eq!(placement(INLINE_UP_TO + 1), Placement::Blob);
}

fn one_format(id: &str, payload: Payload) -> Item {
    Item {
        kind: None,
        formats: vec![Format {
            id: id.into(),
            payload,
        }],
    }
}

#[test]
fn the_prints_of_what_can_be_read_are_the_ones_the_stored_rows_already_hold() {
    assert_eq!(
        Item::plain("copypaste").fingerprint(),
        0x0ec9_ae20_385c_1feb
    );
    assert_eq!(Item::plain("").fingerprint(), 0xcee0_578a_8eb7_fd5a);
    let two = Item {
        kind: Some(crate::kind::Kind::Text),
        formats: vec![
            Format {
                id: SYNTHETIC_TEXT.into(),
                payload: Payload::Inline(b"hola".to_vec()),
            },
            Format {
                id: "public.rtf".into(),
                payload: Payload::Inline(b"{\rtf1 hola}".to_vec()),
            },
        ],
    };
    assert_eq!(
        two.fingerprint(),
        0x6542_1aab_e9c5_2433,
        "every row on disk was deduplicated with these numbers"
    );
}

#[test]
fn the_order_the_formats_arrive_in_does_not_change_the_print() {
    let one = Item {
        kind: None,
        formats: vec![
            Format {
                id: "a".into(),
                payload: Payload::Inline(b"first".to_vec()),
            },
            Format {
                id: "b".into(),
                payload: Payload::TooBig { size: 70_000_000 },
            },
        ],
    };
    let other = Item {
        kind: None,
        formats: vec![
            Format {
                id: "b".into(),
                payload: Payload::TooBig { size: 70_000_000 },
            },
            Format {
                id: "a".into(),
                payload: Payload::Inline(b"first".to_vec()),
            },
        ],
    };
    assert_eq!(one.fingerprint(), other.fingerprint());
}

#[test]
fn two_captures_too_big_to_read_are_not_the_same_capture() {
    let one = one_format("public.png", Payload::TooBig { size: 70_000_000 });
    let other = one_format("public.png", Payload::TooBig { size: 90_000_000 });
    assert_ne!(
        one.fingerprint(),
        other.fingerprint(),
        "they used to share the hash of nothing at all, so the second never arrived"
    );
    let nothing = Item {
        kind: None,
        formats: Vec::new(),
    };
    assert_ne!(one.fingerprint(), nothing.fingerprint());
}

#[test]
fn what_was_never_read_cannot_be_compared_with_anything() {
    assert!(!one_format("public.png", Payload::TooBig { size: 70_000_000 }).is_comparable());
    assert!(!one_format("public.png", Payload::Announced { size: Some(10) }).is_comparable());
    assert!(!one_format("public.png", Payload::Absent).is_comparable());
    assert!(
        !Item {
            kind: None,
            formats: Vec::new()
        }
        .is_comparable()
    );
    assert!(Item::plain("something").is_comparable());
    assert!(one_format("public.png", Payload::Blob(vec![1, 2, 3])).is_comparable());
}
