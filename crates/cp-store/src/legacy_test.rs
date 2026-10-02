use super::*;

#[test]
fn the_numbers_the_2x_stores_are_not_the_positions_of_its_enum() {
    for (said, want) in [
        (0, Kind::Text),
        (1, Kind::Image),
        (2, Kind::File),
        (3, Kind::Folder),
        (4, Kind::Link),
        (5, Kind::Audio),
        (6, Kind::Video),
        (7, Kind::Email),
        (8, Kind::Phone),
        (9, Kind::Color),
        (10, Kind::Ip),
        (11, Kind::Uuid),
        (12, Kind::Json),
    ] {
        assert_eq!(
            kind_of(said),
            Some(want),
            "the 2.x wrote {said} for {want:?} and that number is in its database, not ours"
        );
    }
    assert_eq!(
        kind_of(-1),
        Some(Kind::Text),
        "what the 2.x could not tell apart arrives as text"
    );
    assert_eq!(kind_of(99), Some(Kind::Text));
}

#[test]
fn what_carries_a_path_instead_of_its_content_is_known() {
    assert!(holds_a_path(Kind::Image));
    assert!(holds_a_path(Kind::File));
    assert!(holds_a_path(Kind::Video));
    assert!(!holds_a_path(Kind::Text));
    assert!(!holds_a_path(Kind::Link));
}

#[test]
fn metadata_that_is_not_an_object_brings_nothing_and_does_not_fail() {
    assert!(meta_in(None).is_empty());
    assert!(meta_in(Some("")).is_empty());
    assert!(meta_in(Some("not json at all")).is_empty());
    assert!(meta_in(Some("[1, 2, 3]")).is_empty());
}

#[test]
fn metadata_comes_across_as_pairs_whatever_the_json_held() {
    let mut pairs = meta_in(Some(
        r#"{"width": 1920, "artist": "alguien", "empty": "", "nothing": null}"#,
    ));
    pairs.sort();
    assert_eq!(
        pairs,
        vec![
            ("artist".to_owned(), "alguien".to_owned()),
            ("width".to_owned(), "1920".to_owned())
        ],
        "numbers keep their value, and what says nothing is left out"
    );
}

#[test]
fn a_name_from_the_2x_says_where_it_came_from() {
    let row = Row {
        uuid: "abc-123".to_owned(),
        content: String::new(),
        kind: Kind::Text,
        created_at: 1,
        modified_at: 1,
        app: None,
        pinned: false,
        label: None,
        colour: 0,
        meta: None,
        pastes: 0,
        broken: None,
    };
    assert_eq!(named(&row, 9), "2x-abc-123");
    let nameless = Row {
        uuid: String::new(),
        ..row
    };
    assert!(named(&nameless, 9).starts_with('9'));
}

const FORMER: &str = "CREATE TABLE clipboard_items (
            id TEXT NOT NULL PRIMARY KEY,
            content TEXT NOT NULL,
            type INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            modified_at INTEGER NOT NULL,
            app_source TEXT,
            is_pinned INTEGER NOT NULL DEFAULT 0,
            label TEXT,
            card_color INTEGER NOT NULL DEFAULT 0,
            metadata TEXT,
            paste_count INTEGER NOT NULL DEFAULT 0,
            content_hash TEXT,
            thumb_path TEXT,
            broken_since INTEGER);";

fn a_former_history(dir: &Path, picture: &Path) -> std::path::PathBuf {
    let at = dir.join("clipboard.db");
    let db = Connection::open(&at).expect("opened");
    db.execute_batch(FORMER).expect("made");
    let mut put = db
        .prepare(
            "INSERT INTO clipboard_items
                 (id, content, type, created_at, modified_at, app_source, is_pinned, label,
                  card_color, metadata, paste_count, content_hash, thumb_path, broken_since)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
        )
        .expect("prepared");
    put.execute(rusqlite::params![
        "one",
        "just some text",
        0,
        1_000,
        1_000,
        "Mail",
        0,
        None::<String>,
        0,
        None::<String>,
        3,
        "h1",
        None::<String>,
        None::<i64>
    ])
    .expect("stored");
    put.execute(rusqlite::params![
        "two",
        picture.to_string_lossy(),
        1,
        2_000,
        2_000,
        "Preview",
        1,
        "the receipt",
        2,
        r#"{"width": 800}"#,
        0,
        "h2",
        None::<String>,
        None::<i64>
    ])
    .expect("stored");
    put.execute(rusqlite::params![
        "three",
        "/gone/for/good.png",
        1,
        3_000,
        3_000,
        None::<String>,
        0,
        None::<String>,
        0,
        None::<String>,
        0,
        "h3",
        None::<String>,
        None::<i64>
    ])
    .expect("stored");
    put.execute(rusqlite::params![
        "four",
        "https://example.com",
        4,
        4_000,
        4_000,
        None::<String>,
        0,
        None::<String>,
        0,
        None::<String>,
        0,
        "h4",
        None::<String>,
        9_000
    ])
    .expect("stored");
    put.execute(rusqlite::params![
        "five",
        "{\"a\": 1}",
        12,
        5_000,
        5_000,
        None::<String>,
        0,
        None::<String>,
        0,
        None::<String>,
        0,
        "h5",
        None::<String>,
        None::<i64>
    ])
    .expect("stored");
    drop(put);
    drop(db);
    at
}

