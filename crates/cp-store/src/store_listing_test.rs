use super::*;
use cp_core::item::Format;

fn text_item(text: &str, kind: Kind) -> Item {
    Item {
        kind: Some(kind),
        formats: vec![Format {
            id: "public.utf8-plain-text".into(),
            payload: Payload::Inline(text.as_bytes().to_vec()),
        }],
    }
}

fn history() -> Store {
    let store = Store::in_memory().expect("schema");
    let rows = [
        ("uuid-1", "first note", Kind::Text, 10, "Safari"),
        ("uuid-2", "someone@example.test", Kind::Email, 20, "Slack"),
        ("uuid-3", "#FF8800", Kind::Color, 30, "Slack"),
        ("uuid-4", "second note", Kind::Text, 40, "Code"),
        ("uuid-5", "fn main() {}", Kind::Code, 50, "Code"),
    ];
    for (uuid, text, kind, at, app) in rows {
        let id = store
            .insert_item(uuid, &text_item(text, kind), text, at)
            .expect("insert");
        store.set_source(id, app, at).expect("sourced");
    }
    store
}

fn all(store: &Store, filter: &Filter) -> Vec<Listed> {
    store.list(filter, 100, None).expect("listed").rows
}

fn previews(rows: &[Listed]) -> Vec<&str> {
    rows.iter().map(|one| one.preview.as_str()).collect()
}

#[test]
fn the_card_gets_everything_the_row_knows() {
    let store = history();
    let id = all(&store, &Filter::default())[0].id;
    store.set_label(id, Some("Startup"), 60).expect("labelled");
    store.set_color(id, 5, 61).expect("color");
    store.record_paste(id, 62).expect("pasted");
    store.set_pinned(id, true, 0).expect("pinned");
    let card = all(&store, &Filter::default())
        .into_iter()
        .find(|one| one.id == id)
        .expect("is there");
    assert_eq!(card.preview, "fn main() {}");
    assert_eq!(card.kind, Some(Kind::Code));
    assert_eq!(card.app.as_deref(), Some("Code"));
    assert_eq!(card.label.as_deref(), Some("Startup"));
    assert_eq!(card.color, 5);
    assert_eq!(card.paste_count, 1);
    assert_eq!(card.last_used_at, Some(62));
    assert_eq!(card.created_at, 50);
    assert_eq!(card.modified_at, 50, "pasting does not move the clock");
    assert!(card.pinned);
    assert_eq!(card.broken_since, None);
    assert_eq!(card.thumb_path, None);
    assert_eq!(card.snippet, None, "with no term there is no snippet");
}

#[test]
fn searching_marks_the_fragment_that_matched() {
    let store = history();
    let filter = Filter {
        query: Some("sec".into()),
        ..Default::default()
    };
    let rows = all(&store, &filter);
    assert_eq!(rows.len(), 1);
    let snippet = rows[0].snippet.as_ref().expect("a snippet");
    assert_eq!(snippet.found_in, FoundIn::Text);
    let marked: Vec<&str> = snippet
        .excerpt
        .segments
        .iter()
        .filter(|one| one.matched)
        .map(|one| one.text.as_str())
        .collect();
    assert_eq!(marked, vec!["sec"]);
    assert_eq!(snippet.excerpt.plain(), "second note");
}

#[test]
fn a_hit_on_the_label_says_so() {
    let store = history();
    let id = all(&store, &Filter::default())[0].id;
    store
        .set_label(id, Some("Invoice may"), 60)
        .expect("labelled");
    let filter = Filter {
        query: Some("invoice".into()),
        ..Default::default()
    };
    let rows = all(&store, &filter);
    assert_eq!(rows.len(), 1);
    let snippet = rows[0].snippet.as_ref().expect("a snippet");
    assert_eq!(snippet.found_in, FoundIn::Label);
    assert_eq!(snippet.excerpt.plain(), "Invoice may");
}

#[test]
fn a_hit_on_what_was_read_inside_an_image_says_so() {
    let store = Store::in_memory().expect("schema");
    let image = Item {
        kind: Some(Kind::Image),
        formats: vec![Format {
            id: "public.png".into(),
            payload: Payload::Inline(vec![1]),
        }],
    };
    let id = store
        .insert_item("uuid-img", &image, "", 1)
        .expect("insert");
    store
        .set_ocr_text(id, "Order AB-4417 delivery", 2)
        .expect("ocr");
    let filter = Filter {
        query: Some("ab-4417".into()),
        ..Default::default()
    };
    let rows = all(&store, &filter);
    assert_eq!(rows.len(), 1);
    let snippet = rows[0].snippet.as_ref().expect("a snippet");
    assert_eq!(snippet.found_in, FoundIn::Ocr);
    assert_eq!(
        snippet.excerpt.plain(),
        "Order AB-4417 delivery",
        "the snippet shows what was read exactly as is, not folded"
    );
}

