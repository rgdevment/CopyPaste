use crate::{Error, Result};
use cp_core::item::{Item, Payload};
use cp_core::search::fold;
use rusqlite::{Connection, OptionalExtension, params};

/// Lo que el usuario escribe no es sintaxis FTS5, y tratarlo como si lo fuera
/// rompe la búsqueda con una comilla, un asterisco o un guion delante —y con
/// el buscador vacío, que ocurre cada vez que se borra lo escrito—. Cada
/// palabra se envuelve entre comillas para que FTS la lea como texto literal,
/// y el prefijo se pide fuera de ellas.
fn fts_expression(folded: &str) -> Option<String> {
    let terms: Vec<String> = folded
        .split_whitespace()
        .filter(|word| word.chars().any(char::is_alphanumeric))
        .map(|word| format!("\"{}\"*", word.replace('"', "\"\"")))
        .collect();
    (!terms.is_empty()).then(|| terms.join(" "))
}

/// Lo que el panel enseña de cada ítem en la lista.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    pub id: i64,
    pub modified_at: i64,
    pub kind: Option<String>,
    pub preview: String,
    pub pinned: bool,
}

/// Lo que el usuario tiene puesto en el panel.
#[derive(Debug, Clone, Default)]
pub struct Filter {
    pub query: Option<String>,
    pub kinds: Vec<cp_core::kind::Kind>,
    pub colors: Vec<i64>,
    pub pinned_only: bool,
}

pub struct Store {
    db: Connection,
    blobs: Option<crate::Blobs>,
}

impl Store {
    /// Solo para diagnóstico: mirar planes de consulta desde un ejemplo.
    pub fn raw(&self) -> &Connection {
        &self.db
    }

    pub fn in_memory() -> Result<Self> {
        let db = Connection::open_in_memory()?;
        crate::schema::create(&db)?;
        Ok(Self { db, blobs: None })
    }

