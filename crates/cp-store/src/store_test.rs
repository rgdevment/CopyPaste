use super::*;
use cp_core::item::Format;

#[test]
fn a_pinned_search_lets_the_index_drive_and_never_scans_the_fts_per_row() {
    let store = seeded();
    for filter in [
        Filter {
            query: Some("r".into()),
            pinned_only: true,
            ..Default::default()
        },
        Filter {
            query: Some("r".into()),
            kinds: vec![Kind::Text],
            ..Default::default()
        },
    ] {
        let clauses = Clauses::of(&filter, true, true).expect("clauses");
        let sql = format!("EXPLAIN QUERY PLAN SELECT COUNT(*) {}", clauses.source());
        let mut stmt = store.db.prepare(&sql).expect("a plan");
        let steps: Vec<String> = stmt
            .query_map(params_from_iter(clauses.bound.iter()), |row| {
                row.get::<_, String>(3)
            })
            .expect("a plan")
            .map(|step| step.expect("a step"))
            .collect();
        assert!(
            steps
                .first()
                .is_some_and(|first| first.contains("items_fts")),
            "el FTS tiene que ser el bucle exterior: {steps:?}"
        );
        assert!(
            !steps.iter().any(|step| step.contains("items_pinned")),
            "the partial pinned index made the FTS get swept on every row: {steps:?}"
        );
    }
}

fn seeded() -> Store {
    let store = Store::in_memory().expect("schema");
    for (at, text) in [
        "the café on the corner",
        "Straße Hauptbahnhof",
        "encyclopædia britannica",
        "Łódź centrum",
        "Peçanha e Gonçalves",
    ]
    .iter()
    .enumerate()
    {
        store
            .insert_text(&format!("uuid-{at}"), text, at as i64)
            .expect("insert");
    }
    store
}

#[test]
fn the_four_cases_that_2x_gets_wrong() {
    let store = seeded();
    for (query, expected) in [
        ("cafe", "the café on the corner"),
        ("strasse", "Straße Hauptbahnhof"),
        ("encyclopaedia", "encyclopædia britannica"),
        ("lodz", "Łódź centrum"),
    ] {
        let hits = search(&store, query);
        assert!(
            hits.iter().any(|hit| hit == expected),
            "searching «{query}» did not turn up «{expected}»: {hits:?}"
        );
    }
}

#[test]
fn it_works_in_both_directions() {
    let store = seeded();
    for (query, expected) in [
        ("café", "the café on the corner"),
        ("Straße", "Straße Hauptbahnhof"),
        ("encyclopædia", "encyclopædia britannica"),
        ("Łódź", "Łódź centrum"),
        ("Gonçalves", "Peçanha e Gonçalves"),
    ] {
        let hits = search(&store, query);
        assert!(
            hits.iter().any(|hit| hit == expected),
            "searching «{query}» did not turn up «{expected}»: {hits:?}"
        );
    }
}

#[test]
fn the_stored_text_keeps_its_accents() {
    let store = seeded();
    let hits = search(&store, "cafe");
    assert_eq!(
        hits.first().map(String::as_str),
        Some("the café on the corner"),
        "it is searched without accents but shown just as it was copied"
    );
}

fn sample_item() -> Item {
    Item {
        kind: Some(cp_core::kind::Kind::Text),
        formats: vec![
            Format {
                id: "public.utf8-plain-text".into(),
                payload: Payload::Inline(b"hello".to_vec()),
            },
            Format {
                id: "public.rtf".into(),
                payload: Payload::Inline(vec![0u8; 400]),
            },
            Format {
                id: "com.apple.icns".into(),
                payload: Payload::Announced { size: None },
            },
            Format {
                id: "fndf".into(),
                payload: Payload::Absent,
            },
        ],
    }
}

#[test]
fn an_item_too_big_for_the_row_is_refused_not_emptied() {
    let store = Store::in_memory().expect("schema");
    let big = Item {
        kind: None,
        formats: vec![Format {
            id: "public.png".into(),
            payload: Payload::Blob(vec![0u8; 100_000]),
        }],
    };
    assert!(
        store.insert_item("uuid-grande", &big, "", 1).is_err(),
        "better to refuse than store an item without its bytes"
    );
    assert_eq!(store.count().expect("counted"), 0);
}

#[test]
fn two_images_with_no_preview_are_two_items() {
    let store = Store::in_memory().expect("schema");
    let image = |byte: u8| Item {
        kind: Some(cp_core::kind::Kind::Image),
        formats: vec![Format {
            id: "public.png".into(),
            payload: Payload::Inline(vec![byte; 512]),
        }],
    };
    store
        .insert_item("uuid-a", &image(1), "", 1)
        .expect("insert");
    store
        .insert_item("uuid-b", &image(2), "", 2)
        .expect("insert");
    let hashes: Vec<i64> = store
        .db
        .prepare("SELECT content_hash FROM items ORDER BY id")
        .expect("prepared")
        .query_map([], |row| row.get(0))
        .expect("queried")
        .map(|row| row.expect("a row"))
        .collect();
    assert_ne!(
        hashes[0], hashes[1],
        "hashing the empty preview would make them the same"
    );
}

#[test]
fn an_item_keeps_every_format_it_was_offered() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_item("uuid-multi", &sample_item(), "hello", 1)
        .expect("insert");
    let formats = store.formats_of(id).expect("formats");
    assert_eq!(formats.len(), 4, "all four rows, including the empty ones");
    assert!(formats.contains(&"com.apple.icns".to_string()));
    assert!(formats.contains(&"fndf".to_string()));
}

#[test]
fn the_same_content_is_found_by_its_hash() {
    let store = Store::in_memory().expect("schema");
    store.insert_text("uuid-a", "repetido", 1).expect("insert");
    assert!(
        store
            .find_by_hash(&Item::plain("repetido"))
            .expect("searched")
            .is_some()
    );
    assert!(
        store
            .find_by_hash(&Item::plain("distinto"))
            .expect("searched")
            .is_none()
    );
}

#[test]
fn deleting_an_item_takes_its_formats_with_it() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_item("uuid-cascade", &sample_item(), "hello", 1)
        .expect("insert");
    store.mark_broken(id, 10).expect("marked");
    store.purge_broken_before(20).expect("purged");
    assert_eq!(store.formats_of(id).expect("formats").len(), 0);
}

#[test]
fn a_broken_item_survives_until_its_time_is_up() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_text("uuid-roto", "archivo ido", 1)
        .expect("insert");
    store.mark_broken(id, 100).expect("marked");
    assert_eq!(store.purge_broken_before(50).expect("purged"), 0);
    assert_eq!(
        store.count().expect("counted"),
        1,
        "the deadline has not been met yet"
    );
    assert_eq!(store.purge_broken_before(150).expect("purged"), 1);
    assert_eq!(store.count().expect("counted"), 0);
}