#[test]
fn an_old_folded_ocr_is_still_shown_until_it_is_read_again() {
    let store = Store::in_memory().expect("schema");
    let image = Item {
        kind: Some(cp_core::kind::Kind::Image),
        formats: vec![Format {
            id: "public.png".into(),
            payload: Payload::Inline(vec![1]),
        }],
    };
    let id = store
        .insert_item("uuid-old", &image, "", 1)
        .expect("insert");
    store
        .raw()
        .execute(
            "UPDATE items SET search_ocr = 'folded order' WHERE id = ?1",
            [id],
        )
        .expect("the way version 3 left it");
    assert_eq!(
        store.ocr_text(id).expect("read").as_deref(),
        Some("folded order")
    );
    let rows = all(
        &store,
        &Filter {
            query: Some("folded".into()),
            ..Default::default()
        },
    );
    let snippet = rows[0].snippet.as_ref().expect("a snippet");
    assert_eq!(snippet.found_in, FoundIn::Ocr);
    assert_eq!(snippet.excerpt.plain(), "folded order");
}

#[test]
fn a_hit_on_the_source_application_says_so() {
    let store = history();
    let filter = Filter {
        query: Some("slack".into()),
        ..Default::default()
    };
    let rows = all(&store, &filter);
    assert_eq!(rows.len(), 2);
    assert!(
        rows.iter()
            .all(|one| one.snippet.as_ref().map(|s| s.found_in) == Some(FoundIn::App))
    );
}

#[test]
fn a_class_can_be_left_out() {
    let store = history();
    let filter = Filter {
        exclude_kinds: vec![Kind::Text, Kind::Code],
        ..Default::default()
    };
    assert_eq!(
        previews(&all(&store, &filter)),
        vec!["#FF8800", "someone@example.test"]
    );
}

#[test]
fn the_source_application_filters_by_equality_not_by_search() {
    let store = history();
    let id = all(&store, &Filter::default())[0].id;
    store
        .set_label(id, Some("paste into slack"), 60)
        .expect("labelled");
    let filter = Filter {
        apps: vec!["slack".into()],
        ..Default::default()
    };
    let rows = all(&store, &filter);
    assert_eq!(rows.len(), 2, "only what was copied from Slack, case aside");
    assert!(rows.iter().all(|one| one.app.as_deref() == Some("Slack")));
}

#[test]
fn several_applications_and_a_negated_one() {
    let store = history();
    let either = Filter {
        apps: vec!["Safari".into(), "Code".into()],
        ..Default::default()
    };
    assert_eq!(all(&store, &either).len(), 3);
    let not_code = Filter {
        exclude_apps: vec!["code".into()],
        ..Default::default()
    };
    assert_eq!(all(&store, &not_code).len(), 3);
}

#[test]
fn since_keeps_what_was_copied_from_that_moment_on() {
    let store = history();
    let filter = Filter {
        since: Some(30),
        ..Default::default()
    };
    assert_eq!(all(&store, &filter).len(), 3, "the 30 is included");
}

#[test]
fn since_counts_a_recopy_as_copied_again() {
    let store = history();
    let oldest = all(&store, &Filter::default())
        .last()
        .expect("there is one")
        .id;
    store.reactivate(oldest, 100).expect("copied again");
    let filter = Filter {
        since: Some(100),
        ..Default::default()
    };
    assert_eq!(
        previews(&all(&store, &filter)),
        vec!["first note"],
        "what gets copied again today belongs to today"
    );
}

#[test]
fn broken_items_are_hidden_unless_asked_for() {
    let store = history();
    let id = all(&store, &Filter::default())[0].id;
    store.mark_broken(id, 99).expect("broken");
    assert_eq!(all(&store, &Filter::default()).len(), 4);
    let shown = Filter {
        broken: Broken::Shown,
        ..Default::default()
    };
    assert_eq!(all(&store, &shown).len(), 5);
    let only = Filter {
        broken: Broken::Only,
        ..Default::default()
    };
    let rows = all(&store, &only);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].broken_since, Some(99));
}

