use super::*;
use cp_core::item::Format;

fn image(byte: u8, size: usize) -> Item {
    Item {
        kind: Some(Kind::Image),
        formats: vec![Format {
            id: "public.png".into(),
            payload: Payload::stored(vec![byte; size]),
        }],
    }
}

fn fill(store: &Store, count: i64) -> Vec<i64> {
    (1..=count)
        .map(|at| {
            store
                .insert_text(&format!("uuid-{at}"), &format!("note {at}"), at)
                .expect("insert")
        })
        .collect()
}

fn settle_blobs(dir: &std::path::Path) {
    for path in crate::blobs::files_under(&dir.join("blobs")) {
        std::fs::File::options()
            .write(true)
            .open(&path)
            .expect("opened")
            .set_modified(std::time::UNIX_EPOCH)
            .expect("aged");
    }
}

fn alive(store: &Store) -> Vec<i64> {
    store
        .list(&Filter::default(), 100, None)
        .expect("listed")
        .rows
        .iter()
        .map(|one| one.id)
        .collect()
}

#[test]
fn a_policy_with_nothing_set_sweeps_nothing() {
    let store = Store::in_memory().expect("schema");
    fill(&store, 5);
    let swept = store.sweep(&Policy::default(), 100).expect("swept");
    assert_eq!(
        swept,
        Swept {
            truncated: true,
            ..Swept::default()
        }
    );
    assert_eq!(store.count().expect("counted"), 5);
}

#[test]
fn age_takes_the_old_and_leaves_what_was_pinned() {
    let store = Store::in_memory().expect("schema");
    let ids = fill(&store, 5);
    store.set_pinned(ids[0], true, 0).expect("pinned");
    let policy = Policy {
        keep_for: Some(3),
        ..Default::default()
    };
    let swept = store.sweep(&policy, 6).expect("swept");
    assert_eq!(swept.expired, 1, "the one from before 3 that is not pinned");
    assert_eq!(alive(&store), vec![ids[4], ids[3], ids[2], ids[0]]);
}

#[test]
fn a_count_limit_evicts_the_oldest_unpinned_beyond_it() {
    let store = Store::in_memory().expect("schema");
    let ids = fill(&store, 6);
    store
        .set_pinned(ids[0], true, 0)
        .expect("pins the oldest one");
    let policy = Policy {
        keep_at_most: Some(4),
        ..Default::default()
    };
    let swept = store.sweep(&policy, 10).expect("swept");
    assert_eq!(swept.over_count, 2);
    assert_eq!(
        alive(&store),
        vec![ids[5], ids[4], ids[3], ids[0]],
        "the pinned one counts toward the limit but does not go"
    );
    assert_eq!(store.sweep(&policy, 11).expect("again").over_count, 0);
}

#[test]
fn a_count_limit_already_met_touches_nothing() {
    let store = Store::in_memory().expect("schema");
    fill(&store, 3);
    for keep in [3, 5] {
        let policy = Policy {
            keep_at_most: Some(keep),
            ..Default::default()
        };
        assert_eq!(store.sweep(&policy, 10).expect("swept").over_count, 0);
        assert_eq!(
            store.count().expect("counted"),
            3,
            "with {keep} as the limit"
        );
    }
}

#[test]
fn usage_counts_what_is_actually_kept_and_shared_bytes_once() {
    let (_dir, store) = on_disk();
    let empty = store.usage().expect("usage");
    assert_eq!((empty.items, empty.bytes), (0, 0));
    store.insert_text("uuid-t", "hello", 1).expect("insert");
    store
        .insert_item("uuid-a", &image(1, 200_000), "", 2)
        .expect("a");
    store
        .insert_item("uuid-b", &image(1, 200_000), "", 3)
        .expect("b");
    let announced = Item {
        kind: Some(Kind::Image),
        formats: vec![Format {
            id: "public.tiff".into(),
            payload: Payload::Announced {
                size: Some(4_000_000),
            },
        }],
    };
    store.insert_item("uuid-c", &announced, "", 4).expect("c");
    let usage = store.usage().expect("usage");
    assert_eq!(usage.items, 4);
    assert_eq!(
        usage.bytes, 200_000,
        "the text has no stored formats, the shared image counts once, the announced one counts nothing"
    );
}