#[test]
fn marking_a_broken_item_twice_does_not_restart_its_clock() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_text("uuid-roto", "archivo ido", 1)
        .expect("insert");
    store.mark_broken(id, 100).expect("first");
    store.mark_broken(id, 900).expect("second");
    assert_eq!(
        store.purge_broken_before(150).expect("purged"),
        1,
        "it is the first time it was seen broken that counts, not the last"
    );
}

#[test]
fn a_pinned_item_is_never_purged_even_when_broken() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_text("uuid-fijado", "importante", 1)
        .expect("insert");
    store.set_pinned(id, true, 0).expect("pinned");
    store.mark_broken(id, 100).expect("marked");
    assert_eq!(store.purge_broken_before(9999).expect("purged"), 0);
    assert_eq!(store.count().expect("counted"), 1);
}

#[test]
fn no_query_a_person_can_type_breaks_the_search() {
    let store = seeded();
    for query in [
        "\"",
        "\"\"",
        "*",
        "(",
        ")",
        "()",
        "a AND b",
        "NOT café",
        "search_text:café",
        "NEAR(a b)",
        "-café",
        "^café",
        "café*",
        "{café}",
        "[café]",
        "café OR",
        "OR",
        "AND OR NOT",
        "",
        " ",
        "\t\n",
        "...",
        "!!!",
        "\\",
        "%",
        "_",
        "'; DROP TABLE items; --",
    ] {
        store
            .list(
                &Filter {
                    query: Some(query.into()),
                    ..Default::default()
                },
                10,
                None,
            )
            .unwrap_or_else(|why| panic!("«{query}» broke the search: {why}"));
    }
}

#[test]
fn an_empty_search_returns_nothing_rather_than_everything() {
    let store = seeded();
    for empty in ["", "   ", "\t", "-", "!!", "***"] {
        assert!(
            search(&store, empty).is_empty(),
            "«{empty}» should not return anything"
        );
    }
}

#[test]
fn the_punctuation_around_a_word_does_not_hide_it() {
    let store = seeded();
    for query in ["-café", "^café", "(café)", "«café»", "café!"] {
        let hits = search(&store, query);
        assert!(
            hits.iter().any(|hit| hit.contains("café")),
            "«{query}» did not find the café"
        );
    }
}

#[test]
fn scripts_that_are_not_latin_go_in_and_come_out() {
    let store = Store::in_memory().expect("schema");
    for (at, text) in [
        "日本語のテキスト",
        "Привет мир",
        "مرحبا بالعالم",
        "🎉 party 🎊",
        "한국어 텍스트",
    ]
    .iter()
    .enumerate()
    {
        store
            .insert_text(&format!("uuid-{at}"), text, at as i64)
            .expect("insert");
    }
    for (query, expected) in [
        ("日本語", "日本語のテキスト"),
        ("Привет", "Привет мир"),
        ("party", "🎉 party 🎊"),
        ("한국어", "한국어 텍스트"),
    ] {
        let hits = search(&store, query);
        assert!(
            hits.iter().any(|hit| hit == expected),
            "searching «{query}» was missing «{expected}»: {hits:?}"
        );
    }
}

#[test]
fn a_very_long_text_is_stored_and_found() {
    let store = Store::in_memory().expect("schema");
    let long = format!("{} needle {}", "hay ".repeat(50_000), "hay ".repeat(50_000));
    store.insert_text("uuid-long", &long, 1).expect("insert");
    assert_eq!(search(&store, "needle").len(), 1);
}

#[test]
fn a_multibyte_character_straddling_the_stored_preview_ceiling_does_not_panic() {
    let store = Store::in_memory().expect("schema");
    let padding = "a".repeat(PREVIEW_UP_TO - 1);
    let text = format!("{padding}🎉tail");
    store.insert_text("uuid-straddling", &text, 1).expect(
        "the manual byte-boundary search in head_of must not panic or truncate mid-character",
    );
    assert_eq!(store.count().expect("counted"), 1);
}

#[test]
fn secrets_are_kept_and_searchable_like_any_other_text_with_no_redaction() {
    let store = Store::in_memory().expect("schema");
    let fakes = [
        "AKIAFAKEFAKEFAKEFAKE",
        "-----BEGIN FAKE PRIVATE KEY-----\nnot a real key\n-----END FAKE PRIVATE KEY-----",
        "correct horse battery staple",
        "4111 1111 1111 1111",
    ];
    for (at, secret) in fakes.iter().enumerate() {
        store
            .insert_text(&format!("uuid-secret-{at}"), secret, at as i64)
            .expect("insert");
    }
    assert_eq!(
        search(&store, "AKIAFAKEFAKEFAKEFAKE"),
        vec!["AKIAFAKEFAKEFAKEFAKE"],
        "a clipboard history is, by design, a plain-text record of everything copied, \
         secrets included; nothing here masks, flags, or excludes them"
    );
    assert_eq!(search(&store, "battery staple").len(), 1);
    assert_eq!(search(&store, "4111").len(), 1);
    let found = store
        .find_by_hash(&Item::plain(
            "-----BEGIN FAKE PRIVATE KEY-----\nnot a real key\n-----END FAKE PRIVATE KEY-----",
        ))
        .expect("searched");
    assert!(found.is_some(), "the key text round-trips byte for byte");
}

#[test]
fn the_same_uuid_twice_is_refused_not_duplicated() {
    let store = Store::in_memory().expect("schema");
    store
        .insert_text("uuid-unique", "first", 1)
        .expect("insert");
    assert!(
        store.insert_text("uuid-unique", "second", 2).is_err(),
        "the uuid is unique by contract"
    );
    assert_eq!(store.count().expect("counted"), 1);
}

#[test]
fn marking_an_item_that_does_not_exist_is_not_a_failure() {
    let store = Store::in_memory().expect("schema");
    store
        .mark_broken(9999, 1)
        .expect("it does not exist, and nothing happens");
    assert_eq!(store.count().expect("counted"), 0);
}

#[test]
fn a_purge_with_nothing_to_purge_removes_nothing() {
    let store = seeded();
    let before = store.count().expect("counted");
    assert_eq!(store.purge_broken_before(-1).expect("purged"), 0);
    assert_eq!(store.purge_broken_before(i64::MAX).expect("purged"), 0);
    assert_eq!(store.count().expect("counted"), before);
}

#[test]
fn an_item_with_no_formats_at_all_is_still_an_item() {
    let store = Store::in_memory().expect("schema");
    let empty = Item {
        kind: None,
        formats: vec![],
    };
    let id = store
        .insert_item("uuid-empty", &empty, "", 1)
        .expect("insert");
    assert_eq!(store.formats_of(id).expect("formats").len(), 0);
    assert_eq!(store.count().expect("counted"), 1);
}

