use super::*;

#[test]
fn what_was_captured_is_found_again() {
    let store = Store::in_memory().expect("opened");
    let item = captured("hello");
    store
        .insert_item("uuid-1", &item, "hello", 1)
        .expect("inserted");
    assert_eq!(
        store.find_by_hash(&item).expect("searched"),
        Some(1),
        "what insert_item stores, find_by_hash has to recognise"
    );
}

#[test]
fn a_different_rendering_is_a_different_item() {
    let store = Store::in_memory().expect("opened");
    let plain = captured("hello");
    store
        .insert_item("uuid-1", &plain, "hello", 1)
        .expect("inserted");
    assert!(
        store
            .find_by_hash(&Item::plain("**hello**"))
            .expect("searched")
            .is_none()
    );
}

#[test]
fn a_synthetic_text_is_not_a_captured_one() {
    assert_ne!(
        Item::plain("hello").fingerprint(),
        captured("hello").fingerprint()
    );
}

#[test]
fn copying_the_same_thing_twice_from_google_reactivates_instead_of_duplicating() {
    use cp_core::item::Format;
    let store = Store::in_memory().expect("schema");
    let docs = |guid: &str| Item {
        kind: Some(Kind::Text),
        formats: vec![
            Format {
                id: "public.html".into(),
                payload: Payload::Inline(
                    format!("<b id=\"docs-internal-guid-{guid}\"><span>hello</span></b>")
                        .into_bytes(),
                ),
            },
            Format {
                id: "public.utf8-plain-text".into(),
                payload: Payload::Inline(b"hello".to_vec()),
            },
        ],
    };
    let id = store
        .insert_item("uuid-docs", &docs("4a1e6b2f-7fff-1d3e"), "hello", 1)
        .expect("inserted");
    assert_eq!(
        store
            .find_by_hash(&docs("0c9d8e7f-7fff-aaaa"))
            .expect("searched"),
        Some(id),
        "a different GUID, the same item"
    );
    assert_eq!(
        store.find_by_hash(&Item::plain("hello")).expect("searched"),
        None,
        "plain text copied from where it was pasted is a different item"
    );
}

#[test]
fn the_plain_text_pasted_from_a_rich_item_lands_as_a_second_history_entry() {
    let store = Store::in_memory().expect("schema");
    let rich = captured("hello world");
    let id = store
        .insert_item("uuid-rich", &rich, "hello world", 1)
        .expect("inserted");

    let replayed = Item::plain("hello world");
    assert_eq!(
        store.find_by_hash(&replayed).expect("searched"),
        None,
        "the plain text that paste-as-plain would put on the clipboard does not \
         fingerprint-match the rich item it came from"
    );
    let second_id = store
        .insert_item("uuid-replayed", &replayed, "hello world", 2)
        .expect("inserted");
    assert_ne!(
        id, second_id,
        "the same visible text now lives in two separate history entries instead of \
         reactivating the one it was pasted from"
    );
    assert_eq!(store.count().expect("counted"), 2);

    let original = store.item(id).expect("fetched").expect("still there");
    assert_eq!(
        original.formats.len(),
        2,
        "the original keeps every format it was captured with"
    );
    assert!(
        original.format("public.rtf").is_some(),
        "it can still be pasted rich"
    );
}

#[test]
fn two_different_oversized_captures_collide_into_the_same_identity() {
    use cp_core::item::Format;
    let store = Store::in_memory().expect("schema");
    let refused = |size: usize| Item {
        kind: Some(Kind::Image),
        formats: vec![Format {
            id: "public.png".into(),
            payload: Payload::TooBig { size },
        }],
    };
    let id = store
        .insert_item("uuid-huge-photo", &refused(90_000_000), "", 1)
        .expect("inserted");
    assert_eq!(
        store.find_by_hash(&refused(120_000_000)).expect("searched"),
        Some(id),
        "fingerprint() drops every format whose payload is TooBig before hashing, so two \
         unrelated captures that were both too large to keep hash to the same empty mix \
         regardless of their real size; a caller that checks find_by_hash before inserting \
         (as the panel's capture path does) will reactivate the first oversized item instead \
         of ever recording the second one's existence"
    );
}