#[test]
fn a_scoped_label_query_does_not_match_the_content() {
    let store = history();
    let rows = all(&store, &Filter::default());
    store
        .set_label(rows[1].id, Some("note"), 60)
        .expect("labelled");
    let by_label = Filter {
        label_query: Some("note".into()),
        ..Default::default()
    };
    let found = all(&store, &by_label);
    assert_eq!(found.len(), 1, "«note» is in two contents and one label");
    assert_eq!(found[0].id, rows[1].id);
}

#[test]
fn a_label_query_and_a_text_query_both_apply() {
    let store = history();
    let rows = all(&store, &Filter::default());
    store
        .set_label(rows[0].id, Some("startup"), 60)
        .expect("labelled");
    store
        .set_label(rows[1].id, Some("startup"), 61)
        .expect("labelled");
    let filter = Filter {
        query: Some("main".into()),
        label_query: Some("startup".into()),
        ..Default::default()
    };
    assert_eq!(previews(&all(&store, &filter)), vec!["fn main() {}"]);
}

#[test]
fn a_label_query_with_nothing_usable_finds_nothing() {
    let store = history();
    let filter = Filter {
        label_query: Some("!!!".into()),
        ..Default::default()
    };
    assert!(all(&store, &filter).is_empty());
}

#[test]
fn most_pasted_comes_first_and_ties_break_the_same_way_every_time() {
    let store = history();
    let rows = all(&store, &Filter::default());
    for _ in 0..3 {
        store.record_paste(rows[4].id, 70).expect("pasted");
    }
    store.record_paste(rows[2].id, 71).expect("pasted");
    let filter = Filter {
        order: Order::MostPasted,
        ..Default::default()
    };
    let ordered = all(&store, &filter);
    assert_eq!(ordered[0].id, rows[4].id);
    assert_eq!(ordered[1].id, rows[2].id);
    assert_eq!(
        ordered[2..].iter().map(|one| one.id).collect::<Vec<_>>(),
        vec![rows[0].id, rows[1].id, rows[3].id],
        "at an equal count, the newest one first"
    );
}

#[test]
fn last_used_puts_what_was_never_pasted_at_the_end() {
    let store = history();
    let rows = all(&store, &Filter::default());
    store.record_paste(rows[3].id, 80).expect("pasted");
    store.record_paste(rows[1].id, 90).expect("pasted");
    let filter = Filter {
        order: Order::LastUsed,
        ..Default::default()
    };
    let ordered = all(&store, &filter);
    assert_eq!(ordered[0].id, rows[1].id);
    assert_eq!(ordered[1].id, rows[3].id);
    assert!(ordered[2..].iter().all(|one| one.last_used_at.is_none()));
}

fn walk(store: &Store, filter: &Filter, page: usize) -> Vec<i64> {
    let mut seen = Vec::new();
    let mut after = None;
    loop {
        let got = store.list(filter, page, after).expect("a page");
        seen.extend(got.rows.iter().map(|one| one.id));
        match got.next {
            Some(cursor) => after = Some(cursor),
            None => return seen,
        }
    }
}

#[test]
fn every_order_pages_without_repeating_or_skipping_even_with_ties() {
    let store = Store::in_memory().expect("schema");
    for at in 0..23 {
        let id = store
            .insert_item(
                &format!("uuid-{at}"),
                &text_item(&format!("note {at}"), Kind::Text),
                &format!("note {at}"),
                at % 4,
            )
            .expect("insert");
        for _ in 0..(at % 3) {
            store.record_paste(id, at % 5).expect("pasted");
        }
    }
    for order in [Order::Recent, Order::MostPasted, Order::LastUsed] {
        let filter = Filter {
            order,
            ..Default::default()
        };
        let mut ids = walk(&store, &filter, 4);
        assert_eq!(ids.len(), 23, "{order:?} skipped rows");
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 23, "{order:?} repeated rows");
    }
}