#[test]
fn a_search_that_matches_everything_still_returns_one_page() {
    let store = Store::in_memory().expect("schema");
    for at in 0..250 {
        store
            .insert_text(&format!("uuid-{at}"), &format!("common {at}"), at)
            .expect("insert");
    }
    let page = search(&store, "common");
    assert_eq!(page.len(), Store::PAGE);
}

#[test]
fn the_cursor_walks_a_search_without_repeating_or_skipping() {
    let store = Store::in_memory().expect("schema");
    for at in 0..25 {
        store
            .insert_text(&format!("uuid-{at}"), &format!("cursor {at}"), at)
            .expect("insert");
    }
    let filter = Filter {
        query: Some("cursor".into()),
        ..Default::default()
    };
    let mut seen = Vec::new();
    let mut after = None;
    loop {
        let page = store.list(&filter, 10, after).expect("queried");
        seen.extend(page.rows.into_iter().map(|one| one.preview));
        match page.next {
            Some(cursor) => after = Some(cursor),
            None => break,
        }
    }
    assert_eq!(seen.len(), 25, "went through it all without getting stuck");
    let mut unique = seen.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), 25, "without repeating");
}

#[test]
fn copying_something_again_lifts_it_instead_of_duplicating_it() {
    let store = Store::in_memory().expect("schema");
    let first = store
        .insert_text("uuid-a", "the old one", 10)
        .expect("insert");
    store
        .insert_text("uuid-b", "the new one", 20)
        .expect("insert");

    let before = search(&store, "the");
    assert_eq!(before.first().map(String::as_str), Some("the new one"));

    store.reactivate(first, 30).expect("copied again");
    let after = search(&store, "the");
    assert_eq!(
        after.first().map(String::as_str),
        Some("the old one"),
        "copying something again lifts it to the top"
    );
}

#[test]
fn a_label_can_be_searched_for() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_text("uuid-etq", "some random text", 1)
        .expect("insert");
    assert!(search(&store, "invoice").is_empty());
    store
        .set_label(id, Some("Invoice May"), 2)
        .expect("labelled");
    let hits = search(&store, "invoice");
    assert_eq!(hits.len(), 1, "the label gets into the index");
}

#[test]
fn the_source_application_can_be_searched_for() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_text("uuid-app", "something copied", 1)
        .expect("insert");
    store.set_source(id, "Safari", 2).expect("sourced");
    assert_eq!(search(&store, "safari").len(), 1);
}

#[test]
fn a_label_with_accents_is_found_without_them() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_text("uuid-tilde", "content", 1)
        .expect("insert");
    store
        .set_label(id, Some("Design Meeting"), 2)
        .expect("labelled");
    assert_eq!(search(&store, "meeting").len(), 1);
    assert_eq!(search(&store, "design").len(), 1);
}

#[test]
fn removing_a_label_takes_it_out_of_the_index() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_text("uuid-quita", "content", 1)
        .expect("insert");
    store.set_label(id, Some("temporary"), 2).expect("set");
    assert_eq!(search(&store, "temporary").len(), 1);
    store.set_label(id, None, 3).expect("cleared");
    assert!(search(&store, "temporary").is_empty());
}

#[test]
fn deleting_hides_the_item_from_everything_the_user_can_see() {
    let store = seeded();
    let id = store
        .insert_text("uuid-secreto", "the bank password", 500)
        .expect("insert");
    let before = store.count().expect("counted");

    store.mark_deleted(id, 600).expect("removed");

    assert_eq!(
        store.count().expect("counted"),
        before - 1,
        "stops counting"
    );
    assert!(
        search(&store, "password").is_empty(),
        "it can no longer be found"
    );
    assert!(
        store
            .find_by_hash(&Item::plain("the bank password"))
            .expect("hash")
            .is_none(),
        "copying it again must create a new item, not resurrect the tombstone"
    );
}

#[test]
fn deleting_an_item_takes_its_thumbnail_off_the_disk() {
    let (dir, store) = on_disk();
    let made = dir.path().join("a.png");
    std::fs::write(&made, b"not a real png, but it has heft").expect("written");
    let id = store
        .insert_item("uuid-with-thumbnail", &sample_item(), "something", 1)
        .expect("insert");
    store
        .set_thumb(id, Some(&made.to_string_lossy()), 2)
        .expect("a thumbnail");
    store.mark_deleted(id, 3).expect("removed");
    assert!(!made.exists(), "the thumbnail survived the deletion");
}

#[test]
fn emptying_the_history_takes_every_thumbnail_with_it() {
    let (dir, store) = on_disk();
    let made = dir.path().join("another.png");
    std::fs::write(&made, b"this is not a png either").expect("written");
    let id = store
        .insert_item("uuid-emptied", &sample_item(), "something", 1)
        .expect("insert");
    store
        .set_thumb(id, Some(&made.to_string_lossy()), 2)
        .expect("a thumbnail");
    store.clear_all_unpinned(3).expect("emptied");
    assert!(!made.exists(), "emptying left the thumbnail on disk");
}

#[test]
fn a_deleted_item_leaves_no_content_behind() {
    let store = Store::in_memory().expect("schema");
    let item = sample_item();
    let id = store
        .insert_item("uuid-borrado", &item, "plain text", 1)
        .expect("insert");
    store.set_label(id, Some("label"), 2).expect("labelled");
    store.mark_deleted(id, 3).expect("removed");

    let (preview, search, label): (String, String, Option<String>) = store
        .db
        .query_row(
            "SELECT preview_text, search_text, label FROM items WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("queried");
    assert_eq!(preview, "", "the plain content goes away");
    assert_eq!(search, "", "and so does its copy in the index");
    assert_eq!(label, None);
    assert_eq!(
        store.formats_of(id).expect("formats").len(),
        0,
        "the bytes of the formats go away with the item"
    );
}

#[test]
fn the_tombstone_still_tells_the_sync_what_happened() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_text("uuid-tumba", "goes away", 1)
        .expect("insert");
    store.mark_deleted(id, 50).expect("removed");
    assert!(
        store
            .changed_since(40)
            .expect("changes")
            .contains(&"uuid-tumba".to_string()),
        "without this, another machine resurrects it"
    );
}

#[test]
fn only_what_changed_after_the_mark_is_reported() {
    let store = Store::in_memory().expect("schema");
    let old = store
        .insert_text("uuid-old", "ancient", 10)
        .expect("insert");
    store
        .insert_text("uuid-new", "recent", 100)
        .expect("insert");
    let changed = store.changed_since(50).expect("changes");
    assert_eq!(changed, vec!["uuid-new".to_string()]);

    store.reactivate(old, 200).expect("copied again");
    let after = store.changed_since(50).expect("changes");
    assert_eq!(
        after,
        vec!["uuid-new".to_string(), "uuid-old".to_string()],
        "ordered by version, and the recopy now counts"
    );
}

