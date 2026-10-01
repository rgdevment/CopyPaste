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