#[test]
fn a_byte_quota_evicts_the_oldest_until_it_fits() {
    let (dir, store) = on_disk();
    for at in 1..=4 {
        store
            .insert_item(
                &format!("uuid-{at}"),
                &image(at as u8, 100_000),
                "",
                at as i64,
            )
            .expect("insert");
    }
    settle_blobs(dir.path());
    let policy = Policy {
        bytes_at_most: Some(200_000),
        ..Default::default()
    };
    let swept = store.sweep(&policy, 10).expect("swept");
    assert_eq!(
        swept.over_bytes, 2,
        "landing exactly on the quota still fits"
    );
    assert_eq!(store.usage().expect("usage").bytes, 200_000);
    assert_eq!(alive(&store).len(), 2);
    let files = crate::blobs::files_under(&dir.path().join("blobs")).len();
    assert_eq!(files, 2, "the evicted bytes are gone from disk");
}

#[test]
fn a_quota_never_evicts_what_is_pinned_even_if_it_stays_over() {
    let (_dir, store) = on_disk();
    let id = store
        .insert_item("uuid-1", &image(1, 200_000), "", 1)
        .expect("insert");
    store.set_pinned(id, true, 0).expect("pinned");
    let policy = Policy {
        bytes_at_most: Some(1_000),
        ..Default::default()
    };
    let swept = store.sweep(&policy, 10).expect("swept");
    assert_eq!(swept.over_bytes, 0);
    assert_eq!(store.count().expect("counted"), 1);
}

#[test]
fn a_shared_blob_is_freed_only_when_its_last_owner_goes() {
    let (_dir, store) = on_disk();
    store
        .insert_item("uuid-1", &image(7, 100_000), "", 1)
        .expect("a");
    store
        .insert_item("uuid-2", &image(7, 100_000), "", 2)
        .expect("b");
    let policy = Policy {
        bytes_at_most: Some(50_000),
        ..Default::default()
    };
    let swept = store.sweep(&policy, 10).expect("swept");
    assert_eq!(
        swept.over_bytes, 2,
        "deleting the first one frees nothing, so it moves on to the second"
    );
    assert_eq!(store.usage().expect("usage").bytes, 0);
}

#[test]
fn purging_on_its_own_leaves_nothing_in_the_log_either() {
    let dir = tempfile::tempdir().expect("a folder");
    let store = Store::open(&dir.path().join("history.db")).expect("opened");
    let secret = "secret-file-path";
    let id = store.insert_text("uuid-r", secret, 1).expect("insert");
    store.mark_broken(id, 2).expect("broken");
    store.purge_broken_before(10).expect("purged");
    let wal = std::fs::read(dir.path().join("history.db-wal")).expect("can be read");
    assert!(
        !wal.windows(secret.len())
            .any(|window| window == secret.as_bytes()),
        "purging is a deletion too"
    );
}

#[test]
fn broken_items_go_after_their_grace_and_take_their_bytes_along() {
    let (dir, store) = on_disk();
    let id = store
        .insert_item("uuid-broken", &image(3, 100_000), "", 1)
        .expect("insert");
    store.mark_broken(id, 5).expect("broken");
    settle_blobs(dir.path());
    let policy = Policy {
        broken_for: Some(10),
        ..Default::default()
    };
    assert_eq!(store.sweep(&policy, 14).expect("not yet").broken, 0);
    assert_eq!(store.sweep(&policy, 16).expect("already").broken, 1);
    let files = crate::blobs::files_under(&dir.path().join("blobs")).len();
    assert_eq!(
        files, 0,
        "purging a broken one cannot leave its image on disk"
    );
}