#[test]
fn copying_something_again_does_not_inflate_the_paste_counter() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_text("uuid-recopiado", "something", 1)
        .expect("insert");
    for at in 2..10 {
        store.reactivate(id, at).expect("copied again");
    }
    assert_eq!(
        store.paste_count(id).expect("counted"),
        0,
        "copying again is not pasting, and the card's ×N shows it"
    );
}

#[test]
fn pasting_from_the_history_is_what_counts() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_text("uuid-pegado", "something", 1)
        .expect("insert");
    store.record_paste(id, 2).expect("pasted");
    store.record_paste(id, 3).expect("pasted");
    assert_eq!(store.paste_count(id).expect("counted"), 2);
}

#[test]
fn pasting_moves_the_item_to_the_top_of_the_history() {
    let store = a_little_history();
    let listed = store
        .list(&Filter::default(), 10, None)
        .expect("listed")
        .rows;
    let oldest = listed.last().expect("there is one").id;
    store.record_paste(oldest, 999).expect("pasted");
    let after = store
        .list(&Filter::default(), 10, None)
        .expect("listed")
        .rows;
    assert_eq!(
        after.first().map(|one| one.id),
        Some(oldest),
        "what you paste is what you reach for next"
    );
}

#[test]
fn the_colour_can_be_set_and_filtered_by() {
    let store = a_little_history();
    let listed = store
        .list(&Filter::default(), 10, None)
        .expect("listed")
        .rows;
    store.set_color(listed[0].id, 3, 100).expect("color");
    let filter = Filter {
        colors: vec![3],
        ..Default::default()
    };
    let coloured = store.list(&filter, 10, None).expect("listed").rows;
    assert_eq!(coloured.len(), 1);
    assert_eq!(coloured[0].id, listed[0].id);
}

#[test]
fn an_image_becomes_findable_by_what_is_written_inside_it() {
    let store = Store::in_memory().expect("schema");
    let image = Item {
        kind: Some(cp_core::kind::Kind::Image),
        formats: vec![Format {
            id: "public.png".into(),
            payload: Payload::Inline(vec![137, 80, 78, 71]),
        }],
    };
    let id = store
        .insert_item("uuid-capture", &image, "", 1)
        .expect("insert");

    assert!(
        search(&store, "order").is_empty(),
        "it has not been through OCR yet"
    );
    assert_eq!(store.pending_ocr(10).expect("pending"), vec![id]);

    store
        .set_ocr_text(id, "Order AB-4417 delivery 12 March", 2)
        .expect("ocr");

    assert_eq!(
        search(&store, "order").len(),
        1,
        "a screenshot can be found by what it says inside"
    );
    assert_eq!(search(&store, "AB-4417").len(), 1);
    assert!(
        store.pending_ocr(10).expect("pending").is_empty(),
        "it is no longer pending"
    );
}

#[test]
fn the_ocr_text_is_folded_like_everything_else() {
    let store = Store::in_memory().expect("schema");
    let image = Item {
        kind: Some(cp_core::kind::Kind::Image),
        formats: vec![Format {
            id: "public.png".into(),
            payload: Payload::Inline(vec![1]),
        }],
    };
    let id = store
        .insert_item("uuid-tilde", &image, "", 1)
        .expect("insert");
    store.set_ocr_text(id, "Meeting in Munich", 2).expect("ocr");
    assert_eq!(search(&store, "meeting").len(), 1);
    assert_eq!(search(&store, "munich").len(), 1);
    assert_eq!(
        store.ocr_text(id).expect("read").as_deref(),
        Some("Meeting in Munich"),
        "what gets pasted or shown keeps its capitals and accents"
    );
}

#[test]
fn the_recognised_text_is_gone_with_the_item_and_with_an_edit() {
    let store = Store::in_memory().expect("schema");
    let image = Item {
        kind: Some(cp_core::kind::Kind::Image),
        formats: vec![Format {
            id: "public.png".into(),
            payload: Payload::Inline(vec![1]),
        }],
    };
    let id = store
        .insert_item("uuid-raw", &image, "", 1)
        .expect("insert");
    assert_eq!(store.ocr_text(id).expect("read"), None);
    store.set_ocr_text(id, "Invoice 77", 2).expect("ocr");
    store.update_text(id, "already text", 3).expect("edited");
    assert_eq!(
        store.ocr_text(id).expect("read"),
        None,
        "the edited text replaces the image and what was read inside it"
    );
    store.set_ocr_text(id, "Invoice 78", 4).expect("ocr");
    assert_eq!(
        store.ocr_text(id).expect("read").as_deref(),
        Some("Invoice 78")
    );
    store.mark_deleted(id, 5).expect("removed");
    assert_eq!(store.ocr_text(id).expect("read"), None);
    assert!(
        store
            .raw()
            .query_row(
                "SELECT ocr_text IS NULL AND search_ocr = '' FROM items WHERE id = ?1",
                [id],
                |row| row.get::<_, bool>(0),
            )
            .expect("queried"),
        "deleting clears both columns, not just the searchable one"
    );
    assert!(
        store.set_ocr_text(id, "late", 6).is_ok(),
        "an OCR that arrives after the deletion resurrects nothing"
    );
    assert_eq!(store.ocr_text(id).expect("read"), None);
}

#[test]
fn deleting_takes_the_recognised_text_with_it() {
    let store = Store::in_memory().expect("schema");
    let image = Item {
        kind: Some(cp_core::kind::Kind::Image),
        formats: vec![Format {
            id: "public.png".into(),
            payload: Payload::Inline(vec![1]),
        }],
    };
    let id = store
        .insert_item("uuid-secret", &image, "", 1)
        .expect("insert");
    store.set_ocr_text(id, "recovery key 8842", 2).expect("ocr");
    store.mark_deleted(id, 3).expect("removed");
    assert!(
        search(&store, "recovery").is_empty(),
        "what was read inside the image is also the user's content"
    );
}

fn nowhere_on_disk(dir: &std::path::Path, words: &[&str]) {
    for file in ["history.db", "history.db-wal"] {
        let bytes = std::fs::read(dir.join(file)).expect("can be read");
        for word in words {
            assert!(
                !bytes
                    .windows(word.len())
                    .any(|window| window == word.as_bytes()),
                "«{word}» is still legible in {file}"
            );
        }
    }
}

#[test]
fn a_deleted_secret_is_not_left_lying_in_the_write_ahead_log() {
    let dir = tempfile::tempdir().expect("a folder");
    let path = dir.path().join("history.db");
    let secret = "zqxjkvbnm7hunter2 bank mail";
    let store = Store::open(&path).expect("opened");
    let id = store.insert_text("uuid-secret", secret, 1).expect("insert");

    store.mark_deleted(id, 2).expect("removed");

    nowhere_on_disk(dir.path(), &["zqxjkvbnm7hunter2", "bank", "mail"]);
}