#[test]
fn the_tabs_count_only_the_classes_that_exist_within_the_search() {
    let store = history();
    let facets = store.facets(&Filter::default()).expect("facets");
    assert_eq!(
        facets,
        vec![
            Facet {
                kind: Kind::Text,
                count: 2
            },
            Facet {
                kind: Kind::Code,
                count: 1
            },
            Facet {
                kind: Kind::Color,
                count: 1
            },
            Facet {
                kind: Kind::Email,
                count: 1
            },
        ],
        "by count, and at an equal count by name"
    );
    let within = Filter {
        apps: vec!["Slack".into()],
        kinds: vec![Kind::Text],
        ..Default::default()
    };
    let facets = store.facets(&within).expect("facets");
    assert_eq!(
        facets.iter().map(|one| one.kind).collect::<Vec<_>>(),
        vec![Kind::Color, Kind::Email],
        "the chosen tab does not narrow the others; the app and the term do"
    );
}

#[test]
fn an_item_without_a_class_has_no_tab() {
    let store = Store::in_memory().expect("schema");
    store
        .insert_item(
            "uuid-none",
            &Item {
                kind: None,
                formats: vec![],
            },
            "",
            1,
        )
        .expect("insert");
    assert!(store.facets(&Filter::default()).expect("facets").is_empty());
}

#[test]
fn an_impossible_search_has_no_tabs_either() {
    let store = history();
    let filter = Filter {
        query: Some("!!!".into()),
        ..Default::default()
    };
    assert!(store.facets(&filter).expect("facets").is_empty());
}

#[test]
fn the_applications_come_with_their_counts_most_used_first() {
    let store = history();
    let apps = store.distinct_apps(&Filter::default()).expect("apps");
    assert_eq!(
        apps,
        vec![
            AppCount {
                app: "Code".into(),
                count: 2
            },
            AppCount {
                app: "Slack".into(),
                count: 2
            },
            AppCount {
                app: "Safari".into(),
                count: 1
            },
        ]
    );
}

#[test]
fn two_spellings_of_one_application_are_one_entry() {
    let store = history();
    let id = all(&store, &Filter::default())[0].id;
    store.set_source(id, "slack", 60).expect("sourced");
    let apps = store.distinct_apps(&Filter::default()).expect("apps");
    let slack = apps
        .iter()
        .find(|one| one.app == "Slack")
        .expect("is there");
    assert_eq!(slack.count, 3);
    assert!(apps.iter().all(|one| one.app != "slack"));
}

#[test]
fn deleted_items_count_for_nothing() {
    let store = history();
    let id = all(&store, &Filter::default())[0].id;
    store.mark_deleted(id, 99).expect("removed");
    let facets = store.facets(&Filter::default()).expect("facets");
    assert!(facets.iter().all(|one| one.kind != Kind::Code));
    let apps = store.distinct_apps(&Filter::default()).expect("apps");
    assert_eq!(
        apps.iter()
            .find(|one| one.app == "Code")
            .map(|one| one.count),
        Some(1)
    );
}

#[test]
fn no_order_sorts_the_history_in_memory() {
    let store = history();
    for order in Order::ALL {
        let filter = Filter {
            order,
            ..Default::default()
        };
        let clauses =
            Clauses::of(&filter, true, true).expect("with no term there are still clauses");
        let sql = page_sql(&clauses, order.key(), false);
        let plan: Vec<String> = store
            .db
            .prepare(&format!("EXPLAIN QUERY PLAN {sql}"))
            .expect("prepared")
            .query_map([50i64], |row| row.get::<_, String>(3))
            .expect("a plan")
            .map(|row| row.expect("a row"))
            .collect();
        assert!(
            !plan.iter().any(|step| step.contains("TEMP B-TREE")),
            "{order:?} sorts in memory: {plan:?}"
        );
    }
}

#[test]
fn leaving_a_class_out_keeps_what_has_no_class() {
    let store = history();
    store
        .insert_item(
            "uuid-none",
            &Item {
                kind: None,
                formats: vec![],
            },
            "no class",
            60,
        )
        .expect("insert");
    let filter = Filter {
        exclude_kinds: vec![Kind::Image],
        ..Default::default()
    };
    assert_eq!(
        all(&store, &filter).len(),
        6,
        "excluding images cannot hide what is nothing at all"
    );
}

#[test]
fn an_excluded_class_has_no_tab() {
    let store = history();
    let filter = Filter {
        exclude_kinds: vec![Kind::Text],
        ..Default::default()
    };
    let facets = store.facets(&filter).expect("facets");
    assert!(facets.iter().all(|one| one.kind != Kind::Text));
    assert_eq!(facets.len(), 3);
}