#[test]
fn deleting_an_item_whose_blob_was_just_written_leaves_the_file_to_the_sweep() {
    let (dir, store) = on_disk();
    let id = store
        .insert_item("uuid-fresh", &image(6, 100_000), "", 1)
        .expect("insert");
    store.mark_deleted(id, 2).expect("removed");
    assert_eq!(
        crate::blobs::files_under(&dir.path().join("blobs")).len(),
        1,
        "another connection might be about to reference the same content"
    );
    assert_eq!(
        store.sweep(&Policy::default(), 3).expect("swept").orphans,
        0,
        "still fresh"
    );
    settle_blobs(dir.path());
    assert_eq!(
        store.sweep(&Policy::default(), 4).expect("swept").orphans,
        1
    );
    assert!(crate::blobs::files_under(&dir.path().join("blobs")).is_empty());
}

#[test]
fn a_blob_nobody_points_at_is_swept_once_it_has_settled() {
    let (dir, store) = on_disk();
    let blobs = crate::Blobs::at(&dir.path().join("blobs")).expect("blobs");
    let digest = blobs.put(b"from an interrupted write").expect("stored");
    let path = dir
        .path()
        .join("blobs")
        .join(&digest[0..2])
        .join(&digest[2..4])
        .join(&digest);
    std::fs::File::options()
        .write(true)
        .open(&path)
        .expect("opened")
        .set_modified(std::time::UNIX_EPOCH)
        .expect("aged");
    let swept = store.sweep(&Policy::default(), 10).expect("swept");
    assert_eq!(swept.orphans, 1);
    assert!(!blobs.exists(&digest));
}

#[test]
fn a_blob_with_an_owner_is_never_an_orphan() {
    let (dir, store) = on_disk();
    store
        .insert_item("uuid-1", &image(9, 100_000), "", 1)
        .expect("insert");
    for path in crate::blobs::files_under(&dir.path().join("blobs")) {
        std::fs::File::options()
            .write(true)
            .open(&path)
            .expect("opened")
            .set_modified(std::time::UNIX_EPOCH)
            .expect("aged");
    }
    assert_eq!(
        store.sweep(&Policy::default(), 10).expect("swept").orphans,
        0
    );
    assert!(store.payload_of(1, "public.png").expect("read").is_some());
}

#[test]
fn everything_at_once_reports_each_count() {
    let (_dir, store) = on_disk();
    let ids = fill(&store, 6);
    store.mark_broken(ids[0], 1).expect("broken");
    let policy = Policy {
        keep_for: Some(6),
        keep_at_most: Some(2),
        bytes_at_most: Some(i64::MAX),
        broken_for: Some(1),
    };
    let swept = store.sweep(&policy, 10).expect("swept");
    assert_eq!(
        swept,
        Swept {
            broken: 1,
            expired: 2,
            over_count: 1,
            over_bytes: 0,
            orphans: 0,
            truncated: true,
        },
        "each rule counts what is its own, in order, without counting twice"
    );
    assert_eq!(alive(&store), vec![ids[5], ids[4]]);
}

#[test]
fn the_preview_is_cut_before_a_character_that_straddles_the_limit() {
    let text = format!("{}ñ{}", "a".repeat(PREVIEW_UP_TO - 1), "b".repeat(10));
    let head = head_of(&text);
    assert_eq!(
        head.len(),
        PREVIEW_UP_TO - 1,
        "a multi-byte character does not fit whole and gets left out"
    );
    assert!(head.bytes().all(|b| b == b'a'));
    assert_eq!(head_of("short"), "short");
    let exact = "x".repeat(PREVIEW_UP_TO);
    assert_eq!(head_of(&exact).len(), PREVIEW_UP_TO);
}

