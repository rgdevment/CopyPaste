use rusqlite::{Connection, Result};

pub const SCHEMA_VERSION: u32 = 4;

pub fn migrate(db: &Connection) -> crate::Result<bool> {
    let found: u32 = db.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if found > SCHEMA_VERSION {
        return Err(crate::Error::FromTheFuture {
            found,
            supported: SCHEMA_VERSION,
        });
    }
    if found == SCHEMA_VERSION {
        return Ok(false);
    }
    db.execute_batch("BEGIN IMMEDIATE;")?;
    if ordering_indexes_are_stale(db)? {
        db.execute_batch(&format!(
            "DROP INDEX IF EXISTS items_by_recency;
             DROP INDEX IF EXISTS items_by_kind;
             {ORDERING_INDEXES}"
        ))?;
    }
    if found < 3 {
        db.execute_batch("INSERT INTO items_fts(items_fts) VALUES ('optimize');")?;
    }
    if ocr_text_is_missing(db)? {
        db.execute_batch("ALTER TABLE items ADD COLUMN ocr_text TEXT;")?;
    }
    db.execute_batch(&format!("PRAGMA user_version = {SCHEMA_VERSION}; COMMIT;"))?;
    Ok(true)
}

fn ordering_indexes_are_stale(db: &Connection) -> Result<bool> {
    let mut stmt = db.prepare(
        "SELECT sql FROM sqlite_master
         WHERE type = 'index' AND name IN ('items_by_recency', 'items_by_kind')",
    )?;
    let definitions = stmt.query_map([], |row| row.get::<_, String>(0))?;
    for definition in definitions {
        if !definition?.contains("id DESC") {
            return Ok(true);
        }
    }
    Ok(false)
}

fn ocr_text_is_missing(db: &Connection) -> Result<bool> {
    let mut stmt = db.prepare("SELECT name FROM pragma_table_info('items')")?;
    let columns = stmt.query_map([], |row| row.get::<_, String>(0))?;
    for column in columns {
        if column? == "ocr_text" {
            return Ok(false);
        }
    }
    Ok(true)
}

const ORDERING_INDEXES: &str = "
        CREATE INDEX IF NOT EXISTS items_by_recency ON items(modified_at DESC, id DESC);
        CREATE INDEX IF NOT EXISTS items_by_kind ON items(kind, modified_at DESC, id DESC);
";

pub fn configure(db: &Connection) -> Result<()> {
    db.execute_batch(
        r#"
        PRAGMA auto_vacuum = INCREMENTAL;
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA cache_size = -2000;
        PRAGMA foreign_keys = ON;
        PRAGMA secure_delete = ON;
        "#,
    )
}

pub fn create(db: &Connection) -> Result<()> {
    configure(db)?;
    db.execute_batch(TABLES)?;
    db.execute_batch(ORDERING_INDEXES)?;
    Ok(())
}

