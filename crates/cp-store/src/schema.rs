use rusqlite::{Connection, Result};

pub const SCHEMA_VERSION: u32 = 1;

/// El almacén guarda **el conjunto** de formatos que la fuente ofreció, no
/// uno elegido. `kind` es una clasificación sobre ese conjunto.
pub fn create(db: &Connection) -> Result<()> {
    db.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA secure_delete = ON;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS items (
            id                 INTEGER PRIMARY KEY,
            uuid               TEXT    NOT NULL UNIQUE,
            kind               TEXT,
            preview_text       TEXT,
            created_at         INTEGER NOT NULL,
            last_used_at       INTEGER,
            paste_count        INTEGER NOT NULL DEFAULT 0,
            pinned             INTEGER NOT NULL DEFAULT 0,
            label              TEXT,
            color              TEXT,
            app_source         TEXT,
            content_hash       INTEGER NOT NULL,
            search_text        TEXT NOT NULL,
            broken_since       INTEGER,
            source_modified_at INTEGER
        );

        CREATE INDEX IF NOT EXISTS items_by_recency  ON items(created_at DESC);
        CREATE INDEX IF NOT EXISTS items_by_hash     ON items(content_hash);
        CREATE INDEX IF NOT EXISTS items_broken      ON items(broken_since)
            WHERE broken_since IS NOT NULL;

        CREATE TABLE IF NOT EXISTS item_formats (
            item_id     INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
            format      TEXT    NOT NULL,
            size_bytes  INTEGER,
            inline_data BLOB,
            blob_path   TEXT,
            PRIMARY KEY (item_id, format)
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS items_fts USING fts5(
            search_text,
            content = 'items',
            content_rowid = 'id',
            tokenize = 'unicode61 remove_diacritics 2'
        );

        CREATE TRIGGER IF NOT EXISTS items_ai AFTER INSERT ON items BEGIN
            INSERT INTO items_fts(rowid, search_text) VALUES (new.id, new.search_text);
        END;
        CREATE TRIGGER IF NOT EXISTS items_ad AFTER DELETE ON items BEGIN
            INSERT INTO items_fts(items_fts, rowid, search_text)
                VALUES ('delete', old.id, old.search_text);
        END;
        CREATE TRIGGER IF NOT EXISTS items_au AFTER UPDATE ON items BEGIN
            INSERT INTO items_fts(items_fts, rowid, search_text)
                VALUES ('delete', old.id, old.search_text);
            INSERT INTO items_fts(rowid, search_text) VALUES (new.id, new.search_text);
        END;
        "#,
    )
}