#[test]
fn a_history_from_the_2x_crosses_with_what_the_3_0_can_hold() {
    let there = tempfile::tempdir().expect("a folder");
    let picture = there.path().join("shot.png");
    std::fs::write(&picture, b"\x89PNG-pretend").expect("written");
    let former = a_former_history(there.path(), &picture);

    let looked = look(&former, 0, None).expect("looked");
    assert_eq!(looked.items, 5);
    assert_eq!(looked.pictures, 2);
    assert_eq!(looked.pictures_gone, 1, "one picture is no longer on disk");
    assert_eq!(looked.pinned, 1);
    assert_eq!(looked.labelled, 1);

    let here = tempfile::tempdir().expect("a folder");
    let into = Store::open(&here.path().join("history.db")).expect("opened");
    let brought = bring(&former, &into, 9_999).expect("brought");
    assert_eq!(brought.added, 5);
    assert_eq!(brought.already, 0);
    assert_eq!(brought.refused, 0);
    assert_eq!(
        brought.without_their_picture, 1,
        "the one whose file is gone arrives as the path it pointed at"
    );

    let page = into
        .list(
            &crate::Filter {
                broken: crate::Broken::Shown,
                ..Default::default()
            },
            20,
            None,
        )
        .expect("listed");
    assert_eq!(page.rows.len(), 5);

    let picture_row = page
        .rows
        .iter()
        .find(|one| one.label.as_deref() == Some("the receipt"))
        .expect("the labelled one is here");
    assert_eq!(picture_row.kind, Some(Kind::Image));
    assert!(picture_row.pinned, "what was pinned is still pinned");
    assert_eq!(picture_row.app.as_deref(), Some("Preview"));
    assert_eq!(picture_row.color, 2, "and its colour crossed too");
    let bytes = into
        .payload_of(picture_row.id, SYNTHETIC_IMAGE)
        .expect("read")
        .expect("the picture itself, not its path");
    assert_eq!(bytes, b"\x89PNG-pretend");
    let meta = into.all_meta(picture_row.id).expect("read");
    assert_eq!(meta, vec![("width".to_owned(), "800".to_owned())]);

    let link = page
        .rows
        .iter()
        .find(|one| one.kind == Some(Kind::Link))
        .expect("the link is here");
    assert!(link.broken_since.is_some(), "what was broken stays broken");

    assert!(
        page.rows.iter().any(|one| one.kind == Some(Kind::Json)),
        "the 2.x told json apart and so does this"
    );
}

#[test]
fn bringing_the_same_history_twice_adds_nothing_the_second_time() {
    let there = tempfile::tempdir().expect("a folder");
    let picture = there.path().join("shot.png");
    std::fs::write(&picture, b"png").expect("written");
    let former = a_former_history(there.path(), &picture);

    let here = tempfile::tempdir().expect("a folder");
    let into = Store::open(&here.path().join("history.db")).expect("opened");
    let first = bring(&former, &into, 1).expect("brought");
    let again = bring(&former, &into, 2).expect("brought again");
    assert_eq!(first.added, 5);
    assert_eq!(again.added, 0);
    assert_eq!(again.already, 5);
    assert_eq!(into.count().expect("counted"), 5);
}

#[test]
fn what_was_deleted_here_does_not_come_back_when_the_2x_is_brought_again() {
    let there = tempfile::tempdir().expect("a folder");
    let picture = there.path().join("shot.png");
    std::fs::write(&picture, b"png").expect("written");
    let former = a_former_history(there.path(), &picture);

    let here = tempfile::tempdir().expect("a folder");
    let into = Store::open(&here.path().join("history.db")).expect("opened");
    bring(&former, &into, 1).expect("brought");
    let page = into
        .list(&crate::Filter::default(), 10, None)
        .expect("listed");
    let first = page.rows.first().expect("a row").id;
    into.mark_deleted(first, 2).expect("deleted");

    let again = bring(&former, &into, 3).expect("brought again");
    assert_eq!(again.added, 0);
    assert_eq!(
        into.count().expect("counted"),
        4,
        "deleting something is a decision, and bringing the 2x again must not undo it"
    );
    assert_eq!(
        again.refused, 0,
        "what was deleted on purpose already crossed once, and the page must not call that a failure"
    );
    assert_eq!(again.already, 5);
}