const TABLES: &str = r#"
        CREATE TABLE IF NOT EXISTS items (
            id                 INTEGER PRIMARY KEY,
            uuid               TEXT    NOT NULL UNIQUE,
            kind               TEXT,
            preview_text       TEXT    NOT NULL DEFAULT '',
            app_source         TEXT,
            label              TEXT,
            card_color         INTEGER NOT NULL DEFAULT 0,
            thumb_path         TEXT,
            created_at         INTEGER NOT NULL,
            modified_at        INTEGER NOT NULL,
            last_used_at       INTEGER,
            source_modified_at INTEGER,
            broken_since       INTEGER,
            paste_count        INTEGER NOT NULL DEFAULT 0,
            pinned             INTEGER NOT NULL DEFAULT 0,
            content_hash       INTEGER NOT NULL,
            search_text        TEXT    NOT NULL DEFAULT '',
            search_label       TEXT    NOT NULL DEFAULT '',
            search_app         TEXT    NOT NULL DEFAULT '',
            search_ocr         TEXT    NOT NULL DEFAULT '',
            ocr_text           TEXT,
            updated_at         INTEGER NOT NULL,
            deleted_at         INTEGER
        );

        CREATE INDEX IF NOT EXISTS items_by_creation ON items(created_at);
        CREATE INDEX IF NOT EXISTS items_by_hash ON items(content_hash);
        CREATE INDEX IF NOT EXISTS items_by_color ON items(card_color);
        CREATE INDEX IF NOT EXISTS items_pinned ON items(pinned) WHERE pinned = 1;
        CREATE INDEX IF NOT EXISTS items_broken ON items(broken_since)
            WHERE broken_since IS NOT NULL;
        CREATE INDEX IF NOT EXISTS items_by_version ON items(updated_at);
        CREATE INDEX IF NOT EXISTS items_deleted ON items(deleted_at)
            WHERE deleted_at IS NOT NULL;
        CREATE INDEX IF NOT EXISTS items_by_app ON items(search_app);
        CREATE INDEX IF NOT EXISTS items_live_by_kind ON items(kind)
            WHERE deleted_at IS NULL AND broken_since IS NULL;
        CREATE INDEX IF NOT EXISTS items_live_by_app ON items(search_app, app_source)
            WHERE deleted_at IS NULL AND app_source IS NOT NULL;
        CREATE INDEX IF NOT EXISTS items_by_pastes ON items(paste_count DESC, id DESC);
        CREATE INDEX IF NOT EXISTS items_by_use
            ON items(COALESCE(last_used_at, -1) DESC, id DESC);

        CREATE TABLE IF NOT EXISTS item_formats (
            item_id     INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
            format      TEXT    NOT NULL,
            size_bytes  INTEGER,
            inline_data BLOB,
            blob_path   TEXT,
            PRIMARY KEY (item_id, format)
        );

        -- What is derived from an item and is not its content: dimensions,
        -- duration, size, artist. A table rather than a JSON column so it
        -- can be filtered and indexed by key without pulling in the JSON module.
        CREATE INDEX IF NOT EXISTS formats_by_blob ON item_formats(blob_path)
            WHERE blob_path IS NOT NULL;
        CREATE INDEX IF NOT EXISTS formats_inline_size ON item_formats(size_bytes)
            WHERE inline_data IS NOT NULL;

        CREATE TABLE IF NOT EXISTS item_meta (
            item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
            key     TEXT    NOT NULL,
            value   TEXT    NOT NULL,
            PRIMARY KEY (item_id, key)
        );

        CREATE INDEX IF NOT EXISTS meta_by_key ON item_meta(key, value);

        -- Enrichment goes in a queue: recognising text, generating a
        -- thumbnail or reading a video's duration costs too much for the
        -- capture path. With its state, so what always fails is not retried in a loop.
        CREATE TABLE IF NOT EXISTS pending_work (
            item_id     INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
            job         TEXT    NOT NULL,
            attempts    INTEGER NOT NULL DEFAULT 0,
            last_error  TEXT,
            not_before  INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (item_id, job)
        );

        CREATE INDEX IF NOT EXISTS work_ready ON pending_work(job, not_before);

        CREATE VIRTUAL TABLE IF NOT EXISTS items_fts USING fts5(
            search_text,
            search_label,
            search_app,
            search_ocr,
            content = 'items',
            content_rowid = 'id',
            tokenize = 'unicode61 remove_diacritics 2'
        );

        INSERT INTO items_fts(items_fts, rank) VALUES ('secure-delete', 1);

        CREATE TRIGGER IF NOT EXISTS items_ai AFTER INSERT ON items BEGIN
            INSERT INTO items_fts(rowid, search_text, search_label, search_app, search_ocr)
                VALUES (new.id, new.search_text, new.search_label, new.search_app, new.search_ocr);
        END;
        CREATE TRIGGER IF NOT EXISTS items_ad AFTER DELETE ON items BEGIN
            INSERT INTO items_fts(items_fts, rowid, search_text, search_label, search_app, search_ocr)
                VALUES ('delete', old.id, old.search_text, old.search_label, old.search_app, old.search_ocr);
        END;
        CREATE TRIGGER IF NOT EXISTS items_au AFTER UPDATE ON items BEGIN
            INSERT INTO items_fts(items_fts, rowid, search_text, search_label, search_app, search_ocr)
                VALUES ('delete', old.id, old.search_text, old.search_label, old.search_app, old.search_ocr);
            INSERT INTO items_fts(rowid, search_text, search_label, search_app, search_ocr)
                VALUES (new.id, new.search_text, new.search_label, new.search_app, new.search_ocr);
        END;
        "#;

#[cfg(test)]
mod tests {
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
        create(&db).expect("opening does not add columns to a table that already exists");
        assert!(ocr_text_is_missing(&db).expect("looked"));

        assert!(migrate(&db).expect("migrated"));
        assert!(!ocr_text_is_missing(&db).expect("looked"));
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
}