#[test]
fn a_capture_connection_can_leave_checkpoints_to_maintenance() {
    let (_dir, store) = on_disk();
    assert_eq!(
        store.autocheckpoint().expect("read"),
        1_000,
        "SQLite's factory default"
    );
    store.without_autocheckpoint().expect("turned off");
    assert_eq!(store.autocheckpoint().expect("read"), 0);
    for at in 0..200 {
        store
            .insert_text(&format!("uuid-{at}"), &"x".repeat(2_000), at)
            .expect("insert");
    }
    assert!(
        store.checkpoint_passive().expect("passive") > 0,
        "there are WAL pages to move into the database without waiting for anyone"
    );
    assert!(store.checkpoint().expect("truncated"));
    assert_eq!(
        store.checkpoint_passive().expect("again"),
        0,
        "after truncating there is nothing left to move"
    );
}

#[test]
fn a_sweep_leaves_nothing_in_the_write_ahead_log() {
    let (dir, store) = on_disk();
    let secret = "key-that-goes-away";
    store.insert_text("uuid-s", secret, 1).expect("insert");
    let policy = Policy {
        keep_for: Some(1),
        ..Default::default()
    };
    store.sweep(&policy, 10).expect("swept");
    for file in ["history.db", "history.db-wal"] {
        let bytes = std::fs::read(dir.path().join(file)).expect("can be read");
        assert!(
            !bytes
                .windows(secret.len())
                .any(|window| window == secret.as_bytes()),
            "«{secret}» is still legible in {file}"
        );
    }
}

#[test]
fn editing_replaces_the_content_and_drops_the_renderings_that_no_longer_match() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_item("uuid-e", &captured("hello world"), "hello world", 1)
        .expect("insert");
    store.update_text(id, "goodbye world", 2).expect("edited");

    let card = &store
        .list(&Filter::default(), 10, None)
        .expect("listed")
        .rows[0];
    assert_eq!(card.preview, "goodbye world");
    assert_eq!(
        card.modified_at, 1,
        "editing does not bump it up: the user already has it in front of them"
    );
    assert_eq!(
        store.formats_of(id).expect("formats"),
        vec![cp_core::item::SYNTHETIC_TEXT.to_string()],
        "the RTF said «hello world» and pasting it would mean pasting the old one"
    );
    assert_eq!(
        store
            .payload_of(id, cp_core::item::SYNTHETIC_TEXT)
            .expect("read")
            .as_deref(),
        Some("goodbye world".as_bytes())
    );
}

#[test]
fn the_edited_text_is_what_gets_found_and_classified() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_item("uuid-e", &captured("hello world"), "hello world", 1)
        .expect("insert");
    store.update_text(id, "#FF8800", 2).expect("edited");
    assert!(search(&store, "hello").is_empty());
    assert_eq!(search(&store, "ff8800").len(), 1);
    let card = &store
        .list(&Filter::default(), 10, None)
        .expect("listed")
        .rows[0];
    assert_eq!(card.kind, Some(Kind::Color));
    assert_eq!(
        store.find_by_hash(&Item::plain("#FF8800")).expect("hash"),
        Some(id),
        "the identity is that of the new text"
    );
    assert!(
        store
            .changed_since(1)
            .expect("changes")
            .contains(&"uuid-e".to_string()),
        "the version moves forward"
    );
}

#[test]
fn editing_an_image_into_text_takes_its_bytes_off_the_disk() {
    let (dir, store) = on_disk();
    let id = store
        .insert_item("uuid-img", &image(2, 100_000), "", 1)
        .expect("insert");
    store.set_ocr_text(id, "read text", 2).expect("ocr");
    store.set_meta(id, "width", "800").expect("meta");
    settle_blobs(dir.path());
    store.update_text(id, "read text", 3).expect("edited");
    assert_eq!(
        crate::blobs::files_under(&dir.path().join("blobs")).len(),
        0
    );
    assert!(store.all_meta(id).expect("meta").is_empty());
    assert_eq!(search(&store, "read").len(), 1, "now it is content");
    assert!(store.pending_ocr(10).expect("ocr").is_empty());
}

