use super::*;

#[test]
fn auto_vacuum_actually_took() {
    let db = Connection::open_in_memory().expect("opened");
    create(&db).expect("schema");
    let mode: i64 = db
        .query_row("PRAGMA auto_vacuum", [], |row| row.get(0))
        .expect("queried");
    assert_eq!(
        mode, 2,
        "INCREMENTAL is 2; if it comes out 0 the pragma arrived late and was ignored"
    );
}

#[test]
fn a_touch_that_leaves_the_words_alone_does_not_reindex_them() {
    let db = Connection::open_in_memory().expect("opened");
    create(&db).expect("schema");
    db.execute(
        "INSERT INTO items (id, uuid, preview_text, created_at, modified_at, content_hash,
                                search_text, updated_at)
             VALUES (1, 'u1', 'a copy', 10, 10, 7, 'a copy worth finding', 10)",
        [],
    )
    .expect("stored");

    let indexed = |db: &Connection| -> i64 {
        db.query_row(
            "SELECT count(*) FROM items_fts WHERE items_fts MATCH 'finding'",
            [],
            |row| row.get(0),
        )
        .expect("searched")
    };
    assert_eq!(indexed(&db), 1);

    let before = db.total_changes();
    db.execute(
        "UPDATE items SET modified_at = 20, updated_at = 20 WHERE id = 1",
        [],
    )
    .expect("touched");
    assert_eq!(
        db.total_changes() - before,
        1,
        "one row written and nothing else: a trigger that fired would have added its own"
    );
    assert_eq!(indexed(&db), 1, "the words are still where they were");

    db.execute(
        "UPDATE items SET search_text = 'something else entirely' WHERE id = 1",
        [],
    )
    .expect("rewritten");
    assert_eq!(indexed(&db), 0, "and when they change, the index follows");
    let now: i64 = db
        .query_row(
            "SELECT count(*) FROM items_fts WHERE items_fts MATCH 'entirely'",
            [],
            |row| row.get(0),
        )
        .expect("searched");
    assert_eq!(now, 1);
}

#[test]
fn a_database_carrying_the_broad_trigger_is_healed_by_its_shape_not_its_version() {
    let db = Connection::open_in_memory().expect("opened");
    create(&db).expect("schema");
    db.execute_batch(
        "DROP TRIGGER items_au;
             CREATE TRIGGER items_au AFTER UPDATE ON items BEGIN
                 INSERT INTO items_fts(items_fts, rowid, search_text, search_label, search_app, search_ocr)
                     VALUES ('delete', old.id, old.search_text, old.search_label, old.search_app, old.search_ocr);
                 INSERT INTO items_fts(rowid, search_text, search_label, search_app, search_ocr)
                     VALUES (new.id, new.search_text, new.search_label, new.search_app, new.search_ocr);
             END;
             PRAGMA user_version = 4;",
    )
    .expect("an older shape");
    assert!(search_trigger_is_broad(&db).expect("looked"));

    create(&db).expect("opened again");
    assert!(
        !search_trigger_is_broad(&db).expect("looked"),
        "the shape is what heals it, with the version already up to date"
    );
}

#[test]
fn a_database_that_never_had_the_trigger_gets_one() {
    let db = Connection::open_in_memory().expect("opened");
    create(&db).expect("schema");
    db.execute_batch("DROP TRIGGER items_au;").expect("dropped");
    assert!(!trigger_exists(&db).expect("looked"));
    create(&db).expect("opened again");
    assert!(trigger_exists(&db).expect("looked"));
}

#[test]
fn the_planner_statistics_never_learn_what_was_copied() {
    let db = Connection::open_in_memory().expect("opened");
    create(&db).expect("schema");
    for i in 0..40 {
        db.execute(
            "INSERT INTO items (uuid, kind, preview_text, app_source, search_app,
                                    created_at, modified_at, content_hash, search_text, updated_at)
                 VALUES (?1, 'text', 'x', 'APasswordManager', 'apasswordmanager',
                         ?2, ?2, ?2, 'a password worth hiding', ?2)",
            rusqlite::params![format!("u{i}"), i],
        )
        .expect("stored");
    }
    let stats: i64 = db
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE name LIKE 'sqlite_stat%'",
            [],
            |row| row.get(0),
        )
        .expect("queried");
    assert_eq!(
        stats, 0,
        "ANALYZE samples raw index keys into sqlite_stat4 — app names, meta values, search \
             terms — and nothing scrubs it: not erase, not sweep, not secure_delete"
    );
}

#[test]
fn the_pragmas_that_protect_the_data_are_on() {
    let db = Connection::open_in_memory().expect("opened");
    create(&db).expect("schema");
    let secure: i64 = db
        .query_row("PRAGMA secure_delete", [], |row| row.get(0))
        .expect("queried");
    assert_eq!(secure, 1, "a deleted password cannot remain legible");
    let foreign: i64 = db
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .expect("queried");
    assert_eq!(
        foreign, 1,
        "without this the cascade over formats does not happen"
    );
}

