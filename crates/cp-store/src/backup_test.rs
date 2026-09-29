use super::*;
use cp_core::item::{Format, Item, Payload, SYNTHETIC_IMAGE, SYNTHETIC_TEXT};
use cp_core::kind::Kind;

fn text(what: &str) -> Item {
    Item {
        kind: Some(Kind::Text),
        formats: vec![Format {
            id: SYNTHETIC_TEXT.into(),
            payload: Payload::Inline(what.as_bytes().to_vec()),
        }],
    }
}

fn heavy() -> Item {
    Item {
        kind: Some(Kind::Image),
        formats: vec![Format {
            id: SYNTHETIC_IMAGE.into(),
            payload: Payload::Blob(vec![7u8; cp_core::item::INLINE_UP_TO + 64]),
        }],
    }
}

#[cfg(target_os = "windows")]
fn files(paths: &[&str]) -> Item {
    Item {
        kind: Some(Kind::File),
        formats: vec![Format {
            id: "CF_HDROP".into(),
            payload: Payload::Inline(paths.join("\0").into_bytes()),
        }],
    }
}

#[cfg(not(target_os = "windows"))]
fn files(paths: &[&str]) -> Item {
    let urls = paths
        .iter()
        .map(|path| format!("file://{path}"))
        .collect::<Vec<_>>()
        .join("\n");
    Item {
        kind: Some(Kind::File),
        formats: vec![Format {
            id: "public.file-url".into(),
            payload: Payload::Inline(urls.into_bytes()),
        }],
    }
}

fn somewhere(name: &str) -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().expect("a folder");
    let store = Store::open(&dir.path().join(name)).expect("opened");
    (dir, store)
}

#[test]
fn what_was_kept_travels_whole_from_one_machine_to_another() {
    let (there, store) = somewhere("history.db");
    store
        .insert_item("one", &text("the first thing"), "the first thing", 1_000)
        .expect("stored");
    let heavy_id = store
        .insert_item("two", &heavy(), "", 2_000)
        .expect("stored");
    store.set_pinned(heavy_id, true, 2_000).expect("pinned");
    let backup = there.path().join("mine.cpbackup");
    let made = write(&store, &backup, 3_000).expect("exported");
    assert_eq!(made.items, 2);
    assert_eq!(made.blobs, 1, "the blob had to travel along");
    assert!(made.bytes > 0);
    drop(store);

    let taken = read(&backup).expect("read");
    assert_eq!(taken.format, FORMAT);
    assert_eq!(taken.items, 2);
    assert_eq!(taken.written_at, 3_000);

    let (_here, landed) = somewhere("history.db");
    let brought = bring(&backup, &landed, 4_000).expect("imported");
    assert_eq!(brought.added, 2);
    assert_eq!(brought.already, 0);
    assert_eq!(landed.count().expect("counted"), 2);
    let page = landed
        .list(&crate::Filter::default(), 10, None)
        .expect("listed");
    let picture = page
        .rows
        .iter()
        .find(|one| one.kind == Some(Kind::Image))
        .expect("the image arrived");
    assert!(picture.pinned, "what was pinned is still pinned");
    let payload = landed
        .payload_of(picture.id, SYNTHETIC_IMAGE)
        .expect("read")
        .expect("the blob is there");
    assert_eq!(payload.len(), cp_core::item::INLINE_UP_TO + 64);
}

#[test]
fn importing_adds_and_never_takes_away_what_was_already_there() {
    let (there, store) = somewhere("history.db");
    store
        .insert_item("one", &text("from the backup"), "from the backup", 1_000)
        .expect("stored");
    let backup = there.path().join("mine.cpbackup");
    write(&store, &backup, 1_000).expect("exported");
    drop(store);

    let (_here, landed) = somewhere("history.db");
    for (at, what) in [(1, "one"), (2, "two"), (3, "three")] {
        landed
            .insert_item(&format!("v{at}"), &text(what), what, at)
            .expect("stored");
    }
    let brought = bring(&backup, &landed, 5_000).expect("imported");
    assert_eq!(brought.added, 1);
    assert_eq!(
        landed.count().expect("counted"),
        4,
        "none of what was already there went away"
    );
}