    /// Abre —o crea— la base en disco.
    ///
    /// El archivo se crea con permisos `0600` y su carpeta con `0700`: un
    /// historial de portapapeles es de los archivos más sensibles de una
    /// cuenta, y en un equipo compartido el resto de usuarios no tiene por
    /// qué poder leerlo.
    pub fn open(path: &std::path::Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(Error::Io)?;
            restrict(parent, 0o700)?;
        }
        let db = Connection::open(path)?;
        crate::schema::create(&db)?;
        crate::schema::migrate(&db)?;
        restrict(path, 0o600)?;
        // Las imágenes que CopyPaste captura viven junto a la base, en su
        // propia carpeta. Lo que el usuario copió del disco se queda donde
        // estaba: solo se guarda la ruta.
        let blobs = path
            .parent()
            .map(|parent| crate::Blobs::at(&parent.join("blobs")))
            .transpose()?;
        Ok(Self { db, blobs })
    }

    /// Vuelca el WAL al archivo principal.
    ///
    /// Sin esto, lo recién escrito vive en el `-wal` y una copia del archivo
    /// principal sale incompleta: es la trampa que la 2.x documenta en su
    /// servicio de copia de seguridad.
    pub fn checkpoint(&self) -> Result<()> {
        self.db.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        Ok(())
    }

    /// Recupera espacio de las páginas liberadas, poco a poco.
    pub fn vacuum_step(&self, pages: u32) -> Result<()> {
        self.db
            .execute_batch(&format!("PRAGMA incremental_vacuum({pages});"))?;
        Ok(())
    }

    /// El texto se normaliza **al escribir**, con la misma función que
    /// normaliza el término al buscar. Ese es el invariante que hoy falta: la
    /// 2.x normaliza solo el término, así que `Straße` no se encuentra ni
    /// escribiendo `strasse` ni escribiendo `Straße`.
    pub fn insert_text(&self, uuid: &str, text: &str, created_at: i64) -> Result<i64> {
        let hash = Item::plain(text).fingerprint() as i64;
        self.db.execute(
            "INSERT INTO items (uuid, kind, preview_text, created_at, modified_at, updated_at,
                                content_hash, search_text)
             VALUES (?1, 'text', ?2, ?3, ?3, ?3, ?4, ?5)",
            params![uuid, text, created_at, hash, fold(text)],
        )?;
        Ok(self.db.last_insert_rowid())
    }

    /// Volver a copiar algo que ya estaba lo sube en la lista, no lo duplica.
    ///
    /// **No toca el contador de pegados**, que es lo que la tarjeta enseña
    /// como «×4». Volver a copiar es la operación más frecuente del sistema:
    /// si sumara ahí, el número dejaría de significar lo que dice. La 2.x
    /// separa las dos cosas a propósito.
    pub fn reactivate(&self, id: i64, at: i64) -> Result<()> {
        self.db.execute(
            "UPDATE items SET modified_at = ?2, updated_at = ?2 WHERE id = ?1",
            params![id, at],
        )?;
        Ok(())
    }

    /// Se pegó desde el historial: eso sí cuenta.
    pub fn record_paste(&self, id: i64, at: i64) -> Result<()> {
        self.db.execute(
            "UPDATE items
             SET last_used_at = ?2, updated_at = ?2, paste_count = paste_count + 1
             WHERE id = ?1",
            params![id, at],
        )?;
        Ok(())
    }

    /// El color de la tarjeta, que es una de las vistas del panel.
    pub fn set_color(&self, id: i64, color: i64, at: i64) -> Result<()> {
        self.db.execute(
            "UPDATE items SET card_color = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, color, at],
        )?;
        Ok(())
    }

    pub fn paste_count(&self, id: i64) -> Result<i64> {
        Ok(self
            .db
            .query_row("SELECT paste_count FROM items WHERE id = ?1", [id], |row| {
                row.get(0)
            })?)
    }

    /// El texto que Vision leyó dentro de una imagen.
    ///
    /// Va en su propia columna del índice y no en `search_text` para que se
    /// pueda distinguir «lo que el usuario copió» de «lo que había escrito en
    /// la imagen», y para poder rehacerlo sin tocar lo demás. Se escribe
    /// después de capturar, nunca durante: reconocer texto cuesta unos 180 ms
    /// y el camino de captura no puede pagarlos.
    pub fn set_ocr_text(&self, id: i64, text: &str, at: i64) -> Result<()> {
        self.db.execute(
            "UPDATE items SET search_ocr = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, fold(text), at],
        )?;
        Ok(())
    }

    /// Los ítems de imagen a los que todavía no se les ha pasado el OCR.
    pub fn pending_ocr(&self, limit: usize) -> Result<Vec<i64>> {
        let mut stmt = self.db.prepare(
            "SELECT id FROM items
             WHERE kind = 'image' AND search_ocr = '' AND deleted_at IS NULL
             ORDER BY modified_at DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit as i64], |row| row.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Guarda un dato derivado del ítem.
    pub fn set_meta(&self, id: i64, key: &str, value: &str) -> Result<()> {
        self.db.execute(
            "INSERT INTO item_meta (item_id, key, value) VALUES (?1, ?2, ?3)
             ON CONFLICT(item_id, key) DO UPDATE SET value = excluded.value",
            params![id, key, value],
        )?;
        Ok(())
    }

    pub fn meta(&self, id: i64, key: &str) -> Result<Option<String>> {
        Ok(self
            .db
            .query_row(
                "SELECT value FROM item_meta WHERE item_id = ?1 AND key = ?2",
                params![id, key],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub fn all_meta(&self, id: i64) -> Result<Vec<(String, String)>> {
        let mut stmt = self
            .db
            .prepare("SELECT key, value FROM item_meta WHERE item_id = ?1 ORDER BY key")?;
        let rows = stmt.query_map([id], |row| Ok((row.get(0)?, row.get(1)?)))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Apunta un trabajo de enriquecimiento pendiente.
    pub fn enqueue(&self, id: i64, job: &str) -> Result<()> {
        self.db.execute(
            "INSERT OR IGNORE INTO pending_work (item_id, job) VALUES (?1, ?2)",
            params![id, job],
        )?;
        Ok(())
    }

    /// Los siguientes trabajos de ese tipo que ya se pueden intentar.
    pub fn take_pending(&self, job: &str, now: i64, limit: usize) -> Result<Vec<i64>> {
        let mut stmt = self.db.prepare(
            "SELECT w.item_id
             FROM pending_work w
             JOIN items i ON i.id = w.item_id
             WHERE w.job = ?1 AND w.not_before <= ?2 AND i.deleted_at IS NULL
             ORDER BY i.modified_at DESC
             LIMIT ?3",
        )?;
        let rows = stmt.query_map(params![job, now, limit as i64], |row| row.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn work_done(&self, id: i64, job: &str) -> Result<()> {
        self.db.execute(
            "DELETE FROM pending_work WHERE item_id = ?1 AND job = ?2",
            params![id, job],
        )?;
        Ok(())
    }

    /// Un intento fallido. A partir del tercero el trabajo se abandona: hay
    /// imágenes que Vision no sabe leer y reintentarlas para siempre es
    /// gastar batería en un resultado que no va a cambiar.
    pub const MAX_ATTEMPTS: i64 = 3;

    pub fn work_failed(&self, id: i64, job: &str, why: &str, retry_at: i64) -> Result<bool> {
        self.db.execute(
            "UPDATE pending_work
             SET attempts = attempts + 1, last_error = ?3, not_before = ?4
             WHERE item_id = ?1 AND job = ?2",
            params![id, job, why, retry_at],
        )?;
        let attempts: i64 = self
            .db
            .query_row(
                "SELECT attempts FROM pending_work WHERE item_id = ?1 AND job = ?2",
                params![id, job],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or(0);
        if attempts >= Self::MAX_ATTEMPTS {
            self.work_done(id, job)?;
            return Ok(false);
        }
        Ok(true)
    }

    /// La etiqueta entra en el índice, así que se busca por ella.
    pub fn set_label(&self, id: i64, label: Option<&str>, at: i64) -> Result<()> {
        self.db.execute(
            "UPDATE items SET label = ?2, search_label = ?3, updated_at = ?4 WHERE id = ?1",
            params![id, label, label.map(fold).unwrap_or_default(), at],
        )?;
        Ok(())
    }

    pub fn set_source(&self, id: i64, app: &str, at: i64) -> Result<()> {
        self.db.execute(
            "UPDATE items SET app_source = ?2, search_app = ?3, updated_at = ?4 WHERE id = ?1",
            params![id, app, fold(app), at],
        )?;
        Ok(())
    }

    /// Una tumba, no un borrado: la nube que vendrá necesita saber que algo
    /// dejó de existir, y sin esto una sincronización lo resucitaría.
    pub fn mark_deleted(&self, id: i64, at: i64) -> Result<()> {
        // La lápida guarda identidad y fechas para que la sincronización sepa
        // que esto dejó de existir. Todo lo demás se va: el contenido en claro,
        // su copia plegada en el índice, y las filas de formato con sus bytes.
        // Dejarlo era incumplir lo que PRIVACY.md promete al usuario.
        self.db.execute(
            "UPDATE items
             SET deleted_at = ?2, updated_at = ?2,
                 preview_text = '', search_text = '', search_label = '',
                 search_app = '', search_ocr = '', label = NULL, app_source = NULL,
                 thumb_path = NULL, content_hash = 0
             WHERE id = ?1",
            params![id, at],
        )?;
        // Los blobs de este ítem se sobrescriben antes de desaparecer, y
        // solo si ningún otro ítem los comparte: el nombre es el contenido,
        // así que dos copias iguales apuntan al mismo archivo.
        if let Some(blobs) = &self.blobs {
            for digest in self.blobs_of(id)? {
                if self.blob_is_shared(&digest, id)? {
                    continue;
                }
                blobs.remove(&digest)?;
            }
        }
        self.db
            .execute("DELETE FROM item_formats WHERE item_id = ?1", [id])?;
        // Lo derivado también es del usuario: dimensiones, duración, artista,
        // y el trabajo pendiente que ya no hay que hacer.
        self.db
            .execute("DELETE FROM item_meta WHERE item_id = ?1", [id])?;
        self.db
            .execute("DELETE FROM pending_work WHERE item_id = ?1", [id])?;
        Ok(())
    }

    fn blobs_of(&self, id: i64) -> Result<Vec<String>> {
        let mut stmt = self.db.prepare(
            "SELECT blob_path FROM item_formats WHERE item_id = ?1 AND blob_path IS NOT NULL",
        )?;
        let rows = stmt.query_map([id], |row| row.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    fn blob_is_shared(&self, digest: &str, besides: i64) -> Result<bool> {
        let count: i64 = self.db.query_row(
            "SELECT COUNT(*) FROM item_formats WHERE blob_path = ?1 AND item_id != ?2",
            params![digest, besides],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Los bytes de un formato, vengan de la fila o del disco.
    pub fn payload_of(&self, id: i64, format: &str) -> Result<Option<Vec<u8>>> {
        let found: Option<(Option<Vec<u8>>, Option<String>)> = self
            .db
            .query_row(
                "SELECT inline_data, blob_path FROM item_formats
                 WHERE item_id = ?1 AND format = ?2",
                params![id, format],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        match found {
            Some((Some(bytes), _)) => Ok(Some(bytes)),
            Some((None, Some(digest))) => match &self.blobs {
                Some(blobs) => blobs.get(&digest),
                None => Ok(None),
            },
            _ => Ok(None),
        }
    }

    /// Lo que cambió después de un punto, que es lo que una sincronización
    /// necesita preguntar.
    pub fn changed_since(&self, version: i64) -> Result<Vec<String>> {
        let mut stmt = self
            .db
            .prepare("SELECT uuid FROM items WHERE updated_at > ?1 ORDER BY updated_at")?;
        let rows = stmt.query_map([version], |row| row.get::<_, String>(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Guarda el ítem con **todas** sus filas de formato. El `content_hash`
    /// es del contenido completo, así que dos copias iguales no crean dos
    /// ítems y una copia distinta nunca se toma por repetida.
    pub fn insert_item(
        &self,
        uuid: &str,
        item: &Item,
        preview: &str,
        created_at: i64,
    ) -> Result<i64> {
        // Sin almacén de blobs —una base en memoria— aceptar un `Blob` sería
        // guardar la fila con su tamaño y tirar los bytes. Es preferible
        // negarse a guardar un ítem vacío del que nadie sospecharía.
        if self.blobs.is_none()
            && let Some(oversized) = item.oversized_format()
        {
            return Err(Error::NeedsBlobStore {
                format: oversized.0,
                size: oversized.1,
            });
        }
        let hash = item.fingerprint() as i64;
        let kind = item.kind.map(|k| k.as_str());
        self.db.execute(
            "INSERT INTO items (uuid, kind, preview_text, created_at, modified_at, updated_at,
                                content_hash, search_text)
             VALUES (?1, ?2, ?3, ?4, ?4, ?4, ?5, ?6)",
            params![uuid, kind, preview, created_at, hash, fold(preview)],
        )?;
        let id = self.db.last_insert_rowid();
        for format in &item.formats {
            let (inline, blob, size) = match &format.payload {
                Payload::Inline(bytes) => (Some(bytes.clone()), None, Some(bytes.len() as i64)),
                Payload::Blob(bytes) => {
                    let digest = match &self.blobs {
                        Some(blobs) => Some(blobs.put(bytes)?),
                        None => None,
                    };
                    (None, digest, Some(bytes.len() as i64))
                }
                Payload::TooBig { size } => (None, None, Some(*size as i64)),
                Payload::Announced { size } => (None, None, size.map(|s| s as i64)),
                Payload::Absent => (None, None, None),
            };
            self.db.execute(
                "INSERT INTO item_formats (item_id, format, size_bytes, inline_data, blob_path)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, format.id, size, inline, blob],
            )?;
        }
        Ok(id)
    }

    /// Busca por la **identidad del ítem**, que es la que `insert_item`
    /// guarda. Recibía un `&str` y calculaba el hash del texto desnudo, que no
    /// es lo que hay en la columna: así nunca encontraba nada capturado.
    pub fn find_by_hash(&self, item: &Item) -> Result<Option<i64>> {
        let hash = item.fingerprint() as i64;
        Ok(self
            .db
            .query_row(
                "SELECT id FROM items WHERE content_hash = ?1 AND deleted_at IS NULL LIMIT 1",
                [hash],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub fn formats_of(&self, id: i64) -> Result<Vec<String>> {
        let mut stmt = self
            .db
            .prepare("SELECT format FROM item_formats WHERE item_id = ?1 ORDER BY format")?;
        let rows = stmt.query_map([id], |row| row.get::<_, String>(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Un ítem cuyo archivo ya no existe **no se borra**: se marca. La
    /// diferencia entre «se me borró el historial» y «esto ya no está en el
    /// disco» la nota el usuario de inmediato.
    pub fn mark_broken(&self, id: i64, at: i64) -> Result<()> {
        self.db.execute(
            "UPDATE items SET broken_since = ?2 WHERE id = ?1 AND broken_since IS NULL",
            params![id, at],
        )?;
        Ok(())
    }

    /// Los rotos se van cuando cumplen su plazo, nunca antes. Un ítem fijado
    /// es una decisión del usuario y la limpieza no la revoca.
    pub fn purge_broken_before(&self, cutoff: i64) -> Result<usize> {
        Ok(self.db.execute(
            "DELETE FROM items
             WHERE broken_since IS NOT NULL AND broken_since < ?1 AND pinned = 0",
            [cutoff],
        )?)
    }

    pub fn pin(&self, id: i64) -> Result<()> {
        self.db
            .execute("UPDATE items SET pinned = 1 WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Cuántos ítems tiene el usuario. Las lápidas no se cuentan: para él
    /// están borradas.
    pub fn count(&self) -> Result<i64> {
        Ok(self.db.query_row(
            "SELECT COUNT(*) FROM items WHERE deleted_at IS NULL",
            [],
            |row| row.get(0),
        )?)
    }

    /// Lo que el panel pide: una ventana del historial, con o sin término de
    /// búsqueda y con los filtros que el usuario tenga puestos.
    ///
    /// Una sola consulta para las dos cosas. Tener una para «buscar» y otra
    /// para «listar» es el camino corto a que los filtros funcionen en una y
    /// no en la otra.
    pub fn list(&self, filter: &Filter, limit: usize, after: Option<i64>) -> Result<Vec<Listed>> {
        let expression = filter
            .query
            .as_deref()
            .map(|text| fts_expression(&fold(text)));
        // Se pidió buscar algo que no deja ningún término utilizable.
        if matches!(expression, Some(None)) {
            return Ok(Vec::new());
        }
        let expression = expression.flatten();

        let mut sql = String::from(
            "SELECT items.id, items.modified_at, items.kind, items.preview_text, items.pinned
             FROM items ",
        );
        if expression.is_some() {
            sql.push_str("JOIN items_fts ON items.id = items_fts.rowid ");
        }
        sql.push_str("WHERE items.deleted_at IS NULL ");
        if expression.is_some() {
            sql.push_str("AND items_fts MATCH :match ");
        }
        if !filter.kinds.is_empty() {
            let list = filter
                .kinds
                .iter()
                .map(|kind| format!("'{}'", kind.as_str()))
                .collect::<Vec<_>>()
                .join(", ");
            sql.push_str(&format!("AND items.kind IN ({list}) "));
        }
        if !filter.colors.is_empty() {
            let list = filter
                .colors
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            sql.push_str(&format!("AND items.card_color IN ({list}) "));
        }
        if filter.pinned_only {
            sql.push_str("AND items.pinned = 1 ");
        }
        sql.push_str(
            "AND (:after IS NULL OR items.modified_at < :after)
             ORDER BY items.modified_at DESC LIMIT :limit",
        );

        let mut stmt = self.db.prepare(&sql)?;
        // Solo se pasan los parámetros que la consulta construida usa: SQLite
        // rechaza un nombre que no aparezca en ella.
        let limit = limit as i64;
        let mut bound: Vec<(&str, &dyn rusqlite::ToSql)> =
            vec![(":after", &after), (":limit", &limit)];
        if let Some(text) = &expression {
            bound.push((":match", text));
        }
        let rows = stmt.query_map(bound.as_slice(), |row| {
            Ok(Listed {
                id: row.get(0)?,
                modified_at: row.get(1)?,
                kind: row.get(2)?,
                preview: row.get(3)?,
                pinned: row.get::<_, i64>(4)? == 1,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Retención: lo viejo se va, lo fijado se queda.
    ///
    /// Fijar un ítem es una decisión del usuario y la limpieza no la revoca,
    /// que es exactamente lo que hace `clearOldItems` en la 2.x.
    pub fn clear_older_than(&self, cutoff: i64) -> Result<usize> {
        let doomed: Vec<i64> = {
            let mut stmt = self.db.prepare(
                "SELECT id FROM items
                 WHERE modified_at < ?1 AND pinned = 0 AND deleted_at IS NULL",
            )?;
            let rows = stmt.query_map([cutoff], |row| row.get(0))?;
            rows.collect::<rusqlite::Result<_>>()?
        };
        for id in &doomed {
            self.mark_deleted(*id, cutoff)?;
        }
        Ok(doomed.len())
    }

    /// Vaciar el historial dejando lo fijado.
    pub fn clear_all_unpinned(&self, at: i64) -> Result<usize> {
        self.clear_older_than_matching(at, "pinned = 0")
    }

    fn clear_older_than_matching(&self, at: i64, condition: &str) -> Result<usize> {
        let doomed: Vec<i64> = {
            let mut stmt = self.db.prepare(&format!(
                "SELECT id FROM items WHERE {condition} AND deleted_at IS NULL"
            ))?;
            let rows = stmt.query_map([], |row| row.get(0))?;
            rows.collect::<rusqlite::Result<_>>()?
        };
        for id in &doomed {
            self.mark_deleted(*id, at)?;
        }
        Ok(doomed.len())
    }

    /// Cuántas filas pide una lista antes de que el usuario haga scroll. Sin
    /// tope, una consulta que casa con todo materializa el historial entero
    /// en cada pulsación.
    pub const PAGE: usize = 100;

    pub fn search(&self, query: &str) -> Result<Vec<String>> {
        self.search_page(query, Self::PAGE, 0)
    }

    /// Página siguiente a partir del último visto, en vez de `OFFSET`.
    ///
    /// Con `OFFSET`, SQLite ordena el resultado entero y descarta lo saltado,
    /// así que la página diez cuesta diez veces la primera. Con el corte por
    /// `modified_at` puede recorrer el índice de recencia y parar al llenar
    /// la página: cada página cuesta lo mismo que la primera.
    pub fn search_after(
        &self,
        query: &str,
        limit: usize,
        after: Option<i64>,
    ) -> Result<Vec<(i64, String)>> {
        let Some(expression) = fts_expression(&fold(query)) else {
            return Ok(Vec::new());
        };
        let mut stmt = self.db.prepare(
            "SELECT items.modified_at, items.preview_text
             FROM items_fts
             JOIN items ON items.id = items_fts.rowid
             WHERE items_fts MATCH ?1
               AND items.deleted_at IS NULL
               AND (?2 IS NULL OR items.modified_at < ?2)
             ORDER BY items.modified_at DESC
             LIMIT ?3",
        )?;
        let rows = stmt.query_map(rusqlite::params![expression, after, limit as i64], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn search_page(&self, query: &str, limit: usize, offset: usize) -> Result<Vec<String>> {
        let Some(expression) = fts_expression(&fold(query)) else {
            return Ok(Vec::new());
        };
        let mut stmt = self.db.prepare(
            "SELECT items.preview_text
             FROM items_fts
             JOIN items ON items.id = items_fts.rowid
             WHERE items_fts MATCH ?1 AND items.deleted_at IS NULL
             ORDER BY items.modified_at DESC
             LIMIT ?2 OFFSET ?3",
        )?;
        let rows = stmt.query_map(
            rusqlite::params![expression, limit as i64, offset as i64],
            |row| row.get::<_, String>(0),
        )?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}

/// Ajusta los permisos de un archivo o carpeta a lo que se le pide.
pub(crate) fn restrict(path: &std::path::Path, mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let permissions = std::fs::Permissions::from_mode(mode);
    std::fs::set_permissions(path, permissions).map_err(Error::Io)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cp_core::item::Format;
    use cp_core::kind::Kind;

    fn seeded() -> Store {
        let store = Store::in_memory().expect("esquema");
        for (at, text) in [
            "el café de la esquina",
            "Straße Hauptbahnhof",
            "encyclopædia britannica",
            "Łódź centrum",
            "Peçanha e Gonçalves",
        ]
        .iter()
        .enumerate()
        {
            store
                .insert_text(&format!("uuid-{at}"), text, at as i64)
                .expect("insert");
        }
        store
    }

    /// Los cuatro casos del fallo verificado con SQLite en la 2.x, donde solo
    /// el primero funcionaba —y por suerte, gracias al tokenizer—.
    #[test]
    fn the_four_cases_that_2x_gets_wrong() {
        let store = seeded();
        for (query, expected) in [
            ("cafe", "el café de la esquina"),
            ("strasse", "Straße Hauptbahnhof"),
            ("encyclopaedia", "encyclopædia britannica"),
            ("lodz", "Łódź centrum"),
        ] {
            let hits = store.search(query).expect("consulta");
            assert!(
                hits.iter().any(|hit| hit == expected),
                "buscando «{query}» no apareció «{expected}»: {hits:?}"
            );
        }
    }

    /// Y en la otra dirección, que es la mitad que nadie prueba: escribir la
    /// palabra con el carácter especial también tiene que encontrarla.
    #[test]
    fn it_works_in_both_directions() {
        let store = seeded();
        for (query, expected) in [
            ("café", "el café de la esquina"),
            ("Straße", "Straße Hauptbahnhof"),
            ("encyclopædia", "encyclopædia britannica"),
            ("Łódź", "Łódź centrum"),
            ("Gonçalves", "Peçanha e Gonçalves"),
        ] {
            let hits = store.search(query).expect("consulta");
            assert!(
                hits.iter().any(|hit| hit == expected),
                "buscando «{query}» no apareció «{expected}»: {hits:?}"
            );
        }
    }

    #[test]
    fn the_stored_text_keeps_its_accents() {
        let store = seeded();
        let hits = store.search("cafe").expect("consulta");
        assert_eq!(
            hits.first().map(String::as_str),
            Some("el café de la esquina"),
            "se busca sin tildes pero se muestra como se copió"
        );
    }

    fn sample_item() -> Item {
        Item {
            kind: Some(cp_core::kind::Kind::Text),
            formats: vec![
                Format {
                    id: "public.utf8-plain-text".into(),
                    payload: Payload::Inline(b"hola".to_vec()),
                },
                Format {
                    id: "public.rtf".into(),
                    payload: Payload::Inline(vec![0u8; 400]),
                },
                Format {
                    id: "com.apple.icns".into(),
                    payload: Payload::Announced { size: None },
                },
                Format {
                    id: "fndf".into(),
                    payload: Payload::Absent,
                },
            ],
        }
    }

    #[test]
    fn an_item_too_big_for_the_row_is_refused_not_emptied() {
        let store = Store::in_memory().expect("esquema");
        let big = Item {
            kind: None,
            formats: vec![Format {
                id: "public.png".into(),
                payload: Payload::Blob(vec![0u8; 100_000]),
            }],
        };
        assert!(
            store.insert_item("uuid-grande", &big, "", 1).is_err(),
            "mejor negarse que guardar un ítem sin sus bytes"
        );
        assert_eq!(store.count().expect("cuenta"), 0);
    }

    #[test]
    fn two_images_with_no_preview_are_two_items() {
        let store = Store::in_memory().expect("esquema");
        let image = |byte: u8| Item {
            kind: Some(cp_core::kind::Kind::Image),
            formats: vec![Format {
                id: "public.png".into(),
                payload: Payload::Inline(vec![byte; 512]),
            }],
        };
        store
            .insert_item("uuid-a", &image(1), "", 1)
            .expect("insert");
        store
            .insert_item("uuid-b", &image(2), "", 2)
            .expect("insert");
        let hashes: Vec<i64> = store
            .db
            .prepare("SELECT content_hash FROM items ORDER BY id")
            .expect("prepara")
            .query_map([], |row| row.get(0))
            .expect("consulta")
            .map(|row| row.expect("fila"))
            .collect();
        assert_ne!(
            hashes[0], hashes[1],
            "hashear el preview vacío las haría la misma"
        );
    }

    #[test]
    fn an_item_keeps_every_format_it_was_offered() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_item("uuid-multi", &sample_item(), "hola", 1)
            .expect("insert");
        let formats = store.formats_of(id).expect("formatos");
        assert_eq!(formats.len(), 4, "las cuatro filas, incluidas las vacías");
        assert!(formats.contains(&"com.apple.icns".to_string()));
        assert!(formats.contains(&"fndf".to_string()));
    }

    #[test]
    fn the_same_content_is_found_by_its_hash() {
        let store = Store::in_memory().expect("esquema");
        store.insert_text("uuid-a", "repetido", 1).expect("insert");
        assert!(
            store
                .find_by_hash(&Item::plain("repetido"))
                .expect("busca")
                .is_some()
        );
        assert!(
            store
                .find_by_hash(&Item::plain("distinto"))
                .expect("busca")
                .is_none()
        );
    }

    #[test]
    fn deleting_an_item_takes_its_formats_with_it() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_item("uuid-cascade", &sample_item(), "hola", 1)
            .expect("insert");
        store.mark_broken(id, 10).expect("marca");
        store.purge_broken_before(20).expect("purga");
        assert_eq!(store.formats_of(id).expect("formatos").len(), 0);
    }

    #[test]
    fn a_broken_item_survives_until_its_time_is_up() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_text("uuid-roto", "archivo ido", 1)
            .expect("insert");
        store.mark_broken(id, 100).expect("marca");
        assert_eq!(store.purge_broken_before(50).expect("purga"), 0);
        assert_eq!(store.count().expect("cuenta"), 1, "aún no cumple el plazo");
        assert_eq!(store.purge_broken_before(150).expect("purga"), 1);
        assert_eq!(store.count().expect("cuenta"), 0);
    }

    #[test]
    fn marking_a_broken_item_twice_does_not_restart_its_clock() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_text("uuid-roto", "archivo ido", 1)
            .expect("insert");
        store.mark_broken(id, 100).expect("primera");
        store.mark_broken(id, 900).expect("segunda");
        assert_eq!(
            store.purge_broken_before(150).expect("purga"),
            1,
            "vale la primera vez que se vio roto, no la última"
        );
    }

    #[test]
    fn a_pinned_item_is_never_purged_even_when_broken() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_text("uuid-fijado", "importante", 1)
            .expect("insert");
        store.pin(id).expect("fija");
        store.mark_broken(id, 100).expect("marca");
        assert_eq!(store.purge_broken_before(9999).expect("purga"), 0);
        assert_eq!(store.count().expect("cuenta"), 1);
    }

    /// Lo que un usuario puede escribir en el buscador sin querer decir nada
    /// especial. Ninguna de estas puede devolver un error de SQL.
    #[test]
    fn no_query_a_person_can_type_breaks_the_search() {
        let store = seeded();
        for query in [
            "\"",
            "\"\"",
            "*",
            "(",
            ")",
            "()",
            "a AND b",
            "NOT café",
            "search_text:café",
            "NEAR(a b)",
            "-café",
            "^café",
            "café*",
            "{café}",
            "[café]",
            "café OR",
            "OR",
            "AND OR NOT",
            "",
            " ",
            "\t\n",
            "...",
            "!!!",
            "\\",
            "%",
            "_",
            "'; DROP TABLE items; --",
        ] {
            store
                .search(query)
                .unwrap_or_else(|why| panic!("«{query}» rompió la búsqueda: {why}"));
        }
    }

    #[test]
    fn an_empty_search_returns_nothing_rather_than_everything() {
        let store = seeded();
        for empty in ["", "   ", "\t", "-", "!!", "***"] {
            assert!(
                store.search(empty).expect("consulta").is_empty(),
                "«{empty}» debería no devolver nada"
            );
        }
    }

    #[test]
    fn the_punctuation_around_a_word_does_not_hide_it() {
        let store = seeded();
        for query in ["-café", "^café", "(café)", "«café»", "café!"] {
            let hits = store.search(query).expect("consulta");
            assert!(
                hits.iter().any(|hit| hit.contains("café")),
                "«{query}» no encontró el café"
            );
        }
    }

    #[test]
    fn scripts_that_are_not_latin_go_in_and_come_out() {
        let store = Store::in_memory().expect("esquema");
        for (at, text) in [
            "日本語のテキスト",
            "Привет мир",
            "مرحبا بالعالم",
            "🎉 fiesta 🎊",
            "한국어 텍스트",
        ]
        .iter()
        .enumerate()
        {
            store
                .insert_text(&format!("uuid-{at}"), text, at as i64)
                .expect("insert");
        }
        for (query, expected) in [
            ("日本語", "日本語のテキスト"),
            ("Привет", "Привет мир"),
            ("fiesta", "🎉 fiesta 🎊"),
            ("한국어", "한국어 텍스트"),
        ] {
            let hits = store.search(query).expect("consulta");
            assert!(
                hits.iter().any(|hit| hit == expected),
                "buscando «{query}» faltó «{expected}»: {hits:?}"
            );
        }
    }

    #[test]
    fn a_very_long_text_is_stored_and_found() {
        let store = Store::in_memory().expect("esquema");
        let long = format!(
            "{} aguja {}",
            "paja ".repeat(50_000),
            "paja ".repeat(50_000)
        );
        store.insert_text("uuid-largo", &long, 1).expect("insert");
        assert_eq!(store.search("aguja").expect("consulta").len(), 1);
    }

    #[test]
    fn the_same_uuid_twice_is_refused_not_duplicated() {
        let store = Store::in_memory().expect("esquema");
        store
            .insert_text("uuid-unico", "primero", 1)
            .expect("insert");
        assert!(
            store.insert_text("uuid-unico", "segundo", 2).is_err(),
            "el uuid es único por contrato"
        );
        assert_eq!(store.count().expect("cuenta"), 1);
    }

    #[test]
    fn marking_an_item_that_does_not_exist_is_not_a_failure() {
        let store = Store::in_memory().expect("esquema");
        store.mark_broken(9999, 1).expect("no existe, no pasa nada");
        assert_eq!(store.count().expect("cuenta"), 0);
    }

    #[test]
    fn a_purge_with_nothing_to_purge_removes_nothing() {
        let store = seeded();
        let before = store.count().expect("cuenta");
        assert_eq!(store.purge_broken_before(-1).expect("purga"), 0);
        assert_eq!(store.purge_broken_before(i64::MAX).expect("purga"), 0);
        assert_eq!(store.count().expect("cuenta"), before);
    }

    #[test]
    fn an_item_with_no_formats_at_all_is_still_an_item() {
        let store = Store::in_memory().expect("esquema");
        let empty = Item {
            kind: None,
            formats: vec![],
        };
        let id = store
            .insert_item("uuid-vacio", &empty, "", 1)
            .expect("insert");
        assert_eq!(store.formats_of(id).expect("formatos").len(), 0);
        assert_eq!(store.count().expect("cuenta"), 1);
    }

    #[test]
    fn a_search_that_matches_everything_still_returns_one_page() {
        let store = Store::in_memory().expect("esquema");
        for at in 0..250 {
            store
                .insert_text(&format!("uuid-{at}"), &format!("comun {at}"), at)
                .expect("insert");
        }
        assert_eq!(store.search("comun").expect("consulta").len(), Store::PAGE);
        assert_eq!(
            store.search_page("comun", 10, 0).expect("consulta").len(),
            10
        );
    }

    #[test]
    fn paging_walks_the_whole_result_without_repeating() {
        let store = Store::in_memory().expect("esquema");
        for at in 0..25 {
            store
                .insert_text(&format!("uuid-{at}"), &format!("pagina {at}"), at)
                .expect("insert");
        }
        let first = store.search_page("pagina", 10, 0).expect("consulta");
        let second = store.search_page("pagina", 10, 10).expect("consulta");
        let last = store.search_page("pagina", 10, 20).expect("consulta");
        assert_eq!((first.len(), second.len(), last.len()), (10, 10, 5));
        assert!(
            first.iter().all(|one| !second.contains(one)),
            "las páginas no pueden solaparse"
        );
    }

    #[test]
    fn the_cursor_walks_the_result_without_repeating_or_skipping() {
        let store = Store::in_memory().expect("esquema");
        for at in 0..25 {
            store
                .insert_text(&format!("uuid-{at}"), &format!("cursor {at}"), at)
                .expect("insert");
        }
        let mut seen = Vec::new();
        let mut after = None;
        loop {
            let page = store.search_after("cursor", 10, after).expect("consulta");
            if page.is_empty() {
                break;
            }
            after = page.last().map(|(at, _)| *at);
            seen.extend(page.into_iter().map(|(_, text)| text));
        }
        assert_eq!(seen.len(), 25, "recorrió todo");
        let mut unique = seen.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), 25, "sin repetir");
    }

    #[test]
    fn a_cursor_past_the_oldest_item_is_empty() {
        let store = seeded();
        assert!(
            store
                .search_after("café", 10, Some(-1))
                .expect("consulta")
                .is_empty()
        );
    }

    #[test]
    fn an_offset_past_the_end_is_empty_not_an_error() {
        let store = seeded();
        assert!(
            store
                .search_page("café", 10, 9999)
                .expect("consulta")
                .is_empty()
        );
    }

    #[test]
    fn copying_something_again_lifts_it_instead_of_duplicating_it() {
        let store = Store::in_memory().expect("esquema");
        let first = store.insert_text("uuid-a", "lo viejo", 10).expect("insert");
        store.insert_text("uuid-b", "lo nuevo", 20).expect("insert");

        let before = store.search("lo").expect("consulta");
        assert_eq!(before.first().map(String::as_str), Some("lo nuevo"));

        store.reactivate(first, 30).expect("recopiado");
        let after = store.search("lo").expect("consulta");
        assert_eq!(
            after.first().map(String::as_str),
            Some("lo viejo"),
            "recopiar algo lo sube al principio"
        );
    }

    #[test]
    fn a_label_can_be_searched_for() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_text("uuid-etq", "un texto cualquiera", 1)
            .expect("insert");
        assert!(store.search("factura").expect("consulta").is_empty());
        store
            .set_label(id, Some("Factura Mayo"), 2)
            .expect("etiqueta");
        let hits = store.search("factura").expect("consulta");
        assert_eq!(hits.len(), 1, "la etiqueta entra en el índice");
    }

    #[test]
    fn the_source_application_can_be_searched_for() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_text("uuid-app", "algo copiado", 1)
            .expect("insert");
        store.set_source(id, "Safari", 2).expect("origen");
        assert_eq!(store.search("safari").expect("consulta").len(), 1);
    }

    #[test]
    fn a_label_with_accents_is_found_without_them() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_text("uuid-tilde", "contenido", 1)
            .expect("insert");
        store
            .set_label(id, Some("Reunión Diseño"), 2)
            .expect("etiqueta");
        assert_eq!(store.search("reunion").expect("consulta").len(), 1);
        assert_eq!(store.search("diseño").expect("consulta").len(), 1);
    }

    #[test]
    fn removing_a_label_takes_it_out_of_the_index() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_text("uuid-quita", "contenido", 1)
            .expect("insert");
        store.set_label(id, Some("temporal"), 2).expect("pone");
        assert_eq!(store.search("temporal").expect("consulta").len(), 1);
        store.set_label(id, None, 3).expect("quita");
        assert!(store.search("temporal").expect("consulta").is_empty());
    }

    #[test]
    fn deleting_hides_the_item_from_everything_the_user_can_see() {
        let store = seeded();
        let id = store
            .insert_text("uuid-secreto", "contraseña del banco", 500)
            .expect("insert");
        let before = store.count().expect("cuenta");

        store.mark_deleted(id, 600).expect("borra");

        assert_eq!(store.count().expect("cuenta"), before - 1, "deja de contar");
        assert!(
            store.search("contraseña").expect("consulta").is_empty(),
            "no puede seguir encontrándose"
        );
        assert!(
            store
                .find_by_hash(&Item::plain("contraseña del banco"))
                .expect("hash")
                .is_none(),
            "volver a copiarlo debe crear un ítem nuevo, no resucitar la lápida"
        );
    }

    #[test]
    fn a_deleted_item_leaves_no_content_behind() {
        let store = Store::in_memory().expect("esquema");
        let item = sample_item();
        let id = store
            .insert_item("uuid-borrado", &item, "texto en claro", 1)
            .expect("insert");
        store.set_label(id, Some("etiqueta"), 2).expect("etiqueta");
        store.mark_deleted(id, 3).expect("borra");

        let (preview, search, label): (String, String, Option<String>) = store
            .db
            .query_row(
                "SELECT preview_text, search_text, label FROM items WHERE id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("consulta");
        assert_eq!(preview, "", "el contenido en claro se va");
        assert_eq!(search, "", "y su copia en el índice también");
        assert_eq!(label, None);
        assert_eq!(
            store.formats_of(id).expect("formatos").len(),
            0,
            "los bytes de los formatos se van con el ítem"
        );
    }

    #[test]
    fn the_tombstone_still_tells_the_sync_what_happened() {
        let store = Store::in_memory().expect("esquema");
        let id = store.insert_text("uuid-tumba", "se va", 1).expect("insert");
        store.mark_deleted(id, 50).expect("borra");
        assert!(
            store
                .changed_since(40)
                .expect("cambios")
                .contains(&"uuid-tumba".to_string()),
            "sin esto, otra máquina lo resucita"
        );
    }

    #[test]
    fn only_what_changed_after_the_mark_is_reported() {
        let store = Store::in_memory().expect("esquema");
        let old = store
            .insert_text("uuid-viejo", "antiguo", 10)
            .expect("insert");
        store
            .insert_text("uuid-nuevo", "reciente", 100)
            .expect("insert");
        let changed = store.changed_since(50).expect("cambios");
        assert_eq!(changed, vec!["uuid-nuevo".to_string()]);

        store.reactivate(old, 200).expect("recopiado");
        let after = store.changed_since(50).expect("cambios");
        assert_eq!(
            after,
            vec!["uuid-nuevo".to_string(), "uuid-viejo".to_string()],
            "ordenados por versión, y el recopiado ahora cuenta"
        );
    }

    #[test]
    fn copying_something_again_does_not_inflate_the_paste_counter() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_text("uuid-recopiado", "algo", 1)
            .expect("insert");
        for at in 2..10 {
            store.reactivate(id, at).expect("recopiado");
        }
        assert_eq!(
            store.paste_count(id).expect("cuenta"),
            0,
            "volver a copiar no es pegar, y el ×N de la tarjeta lo enseña"
        );
    }

    #[test]
    fn pasting_from_the_history_is_what_counts() {
        let store = Store::in_memory().expect("esquema");
        let id = store.insert_text("uuid-pegado", "algo", 1).expect("insert");
        store.record_paste(id, 2).expect("pega");
        store.record_paste(id, 3).expect("pega");
        assert_eq!(store.paste_count(id).expect("cuenta"), 2);
    }

    #[test]
    fn pasting_does_not_move_the_item_up_the_list() {
        let store = a_little_history();
        let listed = store.list(&Filter::default(), 10, None).expect("listado");
        let oldest = listed.last().expect("hay").id;
        store.record_paste(oldest, 999).expect("pega");
        let after = store.list(&Filter::default(), 10, None).expect("listado");
        assert_eq!(
            after.last().map(|one| one.id),
            Some(oldest),
            "pegar cuenta, pero no reordena el historial"
        );
    }

    #[test]
    fn the_colour_can_be_set_and_filtered_by() {
        let store = a_little_history();
        let listed = store.list(&Filter::default(), 10, None).expect("listado");
        store.set_color(listed[0].id, 3, 100).expect("color");
        let filter = Filter {
            colors: vec![3],
            ..Default::default()
        };
        let coloured = store.list(&filter, 10, None).expect("listado");
        assert_eq!(coloured.len(), 1);
        assert_eq!(coloured[0].id, listed[0].id);
    }

    #[test]
    fn an_image_becomes_findable_by_what_is_written_inside_it() {
        let store = Store::in_memory().expect("esquema");
        let image = Item {
            kind: Some(cp_core::kind::Kind::Image),
            formats: vec![Format {
                id: "public.png".into(),
                payload: Payload::Inline(vec![137, 80, 78, 71]),
            }],
        };
        let id = store
            .insert_item("uuid-captura", &image, "", 1)
            .expect("insert");

        assert!(
            store.search("pedido").expect("consulta").is_empty(),
            "todavía no se le ha pasado el OCR"
        );
        assert_eq!(store.pending_ocr(10).expect("pendientes"), vec![id]);

        store
            .set_ocr_text(id, "Pedido AB-4417 entrega 12 marzo", 2)
            .expect("ocr");

        assert_eq!(
            store.search("pedido").expect("consulta").len(),
            1,
            "una captura de pantalla se encuentra por lo que pone dentro"
        );
        assert_eq!(store.search("AB-4417").expect("consulta").len(), 1);
        assert!(
            store.pending_ocr(10).expect("pendientes").is_empty(),
            "ya no está pendiente"
        );
    }

    #[test]
    fn the_ocr_text_is_folded_like_everything_else() {
        let store = Store::in_memory().expect("esquema");
        let image = Item {
            kind: Some(cp_core::kind::Kind::Image),
            formats: vec![Format {
                id: "public.png".into(),
                payload: Payload::Inline(vec![1]),
            }],
        };
        let id = store
            .insert_item("uuid-tilde", &image, "", 1)
            .expect("insert");
        store.set_ocr_text(id, "Reunión en Múnich", 2).expect("ocr");
        assert_eq!(store.search("reunion").expect("consulta").len(), 1);
        assert_eq!(store.search("munich").expect("consulta").len(), 1);
    }

    #[test]
    fn deleting_takes_the_recognised_text_with_it() {
        let store = Store::in_memory().expect("esquema");
        let image = Item {
            kind: Some(cp_core::kind::Kind::Image),
            formats: vec![Format {
                id: "public.png".into(),
                payload: Payload::Inline(vec![1]),
            }],
        };
        let id = store
            .insert_item("uuid-secreta", &image, "", 1)
            .expect("insert");
        store
            .set_ocr_text(id, "clave de recuperación 8842", 2)
            .expect("ocr");
        store.mark_deleted(id, 3).expect("borra");
        assert!(
            store.search("recuperacion").expect("consulta").is_empty(),
            "lo leído dentro de la imagen también es contenido del usuario"
        );
    }

    #[test]
    fn what_is_written_survives_closing_the_application() {
        let dir = tempfile::tempdir().expect("carpeta");
        let path = dir.path().join("sub").join("history.db");

        {
            let store = Store::open(&path).expect("abre");
            store
                .insert_text("uuid-persiste", "sobrevive", 1)
                .expect("insert");
            store.checkpoint().expect("checkpoint");
        }

        let reopened = Store::open(&path).expect("reabre");
        assert_eq!(reopened.count().expect("cuenta"), 1);
        assert_eq!(
            reopened.search("sobrevive").expect("consulta").len(),
            1,
            "y el índice también sobrevive"
        );
    }

    #[test]
    fn the_history_is_not_readable_by_other_users() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().expect("carpeta");
        let path = dir.path().join("datos").join("history.db");
        let store = Store::open(&path).expect("abre");
        store
            .insert_text("uuid-privado", "contraseña", 1)
            .expect("insert");
        drop(store);

        let file = std::fs::metadata(&path)
            .expect("archivo")
            .permissions()
            .mode()
            & 0o777;
        let folder = std::fs::metadata(path.parent().expect("padre"))
            .expect("carpeta")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(file, 0o600, "solo su dueño");
        assert_eq!(folder, 0o700, "y la carpeta igual");
    }

    #[test]
    fn a_checkpoint_leaves_the_data_in_the_main_file() {
        let dir = tempfile::tempdir().expect("carpeta");
        let path = dir.path().join("history.db");
        let store = Store::open(&path).expect("abre");
        for at in 0..50 {
            store
                .insert_text(&format!("uuid-{at}"), &format!("linea {at}"), at)
                .expect("insert");
        }
        store.checkpoint().expect("checkpoint");
        let wal = path.with_extension("db-wal");
        let wal_size = std::fs::metadata(&wal).map(|m| m.len()).unwrap_or(0);
        assert!(
            wal_size == 0 || !wal.exists(),
            "tras el checkpoint el WAL queda vacío, no con {wal_size} bytes"
        );
    }

    #[test]
    fn reopening_keeps_the_pragmas_that_protect_the_data() {
        let dir = tempfile::tempdir().expect("carpeta");
        let path = dir.path().join("history.db");
        drop(Store::open(&path).expect("abre"));
        let store = Store::open(&path).expect("reabre");
        let vacuum: i64 = store
            .db
            .query_row("PRAGMA auto_vacuum", [], |row| row.get(0))
            .expect("consulta");
        assert_eq!(
            vacuum, 2,
            "el modo se guarda en el archivo y debe seguir ahí"
        );
    }

    fn on_disk() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().expect("carpeta");
        let store = Store::open(&dir.path().join("history.db")).expect("abre");
        (dir, store)
    }

    fn big_image(byte: u8) -> Item {
        Item {
            kind: Some(cp_core::kind::Kind::Image),
            formats: vec![Format {
                id: "public.png".into(),
                payload: Payload::Blob(vec![byte; 200_000]),
            }],
        }
    }

    #[test]
    fn an_image_too_big_for_the_row_goes_to_disk_and_comes_back() {
        let (_dir, store) = on_disk();
        let id = store
            .insert_item("uuid-imagen", &big_image(7), "", 1)
            .expect("insert");
        let bytes = store
            .payload_of(id, "public.png")
            .expect("lee")
            .expect("está");
        assert_eq!(bytes.len(), 200_000);
        assert!(bytes.iter().all(|b| *b == 7));
    }

    #[test]
    fn two_copies_of_the_same_image_share_one_file() {
        let (dir, store) = on_disk();
        store
            .insert_item("uuid-a", &big_image(9), "", 1)
            .expect("a");
        store
            .insert_item("uuid-b", &big_image(9), "", 2)
            .expect("b");
        let files = std::fs::read_dir(dir.path().join("blobs"))
            .expect("carpeta")
            .count();
        assert_eq!(
            files, 1,
            "el nombre es el contenido, así que es el mismo archivo"
        );
    }

    #[test]
    fn deleting_an_image_takes_its_bytes_off_the_disk() {
        let (_dir, store) = on_disk();
        let id = store
            .insert_item("uuid-borrar", &big_image(3), "", 1)
            .expect("insert");
        assert!(store.payload_of(id, "public.png").expect("lee").is_some());
        store.mark_deleted(id, 2).expect("borra");
        assert!(
            store.payload_of(id, "public.png").expect("lee").is_none(),
            "los bytes de una imagen borrada no pueden seguir en disco"
        );
    }

    #[test]
    fn a_shared_blob_survives_deleting_one_of_its_owners() {
        let (_dir, store) = on_disk();
        let first = store
            .insert_item("uuid-1", &big_image(5), "", 1)
            .expect("a");
        let second = store
            .insert_item("uuid-2", &big_image(5), "", 2)
            .expect("b");
        store.mark_deleted(first, 3).expect("borra el primero");
        assert!(
            store
                .payload_of(second, "public.png")
                .expect("lee")
                .is_some(),
            "el otro ítem sigue necesitando esos bytes"
        );
    }

    #[test]
    fn an_in_memory_store_refuses_what_it_cannot_keep() {
        let store = Store::in_memory().expect("esquema");
        assert!(
            store
                .insert_item("uuid-grande", &big_image(1), "", 1)
                .is_err(),
            "sin carpeta donde escribir, mejor negarse"
        );
    }

    #[test]
    fn the_queue_hands_out_work_and_forgets_it_when_done() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_text("uuid-trabajo", "algo", 1)
            .expect("insert");
        store.enqueue(id, "ocr").expect("encola");
        store
            .enqueue(id, "ocr")
            .expect("encolar dos veces no duplica");
        assert_eq!(
            store.take_pending("ocr", 10, 5).expect("pendientes"),
            vec![id]
        );
        assert!(
            store
                .take_pending("thumbnail", 10, 5)
                .expect("otro tipo")
                .is_empty(),
            "cada cola es la suya"
        );
        store.work_done(id, "ocr").expect("hecho");
        assert!(
            store
                .take_pending("ocr", 10, 5)
                .expect("pendientes")
                .is_empty()
        );
    }

    #[test]
    fn a_job_that_keeps_failing_is_given_up_on() {
        let store = Store::in_memory().expect("esquema");
        let id = store.insert_text("uuid-falla", "algo", 1).expect("insert");
        store.enqueue(id, "ocr").expect("encola");
        for attempt in 1..Store::MAX_ATTEMPTS {
            assert!(
                store
                    .work_failed(id, "ocr", "no se pudo", 0)
                    .expect("falla"),
                "intento {attempt} todavía se reintenta"
            );
        }
        assert!(
            !store
                .work_failed(id, "ocr", "no se pudo", 0)
                .expect("falla"),
            "al agotar los intentos se abandona"
        );
        assert!(
            store
                .take_pending("ocr", 10, 5)
                .expect("pendientes")
                .is_empty()
        );
    }

    #[test]
    fn a_failed_job_waits_before_being_retried() {
        let store = Store::in_memory().expect("esquema");
        let id = store.insert_text("uuid-espera", "algo", 1).expect("insert");
        store.enqueue(id, "ocr").expect("encola");
        store
            .work_failed(id, "ocr", "temporal", 500)
            .expect("falla");
        assert!(
            store
                .take_pending("ocr", 100, 5)
                .expect("aún no")
                .is_empty(),
            "no antes de su hora"
        );
        assert_eq!(store.take_pending("ocr", 500, 5).expect("ya"), vec![id]);
    }

    #[test]
    fn deleted_items_drop_out_of_the_queue() {
        let store = Store::in_memory().expect("esquema");
        let id = store.insert_text("uuid-fuera", "algo", 1).expect("insert");
        store.enqueue(id, "ocr").expect("encola");
        store.mark_deleted(id, 2).expect("borra");
        assert!(
            store
                .take_pending("ocr", 10, 5)
                .expect("pendientes")
                .is_empty(),
            "no se enriquece lo que el usuario borró"
        );
    }

    #[test]
    fn metadata_is_kept_per_key_and_replaced_not_duplicated() {
        let store = Store::in_memory().expect("esquema");
        let id = store
            .insert_text("uuid-meta", "un vídeo", 1)
            .expect("insert");
        store.set_meta(id, "duration", "227").expect("pone");
        store.set_meta(id, "width", "1920").expect("pone");
        store.set_meta(id, "duration", "228").expect("corrige");
        assert_eq!(
            store.meta(id, "duration").expect("lee").as_deref(),
            Some("228")
        );
        assert_eq!(store.all_meta(id).expect("todo").len(), 2);
        assert!(store.meta(id, "artist").expect("lee").is_none());
    }

    #[test]
    fn metadata_goes_away_with_the_item() {
        let store = Store::in_memory().expect("esquema");
        let id = store.insert_text("uuid-meta", "algo", 1).expect("insert");
        store.set_meta(id, "artist", "alguien").expect("pone");
        store.mark_deleted(id, 2).expect("borra");
        assert!(
            store.all_meta(id).expect("todo").is_empty(),
            "los datos derivados son del usuario igual que el contenido"
        );
    }

    fn a_little_history() -> Store {
        let store = Store::in_memory().expect("esquema");
        let rows = [
            ("uuid-1", "primera nota", Kind::Text, 10),
            ("uuid-2", "alguien@ejemplo.test", Kind::Email, 20),
            ("uuid-3", "#FF8800", Kind::Color, 30),
            ("uuid-4", "segunda nota", Kind::Text, 40),
        ];
        for (uuid, text, kind, at) in rows {
            let item = Item {
                kind: Some(kind),
                formats: vec![Format {
                    id: "public.utf8-plain-text".into(),
                    payload: Payload::Inline(text.as_bytes().to_vec()),
                }],
            };
            store.insert_item(uuid, &item, text, at).expect("insert");
        }
        store
    }

    #[test]
    fn the_panel_can_ask_for_the_latest_without_searching_anything() {
        let store = a_little_history();
        let listed = store.list(&Filter::default(), 10, None).expect("listado");
        assert_eq!(listed.len(), 4, "sin término se devuelve el historial");
        assert_eq!(
            listed.first().map(|one| one.preview.as_str()),
            Some("segunda nota"),
            "lo más reciente primero"
        );
    }

    #[test]
    fn the_list_can_be_filtered_by_kind() {
        let store = a_little_history();
        let filter = Filter {
            kinds: vec![Kind::Text],
            ..Default::default()
        };
        let listed = store.list(&filter, 10, None).expect("listado");
        assert_eq!(listed.len(), 2);
        assert!(listed.iter().all(|one| one.kind.as_deref() == Some("text")));
    }

    #[test]
    fn several_kinds_can_be_asked_for_at_once() {
        let store = a_little_history();
        let filter = Filter {
            kinds: vec![Kind::Email, Kind::Color],
            ..Default::default()
        };
        assert_eq!(store.list(&filter, 10, None).expect("listado").len(), 2);
    }

    #[test]
    fn filtering_and_searching_work_together() {
        let store = a_little_history();
        let filter = Filter {
            query: Some("nota".into()),
            kinds: vec![Kind::Text],
            ..Default::default()
        };
        assert_eq!(store.list(&filter, 10, None).expect("listado").len(), 2);

        let narrower = Filter {
            query: Some("nota".into()),
            kinds: vec![Kind::Email],
            ..Default::default()
        };
        assert!(
            store.list(&narrower, 10, None).expect("listado").is_empty(),
            "el filtro y el término se aplican los dos"
        );
    }

    #[test]
    fn only_pinned_can_be_asked_for() {
        let store = a_little_history();
        let listed = store.list(&Filter::default(), 10, None).expect("listado");
        let id = listed.first().expect("hay").id;
        store.pin(id).expect("fija");
        let filter = Filter {
            pinned_only: true,
            ..Default::default()
        };
        let pinned = store.list(&filter, 10, None).expect("listado");
        assert_eq!(pinned.len(), 1);
        assert!(pinned[0].pinned);
    }

    #[test]
    fn the_list_pages_with_the_same_cursor_as_the_search() {
        let store = a_little_history();
        let first = store.list(&Filter::default(), 2, None).expect("página");
        assert_eq!(first.len(), 2);
        let next = store
            .list(
                &Filter::default(),
                2,
                first.last().map(|one| one.modified_at),
            )
            .expect("siguiente");
        assert_eq!(next.len(), 2);
        assert!(next.iter().all(|one| !first.contains(one)));
    }

    #[test]
    fn a_search_with_nothing_usable_returns_nothing_not_everything() {
        let store = a_little_history();
        let filter = Filter {
            query: Some("!!!".into()),
            ..Default::default()
        };
        assert!(
            store.list(&filter, 10, None).expect("listado").is_empty(),
            "pedir buscar algo imposible no puede devolver el historial entero"
        );
    }

    #[test]
    fn deleted_items_never_show_up_in_the_list() {
        let store = a_little_history();
        let listed = store.list(&Filter::default(), 10, None).expect("listado");
        store.mark_deleted(listed[0].id, 99).expect("borra");
        assert_eq!(
            store
                .list(&Filter::default(), 10, None)
                .expect("listado")
                .len(),
            3
        );
    }

    #[test]
    fn retention_takes_the_old_and_leaves_what_was_pinned() {
        let store = a_little_history();
        let listed = store.list(&Filter::default(), 10, None).expect("listado");
        let oldest = listed.last().expect("hay").id;
        store.pin(oldest).expect("fija el más viejo");

        let removed = store.clear_older_than(35).expect("retención");
        assert_eq!(
            removed, 2,
            "se van los de antes del corte que no estén fijados"
        );
        let left = store.list(&Filter::default(), 10, None).expect("listado");
        assert_eq!(left.len(), 2);
        assert!(
            left.iter().any(|one| one.id == oldest),
            "un ítem fijado no lo borra la limpieza"
        );
    }

    #[test]
    fn clearing_everything_still_respects_what_was_pinned() {
        let store = a_little_history();
        let listed = store.list(&Filter::default(), 10, None).expect("listado");
        store.pin(listed[0].id).expect("fija");
        let removed = store.clear_all_unpinned(100).expect("vacía");
        assert_eq!(removed, 3);
        assert_eq!(store.count().expect("cuenta"), 1);
    }

    #[test]
    fn retention_with_nothing_old_enough_removes_nothing() {
        let store = a_little_history();
        assert_eq!(store.clear_older_than(0).expect("retención"), 0);
        assert_eq!(store.count().expect("cuenta"), 4);
    }

    #[test]
    fn a_word_that_is_not_there_finds_nothing() {
        let store = seeded();
        assert!(store.search("berlin").expect("consulta").is_empty());
    }
}

#[cfg(test)]
mod identity {
    use super::*;
    use cp_core::item::Format;

    fn captured(text: &str) -> Item {
        Item {
            kind: None,
            formats: vec![
                Format {
                    id: "public.utf8-plain-text".into(),
                    payload: Payload::Inline(text.as_bytes().to_vec()),
                },
                Format {
                    id: "public.rtf".into(),
                    payload: Payload::Inline(format!("{{\\rtf1 {text}}}").into_bytes()),
                },
            ],
        }
    }

    /// La regresión: `find_by_hash` calculaba el hash del texto desnudo y
    /// `insert_item` guardaba `fingerprint()`. Nunca coincidían, así que la
    /// ruta de captura real no deduplicaba nada.
    #[test]
    fn what_was_captured_is_found_again() {
        let store = Store::in_memory().expect("abre");
        let item = captured("hola");
        store
            .insert_item("uuid-1", &item, "hola", 1)
            .expect("inserta");
        assert_eq!(
            store.find_by_hash(&item).expect("busca"),
            Some(1),
            "lo que guarda insert_item tiene que reconocerlo find_by_hash"
        );
    }

    /// Pegar el mismo texto «como Markdown» o «en plano» produce un contenido
    /// distinto, y eso es un ítem distinto: la identidad mira los bytes.
    #[test]
    fn a_different_rendering_is_a_different_item() {
        let store = Store::in_memory().expect("abre");
        let plain = captured("hola");
        store
            .insert_item("uuid-1", &plain, "hola", 1)
            .expect("inserta");
        assert!(
            store
                .find_by_hash(&Item::plain("**hola**"))
                .expect("busca")
                .is_none()
        );
    }

    /// Un texto guardado sin pasar por el portapapeles no lleva los formatos
    /// que trae una copia real, así que no puede compartir identidad con ella.
    #[test]
    fn a_synthetic_text_is_not_a_captured_one() {
        assert_ne!(
            Item::plain("hola").fingerprint(),
            captured("hola").fingerprint()
        );
    }
}