#[test]
fn the_store_says_when_the_2x_crossed_and_how_much_of_it_is_still_here() {
    let there = tempfile::tempdir().expect("a folder");
    let picture = there.path().join("shot.png");
    std::fs::write(&picture, b"png").expect("written");
    let former = a_former_history(there.path(), &picture);

    let here = tempfile::tempdir().expect("a folder");
    let into = Store::open(&here.path().join("history.db")).expect("opened");
    assert_eq!(
        into.came_from_the_former().expect("asked").count,
        0,
        "nothing crossed yet, so the page may still offer to bring it"
    );

    bring(&former, &into, 7_000).expect("brought");
    let came = into.came_from_the_former().expect("asked");
    assert_eq!(came.count, 5);
    assert_eq!(came.when, Some(7_000));
}

#[test]
fn what_this_machine_copied_is_not_counted_as_coming_from_the_2x() {
    let here = tempfile::tempdir().expect("a folder");
    let into = Store::open(&here.path().join("history.db")).expect("opened");
    into.insert_text("mine", "copiado hoy", 1).expect("insert");
    assert_eq!(
        into.came_from_the_former().expect("asked").count,
        0,
        "only the names the crossing writes count, or the page would say it crossed when it did not"
    );
}

#[test]
fn what_the_2x_left_behind_is_never_written_to() {
    let there = tempfile::tempdir().expect("a folder");
    let picture = there.path().join("shot.png");
    std::fs::write(&picture, b"png").expect("written");
    let former = a_former_history(there.path(), &picture);
    let before = std::fs::metadata(&former).expect("there").len();

    let here = tempfile::tempdir().expect("a folder");
    let into = Store::open(&here.path().join("history.db")).expect("opened");
    bring(&former, &into, 1).expect("brought");

    assert_eq!(
        std::fs::metadata(&former).expect("still there").len(),
        before,
        "the former history is read, never touched"
    );
    assert!(picture.exists(), "and neither are the files it points at");
}

#[test]
fn dropping_the_2x_takes_its_own_and_leaves_everything_else() {
    let dir = tempfile::tempdir().expect("a folder");
    let at = dir.path();
    std::fs::write(at.join("clipboard.db"), b"the former history").expect("written");
    std::fs::write(at.join("clipboard.db-wal"), b"its log").expect("written");
    std::fs::create_dir_all(at.join("images")).expect("made");
    std::fs::write(at.join("images/one.png"), b"a picture").expect("written");
    std::fs::create_dir_all(at.join("config")).expect("made");
    std::fs::write(at.join("config/app.json"), b"its settings").expect("written");
    std::fs::write(at.join(".initialized"), b"").expect("written");
    std::fs::write(at.join("crash.log"), b"where it fell over").expect("written");
    std::fs::write(at.join("last_cleanup.txt"), b"when it last swept").expect("written");
    std::fs::create_dir_all(at.join("logs")).expect("made");
    std::fs::write(at.join("logs/copypaste_2026-09-24.log"), b"its day").expect("written");

    std::fs::write(at.join("history.db"), b"what the 3.0 keeps").expect("written");
    std::fs::create_dir_all(at.join("blobs")).expect("made");
    std::fs::write(at.join("blobs/kept"), b"ours").expect("written");
    std::fs::write(at.join("config.toml"), b"ours too").expect("written");
    std::fs::write(at.join("logs/cp-panel.log"), b"ours as well").expect("written");

    let swept = drop_former(at).expect("swept");
    assert_eq!(
        swept.files, 8,
        "database, log, picture, settings, the flag, the crash, the sweep mark and its day"
    );
    assert!(swept.bytes > 0);

    for gone in [
        "clipboard.db",
        "clipboard.db-wal",
        "images",
        "config",
        ".initialized",
        "crash.log",
        "last_cleanup.txt",
        "logs/copypaste_2026-09-24.log",
    ] {
        assert!(
            !at.join(gone).exists(),
            "{gone} was the 2.x's and had to go"
        );
    }
    for kept in [
        "history.db",
        "blobs/kept",
        "config.toml",
        "logs/cp-panel.log",
    ] {
        assert!(at.join(kept).exists(), "{kept} is the 3.0's and stays");
    }
}

#[test]
fn the_shared_log_folder_loses_only_what_the_2x_wrote_there() {
    let dir = tempfile::tempdir().expect("a folder");
    let logs = dir.path().join("logs");
    std::fs::create_dir_all(&logs).expect("made");
    std::fs::write(logs.join("copypaste_2026-09-24.log"), b"theirs").expect("written");
    std::fs::write(logs.join("copypaste_2026-09-25.log"), b"theirs too").expect("written");
    std::fs::write(logs.join("cp-gui.log"), b"ours").expect("written");
    std::fs::write(logs.join("cp-panel.log"), b"ours too").expect("written");
    std::fs::write(dir.path().join("clipboard.db"), b"history").expect("written");

    let swept = drop_former(dir.path()).expect("swept");
    assert_eq!(swept.files, 3, "the history and their two logs");
    assert!(!logs.join("copypaste_2026-09-24.log").exists());
    assert!(!logs.join("copypaste_2026-09-25.log").exists());
    assert!(logs.join("cp-gui.log").exists(), "ours stays");
    assert!(
        logs.join("cp-panel.log").exists(),
        "and so does the panel's"
    );
    assert!(logs.exists(), "the folder is shared, so it stays");
}

