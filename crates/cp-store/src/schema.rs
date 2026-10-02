use rusqlite::{Connection, Result};

pub const SCHEMA_VERSION: u32 = 7;

const FORMAT_INDEXES: &str = "
        CREATE INDEX IF NOT EXISTS formats_by_digest ON item_formats(digest)
            WHERE digest IS NOT NULL;
        CREATE INDEX IF NOT EXISTS formats_inline_size ON item_formats(size_bytes)
            WHERE inline_data IS NOT NULL;
";

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
    added_columns(db)?;
    if ordering_indexes_are_stale(db)? {
        db.execute_batch(
            "DROP INDEX IF EXISTS items_by_recency;
             DROP INDEX IF EXISTS items_by_kind;",
        )?;
    }
    db.execute_batch(ORDERING_INDEXES)?;
    if found < 3 {
        db.execute_batch("INSERT INTO items_fts(items_fts) VALUES ('optimize');")?;
    }
    if found < 5 {
        db.execute_batch(&format!(
            "DROP TRIGGER IF EXISTS items_au; {SEARCH_TRIGGER}"
        ))?;
    }
    if holds_a_promised_path(db)? {
        db.execute_batch(DIGEST_INSTEAD_OF_A_PATH)?;
    }
    db.execute_batch(FORMAT_INDEXES)?;
    db.execute_batch(&format!("PRAGMA user_version = {SCHEMA_VERSION}; COMMIT;"))?;
    Ok(true)
}

const SEARCH_TRIGGER: &str = "
        CREATE TRIGGER items_au
        AFTER UPDATE OF search_text, search_label, search_app, search_ocr ON items BEGIN
            INSERT INTO items_fts(items_fts, rowid, search_text, search_label, search_app, search_ocr)
                VALUES ('delete', old.id, old.search_text, old.search_label, old.search_app, old.search_ocr);
            INSERT INTO items_fts(rowid, search_text, search_label, search_app, search_ocr)
                VALUES (new.id, new.search_text, new.search_label, new.search_app, new.search_ocr);
        END;
";

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

fn search_trigger_is_broad(db: &Connection) -> Result<bool> {
    let said: Option<String> = db
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'trigger' AND name = 'items_au'",
            [],
            |row| row.get(0),
        )
        .ok();
    Ok(said.is_some_and(|one| !one.contains("UPDATE OF")))
}

fn holds_a_promised_path(db: &Connection) -> Result<bool> {
    let mut stmt = db.prepare("SELECT name FROM pragma_table_info('item_formats')")?;
    let mut columns = stmt.query_map([], |row| row.get::<_, String>(0))?;
    Ok(columns.any(|column| column.is_ok_and(|name| name == "blob_path")))
}

const DIGEST_INSTEAD_OF_A_PATH: &str = "
        UPDATE item_formats SET blob_path = NULL
         WHERE blob_path IS NOT NULL
           AND (length(blob_path) <> 64 OR blob_path GLOB '*[^0-9a-f]*');

        DROP INDEX IF EXISTS formats_by_blob;
        DROP INDEX IF EXISTS formats_inline_size;

        CREATE TABLE item_formats_kept (
            item_id     INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
            format      TEXT    NOT NULL,
            size_bytes  INTEGER,
            inline_data BLOB,
            digest      TEXT    CHECK (digest IS NULL
                OR (length(digest) = 64 AND digest NOT GLOB '*[^0-9a-f]*')),
            PRIMARY KEY (item_id, format)
        );

        INSERT INTO item_formats_kept (item_id, format, size_bytes, inline_data, digest)
            SELECT item_id, format, size_bytes, inline_data, blob_path FROM item_formats;

        DROP TABLE item_formats;
        ALTER TABLE item_formats_kept RENAME TO item_formats;
";

fn added_columns(db: &Connection) -> Result<()> {
    if column_is_missing(db, "ocr_text")? {
        db.execute_batch("ALTER TABLE items ADD COLUMN ocr_text TEXT;")?;
    }
    if column_is_missing(db, "group_key")? {
        db.execute_batch("ALTER TABLE items ADD COLUMN group_key TEXT NOT NULL DEFAULT '';")?;
    }
    if column_is_missing(db, "came_at")? {
        db.execute_batch("ALTER TABLE items ADD COLUMN came_at INTEGER;")?;
        db.execute_batch(
            "UPDATE items SET came_at = updated_at
             WHERE came_at IS NULL AND uuid >= '2x-' AND uuid < '2x.';",
        )?;
    }
    Ok(())
}

fn column_is_missing(db: &Connection, wanted: &str) -> Result<bool> {
    let mut stmt = db.prepare("SELECT name FROM pragma_table_info('items')")?;
    let columns = stmt.query_map([], |row| row.get::<_, String>(0))?;
    for column in columns {
        if column? == wanted {
            return Ok(false);
        }
    }
    Ok(true)
}

const ORDERING_INDEXES: &str = "
        CREATE INDEX IF NOT EXISTS items_by_recency ON items(modified_at DESC, id DESC);
        CREATE INDEX IF NOT EXISTS items_by_touch
            ON items(MAX(modified_at, COALESCE(last_used_at, 0)) DESC, id DESC);
        CREATE INDEX IF NOT EXISTS items_by_kind ON items(kind, modified_at DESC, id DESC);
        CREATE INDEX IF NOT EXISTS items_by_group ON items(group_key DESC, id DESC);
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
    added_columns(db)?;
    db.execute_batch(ORDERING_INDEXES)?;
    if search_trigger_is_broad(db)? {
        db.execute_batch(&format!(
            "DROP TRIGGER IF EXISTS items_au;
             {SEARCH_TRIGGER}
             INSERT INTO items_fts(items_fts) VALUES ('rebuild');"
        ))?;
    } else if !trigger_exists(db)? {
        db.execute_batch(SEARCH_TRIGGER)?;
    }
    Ok(())
}

fn trigger_exists(db: &Connection) -> Result<bool> {
    let found: i64 = db.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type = 'trigger' AND name = 'items_au'",
        [],
        |row| row.get(0),
    )?;
    Ok(found > 0)
}

const TABLES: &str = r#"
        CREATE TABLE IF NOT EXISTS items (
            id                 INTEGER PRIMARY KEY,
            uuid               TEXT    NOT NULL UNIQUE,
            kind               TEXT,
            preview_text       TEXT    NOT NULL DEFAULT '',
            app_source         TEXT,
            group_key          TEXT    NOT NULL DEFAULT '',
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
            came_at            INTEGER,
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
            digest      TEXT    CHECK (digest IS NULL
                OR (length(digest) = 64 AND digest NOT GLOB '*[^0-9a-f]*')),
            PRIMARY KEY (item_id, format)
        );

        -- What is derived from an item and is not its content: dimensions,
        -- duration, size, artist. A table rather than a JSON column so it
        -- can be filtered and indexed by key without pulling in the JSON module.

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
        "#;

#[cfg(test)]
#[path = "schema_test.rs"]
mod tests;
