use super::*;

#[test]
fn an_empty_history_is_all_zeros() {
    let store = Store::in_memory().expect("store");
    assert_eq!(store.glance(0).expect("glance"), Glance::default());
}

#[test]
fn today_counts_what_was_copied_or_copied_again_since_midnight() {
    let store = Store::in_memory().expect("store");
    let old = store.insert_text("uuid-1", "yesterday", 100).expect("old");
    store
        .insert_text("uuid-2", "this morning", 1_000)
        .expect("new");
    store
        .insert_text("uuid-3", "this afternoon", 1_100)
        .expect("later");
    let gone = store
        .insert_text("uuid-4", "deleted", 1_200)
        .expect("deleted");
    store.mark_deleted(gone, 1_300).expect("marked");
    assert_eq!(store.glance(1_000).expect("glance").today, 2);
    store.reactivate(old, 1_400).expect("again");
    let seen = store.glance(1_000).expect("glance");
    assert_eq!((seen.items, seen.today), (3, 3));
}

#[test]
fn pinned_leaves_out_what_was_deleted() {
    let store = Store::in_memory().expect("store");
    let kept = store.insert_text("uuid-1", "kept", 1).expect("kept");
    let gone = store.insert_text("uuid-2", "gone", 2).expect("gone");
    store.set_pinned(kept, true, 3).expect("pinned");
    store.set_pinned(gone, true, 3).expect("pinned");
    store.mark_deleted(gone, 4).expect("deleted");
    assert_eq!(store.glance(0).expect("glance").pinned, 1);
}