#[test]
fn asking_for_no_rows_is_an_empty_page_not_a_panic() {
    let store = history();
    let page = store.list(&Filter::default(), 0, None).expect("a page");
    assert!(page.rows.is_empty());
    assert_eq!(page.next, None);
    let huge = store
        .list(&Filter::default(), usize::MAX, None)
        .expect("a page");
    assert_eq!(huge.rows.len(), 5);
    assert_eq!(huge.next, None);
}

#[test]
fn a_broken_item_can_be_found_again() {
    let store = history();
    let id = all(&store, &Filter::default())[0].id;
    store.mark_broken(id, 99).expect("broken");
    assert_eq!(all(&store, &Filter::default()).len(), 4);
    store.mark_present(id).expect("came back");
    let rows = all(&store, &Filter::default());
    assert_eq!(rows.len(), 5, "the volume got remounted");
    assert_eq!(rows[0].broken_since, None);
    assert_eq!(
        store.purge_broken_before(i64::MAX).expect("purged"),
        0,
        "and it is no longer within anyone's deadline"
    );
}

#[test]
fn the_footer_count_matches_what_the_list_shows() {
    let store = history();
    let id = all(&store, &Filter::default())[0].id;
    store.mark_broken(id, 99).expect("broken");
    assert_eq!(
        store.count().expect("total"),
        5,
        "the total keeps counting broken ones"
    );
    assert_eq!(
        store.count_matching(&Filter::default()).expect("counted"),
        4,
        "what the footer shows is what the list shows"
    );
    let filter = Filter {
        query: Some("note".into()),
        apps: vec!["safari".into()],
        ..Default::default()
    };
    assert_eq!(store.count_matching(&filter).expect("counted"), 1);
    let impossible = Filter {
        query: Some("!!!".into()),
        ..Default::default()
    };
    assert_eq!(store.count_matching(&impossible).expect("counted"), 0);
    let facets: i64 = store
        .facets(&Filter::default())
        .expect("facets")
        .iter()
        .map(|one| one.count)
        .sum();
    assert_eq!(facets, 4, "and the tabs add up to the same");
    assert_eq!(
        store
            .distinct_apps(&Filter::default())
            .expect("apps")
            .iter()
            .map(|one| one.count)
            .sum::<i64>(),
        4,
        "apps do not count broken ones either"
    );
}

#[test]
fn every_order_has_a_stable_name_that_comes_back() {
    for order in Order::ALL {
        assert_eq!(Order::from_name(order.as_str()), Some(order));
    }
    let mut names: Vec<&str> = Order::ALL.iter().map(|order| order.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), 4);
    assert_eq!(Order::from_name("Recent"), None, "the name is exact");
}

#[test]
fn a_cursor_from_another_order_is_refused_not_misread() {
    let store = history();
    let recent = store.list(&Filter::default(), 2, None).expect("a page");
    let cursor = recent.next.expect("there is more");
    let pasted = Filter {
        order: Order::MostPasted,
        ..Default::default()
    };
    assert!(matches!(
        store.list(&pasted, 2, Some(cursor.clone())),
        Err(Error::WrongCursor { .. })
    ));
    let text = cursor.encode();
    assert_eq!(
        Cursor::decode(&text),
        Some(cursor),
        "it goes out and comes back as text"
    );
    assert_eq!(Cursor::decode("recent:1"), None);
    assert_eq!(Cursor::decode("sideways:1:2"), None);
    assert_eq!(Cursor::decode("recent:1:2:3"), None);
    assert_eq!(
        Cursor::decode("recent:x:2"),
        None,
        "a numeric order wants a number"
    );
    let grouped = Cursor::decode("by-group:ejemplo.test:7").expect("a group cursor");
    assert_eq!(Cursor::decode(&grouped.encode()), Some(grouped));
    let colons = Cursor::decode("by-group:carpeta:de:red:7").expect("a key may hold colons");
    assert_eq!(Cursor::decode(&colons.encode()), Some(colons));
}

#[test]
fn the_preview_is_capped_but_the_excerpt_still_sees_the_whole_text() {
    let store = Store::in_memory().expect("schema");
    let text = format!("{}needle", "hay ".repeat(1_000));
    store.insert_text("uuid-long", &text, 1).expect("insert");
    let rows = store
        .list(&Filter::default(), 10, None)
        .expect("listed")
        .rows;
    assert_eq!(rows[0].preview.chars().count(), PREVIEW_CHARS);
    let filter = Filter {
        query: Some("needle".into()),
        ..Default::default()
    };
    let rows = store.list(&filter, 10, None).expect("listed").rows;
    let snippet = rows[0].snippet.as_ref().expect("a snippet");
    assert!(
        snippet
            .excerpt
            .segments
            .iter()
            .any(|one| one.matched && one.text == "needle"),
        "the needle is past the cap on the preview"
    );
}