#[test]
fn dropping_the_2x_never_reaches_the_files_it_only_pointed_at() {
    let dir = tempfile::tempdir().expect("a folder");
    let elsewhere = tempfile::tempdir().expect("another folder");
    let theirs = elsewhere.path().join("the report they copied.pdf");
    std::fs::write(&theirs, b"a document of their own").expect("written");
    std::fs::write(dir.path().join("clipboard.db"), b"history").expect("written");

    drop_former(dir.path()).expect("swept");
    assert!(
        theirs.exists(),
        "a file the 2.x listed is the person's, not CopyPaste's"
    );
}

#[test]
fn dropping_a_folder_that_holds_nothing_of_the_2x_does_nothing_and_says_so() {
    let dir = tempfile::tempdir().expect("a folder");
    std::fs::write(dir.path().join("history.db"), b"only the 3.0 here").expect("written");
    let swept = drop_former(dir.path()).expect("swept");
    assert_eq!(swept.files, 0);
    assert!(dir.path().join("history.db").exists());
}

#[test]
fn a_database_that_is_not_the_former_one_is_refused() {
    let dir = tempfile::tempdir().expect("a folder");
    let stranger = dir.path().join("whatever.db");
    let db = Connection::open(&stranger).expect("opened");
    db.execute_batch("CREATE TABLE something (id INTEGER);")
        .expect("made");
    drop(db);
    assert!(matches!(
        look(&stranger, 0, None),
        Err(Error::NotTheFormerOne)
    ));
}
#[test]
fn the_seconds_the_2x_counts_become_the_milliseconds_the_3_0_counts() {
    let there = tempfile::tempdir().expect("a folder");
    let picture = there.path().join("shot.png");
    std::fs::write(&picture, b"png").expect("written");
    let former = a_former_history(there.path(), &picture);

    let here = tempfile::tempdir().expect("a folder");
    let into = Store::open(&here.path().join("history.db")).expect("opened");
    bring(&former, &into, 9_999).expect("brought");

    let page = into
        .list(
            &crate::Filter {
                broken: crate::Broken::Shown,
                ..Default::default()
            },
            20,
            None,
        )
        .expect("listed");
    for row in &page.rows {
        assert!(
            row.created_at >= 1_000_000,
            "{} is the row of seconds the 2.x wrote, not milliseconds",
            row.created_at
        );
    }
}

const A_DAY: i64 = 24 * 60 * 60 * 1_000;

fn aged(former: &Path, seconds: i64) {
    let db = Connection::open(former).expect("opened");
    db.execute(
        "UPDATE clipboard_items SET created_at = ?1, modified_at = ?1",
        [seconds],
    )
    .expect("aged");
}

#[test]
fn a_history_younger_than_the_kept_time_survives_the_first_sweep() {
    let there = tempfile::tempdir().expect("a folder");
    let picture = there.path().join("shot.png");
    std::fs::write(&picture, b"png").expect("written");
    let former = a_former_history(there.path(), &picture);
    let yesterday = 1_789_950_000;
    aged(&former, yesterday);

    let here = tempfile::tempdir().expect("a folder");
    let into = Store::open(&here.path().join("history.db")).expect("opened");
    let now = in_millis(yesterday) + A_DAY;
    bring(&former, &into, now).expect("brought");
    let before = into.count().expect("counted");
    assert!(before > 0);

    let swept = into
        .sweep(
            &crate::Policy {
                keep_for: Some(30 * A_DAY),
                ..Default::default()
            },
            now,
        )
        .expect("swept");

    assert_eq!(swept.expired, 0, "nothing brought over is a month old yet");
    assert_eq!(into.count().expect("counted"), before);
}