#[test]
fn the_same_copy_brought_twice_does_not_double_anything() {
    let (there, store) = somewhere("history.db");
    store
        .insert_item("one", &text("something"), "something", 1_000)
        .expect("stored");
    store
        .insert_item("two", &text("something else"), "something else", 2_000)
        .expect("stored");
    let backup = there.path().join("mine.cpbackup");
    write(&store, &backup, 1_000).expect("exported");
    drop(store);

    let (_here, landed) = somewhere("history.db");
    let first = bring(&backup, &landed, 3_000).expect("imported");
    let again = bring(&backup, &landed, 4_000).expect("imported again");
    assert_eq!(first.added, 2);
    assert_eq!(again.added, 0);
    assert_eq!(again.already, 2);
    assert_eq!(landed.count().expect("counted"), 2);
}

#[test]
fn a_copy_refuses_to_be_written_over_the_history_it_is_copying() {
    let (there, store) = somewhere("history.db");
    store
        .insert_item("one", &text("what is there"), "what is there", 1_000)
        .expect("stored");
    let live = there.path().join("history.db");
    assert!(
        matches!(write(&store, &live, 2_000), Err(Error::OntoItself)),
        "the history cannot be its own copy"
    );
    assert_eq!(store.count().expect("counted"), 1, "and it is still whole");
    drop(store);
    assert_eq!(
        Store::open(&live)
            .expect("reopened")
            .count()
            .expect("counted"),
        1,
        "reopening it from disk confirms it"
    );
}

#[test]
fn a_file_that_can_no_longer_be_found_still_travels_and_still_arrives() {
    let (there, store) = somewhere("history.db");
    let id = store
        .insert_item("one", &files(&["/tmp/gone.txt"]), "/tmp/gone.txt", 1_000)
        .expect("stored");
    store.mark_broken(id, 2_000).expect("marked broken");
    let backup = there.path().join("mine.cpbackup");
    assert_eq!(write(&store, &backup, 3_000).expect("exported").items, 1);
    drop(store);

    let (_here, landed) = somewhere("history.db");
    let brought = bring(&backup, &landed, 4_000).expect("imported");
    assert_eq!(
        brought.added, 1,
        "what is broken does not get left out silently"
    );
    assert_eq!(landed.count().expect("counted"), 1);
}

#[test]
fn a_backup_that_cannot_be_written_does_not_take_the_previous_one_with_it() {
    let (there, store) = somewhere("history.db");
    store
        .insert_item(
            "one",
            &text("what was already there"),
            "what was already there",
            1_000,
        )
        .expect("stored");
    let backup = there.path().join("mine.cpbackup");
    write(&store, &backup, 1_000).expect("exported");
    let before = std::fs::read(&backup).expect("read");

    let nowhere = there.path().join("dead-end");
    std::fs::write(&nowhere, b"I am not a folder").expect("written");
    assert!(
        write(&store, &nowhere.join("another.cpbackup"), 2_000).is_err(),
        "writing inside a file cannot end well"
    );
    assert_eq!(
        std::fs::read(&backup).expect("read"),
        before,
        "the previous copy is still whole"
    );
}

#[test]
fn a_half_written_backup_never_takes_the_name_of_the_good_one() {
    let (there, store) = somewhere("history.db");
    store
        .insert_item("one", &text("something"), "something", 1_000)
        .expect("stored");
    let backup = there.path().join("mine.cpbackup");
    write(&store, &backup, 1_000).expect("exported");
    assert!(backup.exists());
    assert!(
        !there.path().join("mine.cpbackup.part").exists(),
        "the half-written file does not stick around"
    );
    assert!(read(&backup).is_ok(), "what is left behind is readable");
}

#[test]
fn a_backup_says_where_it_was_made_so_the_other_system_can_warn() {
    let (there, store) = somewhere("history.db");
    store
        .insert_item("one", &text("something"), "something", 1_000)
        .expect("stored");
    let backup = there.path().join("mine.cpbackup");
    write(&store, &backup, 1_000).expect("exported");
    assert_eq!(
        read(&backup).expect("read").platform.as_deref(),
        Some(WHERE_IT_WAS_MADE)
    );

    let (_here, landed) = somewhere("history.db");
    let brought = bring(&backup, &landed, 2_000).expect("imported");
    assert!(!brought.from_elsewhere, "it comes from this very platform");
}