#[test]
fn the_applications_follow_the_search_but_not_their_own_filter() {
    let store = history();
    let within = Filter {
        query: Some("note".into()),
        apps: vec!["Safari".into()],
        ..Default::default()
    };
    let apps = store.distinct_apps(&within).expect("apps");
    assert_eq!(
        apps.iter().map(|one| one.app.as_str()).collect::<Vec<_>>(),
        vec!["Code", "Safari"],
        "both apps with a note, even though the filter asks for only Safari"
    );
    let impossible = Filter {
        query: Some("!!!".into()),
        ..Default::default()
    };
    assert!(store.distinct_apps(&impossible).expect("apps").is_empty());
}

#[test]
fn pinning_can_be_undone_and_moves_the_version() {
    let store = history();
    let id = all(&store, &Filter::default())[0].id;
    store.set_pinned(id, true, 70).expect("pinned");
    assert!(all(&store, &Filter::default())[0].pinned);
    assert!(
        store
            .changed_since(60)
            .expect("changes")
            .contains(&"uuid-5".to_string())
    );
    store.set_pinned(id, false, 71).expect("unpinned");
    assert!(!all(&store, &Filter::default())[0].pinned);
}

#[test]
fn a_thumbnail_path_can_be_set_and_shows_on_the_card() {
    let store = history();
    let id = all(&store, &Filter::default())[0].id;
    store
        .set_thumb(id, Some("thumbs/5.png"), 70)
        .expect("a thumbnail");
    assert_eq!(
        all(&store, &Filter::default())[0].thumb_path.as_deref(),
        Some("thumbs/5.png")
    );
    store.set_thumb(id, None, 71).expect("no thumbnail");
    assert_eq!(all(&store, &Filter::default())[0].thumb_path, None);
}

#[test]
fn an_item_comes_back_with_the_formats_it_was_stored_with() {
    let (_dir, store) = on_disk();
    let big = Item {
        kind: Some(Kind::Image),
        formats: vec![
            Format {
                id: "public.png".into(),
                payload: Payload::Blob(vec![7; 200_000]),
            },
            Format {
                id: "public.tiff".into(),
                payload: Payload::Announced { size: Some(4_000) },
            },
            Format {
                id: "com.apple.icns".into(),
                payload: Payload::Absent,
            },
        ],
    };
    let id = store.insert_item("uuid-item", &big, "", 1).expect("insert");
    let back = store.item(id).expect("read").expect("is there");
    assert_eq!(back.kind, Some(Kind::Image));
    assert_eq!(back.formats.len(), 3);
    assert!(
        matches!(back.format("public.png").expect("png").payload, Payload::Blob(ref b) if b.len() == 200_000)
    );
    assert_eq!(
        back.format("public.tiff").expect("tiff").payload,
        Payload::Announced { size: Some(4_000) }
    );
    assert_eq!(
        back.format("com.apple.icns").expect("icns").payload,
        Payload::Absent
    );
    assert_eq!(store.item(404).expect("read"), None);
    store.mark_deleted(id, 2).expect("removed");
    assert_eq!(store.item(id).expect("read"), None, "lo borrado no vuelve");
}

#[test]
fn a_text_inserted_directly_is_classified_like_a_capture() {
    let store = Store::in_memory().expect("schema");
    store.insert_text("uuid-c", "#FF8800", 1).expect("insert");
    store.insert_text("uuid-t", "una nota", 2).expect("insert");
    let rows = store
        .list(&Filter::default(), 10, None)
        .expect("listed")
        .rows;
    assert_eq!(rows[0].kind, Some(Kind::Text));
    assert_eq!(rows[1].kind, Some(Kind::Color));
}