#[test]
fn the_search_index_forgets_every_token_of_what_was_removed() {
    let dir = tempfile::tempdir().expect("a folder");
    let store = Store::open(&dir.path().join("history.db")).expect("opened");
    type Removal = dyn Fn(&Store, i64);
    let cases: [(&str, &Removal); 4] = [
        ("qwzplk1secret", &|store, id| {
            store.mark_deleted(id, 9).expect("removed")
        }),
        ("qwzplk2secret", &|store, id| {
            store.update_text(id, "innocent", 9).expect("edited")
        }),
        ("qwzplk3secret", &|store, id| {
            store.set_label(id, Some("qwzplk3label"), 8).expect("set");
            store.set_label(id, None, 9).expect("cleared");
        }),
        ("qwzplk4secret", &|store, id| {
            store.mark_broken(id, 8).expect("broken");
            store.purge_broken_before(10).expect("purged");
        }),
    ];
    for (at, (word, remove)) in cases.into_iter().enumerate() {
        let id = store
            .insert_text(&format!("uuid-{at}"), word, 1)
            .expect("insert");
        remove(&store, id);
    }
    nowhere_on_disk(
        dir.path(),
        &[
            "qwzplk1secret",
            "qwzplk2secret",
            "qwzplk3label",
            "qwzplk4secret",
        ],
    );
}

#[test]
fn what_is_written_survives_closing_the_application() {
    let dir = tempfile::tempdir().expect("a folder");
    let path = dir.path().join("sub").join("history.db");

    {
        let store = Store::open(&path).expect("opened");
        store
            .insert_text("uuid-persists", "survives", 1)
            .expect("insert");
        store.checkpoint().expect("checkpoint");
    }

    let reopened = Store::open(&path).expect("reopened");
    assert_eq!(reopened.count().expect("counted"), 1);
    assert_eq!(
        search(&reopened, "survives").len(),
        1,
        "and the index survives too"
    );
}

#[cfg(unix)]
#[test]
fn the_write_ahead_log_is_as_private_as_the_database() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().expect("a folder");
    let path = dir.path().join("history.db");
    let store = Store::open(&path).expect("opened");
    store
        .insert_text("uuid-private", "password", 1)
        .expect("insert");

    let wal = sidecars(&path)
        .into_iter()
        .find(|side| side.exists())
        .expect("the WAL exists while the database is open");
    let mode = std::fs::metadata(&wal).expect("wal").permissions().mode() & 0o777;
    assert_eq!(
        mode, 0o600,
        "what was just copied lives here before it lives in the database"
    );
}

#[test]
fn the_sidecars_are_named_after_the_database() {
    let [wal, shm] = sidecars(std::path::Path::new("/data/history.db"));
    assert!(wal.to_string_lossy().ends_with("history.db-wal"));
    assert!(shm.to_string_lossy().ends_with("history.db-shm"));
}

#[cfg(unix)]
#[test]
fn the_history_is_not_readable_by_other_users() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().expect("a folder");
    let path = dir.path().join("data").join("history.db");
    let store = Store::open(&path).expect("opened");
    store
        .insert_text("uuid-private", "password", 1)
        .expect("insert");
    assert_eq!(store.exposure(), Restricted::Mode(0o600));
    drop(store);

    let file = std::fs::metadata(&path)
        .expect("a file")
        .permissions()
        .mode()
        & 0o777;
    let folder = std::fs::metadata(path.parent().expect("a parent"))
        .expect("a folder")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(file, 0o600, "only its owner");
    assert_eq!(folder, 0o700, "and the folder the same way");
}

#[cfg(windows)]
#[test]
fn on_windows_what_protects_the_history_is_living_under_the_profile() {
    let Some(profile) = std::env::var_os("USERPROFILE") else {
        return;
    };
    let dir = tempfile::Builder::new()
        .prefix("copypaste-")
        .tempdir_in(profile)
        .expect("a folder");
    let path = dir.path().join("data").join("history.db");
    let store = Store::open(&path).expect("opened");
    store
        .insert_text("uuid-private", "password", 1)
        .expect("insert");
    assert_eq!(store.exposure(), Restricted::InheritedFromProfile);
}

fn a_profile_with(entry: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let profile = tempfile::tempdir().expect("profile");
    let path = profile.path().join(entry);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("a folder");
    }
    std::fs::write(&path, b"x").expect("a file");
    (profile, path)
}

#[test]
fn a_history_under_the_profile_is_covered_by_its_permissions() {
    let (profile, history) = a_profile_with("AppData/Local/history.db");
    assert_eq!(
        exposure_of(&history, Some(profile.path())),
        Restricted::InheritedFromProfile
    );
}

#[test]
fn a_history_outside_the_profile_is_unprotected() {
    let (profile, _) = a_profile_with("AppData/history.db");
    let (_shared, elsewhere) = a_profile_with("compartido/history.db");
    assert_eq!(
        exposure_of(&elsewhere, Some(profile.path())),
        Restricted::Unprotected
    );
}

#[test]
fn without_a_profile_nothing_is_promised() {
    let (_profile, history) = a_profile_with("history.db");
    assert_eq!(exposure_of(&history, None), Restricted::Unprotected);
}

#[test]
fn a_path_that_does_not_exist_is_never_taken_for_protected() {
    let profile = tempfile::tempdir().expect("profile");
    let ghost = profile.path().join("not-yet").join("history.db");
    assert!(!under(&ghost, profile.path()));
    assert_eq!(
        exposure_of(&ghost, Some(profile.path())),
        Restricted::Unprotected
    );
}

#[test]
fn being_above_is_not_being_inside() {
    let (profile, history) = a_profile_with("AppData/history.db");
    assert!(under(&history, profile.path()));
    assert!(!under(profile.path(), &history));
}

#[test]
fn a_sibling_that_merely_starts_alike_is_outside() {
    let root = tempfile::tempdir().expect("a root");
    let ann = root.path().join("ann");
    let annabel = root.path().join("annabel");
    std::fs::create_dir_all(&ann).expect("ann");
    std::fs::create_dir_all(&annabel).expect("annabel");
    let history = annabel.join("history.db");
    std::fs::write(&history, b"x").expect("a file");
    assert!(!under(&history, &ann), "a text prefix is not a path prefix");
}

#[test]
fn a_checkpoint_leaves_the_data_in_the_main_file() {
    let dir = tempfile::tempdir().expect("a folder");
    let path = dir.path().join("history.db");
    let store = Store::open(&path).expect("opened");
    for at in 0..50 {
        store
            .insert_text(&format!("uuid-{at}"), &format!("line {at}"), at)
            .expect("insert");
    }
    store.checkpoint().expect("checkpoint");
    let wal = path.with_extension("db-wal");
    let wal_size = std::fs::metadata(&wal).map(|m| m.len()).unwrap_or(0);
    assert!(
        wal_size == 0 || !wal.exists(),
        "after the checkpoint the WAL is left empty, not at {wal_size} bytes"
    );
}