#[test]
fn what_the_look_counts_as_beyond_the_kept_time_is_what_the_sweep_takes() {
    let there = tempfile::tempdir().expect("a folder");
    let picture = there.path().join("shot.png");
    std::fs::write(&picture, b"png").expect("written");
    let former = a_former_history(there.path(), &picture);
    let a_year_ago = 1_758_000_000;
    aged(&former, a_year_ago);
    let now = in_millis(a_year_ago) + 365 * A_DAY;

    let said = look(&former, now, Some(30 * A_DAY)).expect("looked");
    assert!(said.beyond_keep > 0, "a year old is past a month");
    assert_eq!(
        said.beyond_keep,
        said.items - said.pinned,
        "what is pinned is not swept and is not counted"
    );

    let here = tempfile::tempdir().expect("a folder");
    let into = Store::open(&here.path().join("history.db")).expect("opened");
    bring(&former, &into, now).expect("brought");
    let swept = into
        .sweep(
            &crate::Policy {
                keep_for: Some(30 * A_DAY),
                ..Default::default()
            },
            now,
        )
        .expect("swept");

    assert_eq!(
        swept.expired as i64, said.beyond_keep,
        "the warning shown beforehand is what actually happens"
    );
    assert_eq!(into.count().expect("counted"), said.pinned);
}

#[test]
fn a_row_without_a_day_of_its_own_lands_today_and_is_not_counted_as_lost() {
    let there = tempfile::tempdir().expect("a folder");
    let picture = there.path().join("shot.png");
    std::fs::write(&picture, b"png").expect("written");
    let former = a_former_history(there.path(), &picture);
    let long_ago = 1_758_000_000;
    let db = Connection::open(&former).expect("opened");
    db.execute(
        "UPDATE clipboard_items SET created_at = 0, modified_at = ?1",
        [long_ago],
    )
    .expect("aged");
    drop(db);
    let now = in_millis(long_ago) + 365 * A_DAY;

    let said = look(&former, now, Some(30 * A_DAY)).expect("looked");
    assert_eq!(
        said.beyond_keep, 0,
        "a row the migration dates today is not a row the sweep will take"
    );

    let here = tempfile::tempdir().expect("a folder");
    let into = Store::open(&here.path().join("history.db")).expect("opened");
    bring(&former, &into, now).expect("brought");
    let swept = into
        .sweep(
            &crate::Policy {
                keep_for: Some(30 * A_DAY),
                ..Default::default()
            },
            now,
        )
        .expect("swept");
    assert_eq!(swept.expired, 0);
    assert_eq!(into.count().expect("counted"), said.items);
}

#[test]
fn a_picture_the_old_database_points_outside_its_folder_is_not_read() {
    let theirs = tempfile::tempdir().expect("a folder");
    let elsewhere = tempfile::tempdir().expect("another folder");
    let key = elsewhere.path().join("id_rsa");
    let secret = b"zqxjkvbnm-opensshprivatekey";
    std::fs::write(&key, secret).expect("written");
    let former = a_former_history(theirs.path(), &key);

    let here = tempfile::tempdir().expect("a folder");
    let into = Store::open(&here.path().join("history.db")).expect("opened");
    let brought = bring(&former, &into, 10_000).expect("brought");
    drop(into);

    assert!(
        brought.without_their_picture >= 1,
        "it came over, but without a file we were never meant to read"
    );
    for at in crate::blobs::files_under(here.path()) {
        let read = std::fs::read(&at).unwrap_or_default();
        assert!(
            !read.windows(secret.len()).any(|one| one == secret),
            "{} holds what was outside the old folder",
            at.display()
        );
    }
}

#[test]
fn a_picture_bigger_than_we_would_store_today_comes_over_without_it() {
    let there = tempfile::tempdir().expect("a folder");
    let huge = there.path().join("huge.png");
    let file = std::fs::File::create(&huge).expect("made");
    file.set_len(cp_core::item::BLOB_UP_TO as u64 + 1)
        .expect("sized");
    drop(file);
    let former = a_former_history(there.path(), &huge);

    let here = tempfile::tempdir().expect("a folder");
    let into = Store::open(&here.path().join("history.db")).expect("opened");
    let brought = bring(&former, &into, 10_000).expect("brought");

    assert!(brought.without_their_picture >= 1);
    assert!(
        into.usage().expect("weighed").bytes < cp_core::item::BLOB_UP_TO as i64,
        "nothing the capture would refuse today is let in by the migration"
    );
}

#[cfg(unix)]
#[test]
fn a_link_where_the_pictures_were_is_not_followed_out_of_the_folder() {
    let theirs = tempfile::tempdir().expect("a folder");
    let mine = tempfile::tempdir().expect("my own folder");
    let keepsake = mine.path().join("taxes.pdf");
    std::fs::write(&keepsake, b"the only copy I have").expect("written");
    std::os::unix::fs::symlink(mine.path(), theirs.path().join("images")).expect("linked");

    drop_former(theirs.path()).expect("swept");

    assert!(keepsake.exists(), "somebody else's file was deleted");
    assert_eq!(
        std::fs::read(&keepsake).expect("read"),
        b"the only copy I have",
        "somebody else's file was written over"
    );
    assert!(
        !theirs.path().join("images").exists(),
        "the link itself is still ours to remove"
    );
}