#[test]
fn a_fresh_database_is_stamped_with_its_version() {
    let db = Connection::open_in_memory().expect("opened");
    create(&db).expect("schema");
    assert!(
        migrate(&db).expect("migrated"),
        "no version yet: it does migrate"
    );
    let version: u32 = db
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .expect("queried");
    assert_eq!(version, SCHEMA_VERSION);
}

#[test]
fn a_database_from_the_future_is_refused_not_repaired() {
    let db = Connection::open_in_memory().expect("opened");
    create(&db).expect("schema");
    db.execute_batch("PRAGMA user_version = 99;")
        .expect("stamped");
    let refused = migrate(&db);
    assert!(
        matches!(refused, Err(crate::Error::FromTheFuture { found: 99, .. })),
        "opening and «fixing» a newer database destroys the history"
    );
}

#[test]
fn migrating_twice_changes_nothing() {
    let db = Connection::open_in_memory().expect("opened");
    create(&db).expect("schema");
    assert!(
        migrate(&db).expect("first"),
        "the first time it does migrate"
    );
    assert!(
        !migrate(&db).expect("second"),
        "already up to date: there is nothing to do"
    );
}

#[test]
fn creating_twice_is_harmless() {
    let db = Connection::open_in_memory().expect("opened");
    create(&db).expect("first");
    create(&db).expect("second");
}

fn index_sql(db: &Connection, name: &str) -> String {
    db.query_row(
        "SELECT sql FROM sqlite_master WHERE type = 'index' AND name = ?1",
        [name],
        |row| row.get(0),
    )
    .expect("the index exists")
}

#[test]
fn the_index_forgets_what_was_deleted_and_the_setting_survives_reopening() {
    let dir = tempfile::tempdir().expect("a folder");
    let path = dir.path().join("history.db");
    {
        let db = Connection::open(&path).expect("opened");
        create(&db).expect("schema");
    }
    let db = Connection::open(&path).expect("reopened");
    configure(&db).expect("pragmas set");
    let secure: i64 = db
        .query_row(
            "SELECT v FROM items_fts_config WHERE k = 'secure-delete'",
            [],
            |row| row.get(0),
        )
        .expect("the setting lives in the index configuration table");
    assert_eq!(secure, 1, "a deleted token cannot remain in an old segment");
}

#[test]
fn a_version_two_database_forgets_what_its_index_still_remembered() {
    let dir = tempfile::tempdir().expect("a folder");
    let path = dir.path().join("history.db");
    let secret = "qzvrxtoken7secret";
    {
        let db = Connection::open(&path).expect("opened");
        create(&db).expect("schema");
        db.execute_batch(
            "INSERT INTO items_fts(items_fts, rank) VALUES ('secure-delete', 0);
                 PRAGMA user_version = 2;",
        )
        .expect("a database as version 2 left it");
        db.execute(
            "INSERT INTO items (uuid, preview_text, created_at, modified_at, updated_at,
                                    content_hash, search_text)
                 VALUES ('u', ?1, 1, 1, 1, 0, ?1)",
            [secret],
        )
        .expect("inserted");
        db.execute(
            "UPDATE items SET preview_text = '', search_text = '', deleted_at = 2 WHERE uuid = 'u'",
            [],
        )
        .expect("deletes the way version 2 used to delete");
        db.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .expect("checkpointed");
    }
    let bytes = std::fs::read(&path).expect("read");
    assert!(
        bytes.windows(secret.len()).any(|w| w == secret.as_bytes()),
        "without secure-delete the index keeps the deleted token"
    );
    let db = Connection::open(&path).expect("reopened");
    create(&db).expect("created");
    assert!(migrate(&db).expect("migrated"));
    db.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .expect("checkpointed");
    drop(db);
    let bytes = std::fs::read(&path).expect("read");
    assert!(
        !bytes.windows(secret.len()).any(|w| w == secret.as_bytes()),
        "the migration to version 3 merges the segments and what was deleted disappears"
    );
}

#[test]
fn a_version_three_database_gains_the_column_for_the_text_as_read() {
    let db = Connection::open_in_memory().expect("opened");
    create(&db).expect("schema");
    db.execute_batch(
        "ALTER TABLE items DROP COLUMN ocr_text;
             PRAGMA user_version = 3;",
    )
    .expect("a database as version 3 left it");
    assert!(column_is_missing(&db, "ocr_text").expect("looked"));
    create(&db).expect("opening adds the columns its own indexes and queries name");
    assert!(
        !column_is_missing(&db, "ocr_text").expect("looked"),
        "opening cannot leave a column missing: the indexes right below name it"
    );

    assert!(migrate(&db).expect("migrated"));
    db.execute(
        "INSERT INTO items (uuid, created_at, modified_at, updated_at, content_hash, ocr_text)
             VALUES ('u', 1, 1, 1, 0, 'Raw')",
        [],
    )
    .expect("the new column accepts text");
    assert!(
        !migrate(&db).expect("migrated"),
        "and the second time there is nothing to do"
    );
}