#[test]
fn a_blob_the_sweep_took_before_it_could_travel_is_counted_and_not_hidden() {
    let (there, store) = somewhere("history.db");
    store
        .insert_item("one", &heavy(), "", 1_000)
        .expect("stored");
    for digest in std::fs::read_dir(there.path().join("blobs"))
        .expect("blobs")
        .filter_map(|one| one.ok())
    {
        let _ = std::fs::remove_dir_all(digest.path());
    }
    let backup = there.path().join("mine.cpbackup");
    let made = write(&store, &backup, 2_000).expect("exported");
    assert_eq!(made.blobs, 0);
    assert_eq!(
        made.missing, 1,
        "what went missing gets counted, not hidden"
    );
}

#[test]
fn what_was_opened_to_read_a_backup_is_swept_when_it_is_done() {
    let (there, store) = somewhere("history.db");
    store
        .insert_item("one", &text("something"), "something", 1_000)
        .expect("stored");
    let backup = there.path().join("mine.cpbackup");
    write(&store, &backup, 1_000).expect("exported");
    drop(store);

    let dir = {
        let opened = Opened::of(&backup).expect("opened");
        assert!(
            opened.db.exists(),
            "the copy gets materialised so it can be read"
        );
        opened.dir.clone()
    };
    assert!(!dir.exists(), "the opened copy stayed on disk");
}

#[test]
fn what_is_brought_in_is_a_history_and_not_a_backup() {
    let (there, store) = somewhere("history.db");
    store
        .insert_item("one", &text("something"), "something", 1_000)
        .expect("stored");
    let backup = there.path().join("mine.cpbackup");
    write(&store, &backup, 1_000).expect("exported");
    drop(store);

    let (_here, landed) = somewhere("history.db");
    bring(&backup, &landed, 2_000).expect("imported");
    let left: i64 = landed
        .raw()
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE name LIKE 'backup_%'",
            [],
            |row| row.get(0),
        )
        .expect("counted");
    assert_eq!(left, 0);
}

#[test]
fn a_file_that_is_not_a_backup_is_refused_before_anything_is_touched() {
    let dir = tempfile::tempdir().expect("a folder");
    let stranger = dir.path().join("anything.cpbackup");
    std::fs::write(&stranger, b"I am not a database").expect("written");
    assert!(read(&stranger).is_err());

    let plain = dir.path().join("plain.db");
    let store = Store::open(&plain).expect("opened");
    store
        .insert_item("one", &text("something"), "something", 1_000)
        .expect("stored");
    drop(store);
    assert!(
        matches!(read(&plain), Err(Error::NotABackup)),
        "a loose database is not a backup"
    );

    let (_here, landed) = somewhere("history.db");
    landed
        .insert_item("mine", &text("mine"), "mine", 1_000)
        .expect("stored");
    assert!(bring(&plain, &landed, 2_000).is_err());
    assert_eq!(
        landed.count().expect("counted"),
        1,
        "what was theirs is still there"
    );
}

#[test]
fn exporting_twice_over_the_same_file_simply_writes_it_again() {
    let (there, store) = somewhere("history.db");
    store
        .insert_item("one", &text("something"), "something", 1_000)
        .expect("stored");
    let backup = there.path().join("mine.cpbackup");
    let first = write(&store, &backup, 1_000).expect("exported");
    store
        .insert_item("two", &text("something else"), "something else", 2_000)
        .expect("stored");
    let again = write(&store, &backup, 2_000).expect("exported again");
    assert_eq!(first.items, 1);
    assert_eq!(again.items, 2);
    assert_eq!(read(&backup).expect("read").items, 2);
}

#[test]
fn an_empty_history_still_makes_a_backup_that_can_be_brought_back() {
    let (there, store) = somewhere("history.db");
    let backup = there.path().join("empty.cpbackup");
    let made = write(&store, &backup, 1_000).expect("exported");
    assert_eq!(made.items, 0);
    assert_eq!(made.blobs, 0);
    drop(store);

    let (_here, landed) = somewhere("history.db");
    let brought = bring(&backup, &landed, 2_000).expect("imported");
    assert_eq!(brought.added, 0);
    assert_eq!(landed.count().expect("counted"), 0);
}
