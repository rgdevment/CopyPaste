use rusqlite::{Connection, Result};

pub const SCHEMA_VERSION: u32 = 1;

/// Lleva la base a la versión que esta copia entiende.
///
/// `user_version` es un entero que SQLite guarda en la cabecera del archivo y
/// que no cuesta nada leer. La 2.x llegó a la versión 4 de su esquema con
/// cuatro migraciones, así que esto va a hacer falta: montarlo ahora, cuando
/// no hay nada que migrar, es gratis.
///
/// Devuelve si tuvo que migrar algo, para quien quiera distinguir una base
/// que ya estaba al día de una recién puesta al día.
pub fn migrate(db: &Connection) -> crate::Result<bool> {
    let found: u32 = db.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if found > SCHEMA_VERSION {
        // Una base escrita por una versión más nueva no se toca: abrirla y
        // «arreglarla» es la forma más rápida de destrozar el historial de
        // alguien que alterna dos instalaciones.
        return Err(crate::Error::FromTheFuture {
            found,
            supported: SCHEMA_VERSION,
        });
    }
    if found < SCHEMA_VERSION {
        // Aquí irán los pasos, uno por versión. Hoy no hay ninguno: la
        // primera versión la crea `create`.
        db.execute_batch(&format!("PRAGMA user_version = {SCHEMA_VERSION};"))?;
        return Ok(true);
    }
    Ok(false)
}

/// Ajustes de la conexión, en el orden en que hay que darlos.
///
/// `auto_vacuum` es el que tiene trampa: **SQLite lo ignora en silencio si la
/// base ya tiene tablas**, y cambiarlo después exige un `VACUUM` completo. La
/// 2.x lo aprendió en producción, donde el pragma nunca llegó a aplicarse y
/// todos sus `incremental_vacuum` fueron una operación vacía durante meses.
/// Por eso va antes que nada.
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

/// El almacén guarda **el conjunto** de formatos que la fuente ofreció, no
/// uno elegido. `kind` es una clasificación sobre ese conjunto.
pub fn create(db: &Connection) -> Result<()> {
    configure(db)?;
    db.execute_batch(
        r#"
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
            updated_at         INTEGER NOT NULL,
            deleted_at         INTEGER
        );

        CREATE INDEX IF NOT EXISTS items_by_recency ON items(modified_at DESC);
        CREATE INDEX IF NOT EXISTS items_by_creation ON items(created_at);
        CREATE INDEX IF NOT EXISTS items_by_hash ON items(content_hash);
        CREATE INDEX IF NOT EXISTS items_by_kind ON items(kind, modified_at DESC);
        CREATE INDEX IF NOT EXISTS items_by_color ON items(card_color);
        CREATE INDEX IF NOT EXISTS items_pinned ON items(pinned) WHERE pinned = 1;
        CREATE INDEX IF NOT EXISTS items_broken ON items(broken_since)
            WHERE broken_since IS NOT NULL;
        CREATE INDEX IF NOT EXISTS items_by_version ON items(updated_at);
        CREATE INDEX IF NOT EXISTS items_deleted ON items(deleted_at)
            WHERE deleted_at IS NOT NULL;

        CREATE TABLE IF NOT EXISTS item_formats (
            item_id     INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
            format      TEXT    NOT NULL,
            size_bytes  INTEGER,
            inline_data BLOB,
            blob_path   TEXT,
            PRIMARY KEY (item_id, format)
        );

        -- Lo que se deriva de un ítem y no es su contenido: dimensiones,
        -- duración, tamaño, artista. Tabla y no columna JSON porque así se
        -- puede filtrar e indexar por clave sin traer el módulo JSON.
        CREATE TABLE IF NOT EXISTS item_meta (
            item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
            key     TEXT    NOT NULL,
            value   TEXT    NOT NULL,
            PRIMARY KEY (item_id, key)
        );

        CREATE INDEX IF NOT EXISTS meta_by_key ON item_meta(key, value);

        -- El enriquecimiento va en cola: reconocer texto, generar una
        -- miniatura o leer la duración de un vídeo cuestan demasiado para el
        -- camino de captura. Con su estado, para no reintentar en bucle lo
        -- que siempre falla.
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
        "#,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_vacuum_actually_took() {
        let db = Connection::open_in_memory().expect("abre");
        create(&db).expect("esquema");
        let mode: i64 = db
            .query_row("PRAGMA auto_vacuum", [], |row| row.get(0))
            .expect("consulta");
        assert_eq!(
            mode, 2,
            "INCREMENTAL es 2; si sale 0 el pragma llegó tarde y se ignoró"
        );
    }

    #[test]
    fn the_pragmas_that_protect_the_data_are_on() {
        let db = Connection::open_in_memory().expect("abre");
        create(&db).expect("esquema");
        let secure: i64 = db
            .query_row("PRAGMA secure_delete", [], |row| row.get(0))
            .expect("consulta");
        assert_eq!(secure, 1, "una contraseña borrada no puede quedar legible");
        let foreign: i64 = db
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("consulta");
        assert_eq!(foreign, 1, "sin esto la cascada de formatos no ocurre");
    }

    #[test]
    fn a_fresh_database_is_stamped_with_its_version() {
        let db = Connection::open_in_memory().expect("abre");
        create(&db).expect("esquema");
        assert!(migrate(&db).expect("migra"), "no había versión: sí migra");
        let version: u32 = db
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("consulta");
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn a_database_from_the_future_is_refused_not_repaired() {
        let db = Connection::open_in_memory().expect("abre");
        create(&db).expect("esquema");
        db.execute_batch("PRAGMA user_version = 99;")
            .expect("sella");
        let refused = migrate(&db);
        assert!(
            matches!(refused, Err(crate::Error::FromTheFuture { found: 99, .. })),
            "abrir y «arreglar» una base más nueva destroza el historial"
        );
    }

    #[test]
    fn migrating_twice_changes_nothing() {
        let db = Connection::open_in_memory().expect("abre");
        create(&db).expect("esquema");
        assert!(migrate(&db).expect("primera"), "la primera vez sí migra");
        assert!(
            !migrate(&db).expect("segunda"),
            "ya estaba al día: no hay nada que hacer"
        );
    }

    #[test]
    fn creating_twice_is_harmless() {
        let db = Connection::open_in_memory().expect("abre");
        create(&db).expect("primera");
        create(&db).expect("segunda");
    }
}