#[test]
fn a_version_one_database_gets_its_ordering_indexes_rebuilt() {
    let db = Connection::open_in_memory().expect("opened");
    create(&db).expect("schema");
    db.execute_batch(
        "DROP INDEX items_by_recency;
             DROP INDEX items_by_kind;
             CREATE INDEX items_by_recency ON items(modified_at DESC);
             CREATE INDEX items_by_kind ON items(kind, modified_at DESC);
             PRAGMA user_version = 1;",
    )
    .expect("a database as version 1 left it");
    create(&db).expect("opening recreates it without touching what already exists");
    assert!(
        !index_sql(&db, "items_by_recency").contains("id DESC"),
        "IF NOT EXISTS does not redo an old index: that is why a migration is needed"
    );
    assert!(ordering_indexes_are_stale(&db).expect("looked"));

    assert!(migrate(&db).expect("migrated"));
    assert!(!ordering_indexes_are_stale(&db).expect("looked"));
    assert!(index_sql(&db, "items_by_recency").contains("modified_at DESC, id DESC"));
    assert!(index_sql(&db, "items_by_kind").contains("kind, modified_at DESC, id DESC"));
    let version: u32 = db
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .expect("queried");
    assert_eq!(version, SCHEMA_VERSION);
}

#[test]
fn a_database_without_the_group_column_gets_it_before_anything_indexes_it() {
    let db = Connection::open_in_memory().expect("opened");
    create(&db).expect("schema");
    db.execute_batch(
        "DROP INDEX IF EXISTS items_by_group;
         ALTER TABLE items DROP COLUMN group_key;
         PRAGMA user_version = 4;",
    )
    .expect("wound back to before the column existed");

    migrate(&db).expect("the migration adds the column before the index names it");

    let has: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('items') WHERE name = 'group_key'",
            [],
            |row| row.get(0),
        )
        .expect("asked");
    assert_eq!(has, 1, "the column is there");
    let indexed: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'items_by_group'",
            [],
            |row| row.get(0),
        )
        .expect("asked");
    assert_eq!(indexed, 1, "and so is its index");
}

#[test]
fn a_row_that_predates_the_group_column_reads_as_having_no_group() {
    let db = Connection::open_in_memory().expect("opened");
    create(&db).expect("schema");
    db.execute_batch(
        "DROP INDEX IF EXISTS items_by_group;
         ALTER TABLE items DROP COLUMN group_key;",
    )
    .expect("wound back");
    db.execute_batch(
        "INSERT INTO items (uuid, kind, preview_text, created_at, modified_at, updated_at,
                            content_hash)
         VALUES ('antes', 'link', 'https://ejemplo.test/x', 1, 1, 1, 7);",
    )
    .expect("a row from before");
    db.execute_batch("PRAGMA user_version = 4;")
        .expect("stamped");

    migrate(&db).expect("migrated");

    let group: String = db
        .query_row(
            "SELECT group_key FROM items WHERE uuid = 'antes'",
            [],
            |row| row.get(0),
        )
        .expect("read");
    assert_eq!(group, "", "empty means «nobody has grouped it yet»");
}

const AS_IT_WAS_IN_5: &str = "
    DROP INDEX IF EXISTS formats_by_digest;
    DROP INDEX IF EXISTS formats_inline_size;
    CREATE TABLE item_formats_then (
        item_id     INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
        format      TEXT    NOT NULL,
        size_bytes  INTEGER,
        inline_data BLOB,
        blob_path   TEXT,
        PRIMARY KEY (item_id, format)
    );
    INSERT INTO item_formats_then (item_id, format, size_bytes, inline_data, blob_path)
        SELECT item_id, format, size_bytes, inline_data, digest FROM item_formats;
    DROP TABLE item_formats;
    ALTER TABLE item_formats_then RENAME TO item_formats;
    PRAGMA user_version = 5;
";

fn with_one_item(db: &Connection) {
    db.execute(
        "INSERT INTO items (id, uuid, preview_text, created_at, modified_at, content_hash,
                            search_text, updated_at)
             VALUES (1, 'u1', 'a copy', 10, 10, 7, 'a copy worth finding', 10)",
        [],
    )
    .expect("an item to hang formats on");
}

fn digest_of(db: &Connection, format: &str) -> Option<String> {
    db.query_row(
        "SELECT digest FROM item_formats WHERE item_id = 1 AND format = ?1",
        [format],
        |row| row.get(0),
    )
    .expect("the row is there")
}