#[test]
fn an_incremental_vacuum_actually_frees_pages() {
    let store = Store::in_memory().expect("schema");
    for at in 0..2000 {
        let id = store
            .insert_text(&format!("uuid-{at}"), &"x".repeat(200), at)
            .expect("insert");
        store.mark_broken(id, at).expect("marked");
    }
    store.purge_broken_before(i64::MAX).expect("purged");

    let before: i64 = store
        .db
        .query_row("PRAGMA freelist_count", [], |row| row.get(0))
        .expect("queried");
    assert!(
        before > 0,
        "deleting so many rows has to leave free pages, not {before}"
    );

    store
        .vacuum_step(before as u32)
        .expect("empties the free pages");

    let after: i64 = store
        .db
        .query_row("PRAGMA freelist_count", [], |row| row.get(0))
        .expect("queried");
    assert!(
        after < before,
        "incremental_vacuum has to shrink the freelist: before {before}, after {after}"
    );
}

#[test]
fn reopening_keeps_the_pragmas_that_protect_the_data() {
    let dir = tempfile::tempdir().expect("a folder");
    let path = dir.path().join("history.db");
    drop(Store::open(&path).expect("opened"));
    let store = Store::open(&path).expect("reopened");
    let vacuum: i64 = store
        .db
        .query_row("PRAGMA auto_vacuum", [], |row| row.get(0))
        .expect("queried");
    assert_eq!(
        vacuum, 2,
        "the mode gets saved in the file and it should stay that way"
    );
}

fn big_image(byte: u8) -> Item {
    Item {
        kind: Some(cp_core::kind::Kind::Image),
        formats: vec![Format {
            id: "public.png".into(),
            payload: Payload::Blob(vec![byte; 200_000]),
        }],
    }
}

#[test]
fn an_image_too_big_for_the_row_goes_to_disk_and_comes_back() {
    let (_dir, store) = on_disk();
    let id = store
        .insert_item("uuid-image", &big_image(7), "", 1)
        .expect("insert");
    let bytes = store
        .payload_of(id, "public.png")
        .expect("read")
        .expect("is there");
    assert_eq!(bytes.len(), 200_000);
    assert!(bytes.iter().all(|b| *b == 7));
}

#[test]
fn two_copies_of_the_same_image_share_one_file() {
    let (dir, store) = on_disk();
    store
        .insert_item("uuid-a", &big_image(9), "", 1)
        .expect("a");
    store
        .insert_item("uuid-b", &big_image(9), "", 2)
        .expect("b");
    let files = std::fs::read_dir(dir.path().join("blobs"))
        .expect("a folder")
        .count();
    assert_eq!(files, 1, "the name is the content, so it is the same file");
}

#[test]
fn blobs_of_reports_the_digests_the_item_has() {
    let (_dir, store) = on_disk();
    let id = store
        .insert_item("uuid-blobs", &big_image(4), "", 1)
        .expect("insert");
    assert_eq!(store.blobs_of(id).expect("blobs").len(), 1);
}

#[test]
fn blobs_of_an_item_with_no_blobs_is_empty() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_text("uuid-no-blobs", "just text", 1)
        .expect("insert");
    assert!(store.blobs_of(id).expect("blobs").is_empty());
}

#[test]
fn the_blob_of_the_only_item_that_held_it_stops_being_referenced_when_it_goes() {
    let (_dir, store) = on_disk();
    let id = store
        .insert_item("uuid-only", &big_image(11), "", 1)
        .expect("insert");
    let digest = store
        .blobs_of(id)
        .expect("blobs")
        .pop()
        .expect("there is one");
    assert!(store.blob_is_referenced(&digest).expect("queried"));
    store.mark_deleted(id, 2).expect("deleted");
    assert!(
        !store.blob_is_referenced(&digest).expect("queried"),
        "nobody points at it any more, so it can be let go"
    );
}

#[test]
fn a_blob_two_items_hold_survives_the_first_of_them_going() {
    let (_dir, store) = on_disk();
    let first = store
        .insert_item("uuid-1", &big_image(12), "", 1)
        .expect("a");
    store
        .insert_item("uuid-2", &big_image(12), "", 2)
        .expect("b");
    let digest = store
        .blobs_of(first)
        .expect("blobs")
        .pop()
        .expect("there is one");
    store.mark_deleted(first, 3).expect("deleted");
    assert!(
        store.blob_is_referenced(&digest).expect("queried"),
        "the other item still needs it"
    );
    assert!(
        store.blobs().expect("a blob store").exists(&digest),
        "and so its bytes are still on disk"
    );
}

#[test]
fn an_inline_payload_comes_back_as_is() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_item("uuid-inline", &sample_item(), "hello", 1)
        .expect("insert");
    let bytes = store
        .payload_of(id, "public.utf8-plain-text")
        .expect("read")
        .expect("is there");
    assert_eq!(bytes, b"hello");
}

#[test]
fn deleting_an_image_takes_its_bytes_off_the_disk() {
    let (_dir, store) = on_disk();
    let id = store
        .insert_item("uuid-delete", &big_image(3), "", 1)
        .expect("insert");
    assert!(store.payload_of(id, "public.png").expect("read").is_some());
    store.mark_deleted(id, 2).expect("removed");
    assert!(
        store.payload_of(id, "public.png").expect("read").is_none(),
        "the bytes of a deleted image cannot remain on disk"
    );
}

#[test]
fn a_shared_blob_survives_deleting_one_of_its_owners() {
    let (_dir, store) = on_disk();
    let first = store
        .insert_item("uuid-1", &big_image(5), "", 1)
        .expect("a");
    let second = store
        .insert_item("uuid-2", &big_image(5), "", 2)
        .expect("b");
    store.mark_deleted(first, 3).expect("deletes the first one");
    assert!(
        store
            .payload_of(second, "public.png")
            .expect("read")
            .is_some(),
        "the other item still needs those bytes"
    );
}

#[test]
fn an_in_memory_store_refuses_what_it_cannot_keep() {
    let store = Store::in_memory().expect("schema");
    assert!(
        store.insert_item("uuid-big", &big_image(1), "", 1).is_err(),
        "with no folder to write to, better to refuse"
    );
}

