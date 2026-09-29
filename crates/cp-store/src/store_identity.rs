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