#[test]
fn every_filter_at_once_narrows_to_the_one_row() {
    let store = history();
    let rows = all(&store, &Filter::default());
    let target = rows
        .iter()
        .find(|one| one.preview == "second note")
        .expect("is there");
    store
        .set_label(target.id, Some("key"), 60)
        .expect("labelled");
    store.set_color(target.id, 2, 61).expect("color");
    store.set_pinned(target.id, true, 62).expect("pinned");
    let filter = Filter {
        query: Some("note".into()),
        label_query: Some("key".into()),
        kinds: vec![Kind::Text, Kind::Code],
        exclude_kinds: vec![Kind::Email],
        apps: vec!["Code".into(), "Safari".into()],
        exclude_apps: vec!["Slack".into()],
        colors: vec![2],
        pinned_only: true,
        since: Some(20),
        broken: Broken::Shown,
        order: Order::MostPasted,
    };
    let found = all(&store, &filter);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, target.id);
    assert_eq!(store.count_matching(&filter).expect("counted"), 1);
    assert_eq!(
        store.facets(&filter).expect("facets"),
        vec![Facet {
            kind: Kind::Text,
            count: 1
        }]
    );
}

#[test]
fn editing_a_pinned_item_keeps_its_pin_label_colour_and_app() {
    let store = history();
    let id = all(&store, &Filter::default())[0].id;
    store.set_label(id, Some("fixed"), 60).expect("labelled");
    store.set_color(id, 3, 61).expect("color");
    store.set_pinned(id, true, 62).expect("pinned");
    store
        .update_text(id, "different content", 63)
        .expect("edited");
    let card = all(&store, &Filter::default())
        .into_iter()
        .find(|one| one.id == id)
        .expect("is there");
    assert!(card.pinned);
    assert_eq!(card.label.as_deref(), Some("fixed"));
    assert_eq!(card.color, 3);
    assert_eq!(card.app.as_deref(), Some("Code"));
}

#[test]
fn a_row_with_a_class_nobody_knows_lists_as_no_class() {
    let store = history();
    store
        .db
        .execute(
            "UPDATE items SET kind = 'hologram' WHERE uuid = 'uuid-5'",
            [],
        )
        .expect("a newer database");
    let rows = all(&store, &Filter::default());
    assert_eq!(rows[0].kind, None);
    assert!(store.facets(&Filter::default()).expect("facets").len() == 3);
}

#[test]
fn walking_the_pages_reads_the_same_order_as_one_big_page() {
    let store = Store::in_memory().expect("schema");
    for at in 0..23 {
        let id = store
            .insert_item(
                &format!("uuid-{at}"),
                &text_item(&format!("note {at}"), Kind::Text),
                &format!("note {at}"),
                at % 4,
            )
            .expect("insert");
        for _ in 0..(at % 3) {
            store.record_paste(id, at % 5).expect("pasted");
        }
    }
    for order in Order::ALL {
        let filter = Filter {
            order,
            ..Default::default()
        };
        let whole: Vec<i64> = all(&store, &filter).iter().map(|one| one.id).collect();
        assert_eq!(walk(&store, &filter, 4), whole, "{order:?}");
    }
}

#[test]
fn nothing_a_person_can_type_as_an_application_breaks_the_query() {
    let store = history();
    for app in ["'; DROP TABLE items; --", "\"", "%", "Straße"] {
        let filter = Filter {
            apps: vec![app.into()],
            ..Default::default()
        };
        store
            .list(&filter, 10, None)
            .unwrap_or_else(|why| panic!("«{app}» broke the listing: {why}"));
    }
    assert_eq!(
        store.count().expect("counted"),
        5,
        "the table is still there"
    );
}

#[test]
fn the_ordering_expression_is_the_one_the_index_was_built_for() {
    const SCHEMA: &str = include_str!("schema.rs");
    let bare = TOUCHED.replace("items.", "");
    assert!(
        SCHEMA.contains(&bare),
        "items_by_touch has to spell «{bare}» or sqlite plans a scan"
    );
}

#[test]
fn pasting_does_not_move_the_retention_clock() {
    let store = history();
    let one = all(&store, &Filter::default())[0].id;
    let was = all(&store, &Filter::default())[0].modified_at;
    store.record_paste(one, 10_000).expect("pasted");
    let after = all(&store, &Filter::default())
        .into_iter()
        .find(|row| row.id == one)
        .expect("still there");
    assert_eq!(after.modified_at, was, "expiry and d: read this one");
    assert_eq!(after.last_used_at, Some(10_000));
}