#[test]
fn the_queue_hands_out_work_and_forgets_it_when_done() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_text("uuid-work", "something", 1)
        .expect("insert");
    store.enqueue(id, "ocr").expect("queued");
    store
        .enqueue(id, "ocr")
        .expect("queuing twice does not duplicate");
    assert_eq!(store.take_pending("ocr", 10, 5).expect("pending"), vec![id]);
    assert!(
        store
            .take_pending("thumbnail", 10, 5)
            .expect("a different kind")
            .is_empty(),
        "each queue is its own"
    );
    store.work_done(id, "ocr").expect("done");
    assert!(
        store
            .take_pending("ocr", 10, 5)
            .expect("pending")
            .is_empty()
    );
}

#[test]
fn a_job_that_keeps_failing_is_given_up_on() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_text("uuid-failure", "something", 1)
        .expect("insert");
    store.enqueue(id, "ocr").expect("queued");
    for attempt in 1..Store::MAX_ATTEMPTS {
        assert!(
            store
                .work_failed(id, "ocr", "could not do it", 0)
                .expect("failed"),
            "attempt {attempt} is still retried"
        );
    }
    assert!(
        !store
            .work_failed(id, "ocr", "could not do it", 0)
            .expect("failed"),
        "once the attempts run out, it gives up"
    );
    assert!(
        store
            .take_pending("ocr", 10, 5)
            .expect("pending")
            .is_empty()
    );
}

#[test]
fn a_failed_job_waits_before_being_retried() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_text("uuid-wait", "something", 1)
        .expect("insert");
    store.enqueue(id, "ocr").expect("queued");
    store
        .work_failed(id, "ocr", "temporary", 500)
        .expect("failed");
    assert!(
        store
            .take_pending("ocr", 100, 5)
            .expect("not yet")
            .is_empty(),
        "not before its time"
    );
    assert_eq!(
        store.take_pending("ocr", 500, 5).expect("already"),
        vec![id]
    );
}

#[test]
fn deleted_items_drop_out_of_the_queue() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_text("uuid-out", "something", 1)
        .expect("insert");
    store.enqueue(id, "ocr").expect("queued");
    store.mark_deleted(id, 2).expect("removed");
    assert!(
        store
            .take_pending("ocr", 10, 5)
            .expect("pending")
            .is_empty(),
        "what the user deleted does not get enriched"
    );
}

#[test]
fn failing_a_job_for_an_item_already_deleted_claims_it_will_retry_but_there_is_nothing_left_to_retry()
 {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_text("uuid-gone", "something", 1)
        .expect("insert");
    store.enqueue(id, "ocr").expect("queued");
    store.mark_deleted(id, 2).expect("removed");
    assert!(
        store
            .work_failed(id, "ocr", "too slow", 100)
            .expect("recorded"),
        "the queue row is already gone, but work_failed still reports that it will retry"
    );
    assert!(
        store
            .take_pending("ocr", 1_000, 5)
            .expect("pending")
            .is_empty(),
        "there is nothing left to retry: the row disappeared along with the item"
    );
}

#[test]
fn different_capitalisation_is_a_different_identity_but_the_same_search_result() {
    let store = Store::in_memory().expect("schema");
    let upper_id = store
        .insert_text("uuid-upper", "TEXTO IMPORTANTE", 1)
        .expect("insert");
    assert_eq!(
        store
            .find_by_hash(&Item::plain("texto importante"))
            .expect("searched"),
        None,
        "case is part of the identity, so the same words in lowercase do not match"
    );
    let lower_id = store
        .insert_text("uuid-lower", "texto importante", 2)
        .expect("insert");
    assert_ne!(upper_id, lower_id);
    assert_eq!(
        search(&store, "TeXto").len(),
        2,
        "search folds case, so both entries turn up for either spelling"
    );
}

#[test]
fn metadata_is_kept_per_key_and_replaced_not_duplicated() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_text("uuid-meta", "a video", 1)
        .expect("insert");
    store.set_meta(id, "duration", "227").expect("set");
    store.set_meta(id, "width", "1920").expect("set");
    store.set_meta(id, "duration", "228").expect("corrected");
    assert_eq!(
        store.meta(id, "duration").expect("read").as_deref(),
        Some("228")
    );
    assert_eq!(store.all_meta(id).expect("everything").len(), 2);
    assert!(store.meta(id, "artist").expect("read").is_none());
}

#[test]
fn metadata_goes_away_with_the_item() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_text("uuid-meta", "something", 1)
        .expect("insert");
    store.set_meta(id, "artist", "someone").expect("set");
    store.mark_deleted(id, 2).expect("removed");
    assert!(
        store.all_meta(id).expect("everything").is_empty(),
        "derived data belongs to the user just like the content does"
    );
}

fn a_little_history() -> Store {
    let store = Store::in_memory().expect("schema");
    let rows = [
        ("uuid-1", "first note", Kind::Text, 10),
        ("uuid-2", "someone@example.test", Kind::Email, 20),
        ("uuid-3", "#FF8800", Kind::Color, 30),
        ("uuid-4", "second note", Kind::Text, 40),
    ];
    for (uuid, text, kind, at) in rows {
        let item = Item {
            kind: Some(kind),
            formats: vec![Format {
                id: "public.utf8-plain-text".into(),
                payload: Payload::Inline(text.as_bytes().to_vec()),
            }],
        };
        store.insert_item(uuid, &item, text, at).expect("insert");
    }
    store
}

#[test]
fn the_panel_can_ask_for_the_latest_without_searching_anything() {
    let store = a_little_history();
    let listed = store
        .list(&Filter::default(), 10, None)
        .expect("listed")
        .rows;
    assert_eq!(listed.len(), 4, "with no term the history comes back");
    assert_eq!(
        listed.first().map(|one| one.preview.as_str()),
        Some("second note"),
        "the most recent one comes first"
    );
}

#[test]
fn the_list_can_be_filtered_by_kind() {
    let store = a_little_history();
    let filter = Filter {
        kinds: vec![Kind::Text],
        ..Default::default()
    };
    let listed = store.list(&filter, 10, None).expect("listed").rows;
    assert_eq!(listed.len(), 2);
    assert!(listed.iter().all(|one| one.kind == Some(Kind::Text)));
}

#[test]
fn several_kinds_can_be_asked_for_at_once() {
    let store = a_little_history();
    let filter = Filter {
        kinds: vec![Kind::Email, Kind::Color],
        ..Default::default()
    };
    assert_eq!(store.list(&filter, 10, None).expect("listed").rows.len(), 2);
}

#[test]
fn filtering_and_searching_work_together() {
    let store = a_little_history();
    let filter = Filter {
        query: Some("note".into()),
        kinds: vec![Kind::Text],
        ..Default::default()
    };
    assert_eq!(store.list(&filter, 10, None).expect("listed").rows.len(), 2);

    let narrower = Filter {
        query: Some("note".into()),
        kinds: vec![Kind::Email],
        ..Default::default()
    };
    assert!(
        store
            .list(&narrower, 10, None)
            .expect("listed")
            .rows
            .is_empty(),
        "both the filter and the term get applied"
    );
}