#[test]
fn what_was_edited_away_is_not_left_lying_in_the_database_or_its_log() {
    let dir = tempfile::tempdir().expect("a folder");
    let store = Store::open(&dir.path().join("history.db")).expect("opened");
    let secret = "hunter2-the-old-one";
    let id = store.insert_text("uuid-s", secret, 1).expect("insert");
    store
        .checkpoint()
        .expect("it is already in the main database");
    store.update_text(id, "innocent text", 2).expect("edited");
    for file in ["history.db", "history.db-wal"] {
        let bytes = std::fs::read(dir.path().join(file)).expect("can be read");
        assert!(
            !bytes
                .windows(secret.len())
                .any(|window| window == secret.as_bytes()),
            "«{secret}» is still legible in {file}"
        );
    }
}

#[test]
fn editing_a_broken_file_makes_it_a_whole_text_again() {
    let store = Store::in_memory().expect("schema");
    let id = store
        .insert_text("uuid-broken", "/tmp/gone.txt", 1)
        .expect("insert");
    store.mark_broken(id, 5).expect("broken");
    store
        .update_text(id, "what it used to say", 6)
        .expect("edited");
    let rows = store
        .list(&Filter::default(), 10, None)
        .expect("listed")
        .rows;
    assert_eq!(rows.len(), 1, "a text item cannot be broken");
    assert_eq!(rows[0].broken_since, None);
}

#[test]
fn an_edit_that_cannot_be_written_leaves_the_item_as_it_was() {
    let (dir, store) = on_disk();
    let id = store
        .insert_item("uuid-e", &captured("intact"), "intact", 1)
        .expect("insert");
    let blobs = dir.path().join("blobs");
    let long = "x".repeat(cp_core::item::INLINE_UP_TO + 1);
    std::fs::remove_dir_all(&blobs).expect("blobs folder gone");
    std::fs::write(&blobs, b"I am not a folder").expect("gets in the way");
    assert!(
        store.update_text(id, &long, 2).is_err(),
        "there is nowhere to put the blob"
    );
    let card = &store
        .list(&Filter::default(), 10, None)
        .expect("listed")
        .rows[0];
    assert_eq!(card.preview, "intact", "the row did not change");
    assert_eq!(
        store.formats_of(id).expect("formats").len(),
        2,
        "and the previous formats are still there"
    );
    assert_eq!(
        store
            .payload_of(id, "public.utf8-plain-text")
            .expect("read")
            .as_deref(),
        Some("intact".as_bytes())
    );
}

#[test]
fn an_edit_bigger_than_a_blob_is_refused_before_touching_the_row() {
    let (_dir, store) = on_disk();
    let id = store.insert_text("uuid-x", "short", 1).expect("insert");
    let absurd = "x".repeat(cp_core::item::BLOB_UP_TO + 1);
    assert!(matches!(
        store.update_text(id, &absurd, 2),
        Err(Error::TooBig { .. })
    ));
    assert_eq!(
        store
            .list(&Filter::default(), 10, None)
            .expect("listed")
            .rows[0]
            .preview,
        "short"
    );
    assert!(matches!(
        store.insert_text("uuid-y", &absurd, 3),
        Err(Error::TooBig { .. })
    ));
}

#[test]
fn editing_an_image_forgets_what_was_read_in_it() {
    let (dir, store) = on_disk();
    let id = store
        .insert_item("uuid-img", &image(2, 100_000), "", 1)
        .expect("insert");
    store.set_ocr_text(id, "qzzsecret read", 2).expect("ocr");
    assert_eq!(search(&store, "qzzsecret").len(), 1);
    settle_blobs(dir.path());
    store.update_text(id, "something else", 3).expect("edited");
    assert!(
        search(&store, "qzzsecret").is_empty(),
        "what was read belonged to the image that is no longer there"
    );
}