#[test]
fn the_spare_copy_a_broken_restore_left_behind_goes_with_the_rest() {
    let theirs = tempfile::tempdir().expect("a folder");
    let spare = theirs.path().join(format!("{THEIR_SPARE}1700000000"));
    std::fs::create_dir_all(&spare).expect("made");
    let inside = spare.join("clipboard.db");
    std::fs::write(&inside, b"a whole second history").expect("written");

    let swept = drop_former(theirs.path()).expect("swept");

    assert!(!inside.exists());
    assert!(!spare.exists());
    assert!(swept.files >= 1);
}

#[test]
fn what_came_over_is_searchable_by_the_application_it_came_from() {
    let there = tempfile::tempdir().expect("a folder");
    let picture = there.path().join("shot.png");
    std::fs::write(&picture, b"png").expect("written");
    let former = a_former_history(there.path(), &picture);

    let here = tempfile::tempdir().expect("a folder");
    let into = Store::open(&here.path().join("history.db")).expect("opened");
    bring(&former, &into, 10_000).expect("brought");

    let found = |query: &str| {
        into.list(
            &crate::Filter {
                query: Some(query.into()),
                ..Default::default()
            },
            Store::PAGE,
            None,
        )
        .expect("queried")
        .rows
        .len()
    };
    assert_eq!(found("mail"), 1, "the application it came from");
    assert_eq!(found("receipt"), 1, "the label it was given");
}

#[test]
fn what_was_copied_again_in_the_2x_keeps_the_day_it_was_copied_again() {
    let there = tempfile::tempdir().expect("a folder");
    let picture = there.path().join("shot.png");
    std::fs::write(&picture, b"png").expect("written");
    let former = a_former_history(there.path(), &picture);
    let db = Connection::open(&former).expect("opened");
    db.execute(
        "UPDATE clipboard_items SET modified_at = 8_000 WHERE id = 'one'",
        [],
    )
    .expect("touched");
    drop(db);

    let here = tempfile::tempdir().expect("a folder");
    let into = Store::open(&here.path().join("history.db")).expect("opened");
    bring(&former, &into, 9_999).expect("brought");

    let page = into
        .list(&crate::Filter::default(), 20, None)
        .expect("listed");
    let first = page.rows.first().expect("something came over");
    assert_eq!(
        first.created_at,
        in_millis(1_000),
        "the one copied again is at the top, and it was created first"
    );
}

fn as_windows_wraps_it(markup: &str) -> String {
    let head = |at: usize| format!("Version:0.9\r\nStartHTML:{at:08}\r\n");
    let mut at = head(0).len();
    loop {
        let said = format!("{}{markup}", head(at));
        let found = said.find('<').expect("markup");
        if found == at {
            return said;
        }
        at = found;
    }
}

#[test]
fn the_windows_wrapper_is_taken_off_by_its_own_offset() {
    let wrapped = as_windows_wraps_it("<b>hi</b>");
    let kept = unwrapped(wrapped.as_bytes()).expect("the html is in there");
    assert_eq!(kept, b"<b>hi</b>", "the header goes, the markup stays");
}

#[test]
fn a_wrapper_whose_offset_lies_still_gives_back_the_markup() {
    let lying = b"Version:0.9\r\nStartHTML:00000002\r\n<p>still here</p>";
    let kept = unwrapped(lying).expect("an offset that points at nothing is not trusted");
    assert_eq!(kept, b"<p>still here</p>");

    let far = b"Version:0.9\r\nStartHTML:99999999\r\n<p>still here</p>";
    assert_eq!(
        unwrapped(far).expect("found by its tag"),
        b"<p>still here</p>"
    );

    let none = b"Version:0.9\r\nnothing that looks like markup";
    assert!(unwrapped(none).is_none(), "and nothing is invented");
}

#[test]
fn a_history_carried_from_windows_keeps_its_styles_on_a_mac() {
    let from_windows = as_windows_wraps_it("<b>the receipt</b>");
    let carried = rich_of(Some(&format!(
        r#"{{"html": "{}"}}"#,
        to_base64(from_windows.as_bytes())
    )));
    if cfg!(target_os = "windows") {
        let (id, bytes) = carried.first().expect("it crossed whole");
        assert_eq!(*id, RICH_HTML);
        assert!(
            bytes.starts_with(THE_WINDOWS_HEADER),
            "Windows wants CF_HTML"
        );
    } else {
        let (id, bytes) = carried.first().expect("it crossed unwrapped");
        assert_eq!(*id, RICH_HTML);
        assert_eq!(
            bytes, b"<b>the receipt</b>",
            "a Mac pasteboard wants the markup on its own"
        );
    }
}