#[test]
fn only_pinned_can_be_asked_for() {
    let store = a_little_history();
    let listed = store
        .list(&Filter::default(), 10, None)
        .expect("listed")
        .rows;
    let id = listed.first().expect("there is one").id;
    store.set_pinned(id, true, 0).expect("pinned");
    let filter = Filter {
        pinned_only: true,
        ..Default::default()
    };
    let pinned = store.list(&filter, 10, None).expect("listed").rows;
    assert_eq!(pinned.len(), 1);
    assert!(pinned[0].pinned);
}

#[test]
fn the_list_hands_out_a_cursor_only_while_there_is_more() {
    let store = a_little_history();
    let first = store.list(&Filter::default(), 2, None).expect("a page");
    assert_eq!(first.rows.len(), 2);
    let cursor = first.next.expect("two more remain");
    let second = store
        .list(&Filter::default(), 2, Some(cursor))
        .expect("next");
    assert_eq!(second.rows.len(), 2);
    assert!(second.rows.iter().all(|one| !first.rows.contains(one)));
    assert_eq!(second.next, None, "the last page promises no other");
}

#[test]
fn a_page_that_ends_exactly_at_the_last_row_promises_nothing_more() {
    let store = a_little_history();
    let whole = store.list(&Filter::default(), 4, None).expect("a page");
    assert_eq!(whole.rows.len(), 4);
    assert_eq!(whole.next, None);
}

#[test]
fn a_search_with_nothing_usable_returns_nothing_not_everything() {
    let store = a_little_history();
    let filter = Filter {
        query: Some("!!!".into()),
        ..Default::default()
    };
    assert!(
        store
            .list(&filter, 10, None)
            .expect("listed")
            .rows
            .is_empty(),
        "asking to search for something impossible cannot return the whole history"
    );
}

#[test]
fn deleted_items_never_show_up_in_the_list() {
    let store = a_little_history();
    let listed = store
        .list(&Filter::default(), 10, None)
        .expect("listed")
        .rows;
    store.mark_deleted(listed[0].id, 99).expect("removed");
    assert_eq!(
        store
            .list(&Filter::default(), 10, None)
            .expect("listed")
            .rows
            .len(),
        3
    );
}

#[test]
fn retention_takes_the_old_and_leaves_what_was_pinned() {
    let store = a_little_history();
    let listed = store
        .list(&Filter::default(), 10, None)
        .expect("listed")
        .rows;
    let oldest = listed.last().expect("there is one").id;
    store
        .set_pinned(oldest, true, 0)
        .expect("pins the oldest one");

    let removed = store.clear_older_than(35).expect("retention");
    assert_eq!(
        removed, 2,
        "what is from before the cutoff and not pinned goes away"
    );
    let left = store
        .list(&Filter::default(), 10, None)
        .expect("listed")
        .rows;
    assert_eq!(left.len(), 2);
    assert!(
        left.iter().any(|one| one.id == oldest),
        "a pinned item does not get removed by cleanup"
    );
}

#[test]
fn clearing_everything_still_respects_what_was_pinned() {
    let store = a_little_history();
    let listed = store
        .list(&Filter::default(), 10, None)
        .expect("listed")
        .rows;
    store.set_pinned(listed[0].id, true, 0).expect("pinned");
    let removed = store.clear_all_unpinned(100).expect("emptied");
    assert_eq!(removed, 3);
    assert_eq!(store.count().expect("counted"), 1);
}

#[test]
fn retention_with_nothing_old_enough_removes_nothing() {
    let store = a_little_history();
    assert_eq!(store.clear_older_than(0).expect("retention"), 0);
    assert_eq!(store.count().expect("counted"), 4);
}

#[test]
fn a_word_that_is_not_there_finds_nothing() {
    let store = seeded();
    assert!(search(&store, "berlin").is_empty());
}

fn aged(path: &std::path::Path) {
    let file = std::fs::File::options()
        .write(true)
        .open(path)
        .expect("opened");
    file.set_modified(std::time::UNIX_EPOCH).expect("aged");
}

#[test]
fn editing_a_text_into_the_very_same_text_does_not_throw_its_blob_away() {
    let (_dir, store) = on_disk();
    let long = "a".repeat(cp_core::item::INLINE_UP_TO + 1);
    let as_a_blob = Item {
        kind: Some(cp_core::kind::Kind::Text),
        formats: vec![Format {
            id: cp_core::item::SYNTHETIC_TEXT.into(),
            payload: Payload::stored(long.as_bytes().to_vec()),
        }],
    };
    let id = store
        .insert_item("uuid-edited", &as_a_blob, &long[..10], 1)
        .expect("insert");
    let digest = store
        .blobs_of(id)
        .expect("blobs")
        .pop()
        .expect("a text that long lives in a blob");
    let blobs = store.blobs().expect("a blob store");
    aged(&blobs.where_it_is(&digest).expect("a real digest"));

    store.update_text(id, &long, 2).expect("edited");

    assert!(
        blobs.exists(&digest),
        "the row still points at it, so the bytes cannot be gone"
    );
    assert_eq!(
        store
            .payload_of(id, cp_core::item::SYNTHETIC_TEXT)
            .expect("read"),
        Some(long.as_bytes().to_vec()),
        "and the item still reads back whole"
    );
}

#[test]
fn a_clear_that_cannot_be_written_leaves_every_file_where_it_was() {
    let (dir, store) = on_disk();
    let id = store
        .insert_item("uuid-kept", &big_image(21), "", 1)
        .expect("insert");
    let digest = store
        .blobs_of(id)
        .expect("blobs")
        .pop()
        .expect("there is one");
    let blobs = store.blobs().expect("a blob store");
    let blob = blobs.where_it_is(&digest).expect("a real digest");
    aged(&blob);
    let thumb = dir.path().join("thumbs").join("kept.png");
    std::fs::create_dir_all(thumb.parent().expect("a folder")).expect("a folder");
    std::fs::write(&thumb, b"a drawn thumbnail").expect("a thumb");
    store
        .set_thumb(id, Some(&thumb.to_string_lossy()), 2)
        .expect("noted");

    let other = rusqlite::Connection::open(dir.path().join("history.db")).expect("a second reader");
    other
        .execute_batch("BEGIN EXCLUSIVE;")
        .expect("the database is held by somebody else");

    assert!(
        store.clear_all_unpinned(3).is_err(),
        "nothing can be written while another connection holds it"
    );

    assert!(blob.exists(), "the blob outlived a clear that never landed");
    assert!(thumb.exists(), "and so did the thumbnail");
    other.execute_batch("ROLLBACK;").expect("let go");
    assert_eq!(
        store
            .payload_of(id, "public.png")
            .expect("read")
            .map(|b| b.len()),
        Some(200_000),
        "the item still reads back whole"
    );
}