#[test]
fn what_you_paste_rises_to_the_top_of_the_recent_order() {
    let store = history();
    let rows = all(&store, &Filter::default());
    let buried = rows.last().expect("there are rows").id;
    assert_ne!(rows[0].id, buried, "it starts at the bottom");

    store.record_paste(buried, 100).expect("pasted");

    let ordered = all(&store, &Filter::default());
    assert_eq!(
        ordered[0].id, buried,
        "pasting moves it up, not just its count"
    );
    assert_eq!(ordered[0].paste_count, 1);
}

#[test]
fn the_meta_of_a_whole_page_comes_in_one_query() {
    let store = history();
    let rows = all(&store, &Filter::default());
    store.set_meta(rows[0].id, "duration", "138").expect("set");
    store.set_meta(rows[0].id, "width", "1920").expect("set");
    store.set_meta(rows[0].id, "artist", "nadie").expect("set");
    store.set_meta(rows[1].id, "duration", "3").expect("set");

    let ids: Vec<i64> = rows.iter().map(|one| one.id).collect();
    let found = store.meta_for(&ids, &["duration", "width"]).expect("meta");

    assert_eq!(found.len(), 2, "only the two that have any of those keys");
    let first = &found[&rows[0].id];
    assert_eq!(first.get("duration").map(String::as_str), Some("138"));
    assert_eq!(first.get("width").map(String::as_str), Some("1920"));
    assert!(first.get("artist").is_none(), "a key nobody asked for");
    assert_eq!(found[&rows[1].id].len(), 1);
}

#[test]
fn asking_for_nothing_reads_nothing() {
    let store = history();
    let rows = all(&store, &Filter::default());
    store.set_meta(rows[0].id, "duration", "138").expect("set");
    assert!(store.meta_for(&[], &["duration"]).expect("meta").is_empty());
    assert!(store.meta_for(&[rows[0].id], &[]).expect("meta").is_empty());
}

#[test]
fn an_item_with_no_meta_at_all_is_simply_absent() {
    let store = history();
    let rows = all(&store, &Filter::default());
    let ids: Vec<i64> = rows.iter().map(|one| one.id).collect();
    let found = store.meta_for(&ids, &["duration"]).expect("meta");
    assert!(found.is_empty(), "nobody wrote any, so nobody answers");
}

#[test]
fn deleting_an_item_takes_its_meta_with_it() {
    let store = history();
    let rows = all(&store, &Filter::default());
    store.set_meta(rows[0].id, "duration", "138").expect("set");
    store.mark_deleted(rows[0].id, 99).expect("deleted");
    let found = store.meta_for(&[rows[0].id], &["duration"]).expect("meta");
    assert!(found.is_empty(), "erasing a row leaves no meta behind");
}

#[test]
fn what_is_missing_a_measurement_can_be_found_without_touching_the_rest() {
    let store = history();
    let rows = all(&store, &Filter::default());
    let kinds: Vec<&str> = vec!["text", "email", "color"];
    let of_those = rows
        .iter()
        .filter(|row| {
            row.kind
                .map(|kind| kinds.contains(&kind.as_str()))
                .unwrap_or(false)
        })
        .count();
    assert!(of_those > 1, "the fixture has a few of those kinds");

    let waiting = store
        .missing_meta(&kinds, "pixels-wide", 100)
        .expect("asked");
    assert_eq!(waiting.len(), of_those, "nobody has been measured yet");

    store
        .set_meta(waiting[0], "pixels-wide", "1920")
        .expect("measured");
    let after = store
        .missing_meta(&kinds, "pixels-wide", 100)
        .expect("asked");
    assert_eq!(after.len(), of_those - 1, "the measured one drops out");
    assert!(!after.contains(&waiting[0]));
}

#[test]
fn a_kind_nobody_asked_about_is_never_queued() {
    let store = history();
    let only_colour = store
        .missing_meta(&["color"], "duration-ms", 100)
        .expect("asked");
    let every_row = all(&store, &Filter::default()).len();
    assert!(only_colour.len() < every_row, "only the colours");
    assert!(
        store
            .missing_meta(&[], "duration-ms", 100)
            .expect("asked")
            .is_empty(),
        "asking about no kind at all reads nothing"
    );
}

#[test]
fn a_deleted_row_is_not_sent_back_to_be_measured() {
    let store = history();
    let rows = all(&store, &Filter::default());
    store.mark_deleted(rows[0].id, 99).expect("deleted");
    let waiting = store
        .missing_meta(&["text", "email", "color"], "pixels-wide", 100)
        .expect("asked");
    assert!(!waiting.contains(&rows[0].id), "it is gone, not unmeasured");
}