fn to_base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let mut held = [0u8; 3];
        held[..chunk.len()].copy_from_slice(chunk);
        let joined = u32::from(held[0]) << 16 | u32::from(held[1]) << 8 | u32::from(held[2]);
        for at in 0..4 {
            if at <= chunk.len() {
                let six = (joined >> (18 - at * 6)) & 0x3F;
                out.push(ALPHABET[six as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[test]
fn the_styles_the_2x_kept_cross_as_a_format_and_not_as_a_note() {
    let carried = rich_of(Some(
        r#"{"html": "VmVyc2lvbjowLjkNCjxiPmhvbGE8L2I+", "width": 8}"#,
    ));
    let (id, bytes) = carried.first().expect("the html crossed");
    assert_eq!(*id, RICH_HTML);
    assert!(
        std::str::from_utf8(bytes)
            .expect("utf8")
            .contains("<b>hola</b>")
    );
    assert!(
        !meta_in(Some(r#"{"html": "VmVyc2lvbjowLjk=", "width": 8}"#))
            .iter()
            .any(|(key, _)| key == "html"),
        "what crossed as a format is not written down twice"
    );
}

#[test]
fn base64_the_2x_wrote_comes_back_byte_for_byte() {
    assert_eq!(un_base64("aG9sYQ==").expect("decoded"), b"hola");
    assert_eq!(un_base64("aG9sYQ").expect("decoded"), b"hola");
    assert_eq!(
        un_base64("aG9s\r\nYQ==").expect("decoded"),
        b"hola",
        "a line break inside it is not a byte"
    );
    assert!(un_base64("not base64 at all").is_none());
    assert!(un_base64("").is_none());
}
#[test]
fn a_picture_that_crossed_asks_for_its_thumbnail_and_its_reading() {
    let there = tempfile::tempdir().expect("a folder");
    let picture = there.path().join("shot.png");
    std::fs::write(&picture, b"png").expect("written");
    let former = a_former_history(there.path(), &picture);

    let here = tempfile::tempdir().expect("a folder");
    let into = Store::open(&here.path().join("history.db")).expect("opened");
    bring(&former, &into, 9_999).expect("brought");

    let waiting = into.take_pending("thumb", 9_999, 20).expect("asked");
    let reading = into.take_pending("ocr", 9_999, 20).expect("asked");
    assert_eq!(waiting.len(), 1, "the picture that came with its bytes");
    assert_eq!(reading.len(), 1);

    let page = into
        .list(&crate::Filter::default(), 20, None)
        .expect("listed");
    let carried = page
        .rows
        .iter()
        .find(|one| one.label.as_deref() == Some("the receipt"))
        .expect("the picture is here");
    assert!(
        waiting.contains(&carried.id),
        "the one queued is the picture itself"
    );
}

#[test]
fn a_picture_whose_file_is_gone_asks_for_nothing() {
    assert!(jobs_for(Kind::Image, false).is_empty());
    assert_eq!(jobs_for(Kind::Image, true), ["thumb", "ocr"]);
    assert!(jobs_for(Kind::Text, true).is_empty());
}

#[test]
fn the_path_the_2x_kept_a_picture_at_is_not_what_the_card_says() {
    let there = tempfile::tempdir().expect("a folder");
    let picture = there.path().join("shot.png");
    std::fs::write(&picture, b"png").expect("written");
    let former = a_former_history(there.path(), &picture);

    let here = tempfile::tempdir().expect("a folder");
    let into = Store::open(&here.path().join("history.db")).expect("opened");
    bring(&former, &into, 9_999).expect("brought");

    let page = into
        .list(&crate::Filter::default(), 20, None)
        .expect("listed");
    let carried = page
        .rows
        .iter()
        .find(|one| one.label.as_deref() == Some("the receipt"))
        .expect("the picture is here");
    assert!(
        carried.preview.is_empty(),
        "a picture that crossed as bytes shows its picture, not where it used to live: {:?}",
        carried.preview
    );
    assert!(
        into.list(
            &crate::Filter {
                query: Some("shot".to_owned()),
                ..Default::default()
            },
            20,
            None,
        )
        .expect("searched")
        .rows
        .is_empty(),
        "and the path it used to live at is not searchable either"
    );
}

#[test]
fn how_many_times_something_was_pasted_crosses_too() {
    let there = tempfile::tempdir().expect("a folder");
    let picture = there.path().join("shot.png");
    std::fs::write(&picture, b"png").expect("written");
    let former = a_former_history(there.path(), &picture);

    let here = tempfile::tempdir().expect("a folder");
    let into = Store::open(&here.path().join("history.db")).expect("opened");
    bring(&former, &into, 9_999).expect("brought");

    let page = into
        .list(&crate::Filter::default(), 20, None)
        .expect("listed");
    let counted: Vec<i64> = page
        .rows
        .iter()
        .map(|one| one.paste_count)
        .filter(|times| *times > 0)
        .collect();
    assert_eq!(
        counted,
        vec![3],
        "the 2.x had pasted one of them three times"
    );
}

#[test]
fn a_thumbnail_the_2x_left_in_its_own_folder_is_not_adopted() {
    let there = tempfile::tempdir().expect("a folder");
    let picture = there.path().join("shot.png");
    std::fs::write(&picture, b"png").expect("written");
    let former = a_former_history(there.path(), &picture);
    let thumb = there.path().join("images");
    std::fs::create_dir_all(&thumb).expect("made");
    let thumb = thumb.join("shot_thumb.png");
    std::fs::write(&thumb, b"png").expect("written");
    let db = Connection::open(&former).expect("opened");
    db.execute(
        "UPDATE clipboard_items SET thumb_path = ?1 WHERE id = 'two'",
        [thumb.to_string_lossy()],
    )
    .expect("pointed");
    drop(db);

    let here = tempfile::tempdir().expect("a folder");
    let into = Store::open(&here.path().join("history.db")).expect("opened");
    bring(&former, &into, 9_999).expect("brought");

    let page = into
        .list(&crate::Filter::default(), 20, None)
        .expect("listed");
    assert!(
        page.rows.iter().all(|one| one.thumb_path.is_none()),
        "it lives in the folder that deleting the 2.x empties, so the 3.0 draws its own"
    );
}

#[test]
fn a_picture_is_taken_only_if_it_is_a_real_file_with_bytes_inside_its_folder() {
    let there = tempfile::tempdir().expect("a folder");
    let root = std::fs::canonicalize(there.path()).expect("canonical");

    assert_eq!(picture_at(Some(&root), ""), None, "the 2.x stored no path");
    assert_eq!(picture_at(None, "shot.png"), None, "and we know no folder");

    let empty = root.join("empty.png");
    std::fs::write(&empty, b"").expect("written");
    assert_eq!(
        picture_at(Some(&root), empty.to_str().expect("utf8")),
        None,
        "a file of zero bytes carries no picture"
    );

    let folder = root.join("inside");
    std::fs::create_dir(&folder).expect("created");
    assert_eq!(
        picture_at(Some(&root), folder.to_str().expect("utf8")),
        None,
        "a folder is not a picture"
    );

    let elsewhere = tempfile::tempdir().expect("a folder");
    let stray = elsewhere.path().join("stray.png");
    std::fs::write(&stray, b"png").expect("written");
    assert_eq!(
        picture_at(Some(&root), stray.to_str().expect("utf8")),
        None,
        "and what the database points at outside its own folder is not followed"
    );

    let good = root.join("shot.png");
    std::fs::write(&good, b"png").expect("written");
    assert_eq!(
        picture_at(Some(&root), good.to_str().expect("utf8")),
        Some(std::fs::canonicalize(&good).expect("canonical"))
    );
}

#[test]
fn a_row_the_2x_left_unreadable_is_counted_and_the_rest_still_crosses() {
    let there = tempfile::tempdir().expect("a folder");
    let at = there.path().join("clipboard.db");
    let db = Connection::open(&at).expect("opened");
    db.execute_batch(FORMER).expect("made");
    db.execute(
        "INSERT INTO clipboard_items (id, content, type, created_at, modified_at)
             VALUES ('good', 'plain text', 0, 1000, 1000)",
        [],
    )
    .expect("stored");
    db.execute(
        "INSERT INTO clipboard_items (id, content, type, created_at, modified_at)
             VALUES ('bad', 'plain text', 'not a number', 2000, 2000)",
        [],
    )
    .expect("stored");
    drop(db);

    let here = tempfile::tempdir().expect("a folder");
    let into = Store::open(&here.path().join("history.db")).expect("opened");
    let brought = bring(&at, &into, 9_999).expect("brought");
    assert_eq!(brought.added, 1);
    assert_eq!(
        brought.refused, 1,
        "a row that cannot be read is counted, not swallowed, and does not stop the others"
    );
}

#[test]
fn deleting_everything_that_crossed_does_not_make_it_look_like_it_never_did() {
    let there = tempfile::tempdir().expect("a folder");
    let picture = there.path().join("shot.png");
    std::fs::write(&picture, b"png").expect("written");
    let former = a_former_history(there.path(), &picture);

    let here = tempfile::tempdir().expect("a folder");
    let into = Store::open(&here.path().join("history.db")).expect("opened");
    bring(&former, &into, 7_000).expect("brought");
    let every = crate::Filter {
        broken: crate::Broken::Shown,
        ..Default::default()
    };
    let page = into.list(&every, 10, None).expect("listed");
    assert_eq!(page.rows.len(), 5);
    for row in &page.rows {
        into.mark_deleted(row.id, 8_000).expect("deleted");
    }

    let came = into.came_from_the_former().expect("asked");
    assert_eq!(
        came.count, 5,
        "it crossed, and deleting is not undoing that"
    );
    assert_eq!(came.still, 0, "none of them are here any more");
    assert!(
        came.when.is_some(),
        "there is still a moment to show, even if no row of it is left"
    );
}