#[test]
fn marking_present_what_was_already_purged_is_a_quiet_no_op() {
    let store = Store::in_memory().expect("schema");
    let id = store.insert_text("uuid-r", "/tmp/gone", 1).expect("insert");
    store.mark_broken(id, 5).expect("broken");
    store.purge_broken_before(10).expect("purged");
    store
        .mark_present(id)
        .expect("it no longer exists, and nothing happens");
    assert_eq!(store.count().expect("counted"), 0);
}

#[test]
fn a_broken_item_exactly_at_the_cutoff_is_not_purged_yet() {
    let store = Store::in_memory().expect("schema");
    let id = store.insert_text("uuid-r", "/tmp/gone", 1).expect("insert");
    store.mark_broken(id, 5).expect("broken");
    let policy = Policy {
        broken_for: Some(10),
        ..Default::default()
    };
    assert_eq!(
        store.sweep(&policy, 15).expect("precisely").broken,
        0,
        "5 is not less than 15 - 10"
    );
    assert_eq!(store.sweep(&policy, 16).expect("already").broken, 1);
}

#[test]
fn the_count_limit_runs_before_the_byte_quota() {
    let (dir, store) = on_disk();
    for at in 1..=4 {
        store
            .insert_item(
                &format!("uuid-{at}"),
                &image(at as u8, 100_000),
                "",
                at as i64,
            )
            .expect("insert");
    }
    settle_blobs(dir.path());
    let policy = Policy {
        keep_at_most: Some(2),
        bytes_at_most: Some(150_000),
        ..Default::default()
    };
    let swept = store.sweep(&policy, 10).expect("swept");
    assert_eq!(
        (swept.over_count, swept.over_bytes),
        (2, 1),
        "the quota sees what the limit left behind"
    );
    assert_eq!(store.count().expect("counted"), 1);
}

#[test]
fn editing_what_does_not_exist_or_was_deleted_is_refused() {
    let store = Store::in_memory().expect("schema");
    assert!(matches!(
        store.update_text(404, "nothing", 1),
        Err(Error::NoSuchItem { id: 404 })
    ));
    let id = store.insert_text("uuid-d", "something", 1).expect("insert");
    store.mark_deleted(id, 2).expect("removed");
    assert!(store.update_text(id, "resurrects", 3).is_err());
    assert_eq!(store.count().expect("counted"), 0);
}

#[test]
fn a_long_edit_goes_to_disk_and_an_absurd_one_is_refused() {
    let (_dir, store) = on_disk();
    let id = store.insert_text("uuid-l", "short", 1).expect("insert");
    let long = "x".repeat(cp_core::item::INLINE_UP_TO + 1);
    store.update_text(id, &long, 2).expect("edited");
    assert_eq!(store.usage().expect("usage").bytes as usize, long.len());

    let memory = Store::in_memory().expect("schema");
    let id = memory.insert_text("uuid-m", "short", 1).expect("insert");
    assert!(
        matches!(
            memory.update_text(id, &long, 2),
            Err(Error::NeedsBlobStore { .. })
        ),
        "with no folder there is nowhere to put it"
    );
    assert_eq!(
        search(&memory, "short").len(),
        1,
        "and what was there before stays intact"
    );
}

#[test]
fn the_parser_and_the_list_speak_the_same_filter() {
    let store = Store::in_memory().expect("schema");
    let rows = [
        ("uuid-1", "monday meeting", "Slack", 10),
        ("uuid-2", "tuesday meeting", "Code", 20),
        ("uuid-3", "something else", "Slack", 30),
    ];
    for (uuid, text, app, at) in rows {
        let id = store.insert_text(uuid, text, at).expect("insert");
        store.set_source(id, app, at).expect("sourced");
    }
    let clock = crate::query::Clock {
        now: 40,
        day_start: 0,
    };
    let filter = crate::query::parse("meeting @slack", &clock);
    let found = store.list(&filter, 10, None).expect("listed").rows;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].preview, "monday meeting");
    assert_eq!(
        found[0]
            .snippet
            .as_ref()
            .map(|snippet| snippet.excerpt.plain()),
        Some("monday meeting".into())
    );
}