#[test]
fn a_stored_path_that_was_never_a_digest_does_not_survive_the_migration() {
    let db = Connection::open_in_memory().expect("opened");
    create(&db).expect("schema");
    with_one_item(&db);
    db.execute_batch(AS_IT_WAS_IN_5)
        .expect("wound back to when the column promised a path");

    let good = "a".repeat(64);
    db.execute(
        "INSERT INTO item_formats (item_id, format, blob_path) VALUES (1, 'good', ?1)",
        [&good],
    )
    .expect("a real digest");
    for (format, said) in [
        ("short", "a"),
        ("letters", "zz"),
        ("accented", "añ"),
        ("walks", "../../x"),
    ] {
        db.execute(
            "INSERT INTO item_formats (item_id, format, blob_path) VALUES (1, ?1, ?2)",
            [format, said],
        )
        .expect("the old column took anything at all");
    }

    migrate(&db).expect("the migration cleans up before it constrains");

    assert_eq!(digest_of(&db, "good").as_deref(), Some(good.as_str()));
    for format in ["short", "letters", "accented", "walks"] {
        assert_eq!(
            digest_of(&db, format),
            None,
            "«{format}» kept a value no blob store could ever answer for"
        );
    }
    let left: i64 = db
        .query_row("SELECT COUNT(*) FROM item_formats", [], |row| row.get(0))
        .expect("counted");
    assert_eq!(
        left, 5,
        "the rows stay; only what they pointed at is let go"
    );
}

#[test]
fn after_the_migration_the_column_is_named_and_shaped_like_a_digest() {
    let db = Connection::open_in_memory().expect("opened");
    create(&db).expect("schema");
    with_one_item(&db);
    db.execute_batch(AS_IT_WAS_IN_5).expect("wound back");

    migrate(&db).expect("migrated");

    let named: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('item_formats') WHERE name = 'digest'",
            [],
            |row| row.get(0),
        )
        .expect("queried");
    assert_eq!(named, 1, "the column no longer promises a path");
    let promised: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('item_formats') WHERE name = 'blob_path'",
            [],
            |row| row.get(0),
        )
        .expect("queried");
    assert_eq!(promised, 0);

    assert!(
        db.execute(
            "INSERT INTO item_formats (item_id, format, digest) VALUES (1, 'bad', 'añ')",
            [],
        )
        .is_err(),
        "the schema itself refuses what is not a digest now"
    );
    db.execute(
        "INSERT INTO item_formats (item_id, format, digest) VALUES (1, 'fine', ?1)",
        [&"f".repeat(64)],
    )
    .expect("and still takes a real one");
}

#[test]
fn the_indexes_on_formats_are_there_after_a_migration_and_after_a_fresh_start() {
    for wound_back in [false, true] {
        let db = Connection::open_in_memory().expect("opened");
        create(&db).expect("schema");
        with_one_item(&db);
        if wound_back {
            db.execute_batch(AS_IT_WAS_IN_5).expect("wound back");
        }
        migrate(&db).expect("migrated");
        for index in ["formats_by_digest", "formats_inline_size"] {
            let found: i64 = db
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?1",
                    [index],
                    |row| row.get(0),
                )
                .expect("queried");
            assert_eq!(found, 1, "{index}, wound back: {wound_back}");
        }
    }
}

#[test]
fn a_history_from_before_the_column_keeps_the_day_the_2x_crossed() {
    let db = Connection::open_in_memory().expect("memory");
    create(&db).expect("schema");
    db.execute_batch("ALTER TABLE items DROP COLUMN came_at;")
        .expect("as it was before");
    db.execute(
        "INSERT INTO items (uuid, created_at, modified_at, updated_at, content_hash)
         VALUES ('2x-abc', 100, 100, 4_000, 7)",
        [],
    )
    .expect("a row that crossed");
    db.execute(
        "INSERT INTO items (uuid, created_at, modified_at, updated_at, content_hash)
         VALUES ('mine', 200, 200, 5_000, 8)",
        [],
    )
    .expect("a row of this machine");
    db.execute_batch("PRAGMA user_version = 6;").expect("older");

    migrate(&db).expect("migrated");

    let came: Option<i64> = db
        .query_row(
            "SELECT came_at FROM items WHERE uuid = '2x-abc'",
            [],
            |row| row.get(0),
        )
        .expect("asked");
    assert_eq!(
        came,
        Some(4_000),
        "what already crossed keeps the best moment we have for it instead of looking like it \
         never did"
    );
    let mine: Option<i64> = db
        .query_row("SELECT came_at FROM items WHERE uuid = 'mine'", [], |row| {
            row.get(0)
        })
        .expect("asked");
    assert_eq!(mine, None, "what was copied here never crossed anything");
}
