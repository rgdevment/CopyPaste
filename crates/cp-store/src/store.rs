use crate::{Error, Result};
use cp_core::item::{Item, Payload};
use cp_core::kind::Kind;
use cp_core::search::{EXCERPT_CHARS, Excerpt, excerpt, fold, terms_of};
use rusqlite::types::ToSql;
use rusqlite::{Connection, OptionalExtension, params, params_from_iter};

fn fts_expression(query: &str) -> Option<String> {
    let terms: Vec<String> = terms_of(query)
        .iter()
        .map(|word| format!("\"{}\"*", word.replace('"', "\"\"")))
        .collect();
    (!terms.is_empty()).then(|| terms.join(" "))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoundIn {
    Text,
    Label,
    App,
    Ocr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snippet {
    pub found_in: FoundIn,
    pub excerpt: Excerpt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    pub id: i64,
    pub modified_at: i64,
    pub created_at: i64,
    pub kind: Option<Kind>,
    pub preview: String,
    pub app: Option<String>,
    pub label: Option<String>,
    pub color: i64,
    pub thumb_path: Option<String>,
    pub paste_count: i64,
    pub last_used_at: Option<i64>,
    pub broken_since: Option<i64>,
    pub pinned: bool,
    pub snippet: Option<Snippet>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    order: Order,
    key: i64,
    id: i64,
}

impl Cursor {
    pub fn encode(self) -> String {
        format!("{}:{}:{}", self.order.as_str(), self.key, self.id)
    }

    pub fn decode(text: &str) -> Option<Cursor> {
        let mut parts = text.split(':');
        let order = Order::from_name(parts.next()?)?;
        let key = parts.next()?.parse().ok()?;
        let id = parts.next()?.parse().ok()?;
        parts.next().is_none().then_some(Cursor { order, key, id })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Page {
    pub rows: Vec<Listed>,
    pub next: Option<Cursor>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Order {
    #[default]
    Recent,
    MostPasted,
    LastUsed,
}

impl Order {
    pub const ALL: [Order; 3] = [Order::Recent, Order::MostPasted, Order::LastUsed];

    pub fn as_str(self) -> &'static str {
        match self {
            Order::Recent => "recent",
            Order::MostPasted => "most-pasted",
            Order::LastUsed => "last-used",
        }
    }

    pub fn from_name(name: &str) -> Option<Order> {
        Order::ALL.into_iter().find(|order| order.as_str() == name)
    }

    fn key(self) -> &'static str {
        match self {
            Order::Recent => "items.modified_at",
            Order::MostPasted => "items.paste_count",
            Order::LastUsed => "COALESCE(items.last_used_at, -1)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Broken {
    #[default]
    Hidden,
    Shown,
    Only,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filter {
    pub query: Option<String>,
    pub label_query: Option<String>,
    pub kinds: Vec<Kind>,
    pub exclude_kinds: Vec<Kind>,
    pub apps: Vec<String>,
    pub exclude_apps: Vec<String>,
    pub colors: Vec<i64>,
    pub pinned_only: bool,
    pub since: Option<i64>,
    pub broken: Broken,
    pub order: Order,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Facet {
    pub kind: Kind,
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppCount {
    pub app: String,
    pub count: i64,
}

struct Clauses {
    joins_index: bool,
    conditions: Vec<String>,
    bound: Vec<Box<dyn ToSql>>,
}

impl Clauses {
    fn of(filter: &Filter, with_kinds: bool, with_apps: bool) -> Option<Self> {
        let mut clauses = Self {
            joins_index: false,
            conditions: vec!["items.deleted_at IS NULL".into()],
            bound: Vec::new(),
        };
        let mut index = Vec::new();
        if let Some(text) = &filter.query {
            index.push(fts_expression(text)?);
        }
        if let Some(label) = &filter.label_query {
            index.push(format!("search_label : ({})", fts_expression(label)?));
        }
        if !index.is_empty() {
            clauses.joins_index = true;
            clauses.conditions.push("items_fts MATCH ?".into());
            clauses.bound.push(Box::new(index.join(" AND ")));
        }
        if with_kinds {
            clauses.kinds(&filter.kinds, false);
        }
        clauses.kinds(&filter.exclude_kinds, true);
        if with_apps {
            clauses.apps("IN", &filter.apps);
        }
        clauses.apps("NOT IN", &filter.exclude_apps);
        if !filter.colors.is_empty() {
            let list = filter
                .colors
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            clauses
                .conditions
                .push(format!("items.card_color IN ({list})"));
        }
        if filter.pinned_only {
            clauses.conditions.push("items.pinned = 1".into());
        }
        if let Some(since) = filter.since {
            clauses.conditions.push("items.modified_at >= ?".into());
            clauses.bound.push(Box::new(since));
        }
        match filter.broken {
            Broken::Hidden => clauses.conditions.push("items.broken_since IS NULL".into()),
            Broken::Only => clauses
                .conditions
                .push("items.broken_since IS NOT NULL".into()),
            Broken::Shown => {}
        }
        Some(clauses)
    }

    fn kinds(&mut self, kinds: &[Kind], excluded: bool) {
        if kinds.is_empty() {
            return;
        }
        let list = kinds
            .iter()
            .map(|kind| format!("'{}'", kind.as_str()))
            .collect::<Vec<_>>()
            .join(", ");
        self.conditions.push(if excluded {
            format!("(items.kind IS NULL OR items.kind NOT IN ({list}))")
        } else {
            format!("items.kind IN ({list})")
        });
    }

    fn apps(&mut self, verb: &str, apps: &[String]) {
        if apps.is_empty() {
            return;
        }
        let marks = vec!["?"; apps.len()].join(", ");
        self.conditions
            .push(format!("items.search_app {verb} ({marks})"));
        for app in apps {
            self.bound.push(Box::new(fold(app)));
        }
    }

    fn source(&self) -> String {
        let conditions = self.conditions.join(" AND ");
        if self.joins_index {
            format!(
                "FROM items_fts CROSS JOIN items ON items.id = items_fts.rowid WHERE {conditions}"
            )
        } else {
            format!("FROM items WHERE {conditions}")
        }
    }
}

pub struct Store {
    db: Connection,
    blobs: Option<crate::Blobs>,
    exposure: Restricted,
}

impl Drop for Store {
    fn drop(&mut self) {
        let _ = self.db.execute_batch("PRAGMA optimize;");
    }
}

impl Store {
    pub fn raw(&self) -> &Connection {
        &self.db
    }

    pub fn blobs(&self) -> Option<&crate::Blobs> {
        self.blobs.as_ref()
    }

    pub fn in_memory() -> Result<Self> {
        let db = Connection::open_in_memory()?;
        crate::schema::create(&db)?;
        Ok(Self {
            db,
            blobs: None,
            exposure: Restricted::Mode(0o600),
        })
    }

    pub fn open(path: &std::path::Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(Error::Io)?;
            restrict(parent, 0o700)?;
        }
        let db = Connection::open(path)?;
        crate::schema::create(&db)?;
        crate::schema::migrate(&db)?;
        let exposure = restrict(path, 0o600)?;
        for side in sidecars(path) {
            if side.exists() {
                restrict(&side, 0o600)?;
            }
        }
        let blobs = path
            .parent()
            .map(|parent| crate::Blobs::at(&parent.join("blobs")))
            .transpose()?;
        Ok(Self {
            db,
            blobs,
            exposure,
        })
    }

    pub fn exposure(&self) -> Restricted {
        self.exposure
    }

    pub fn checkpoint(&self) -> Result<bool> {
        let busy: i64 = self
            .db
            .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| row.get(0))?;
        Ok(busy == 0)
    }

    pub const INTERACTIVE_WAIT: std::time::Duration = std::time::Duration::from_millis(50);

    fn checkpoint_briefly(&self) -> Result<bool> {
        self.db.busy_timeout(Self::INTERACTIVE_WAIT)?;
        let done = self.checkpoint();
        self.db.busy_timeout(std::time::Duration::from_secs(5))?;
        done
    }

    pub fn checkpoint_passive(&self) -> Result<i64> {
        let (_, _, written): (i64, i64, i64) =
            self.db
                .query_row("PRAGMA wal_checkpoint(PASSIVE)", [], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })?;
        Ok(written)
    }

    pub fn autocheckpoint(&self) -> Result<i64> {
        Ok(self
            .db
            .query_row("PRAGMA wal_autocheckpoint", [], |row| row.get(0))?)
    }

    pub fn without_autocheckpoint(&self) -> Result<()> {
        self.db.execute_batch("PRAGMA wal_autocheckpoint = 0;")?;
        Ok(())
    }

    pub fn vacuum_step(&self, pages: u32) -> Result<()> {
        self.db
            .execute_batch(&format!("PRAGMA incremental_vacuum({pages});"))?;
        Ok(())
    }

    pub fn insert_text(&self, uuid: &str, text: &str, created_at: i64) -> Result<i64> {
        if cp_core::item::placement(text.len()) == cp_core::item::Placement::Refused {
            return Err(Error::TooBig { size: text.len() });
        }
        let hash = Item::plain(text).fingerprint() as i64;
        let kind = cp_core::kind::classify_text(text).as_str();
        self.db.execute(
            "INSERT INTO items (uuid, kind, preview_text, created_at, modified_at, updated_at,
                                content_hash, search_text)
             VALUES (?1, ?6, ?2, ?3, ?3, ?3, ?4, ?5)",
            params![uuid, head_of(text), created_at, hash, fold(text), kind],
        )?;
        Ok(self.db.last_insert_rowid())
    }

    pub fn reactivate(&self, id: i64, at: i64) -> Result<()> {
        self.db.execute(
            "UPDATE items SET modified_at = ?2, updated_at = ?2 WHERE id = ?1",
            params![id, at],
        )?;
        Ok(())
    }

    pub fn record_paste(&self, id: i64, at: i64) -> Result<()> {
        self.db.execute(
            "UPDATE items
             SET last_used_at = ?2, updated_at = ?2, paste_count = paste_count + 1
             WHERE id = ?1",
            params![id, at],
        )?;
        Ok(())
    }

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

    pub fn set_ocr_text(&self, id: i64, text: &str, at: i64) -> Result<()> {
        self.db.execute(
            "UPDATE items SET ocr_text = ?2, search_ocr = ?3, updated_at = ?4
             WHERE id = ?1 AND deleted_at IS NULL",
            params![id, text, fold(text), at],
        )?;
        Ok(())
    }

    pub fn ocr_text(&self, id: i64) -> Result<Option<String>> {
        Ok(self
            .db
            .query_row(
                "SELECT COALESCE(ocr_text, search_ocr) FROM items
                 WHERE id = ?1 AND deleted_at IS NULL",
                [id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .filter(|text| !text.is_empty()))
    }

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

    pub fn enqueue(&self, id: i64, job: &str) -> Result<()> {
        self.db.execute(
            "INSERT OR IGNORE INTO pending_work (item_id, job) VALUES (?1, ?2)",
            params![id, job],
        )?;
        Ok(())
    }

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

    pub fn mark_deleted(&self, id: i64, at: i64) -> Result<()> {
        self.erase(id, at)?;
        self.checkpoint_briefly()?;
        Ok(())
    }

    fn erase(&self, id: i64, at: i64) -> Result<()> {
        self.drop_thumb(id)?;
        self.db.execute(
            "UPDATE items
             SET deleted_at = ?2, updated_at = ?2,
                 preview_text = '', search_text = '', search_label = '',
                 search_app = '', search_ocr = '', ocr_text = NULL, label = NULL,
                 app_source = NULL, thumb_path = NULL, content_hash = 0
             WHERE id = ?1",
            params![id, at],
        )?;
        self.release(id)
    }

    fn drop_thumb(&self, id: i64) -> Result<()> {
        let path: Option<String> = self
            .db
            .query_row("SELECT thumb_path FROM items WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .optional()?
            .flatten();
        if let Some(path) = path {
            let _ = crate::blobs::remove_at(std::path::Path::new(&path));
        }
        Ok(())
    }

    fn release(&self, id: i64) -> Result<()> {
        if let Some(blobs) = &self.blobs {
            for digest in self.blobs_of(id)? {
                if self.blob_is_shared(&digest, id)? {
                    continue;
                }
                blobs.remove_if_settled(&digest)?;
            }
        }
        self.db
            .execute("DELETE FROM item_formats WHERE item_id = ?1", [id])?;
        self.db
            .execute("DELETE FROM item_meta WHERE item_id = ?1", [id])?;
        self.db
            .execute("DELETE FROM pending_work WHERE item_id = ?1", [id])?;
        Ok(())
    }

    fn ids_where(&self, condition: &str, bound: &[&dyn ToSql]) -> Result<Vec<i64>> {
        let mut stmt = self
            .db
            .prepare(&format!("SELECT id FROM items WHERE {condition}"))?;
        let rows = stmt.query_map(bound, |row| row.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    fn erase_all(&self, ids: &[i64], at: i64) -> Result<usize> {
        let transaction = self.db.unchecked_transaction()?;
        for id in ids {
            self.erase(*id, at)?;
        }
        transaction.commit()?;
        Ok(ids.len())
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

    pub fn changed_since(&self, version: i64) -> Result<Vec<String>> {
        let mut stmt = self
            .db
            .prepare("SELECT uuid FROM items WHERE updated_at > ?1 ORDER BY updated_at")?;
        let rows = stmt.query_map([version], |row| row.get::<_, String>(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn insert_item(
        &self,
        uuid: &str,
        item: &Item,
        preview: &str,
        created_at: i64,
    ) -> Result<i64> {
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
        let rows = self.rows_of(item)?;
        let transaction = self.db.unchecked_transaction()?;
        self.db.execute(
            "INSERT INTO items (uuid, kind, preview_text, created_at, modified_at, updated_at,
                                content_hash, search_text)
             VALUES (?1, ?2, ?3, ?4, ?4, ?4, ?5, ?6)",
            params![
                uuid,
                kind,
                head_of(preview),
                created_at,
                hash,
                fold(preview)
            ],
        )?;
        let id = self.db.last_insert_rowid();
        self.write_rows(id, &rows)?;
        transaction.commit()?;
        Ok(id)
    }

    fn rows_of(&self, item: &Item) -> Result<Vec<FormatRow>> {
        let mut rows = Vec::with_capacity(item.formats.len());
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
            rows.push(FormatRow {
                id: format.id.clone(),
                size,
                inline,
                blob,
            });
        }
        Ok(rows)
    }

    fn write_rows(&self, id: i64, rows: &[FormatRow]) -> Result<()> {
        for row in rows {
            self.db.execute(
                "INSERT INTO item_formats (item_id, format, size_bytes, inline_data, blob_path)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, row.id, row.size, row.inline, row.blob],
            )?;
        }
        Ok(())
    }

    pub fn update_text(&self, id: i64, text: &str, at: i64) -> Result<()> {
        let payload = Payload::stored(text.as_bytes().to_vec());
        if let Payload::TooBig { size } = payload {
            return Err(Error::TooBig { size });
        }
        let edited = Item {
            kind: Some(cp_core::kind::classify_text(text)),
            formats: vec![cp_core::item::Format {
                id: cp_core::item::SYNTHETIC_TEXT.into(),
                payload,
            }],
        };
        if self.blobs.is_none() && edited.needs_blob_store() {
            let (format, size) = edited.oversized_format().expect("it just said so");
            return Err(Error::NeedsBlobStore { format, size });
        }
        let rows = self.rows_of(&edited)?;
        let transaction = self.db.unchecked_transaction()?;
        let changed = self.db.execute(
            "UPDATE items
             SET kind = ?2, preview_text = ?3, search_text = ?4, search_ocr = '',
                 ocr_text = NULL, content_hash = ?5, thumb_path = NULL, broken_since = NULL,
                 updated_at = ?6
             WHERE id = ?1 AND deleted_at IS NULL",
            params![
                id,
                edited.kind.map(|kind| kind.as_str()),
                head_of(text),
                fold(text),
                edited.fingerprint() as i64,
                at
            ],
        )?;
        if changed == 0 {
            return Err(Error::NoSuchItem { id });
        }
        let previous = self.blobs_of(id)?;
        self.db
            .execute("DELETE FROM item_formats WHERE item_id = ?1", [id])?;
        self.db
            .execute("DELETE FROM item_meta WHERE item_id = ?1", [id])?;
        self.db
            .execute("DELETE FROM pending_work WHERE item_id = ?1", [id])?;
        self.write_rows(id, &rows)?;
        transaction.commit()?;
        if let Some(blobs) = &self.blobs {
            for digest in previous {
                if !self.blob_is_shared(&digest, id)? {
                    blobs.remove_if_settled(&digest)?;
                }
            }
        }
        self.checkpoint_briefly()?;
        Ok(())
    }

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

    pub fn item(&self, id: i64) -> Result<Option<Item>> {
        let kind: Option<Option<String>> = self
            .db
            .query_row(
                "SELECT kind FROM items WHERE id = ?1 AND deleted_at IS NULL",
                [id],
                |row| row.get(0),
            )
            .optional()?;
        let Some(kind) = kind else {
            return Ok(None);
        };
        let mut stmt = self.db.prepare(
            "SELECT format, size_bytes, inline_data, blob_path FROM item_formats
             WHERE item_id = ?1 ORDER BY format",
        )?;
        let rows = stmt.query_map([id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<i64>>(1)?,
                row.get::<_, Option<Vec<u8>>>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })?;
        let mut formats = Vec::new();
        for row in rows {
            let (format, size, inline, blob) = row?;
            let payload = match (inline, blob, size) {
                (Some(bytes), _, _) => Payload::Inline(bytes),
                (None, Some(digest), _) => match self
                    .blobs
                    .as_ref()
                    .and_then(|blobs| blobs.get(&digest).ok().flatten())
                {
                    Some(bytes) => Payload::Blob(bytes),
                    None => Payload::Announced {
                        size: size.map(|s| s as usize),
                    },
                },
                (None, None, Some(size)) => Payload::Announced {
                    size: Some(size as usize),
                },
                (None, None, None) => Payload::Absent,
            };
            formats.push(cp_core::item::Format {
                id: format,
                payload,
            });
        }
        Ok(Some(Item {
            kind: kind.and_then(|name| Kind::from_name(&name)),
            formats,
        }))
    }

    pub fn formats_of(&self, id: i64) -> Result<Vec<String>> {
        let mut stmt = self
            .db
            .prepare("SELECT format FROM item_formats WHERE item_id = ?1 ORDER BY format")?;
        let rows = stmt.query_map([id], |row| row.get::<_, String>(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn mark_broken(&self, id: i64, at: i64) -> Result<()> {
        self.db.execute(
            "UPDATE items SET broken_since = ?2 WHERE id = ?1 AND broken_since IS NULL",
            params![id, at],
        )?;
        Ok(())
    }

    pub fn purge_broken_before(&self, cutoff: i64) -> Result<usize> {
        let doomed = self.ids_where(
            "broken_since IS NOT NULL AND broken_since < ?1 AND pinned = 0",
            &[&cutoff],
        )?;
        for id in &doomed {
            self.release(*id)?;
            self.db.execute("DELETE FROM items WHERE id = ?1", [id])?;
        }
        self.checkpoint_briefly()?;
        Ok(doomed.len())
    }

    pub fn mark_present(&self, id: i64) -> Result<()> {
        self.db
            .execute("UPDATE items SET broken_since = NULL WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn set_pinned(&self, id: i64, pinned: bool, at: i64) -> Result<()> {
        self.db.execute(
            "UPDATE items SET pinned = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, i64::from(pinned), at],
        )?;
        Ok(())
    }

    pub fn set_thumb(&self, id: i64, path: Option<&str>, at: i64) -> Result<()> {
        self.db.execute(
            "UPDATE items SET thumb_path = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, path, at],
        )?;
        Ok(())
    }

    pub fn count(&self) -> Result<i64> {
        Ok(self.db.query_row(
            "SELECT (SELECT COUNT(*) FROM items)
                  - (SELECT COUNT(*) FROM items WHERE deleted_at IS NOT NULL)",
            [],
            |row| row.get(0),
        )?)
    }

    pub fn count_matching(&self, filter: &Filter) -> Result<i64> {
        let Some(clauses) = Clauses::of(filter, true, true) else {
            return Ok(0);
        };
        let sql = format!("SELECT COUNT(*) {}", clauses.source());
        Ok(self
            .db
            .query_row(&sql, params_from_iter(clauses.bound.iter()), |row| {
                row.get(0)
            })?)
    }

    pub fn list(&self, filter: &Filter, limit: usize, after: Option<Cursor>) -> Result<Page> {
        let Some(mut clauses) = Clauses::of(filter, true, true) else {
            return Ok(Page::default());
        };
        let key = filter.order.key();
        let terms = filter.query.as_deref().map(terms_of).unwrap_or_default();
        if let Some(cursor) = after {
            if cursor.order != filter.order {
                return Err(Error::WrongCursor {
                    cursor: cursor.order.as_str(),
                    order: filter.order.as_str(),
                });
            }
            clauses
                .conditions
                .push(format!("({key} < ? OR ({key} = ? AND items.id < ?))"));
            clauses.bound.push(Box::new(cursor.key));
            clauses.bound.push(Box::new(cursor.key));
            clauses.bound.push(Box::new(cursor.id));
        }
        let sql = page_sql(&clauses, key, !terms.is_empty());
        let fetch = i64::try_from(limit).map_or(i64::MAX, |limit| limit.saturating_add(1));
        clauses.bound.push(Box::new(fetch));

        let mut stmt = self.db.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(clauses.bound.iter()), |row| {
            let ocr: String = row.get(13)?;
            let whole: String = row.get(15)?;
            let mut listed = Listed {
                id: row.get(0)?,
                modified_at: row.get(1)?,
                created_at: row.get(2)?,
                kind: row
                    .get::<_, Option<String>>(3)?
                    .and_then(|name| Kind::from_name(&name)),
                preview: row.get(4)?,
                app: row.get(5)?,
                label: row.get(6)?,
                color: row.get(7)?,
                thumb_path: row.get(8)?,
                paste_count: row.get(9)?,
                last_used_at: row.get(10)?,
                broken_since: row.get(11)?,
                pinned: row.get::<_, i64>(12)? == 1,
                snippet: None,
            };
            listed.snippet = snippet_of(&listed, &whole, &ocr, &terms);
            Ok((listed, row.get::<_, i64>(14)?))
        })?;
        let mut keyed: Vec<(Listed, i64)> = rows.collect::<rusqlite::Result<_>>()?;
        let more = keyed.len() > limit;
        keyed.truncate(limit);
        let next = match keyed.last() {
            Some((last, key)) if more => Some(Cursor {
                order: filter.order,
                key: *key,
                id: last.id,
            }),
            _ => None,
        };
        Ok(Page {
            rows: keyed.into_iter().map(|(listed, _)| listed).collect(),
            next,
        })
    }

    pub fn facets(&self, filter: &Filter) -> Result<Vec<Facet>> {
        let Some(mut clauses) = Clauses::of(filter, false, true) else {
            return Ok(Vec::new());
        };
        clauses.conditions.push("items.kind IS NOT NULL".into());
        let sql = format!(
            "SELECT items.kind, COUNT(*) {} GROUP BY items.kind
             ORDER BY COUNT(*) DESC, items.kind",
            clauses.source()
        );
        let mut stmt = self.db.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(clauses.bound.iter()), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        let mut facets = Vec::new();
        for row in rows {
            let (name, count) = row?;
            if let Some(kind) = Kind::from_name(&name) {
                facets.push(Facet { kind, count });
            }
        }
        Ok(facets)
    }

    pub fn distinct_apps(&self, filter: &Filter) -> Result<Vec<AppCount>> {
        let Some(mut clauses) = Clauses::of(filter, true, false) else {
            return Ok(Vec::new());
        };
        clauses
            .conditions
            .push("items.app_source IS NOT NULL".into());
        let sql = format!(
            "SELECT MIN(items.app_source), COUNT(*) {} GROUP BY items.search_app
             ORDER BY COUNT(*) DESC, MIN(items.app_source)",
            clauses.source()
        );
        let mut stmt = self.db.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(clauses.bound.iter()), |row| {
            Ok(AppCount {
                app: row.get(0)?,
                count: row.get(1)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn clear_older_than(&self, cutoff: i64) -> Result<usize> {
        let removed = self.expire(cutoff, cutoff)?;
        self.checkpoint_briefly()?;
        Ok(removed)
    }

    fn expire(&self, cutoff: i64, at: i64) -> Result<usize> {
        let doomed = self.ids_where(
            "modified_at < ?1 AND pinned = 0 AND deleted_at IS NULL",
            &[&cutoff],
        )?;
        self.erase_all(&doomed, at)
    }

    pub fn clear_all_unpinned(&self, at: i64) -> Result<usize> {
        let doomed = self.ids_where("pinned = 0 AND deleted_at IS NULL", &[])?;
        let removed = self.erase_all(&doomed, at)?;
        self.checkpoint_briefly()?;
        Ok(removed)
    }

    pub fn usage(&self) -> Result<Usage> {
        let items = self.count()?;
        let inline: i64 = self.db.query_row(
            "SELECT COALESCE(SUM(size_bytes), 0) FROM item_formats WHERE inline_data IS NOT NULL",
            [],
            |row| row.get(0),
        )?;
        let blobs: i64 = self.db.query_row(
            "SELECT COALESCE(SUM(size_bytes), 0) FROM (
                 SELECT DISTINCT blob_path, size_bytes FROM item_formats
                 WHERE blob_path IS NOT NULL)",
            [],
            |row| row.get(0),
        )?;
        Ok(Usage {
            items,
            bytes: inline + blobs,
        })
    }

    pub fn sweep(&self, policy: &Policy, now: i64) -> Result<Swept> {
        let mut swept = Swept::default();
        if let Some(grace) = policy.broken_for {
            swept.broken = self.purge_broken_before(now - grace)?;
        }
        if let Some(age) = policy.keep_for {
            swept.expired = self.expire(now - age, now)?;
        }
        if let Some(keep) = policy.keep_at_most {
            let excess = (self.count()? - keep).max(0);
            let doomed = self.ids_where(
                "pinned = 0 AND deleted_at IS NULL ORDER BY modified_at, id LIMIT ?1",
                &[&excess],
            )?;
            swept.over_count = self.erase_all(&doomed, now)?;
        }
        if let Some(limit) = policy.bytes_at_most {
            swept.over_bytes = self.evict_until_under(limit, now)?;
        }
        if let Some(blobs) = &self.blobs {
            let referenced = self.referenced_blobs()?;
            swept.orphans = blobs.sweep(&|digest| referenced.contains(digest))?;
        }
        swept.truncated = self.checkpoint()?;
        Ok(swept)
    }

    fn evict_until_under(&self, limit: i64, at: i64) -> Result<usize> {
        let mut evicted = 0;
        let mut left = usize::MAX;
        loop {
            let usage = self.usage()?.bytes;
            if usage <= limit {
                return Ok(evicted);
            }
            let candidates = self.eviction_candidates()?;
            if candidates.len() >= left {
                return Ok(evicted);
            }
            left = candidates.len();
            let mut freed = 0;
            let transaction = self.db.unchecked_transaction()?;
            for (id, bytes) in candidates {
                self.erase(id, at)?;
                evicted += 1;
                freed += bytes;
                if freed >= usage - limit {
                    break;
                }
            }
            transaction.commit()?;
        }
    }

    fn eviction_candidates(&self) -> Result<Vec<(i64, i64)>> {
        let mut stmt = self.db.prepare(
            "SELECT items.id, COALESCE(SUM(COALESCE(LENGTH(f.inline_data), f.size_bytes, 0)), 0)
             FROM items LEFT JOIN item_formats f
               ON f.item_id = items.id AND (f.inline_data IS NOT NULL OR f.blob_path IS NOT NULL)
             WHERE items.pinned = 0 AND items.deleted_at IS NULL
             GROUP BY items.id
             ORDER BY items.modified_at, items.id",
        )?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    fn referenced_blobs(&self) -> Result<std::collections::HashSet<String>> {
        let mut stmt = self
            .db
            .prepare("SELECT DISTINCT blob_path FROM item_formats WHERE blob_path IS NOT NULL")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub const PAGE: usize = 100;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Policy {
    pub keep_for: Option<i64>,
    pub keep_at_most: Option<i64>,
    pub bytes_at_most: Option<i64>,
    pub broken_for: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Swept {
    pub broken: usize,
    pub expired: usize,
    pub over_count: usize,
    pub over_bytes: usize,
    pub orphans: usize,
    pub truncated: bool,
}

pub const PREVIEW_UP_TO: usize = 64 * 1024;

fn head_of(text: &str) -> &str {
    let mut end = text.len().min(PREVIEW_UP_TO);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

struct FormatRow {
    id: String,
    size: Option<i64>,
    inline: Option<Vec<u8>>,
    blob: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Usage {
    pub items: i64,
    pub bytes: i64,
}

pub const PREVIEW_CHARS: usize = 2_000;

fn page_sql(clauses: &Clauses, key: &str, with_query: bool) -> String {
    format!(
        "SELECT items.id, items.modified_at, items.created_at, items.kind,
                SUBSTR(items.preview_text, 1, {preview}), items.app_source, items.label,
                items.card_color, items.thumb_path, items.paste_count, items.last_used_at,
                items.broken_since, items.pinned, {ocr}, page.key, {whole}
         FROM (SELECT items.id AS id, {key} AS key {}
               ORDER BY {key} DESC, items.id DESC LIMIT ?) AS page
         JOIN items ON items.id = page.id
         ORDER BY page.key DESC, page.id DESC",
        clauses.source(),
        preview = PREVIEW_CHARS,
        ocr = if with_query {
            "COALESCE(items.ocr_text, items.search_ocr)"
        } else {
            "''"
        },
        whole = if with_query {
            "items.preview_text"
        } else {
            "''"
        }
    )
}

fn snippet_of(listed: &Listed, whole: &str, ocr: &str, terms: &[String]) -> Option<Snippet> {
    if terms.is_empty() {
        return None;
    }
    let sources = [
        (FoundIn::Text, Some(whole)),
        (FoundIn::Label, listed.label.as_deref()),
        (FoundIn::App, listed.app.as_deref()),
        (FoundIn::Ocr, Some(ocr)),
    ];
    sources.into_iter().find_map(|(found_in, text)| {
        let excerpt = excerpt(text?, terms, EXCERPT_CHARS)?;
        Some(Snippet { found_in, excerpt })
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Restricted {
    Mode(u32),
    InheritedFromProfile,
    Unprotected,
}

fn sidecars(path: &std::path::Path) -> [std::path::PathBuf; 2] {
    ["-wal", "-shm"].map(|suffix| {
        let mut name = path.as_os_str().to_os_string();
        name.push(suffix);
        std::path::PathBuf::from(name)
    })
}

pub fn restrict(path: &std::path::Path, mode: u32) -> Result<Restricted> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(mode);
        std::fs::set_permissions(path, permissions).map_err(Error::Io)?;
        Ok(Restricted::Mode(mode))
    }
    #[cfg(not(unix))]
    {
        let _ = mode;
        let profile = std::env::var_os("USERPROFILE").map(std::path::PathBuf::from);
        Ok(exposure_of(path, profile.as_deref()))
    }
}

#[cfg_attr(unix, allow(dead_code))]
fn exposure_of(path: &std::path::Path, profile: Option<&std::path::Path>) -> Restricted {
    match profile {
        Some(profile) if under(path, profile) => Restricted::InheritedFromProfile,
        _ => Restricted::Unprotected,
    }
}

#[cfg_attr(unix, allow(dead_code))]
fn under(path: &std::path::Path, root: &std::path::Path) -> bool {
    match (path.canonicalize(), root.canonicalize()) {
        (Ok(path), Ok(root)) => path.starts_with(root),
        _ => false,
    }
}

#[cfg(test)]
fn on_disk() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().expect("a folder");
    let store = Store::open(&dir.path().join("history.db")).expect("opened");
    (dir, store)
}

#[cfg(test)]
fn captured(text: &str) -> Item {
    Item {
        kind: Some(Kind::Text),
        formats: vec![
            cp_core::item::Format {
                id: "public.utf8-plain-text".into(),
                payload: Payload::Inline(text.as_bytes().to_vec()),
            },
            cp_core::item::Format {
                id: "public.rtf".into(),
                payload: Payload::Inline(format!("{{\\rtf1 {text}}}").into_bytes()),
            },
        ],
    }
}

#[cfg(test)]
fn search(store: &Store, query: &str) -> Vec<String> {
    let filter = Filter {
        query: Some(query.into()),
        ..Default::default()
    };
    store
        .list(&filter, Store::PAGE, None)
        .expect("queried")
        .rows
        .into_iter()
        .map(|one| one.preview)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cp_core::item::Format;

    #[test]
    fn a_pinned_search_lets_the_index_drive_and_never_scans_the_fts_per_row() {
        let store = seeded();
        for filter in [
            Filter {
                query: Some("r".into()),
                pinned_only: true,
                ..Default::default()
            },
            Filter {
                query: Some("r".into()),
                kinds: vec![Kind::Text],
                ..Default::default()
            },
        ] {
            let clauses = Clauses::of(&filter, true, true).expect("clauses");
            let sql = format!("EXPLAIN QUERY PLAN SELECT COUNT(*) {}", clauses.source());
            let mut stmt = store.db.prepare(&sql).expect("a plan");
            let steps: Vec<String> = stmt
                .query_map(params_from_iter(clauses.bound.iter()), |row| {
                    row.get::<_, String>(3)
                })
                .expect("a plan")
                .map(|step| step.expect("a step"))
                .collect();
            assert!(
                steps
                    .first()
                    .is_some_and(|first| first.contains("items_fts")),
                "el FTS tiene que ser el bucle exterior: {steps:?}"
            );
            assert!(
                !steps.iter().any(|step| step.contains("items_pinned")),
                "the partial pinned index made the FTS get swept on every row: {steps:?}"
            );
        }
    }

    fn seeded() -> Store {
        let store = Store::in_memory().expect("schema");
        for (at, text) in [
            "the café on the corner",
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

    #[test]
    fn the_four_cases_that_2x_gets_wrong() {
        let store = seeded();
        for (query, expected) in [
            ("cafe", "the café on the corner"),
            ("strasse", "Straße Hauptbahnhof"),
            ("encyclopaedia", "encyclopædia britannica"),
            ("lodz", "Łódź centrum"),
        ] {
            let hits = search(&store, query);
            assert!(
                hits.iter().any(|hit| hit == expected),
                "searching «{query}» did not turn up «{expected}»: {hits:?}"
            );
        }
    }

    #[test]
    fn it_works_in_both_directions() {
        let store = seeded();
        for (query, expected) in [
            ("café", "the café on the corner"),
            ("Straße", "Straße Hauptbahnhof"),
            ("encyclopædia", "encyclopædia britannica"),
            ("Łódź", "Łódź centrum"),
            ("Gonçalves", "Peçanha e Gonçalves"),
        ] {
            let hits = search(&store, query);
            assert!(
                hits.iter().any(|hit| hit == expected),
                "searching «{query}» did not turn up «{expected}»: {hits:?}"
            );
        }
    }

    #[test]
    fn the_stored_text_keeps_its_accents() {
        let store = seeded();
        let hits = search(&store, "cafe");
        assert_eq!(
            hits.first().map(String::as_str),
            Some("the café on the corner"),
            "it is searched without accents but shown just as it was copied"
        );
    }

    fn sample_item() -> Item {
        Item {
            kind: Some(cp_core::kind::Kind::Text),
            formats: vec![
                Format {
                    id: "public.utf8-plain-text".into(),
                    payload: Payload::Inline(b"hello".to_vec()),
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
        let store = Store::in_memory().expect("schema");
        let big = Item {
            kind: None,
            formats: vec![Format {
                id: "public.png".into(),
                payload: Payload::Blob(vec![0u8; 100_000]),
            }],
        };
        assert!(
            store.insert_item("uuid-grande", &big, "", 1).is_err(),
            "better to refuse than store an item without its bytes"
        );
        assert_eq!(store.count().expect("counted"), 0);
    }

    #[test]
    fn two_images_with_no_preview_are_two_items() {
        let store = Store::in_memory().expect("schema");
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
            .expect("prepared")
            .query_map([], |row| row.get(0))
            .expect("queried")
            .map(|row| row.expect("a row"))
            .collect();
        assert_ne!(
            hashes[0], hashes[1],
            "hashing the empty preview would make them the same"
        );
    }

    #[test]
    fn an_item_keeps_every_format_it_was_offered() {
        let store = Store::in_memory().expect("schema");
        let id = store
            .insert_item("uuid-multi", &sample_item(), "hello", 1)
            .expect("insert");
        let formats = store.formats_of(id).expect("formats");
        assert_eq!(formats.len(), 4, "all four rows, including the empty ones");
        assert!(formats.contains(&"com.apple.icns".to_string()));
        assert!(formats.contains(&"fndf".to_string()));
    }

    #[test]
    fn the_same_content_is_found_by_its_hash() {
        let store = Store::in_memory().expect("schema");
        store.insert_text("uuid-a", "repetido", 1).expect("insert");
        assert!(
            store
                .find_by_hash(&Item::plain("repetido"))
                .expect("searched")
                .is_some()
        );
        assert!(
            store
                .find_by_hash(&Item::plain("distinto"))
                .expect("searched")
                .is_none()
        );
    }

    #[test]
    fn deleting_an_item_takes_its_formats_with_it() {
        let store = Store::in_memory().expect("schema");
        let id = store
            .insert_item("uuid-cascade", &sample_item(), "hello", 1)
            .expect("insert");
        store.mark_broken(id, 10).expect("marked");
        store.purge_broken_before(20).expect("purged");
        assert_eq!(store.formats_of(id).expect("formats").len(), 0);
    }

    #[test]
    fn a_broken_item_survives_until_its_time_is_up() {
        let store = Store::in_memory().expect("schema");
        let id = store
            .insert_text("uuid-roto", "archivo ido", 1)
            .expect("insert");
        store.mark_broken(id, 100).expect("marked");
        assert_eq!(store.purge_broken_before(50).expect("purged"), 0);
        assert_eq!(
            store.count().expect("counted"),
            1,
            "the deadline has not been met yet"
        );
        assert_eq!(store.purge_broken_before(150).expect("purged"), 1);
        assert_eq!(store.count().expect("counted"), 0);
    }

    #[test]
    fn marking_a_broken_item_twice_does_not_restart_its_clock() {
        let store = Store::in_memory().expect("schema");
        let id = store
            .insert_text("uuid-roto", "archivo ido", 1)
            .expect("insert");
        store.mark_broken(id, 100).expect("first");
        store.mark_broken(id, 900).expect("second");
        assert_eq!(
            store.purge_broken_before(150).expect("purged"),
            1,
            "it is the first time it was seen broken that counts, not the last"
        );
    }

    #[test]
    fn a_pinned_item_is_never_purged_even_when_broken() {
        let store = Store::in_memory().expect("schema");
        let id = store
            .insert_text("uuid-fijado", "importante", 1)
            .expect("insert");
        store.set_pinned(id, true, 0).expect("pinned");
        store.mark_broken(id, 100).expect("marked");
        assert_eq!(store.purge_broken_before(9999).expect("purged"), 0);
        assert_eq!(store.count().expect("counted"), 1);
    }

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
                .list(
                    &Filter {
                        query: Some(query.into()),
                        ..Default::default()
                    },
                    10,
                    None,
                )
                .unwrap_or_else(|why| panic!("«{query}» broke the search: {why}"));
        }
    }

    #[test]
    fn an_empty_search_returns_nothing_rather_than_everything() {
        let store = seeded();
        for empty in ["", "   ", "\t", "-", "!!", "***"] {
            assert!(
                search(&store, empty).is_empty(),
                "«{empty}» should not return anything"
            );
        }
    }

    #[test]
    fn the_punctuation_around_a_word_does_not_hide_it() {
        let store = seeded();
        for query in ["-café", "^café", "(café)", "«café»", "café!"] {
            let hits = search(&store, query);
            assert!(
                hits.iter().any(|hit| hit.contains("café")),
                "«{query}» did not find the café"
            );
        }
    }

    #[test]
    fn scripts_that_are_not_latin_go_in_and_come_out() {
        let store = Store::in_memory().expect("schema");
        for (at, text) in [
            "日本語のテキスト",
            "Привет мир",
            "مرحبا بالعالم",
            "🎉 party 🎊",
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
            ("party", "🎉 party 🎊"),
            ("한국어", "한국어 텍스트"),
        ] {
            let hits = search(&store, query);
            assert!(
                hits.iter().any(|hit| hit == expected),
                "searching «{query}» was missing «{expected}»: {hits:?}"
            );
        }
    }

    #[test]
    fn a_very_long_text_is_stored_and_found() {
        let store = Store::in_memory().expect("schema");
        let long = format!("{} needle {}", "hay ".repeat(50_000), "hay ".repeat(50_000));
        store.insert_text("uuid-long", &long, 1).expect("insert");
        assert_eq!(search(&store, "needle").len(), 1);
    }

    #[test]
    fn the_same_uuid_twice_is_refused_not_duplicated() {
        let store = Store::in_memory().expect("schema");
        store
            .insert_text("uuid-unique", "first", 1)
            .expect("insert");
        assert!(
            store.insert_text("uuid-unique", "second", 2).is_err(),
            "the uuid is unique by contract"
        );
        assert_eq!(store.count().expect("counted"), 1);
    }

    #[test]
    fn marking_an_item_that_does_not_exist_is_not_a_failure() {
        let store = Store::in_memory().expect("schema");
        store
            .mark_broken(9999, 1)
            .expect("it does not exist, and nothing happens");
        assert_eq!(store.count().expect("counted"), 0);
    }

    #[test]
    fn a_purge_with_nothing_to_purge_removes_nothing() {
        let store = seeded();
        let before = store.count().expect("counted");
        assert_eq!(store.purge_broken_before(-1).expect("purged"), 0);
        assert_eq!(store.purge_broken_before(i64::MAX).expect("purged"), 0);
        assert_eq!(store.count().expect("counted"), before);
    }

    #[test]
    fn an_item_with_no_formats_at_all_is_still_an_item() {
        let store = Store::in_memory().expect("schema");
        let empty = Item {
            kind: None,
            formats: vec![],
        };
        let id = store
            .insert_item("uuid-empty", &empty, "", 1)
            .expect("insert");
        assert_eq!(store.formats_of(id).expect("formats").len(), 0);
        assert_eq!(store.count().expect("counted"), 1);
    }

    #[test]
    fn a_search_that_matches_everything_still_returns_one_page() {
        let store = Store::in_memory().expect("schema");
        for at in 0..250 {
            store
                .insert_text(&format!("uuid-{at}"), &format!("common {at}"), at)
                .expect("insert");
        }
        let page = search(&store, "common");
        assert_eq!(page.len(), Store::PAGE);
    }

    #[test]
    fn the_cursor_walks_a_search_without_repeating_or_skipping() {
        let store = Store::in_memory().expect("schema");
        for at in 0..25 {
            store
                .insert_text(&format!("uuid-{at}"), &format!("cursor {at}"), at)
                .expect("insert");
        }
        let filter = Filter {
            query: Some("cursor".into()),
            ..Default::default()
        };
        let mut seen = Vec::new();
        let mut after = None;
        loop {
            let page = store.list(&filter, 10, after).expect("queried");
            seen.extend(page.rows.into_iter().map(|one| one.preview));
            match page.next {
                Some(cursor) => after = Some(cursor),
                None => break,
            }
        }
        assert_eq!(seen.len(), 25, "went through it all without getting stuck");
        let mut unique = seen.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), 25, "without repeating");
    }

    #[test]
    fn copying_something_again_lifts_it_instead_of_duplicating_it() {
        let store = Store::in_memory().expect("schema");
        let first = store
            .insert_text("uuid-a", "the old one", 10)
            .expect("insert");
        store
            .insert_text("uuid-b", "the new one", 20)
            .expect("insert");

        let before = search(&store, "the");
        assert_eq!(before.first().map(String::as_str), Some("the new one"));

        store.reactivate(first, 30).expect("copied again");
        let after = search(&store, "the");
        assert_eq!(
            after.first().map(String::as_str),
            Some("the old one"),
            "copying something again lifts it to the top"
        );
    }

    #[test]
    fn a_label_can_be_searched_for() {
        let store = Store::in_memory().expect("schema");
        let id = store
            .insert_text("uuid-etq", "some random text", 1)
            .expect("insert");
        assert!(search(&store, "invoice").is_empty());
        store
            .set_label(id, Some("Invoice May"), 2)
            .expect("labelled");
        let hits = search(&store, "invoice");
        assert_eq!(hits.len(), 1, "the label gets into the index");
    }

    #[test]
    fn the_source_application_can_be_searched_for() {
        let store = Store::in_memory().expect("schema");
        let id = store
            .insert_text("uuid-app", "something copied", 1)
            .expect("insert");
        store.set_source(id, "Safari", 2).expect("sourced");
        assert_eq!(search(&store, "safari").len(), 1);
    }

    #[test]
    fn a_label_with_accents_is_found_without_them() {
        let store = Store::in_memory().expect("schema");
        let id = store
            .insert_text("uuid-tilde", "content", 1)
            .expect("insert");
        store
            .set_label(id, Some("Design Meeting"), 2)
            .expect("labelled");
        assert_eq!(search(&store, "meeting").len(), 1);
        assert_eq!(search(&store, "design").len(), 1);
    }

    #[test]
    fn removing_a_label_takes_it_out_of_the_index() {
        let store = Store::in_memory().expect("schema");
        let id = store
            .insert_text("uuid-quita", "content", 1)
            .expect("insert");
        store.set_label(id, Some("temporary"), 2).expect("set");
        assert_eq!(search(&store, "temporary").len(), 1);
        store.set_label(id, None, 3).expect("cleared");
        assert!(search(&store, "temporary").is_empty());
    }

    #[test]
    fn deleting_hides_the_item_from_everything_the_user_can_see() {
        let store = seeded();
        let id = store
            .insert_text("uuid-secreto", "the bank password", 500)
            .expect("insert");
        let before = store.count().expect("counted");

        store.mark_deleted(id, 600).expect("removed");

        assert_eq!(
            store.count().expect("counted"),
            before - 1,
            "stops counting"
        );
        assert!(
            search(&store, "password").is_empty(),
            "it can no longer be found"
        );
        assert!(
            store
                .find_by_hash(&Item::plain("the bank password"))
                .expect("hash")
                .is_none(),
            "copying it again must create a new item, not resurrect the tombstone"
        );
    }

    #[test]
    fn deleting_an_item_takes_its_thumbnail_off_the_disk() {
        let (dir, store) = on_disk();
        let made = dir.path().join("a.png");
        std::fs::write(&made, b"not a real png, but it has heft").expect("written");
        let id = store
            .insert_item("uuid-with-thumbnail", &sample_item(), "something", 1)
            .expect("insert");
        store
            .set_thumb(id, Some(&made.to_string_lossy()), 2)
            .expect("a thumbnail");
        store.mark_deleted(id, 3).expect("removed");
        assert!(!made.exists(), "the thumbnail survived the deletion");
    }

    #[test]
    fn emptying_the_history_takes_every_thumbnail_with_it() {
        let (dir, store) = on_disk();
        let made = dir.path().join("another.png");
        std::fs::write(&made, b"this is not a png either").expect("written");
        let id = store
            .insert_item("uuid-emptied", &sample_item(), "something", 1)
            .expect("insert");
        store
            .set_thumb(id, Some(&made.to_string_lossy()), 2)
            .expect("a thumbnail");
        store.clear_all_unpinned(3).expect("emptied");
        assert!(!made.exists(), "emptying left the thumbnail on disk");
    }

    #[test]
    fn a_deleted_item_leaves_no_content_behind() {
        let store = Store::in_memory().expect("schema");
        let item = sample_item();
        let id = store
            .insert_item("uuid-borrado", &item, "plain text", 1)
            .expect("insert");
        store.set_label(id, Some("label"), 2).expect("labelled");
        store.mark_deleted(id, 3).expect("removed");

        let (preview, search, label): (String, String, Option<String>) = store
            .db
            .query_row(
                "SELECT preview_text, search_text, label FROM items WHERE id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("queried");
        assert_eq!(preview, "", "the plain content goes away");
        assert_eq!(search, "", "and so does its copy in the index");
        assert_eq!(label, None);
        assert_eq!(
            store.formats_of(id).expect("formats").len(),
            0,
            "the bytes of the formats go away with the item"
        );
    }

    #[test]
    fn the_tombstone_still_tells_the_sync_what_happened() {
        let store = Store::in_memory().expect("schema");
        let id = store
            .insert_text("uuid-tumba", "goes away", 1)
            .expect("insert");
        store.mark_deleted(id, 50).expect("removed");
        assert!(
            store
                .changed_since(40)
                .expect("changes")
                .contains(&"uuid-tumba".to_string()),
            "without this, another machine resurrects it"
        );
    }

    #[test]
    fn only_what_changed_after_the_mark_is_reported() {
        let store = Store::in_memory().expect("schema");
        let old = store
            .insert_text("uuid-old", "ancient", 10)
            .expect("insert");
        store
            .insert_text("uuid-new", "recent", 100)
            .expect("insert");
        let changed = store.changed_since(50).expect("changes");
        assert_eq!(changed, vec!["uuid-new".to_string()]);

        store.reactivate(old, 200).expect("copied again");
        let after = store.changed_since(50).expect("changes");
        assert_eq!(
            after,
            vec!["uuid-new".to_string(), "uuid-old".to_string()],
            "ordered by version, and the recopy now counts"
        );
    }

    #[test]
    fn copying_something_again_does_not_inflate_the_paste_counter() {
        let store = Store::in_memory().expect("schema");
        let id = store
            .insert_text("uuid-recopiado", "something", 1)
            .expect("insert");
        for at in 2..10 {
            store.reactivate(id, at).expect("copied again");
        }
        assert_eq!(
            store.paste_count(id).expect("counted"),
            0,
            "copying again is not pasting, and the card's ×N shows it"
        );
    }

    #[test]
    fn pasting_from_the_history_is_what_counts() {
        let store = Store::in_memory().expect("schema");
        let id = store
            .insert_text("uuid-pegado", "something", 1)
            .expect("insert");
        store.record_paste(id, 2).expect("pasted");
        store.record_paste(id, 3).expect("pasted");
        assert_eq!(store.paste_count(id).expect("counted"), 2);
    }

    #[test]
    fn pasting_does_not_move_the_item_up_the_list() {
        let store = a_little_history();
        let listed = store
            .list(&Filter::default(), 10, None)
            .expect("listed")
            .rows;
        let oldest = listed.last().expect("there is one").id;
        store.record_paste(oldest, 999).expect("pasted");
        let after = store
            .list(&Filter::default(), 10, None)
            .expect("listed")
            .rows;
        assert_eq!(
            after.last().map(|one| one.id),
            Some(oldest),
            "pasting counts, but it does not reorder the history"
        );
    }

    #[test]
    fn the_colour_can_be_set_and_filtered_by() {
        let store = a_little_history();
        let listed = store
            .list(&Filter::default(), 10, None)
            .expect("listed")
            .rows;
        store.set_color(listed[0].id, 3, 100).expect("color");
        let filter = Filter {
            colors: vec![3],
            ..Default::default()
        };
        let coloured = store.list(&filter, 10, None).expect("listed").rows;
        assert_eq!(coloured.len(), 1);
        assert_eq!(coloured[0].id, listed[0].id);
    }

    #[test]
    fn an_image_becomes_findable_by_what_is_written_inside_it() {
        let store = Store::in_memory().expect("schema");
        let image = Item {
            kind: Some(cp_core::kind::Kind::Image),
            formats: vec![Format {
                id: "public.png".into(),
                payload: Payload::Inline(vec![137, 80, 78, 71]),
            }],
        };
        let id = store
            .insert_item("uuid-capture", &image, "", 1)
            .expect("insert");

        assert!(
            search(&store, "order").is_empty(),
            "it has not been through OCR yet"
        );
        assert_eq!(store.pending_ocr(10).expect("pending"), vec![id]);

        store
            .set_ocr_text(id, "Order AB-4417 delivery 12 March", 2)
            .expect("ocr");

        assert_eq!(
            search(&store, "order").len(),
            1,
            "a screenshot can be found by what it says inside"
        );
        assert_eq!(search(&store, "AB-4417").len(), 1);
        assert!(
            store.pending_ocr(10).expect("pending").is_empty(),
            "it is no longer pending"
        );
    }

    #[test]
    fn the_ocr_text_is_folded_like_everything_else() {
        let store = Store::in_memory().expect("schema");
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
        store.set_ocr_text(id, "Meeting in Munich", 2).expect("ocr");
        assert_eq!(search(&store, "meeting").len(), 1);
        assert_eq!(search(&store, "munich").len(), 1);
        assert_eq!(
            store.ocr_text(id).expect("read").as_deref(),
            Some("Meeting in Munich"),
            "what gets pasted or shown keeps its capitals and accents"
        );
    }

    #[test]
    fn the_recognised_text_is_gone_with_the_item_and_with_an_edit() {
        let store = Store::in_memory().expect("schema");
        let image = Item {
            kind: Some(cp_core::kind::Kind::Image),
            formats: vec![Format {
                id: "public.png".into(),
                payload: Payload::Inline(vec![1]),
            }],
        };
        let id = store
            .insert_item("uuid-raw", &image, "", 1)
            .expect("insert");
        assert_eq!(store.ocr_text(id).expect("read"), None);
        store.set_ocr_text(id, "Invoice 77", 2).expect("ocr");
        store.update_text(id, "already text", 3).expect("edited");
        assert_eq!(
            store.ocr_text(id).expect("read"),
            None,
            "the edited text replaces the image and what was read inside it"
        );
        store.set_ocr_text(id, "Invoice 78", 4).expect("ocr");
        assert_eq!(
            store.ocr_text(id).expect("read").as_deref(),
            Some("Invoice 78")
        );
        store.mark_deleted(id, 5).expect("removed");
        assert_eq!(store.ocr_text(id).expect("read"), None);
        assert!(
            store
                .raw()
                .query_row(
                    "SELECT ocr_text IS NULL AND search_ocr = '' FROM items WHERE id = ?1",
                    [id],
                    |row| row.get::<_, bool>(0),
                )
                .expect("queried"),
            "deleting clears both columns, not just the searchable one"
        );
        assert!(
            store.set_ocr_text(id, "late", 6).is_ok(),
            "an OCR that arrives after the deletion resurrects nothing"
        );
        assert_eq!(store.ocr_text(id).expect("read"), None);
    }

    #[test]
    fn deleting_takes_the_recognised_text_with_it() {
        let store = Store::in_memory().expect("schema");
        let image = Item {
            kind: Some(cp_core::kind::Kind::Image),
            formats: vec![Format {
                id: "public.png".into(),
                payload: Payload::Inline(vec![1]),
            }],
        };
        let id = store
            .insert_item("uuid-secret", &image, "", 1)
            .expect("insert");
        store.set_ocr_text(id, "recovery key 8842", 2).expect("ocr");
        store.mark_deleted(id, 3).expect("removed");
        assert!(
            search(&store, "recovery").is_empty(),
            "what was read inside the image is also the user's content"
        );
    }

    fn nowhere_on_disk(dir: &std::path::Path, words: &[&str]) {
        for file in ["history.db", "history.db-wal"] {
            let bytes = std::fs::read(dir.join(file)).expect("can be read");
            for word in words {
                assert!(
                    !bytes
                        .windows(word.len())
                        .any(|window| window == word.as_bytes()),
                    "«{word}» is still legible in {file}"
                );
            }
        }
    }

    #[test]
    fn a_deleted_secret_is_not_left_lying_in_the_write_ahead_log() {
        let dir = tempfile::tempdir().expect("a folder");
        let path = dir.path().join("history.db");
        let secret = "zqxjkvbnm7hunter2 bank mail";
        let store = Store::open(&path).expect("opened");
        let id = store.insert_text("uuid-secret", secret, 1).expect("insert");

        store.mark_deleted(id, 2).expect("removed");

        nowhere_on_disk(dir.path(), &["zqxjkvbnm7hunter2", "bank", "mail"]);
    }

    #[test]
    fn the_search_index_forgets_every_token_of_what_was_removed() {
        let dir = tempfile::tempdir().expect("a folder");
        let store = Store::open(&dir.path().join("history.db")).expect("opened");
        type Removal = dyn Fn(&Store, i64);
        let cases: [(&str, &Removal); 4] = [
            ("qwzplk1secret", &|store, id| {
                store.mark_deleted(id, 9).expect("removed")
            }),
            ("qwzplk2secret", &|store, id| {
                store.update_text(id, "innocent", 9).expect("edited")
            }),
            ("qwzplk3secret", &|store, id| {
                store.set_label(id, Some("qwzplk3label"), 8).expect("set");
                store.set_label(id, None, 9).expect("cleared");
            }),
            ("qwzplk4secret", &|store, id| {
                store.mark_broken(id, 8).expect("broken");
                store.purge_broken_before(10).expect("purged");
            }),
        ];
        for (at, (word, remove)) in cases.into_iter().enumerate() {
            let id = store
                .insert_text(&format!("uuid-{at}"), word, 1)
                .expect("insert");
            remove(&store, id);
        }
        nowhere_on_disk(
            dir.path(),
            &[
                "qwzplk1secret",
                "qwzplk2secret",
                "qwzplk3label",
                "qwzplk4secret",
            ],
        );
    }

    #[test]
    fn what_is_written_survives_closing_the_application() {
        let dir = tempfile::tempdir().expect("a folder");
        let path = dir.path().join("sub").join("history.db");

        {
            let store = Store::open(&path).expect("opened");
            store
                .insert_text("uuid-persists", "survives", 1)
                .expect("insert");
            store.checkpoint().expect("checkpoint");
        }

        let reopened = Store::open(&path).expect("reopened");
        assert_eq!(reopened.count().expect("counted"), 1);
        assert_eq!(
            search(&reopened, "survives").len(),
            1,
            "and the index survives too"
        );
    }

    #[cfg(unix)]
    #[test]
    fn the_write_ahead_log_is_as_private_as_the_database() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().expect("a folder");
        let path = dir.path().join("history.db");
        let store = Store::open(&path).expect("opened");
        store
            .insert_text("uuid-private", "password", 1)
            .expect("insert");

        let wal = sidecars(&path)
            .into_iter()
            .find(|side| side.exists())
            .expect("the WAL exists while the database is open");
        let mode = std::fs::metadata(&wal).expect("wal").permissions().mode() & 0o777;
        assert_eq!(
            mode, 0o600,
            "what was just copied lives here before it lives in the database"
        );
    }

    #[test]
    fn the_sidecars_are_named_after_the_database() {
        let [wal, shm] = sidecars(std::path::Path::new("/data/history.db"));
        assert!(wal.to_string_lossy().ends_with("history.db-wal"));
        assert!(shm.to_string_lossy().ends_with("history.db-shm"));
    }

    #[cfg(unix)]
    #[test]
    fn the_history_is_not_readable_by_other_users() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().expect("a folder");
        let path = dir.path().join("data").join("history.db");
        let store = Store::open(&path).expect("opened");
        store
            .insert_text("uuid-private", "password", 1)
            .expect("insert");
        assert_eq!(store.exposure(), Restricted::Mode(0o600));
        drop(store);

        let file = std::fs::metadata(&path)
            .expect("a file")
            .permissions()
            .mode()
            & 0o777;
        let folder = std::fs::metadata(path.parent().expect("a parent"))
            .expect("a folder")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(file, 0o600, "only its owner");
        assert_eq!(folder, 0o700, "and the folder the same way");
    }

    #[cfg(windows)]
    #[test]
    fn on_windows_what_protects_the_history_is_living_under_the_profile() {
        let Some(profile) = std::env::var_os("USERPROFILE") else {
            return;
        };
        let dir = tempfile::Builder::new()
            .prefix("copypaste-")
            .tempdir_in(profile)
            .expect("a folder");
        let path = dir.path().join("data").join("history.db");
        let store = Store::open(&path).expect("opened");
        store
            .insert_text("uuid-private", "password", 1)
            .expect("insert");
        assert_eq!(store.exposure(), Restricted::InheritedFromProfile);
    }

    fn a_profile_with(entry: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let profile = tempfile::tempdir().expect("profile");
        let path = profile.path().join(entry);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("a folder");
        }
        std::fs::write(&path, b"x").expect("a file");
        (profile, path)
    }

    #[test]
    fn a_history_under_the_profile_is_covered_by_its_permissions() {
        let (profile, history) = a_profile_with("AppData/Local/history.db");
        assert_eq!(
            exposure_of(&history, Some(profile.path())),
            Restricted::InheritedFromProfile
        );
    }

    #[test]
    fn a_history_outside_the_profile_is_unprotected() {
        let (profile, _) = a_profile_with("AppData/history.db");
        let (_shared, elsewhere) = a_profile_with("compartido/history.db");
        assert_eq!(
            exposure_of(&elsewhere, Some(profile.path())),
            Restricted::Unprotected
        );
    }

    #[test]
    fn without_a_profile_nothing_is_promised() {
        let (_profile, history) = a_profile_with("history.db");
        assert_eq!(exposure_of(&history, None), Restricted::Unprotected);
    }

    #[test]
    fn a_path_that_does_not_exist_is_never_taken_for_protected() {
        let profile = tempfile::tempdir().expect("profile");
        let ghost = profile.path().join("not-yet").join("history.db");
        assert!(!under(&ghost, profile.path()));
        assert_eq!(
            exposure_of(&ghost, Some(profile.path())),
            Restricted::Unprotected
        );
    }

    #[test]
    fn being_above_is_not_being_inside() {
        let (profile, history) = a_profile_with("AppData/history.db");
        assert!(under(&history, profile.path()));
        assert!(!under(profile.path(), &history));
    }

    #[test]
    fn a_sibling_that_merely_starts_alike_is_outside() {
        let root = tempfile::tempdir().expect("a root");
        let ann = root.path().join("ann");
        let annabel = root.path().join("annabel");
        std::fs::create_dir_all(&ann).expect("ann");
        std::fs::create_dir_all(&annabel).expect("annabel");
        let history = annabel.join("history.db");
        std::fs::write(&history, b"x").expect("a file");
        assert!(!under(&history, &ann), "a text prefix is not a path prefix");
    }

    #[test]
    fn a_checkpoint_leaves_the_data_in_the_main_file() {
        let dir = tempfile::tempdir().expect("a folder");
        let path = dir.path().join("history.db");
        let store = Store::open(&path).expect("opened");
        for at in 0..50 {
            store
                .insert_text(&format!("uuid-{at}"), &format!("line {at}"), at)
                .expect("insert");
        }
        store.checkpoint().expect("checkpoint");
        let wal = path.with_extension("db-wal");
        let wal_size = std::fs::metadata(&wal).map(|m| m.len()).unwrap_or(0);
        assert!(
            wal_size == 0 || !wal.exists(),
            "after the checkpoint the WAL is left empty, not at {wal_size} bytes"
        );
    }

    #[test]
    fn an_incremental_vacuum_actually_frees_pages() {
        let store = Store::in_memory().expect("schema");
        for at in 0..2000 {
            let id = store
                .insert_text(&format!("uuid-{at}"), &"x".repeat(200), at)
                .expect("insert");
            store.mark_broken(id, at).expect("marked");
        }
        store.purge_broken_before(i64::MAX).expect("purged");

        let before: i64 = store
            .db
            .query_row("PRAGMA freelist_count", [], |row| row.get(0))
            .expect("queried");
        assert!(
            before > 0,
            "deleting so many rows has to leave free pages, not {before}"
        );

        store
            .vacuum_step(before as u32)
            .expect("empties the free pages");

        let after: i64 = store
            .db
            .query_row("PRAGMA freelist_count", [], |row| row.get(0))
            .expect("queried");
        assert!(
            after < before,
            "incremental_vacuum has to shrink the freelist: before {before}, after {after}"
        );
    }

    #[test]
    fn reopening_keeps_the_pragmas_that_protect_the_data() {
        let dir = tempfile::tempdir().expect("a folder");
        let path = dir.path().join("history.db");
        drop(Store::open(&path).expect("opened"));
        let store = Store::open(&path).expect("reopened");
        let vacuum: i64 = store
            .db
            .query_row("PRAGMA auto_vacuum", [], |row| row.get(0))
            .expect("queried");
        assert_eq!(
            vacuum, 2,
            "the mode gets saved in the file and it should stay that way"
        );
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
            .insert_item("uuid-image", &big_image(7), "", 1)
            .expect("insert");
        let bytes = store
            .payload_of(id, "public.png")
            .expect("read")
            .expect("is there");
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
            .expect("a folder")
            .count();
        assert_eq!(files, 1, "the name is the content, so it is the same file");
    }

    #[test]
    fn blobs_of_reports_the_digests_the_item_has() {
        let (_dir, store) = on_disk();
        let id = store
            .insert_item("uuid-blobs", &big_image(4), "", 1)
            .expect("insert");
        assert_eq!(store.blobs_of(id).expect("blobs").len(), 1);
    }

    #[test]
    fn blobs_of_an_item_with_no_blobs_is_empty() {
        let store = Store::in_memory().expect("schema");
        let id = store
            .insert_text("uuid-no-blobs", "just text", 1)
            .expect("insert");
        assert!(store.blobs_of(id).expect("blobs").is_empty());
    }

    #[test]
    fn a_blob_used_by_only_one_item_is_not_shared() {
        let (_dir, store) = on_disk();
        let id = store
            .insert_item("uuid-only", &big_image(11), "", 1)
            .expect("insert");
        let digest = store
            .blobs_of(id)
            .expect("blobs")
            .pop()
            .expect("there is one");
        assert!(!store.blob_is_shared(&digest, id).expect("queried"));
    }

    #[test]
    fn a_blob_used_by_two_items_is_shared() {
        let (_dir, store) = on_disk();
        let first = store
            .insert_item("uuid-1", &big_image(12), "", 1)
            .expect("a");
        store
            .insert_item("uuid-2", &big_image(12), "", 2)
            .expect("b");
        let digest = store
            .blobs_of(first)
            .expect("blobs")
            .pop()
            .expect("there is one");
        assert!(store.blob_is_shared(&digest, first).expect("queried"));
    }

    #[test]
    fn an_inline_payload_comes_back_as_is() {
        let store = Store::in_memory().expect("schema");
        let id = store
            .insert_item("uuid-inline", &sample_item(), "hello", 1)
            .expect("insert");
        let bytes = store
            .payload_of(id, "public.utf8-plain-text")
            .expect("read")
            .expect("is there");
        assert_eq!(bytes, b"hello");
    }

    #[test]
    fn deleting_an_image_takes_its_bytes_off_the_disk() {
        let (_dir, store) = on_disk();
        let id = store
            .insert_item("uuid-delete", &big_image(3), "", 1)
            .expect("insert");
        assert!(store.payload_of(id, "public.png").expect("read").is_some());
        store.mark_deleted(id, 2).expect("removed");
        assert!(
            store.payload_of(id, "public.png").expect("read").is_none(),
            "the bytes of a deleted image cannot remain on disk"
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
        store.mark_deleted(first, 3).expect("deletes the first one");
        assert!(
            store
                .payload_of(second, "public.png")
                .expect("read")
                .is_some(),
            "the other item still needs those bytes"
        );
    }

    #[test]
    fn an_in_memory_store_refuses_what_it_cannot_keep() {
        let store = Store::in_memory().expect("schema");
        assert!(
            store.insert_item("uuid-big", &big_image(1), "", 1).is_err(),
            "with no folder to write to, better to refuse"
        );
    }

    #[test]
    fn the_queue_hands_out_work_and_forgets_it_when_done() {
        let store = Store::in_memory().expect("schema");
        let id = store
            .insert_text("uuid-work", "something", 1)
            .expect("insert");
        store.enqueue(id, "ocr").expect("queued");
        store
            .enqueue(id, "ocr")
            .expect("queuing twice does not duplicate");
        assert_eq!(store.take_pending("ocr", 10, 5).expect("pending"), vec![id]);
        assert!(
            store
                .take_pending("thumbnail", 10, 5)
                .expect("a different kind")
                .is_empty(),
            "each queue is its own"
        );
        store.work_done(id, "ocr").expect("done");
        assert!(
            store
                .take_pending("ocr", 10, 5)
                .expect("pending")
                .is_empty()
        );
    }

    #[test]
    fn a_job_that_keeps_failing_is_given_up_on() {
        let store = Store::in_memory().expect("schema");
        let id = store
            .insert_text("uuid-failure", "something", 1)
            .expect("insert");
        store.enqueue(id, "ocr").expect("queued");
        for attempt in 1..Store::MAX_ATTEMPTS {
            assert!(
                store
                    .work_failed(id, "ocr", "could not do it", 0)
                    .expect("failed"),
                "attempt {attempt} is still retried"
            );
        }
        assert!(
            !store
                .work_failed(id, "ocr", "could not do it", 0)
                .expect("failed"),
            "once the attempts run out, it gives up"
        );
        assert!(
            store
                .take_pending("ocr", 10, 5)
                .expect("pending")
                .is_empty()
        );
    }

    #[test]
    fn a_failed_job_waits_before_being_retried() {
        let store = Store::in_memory().expect("schema");
        let id = store
            .insert_text("uuid-wait", "something", 1)
            .expect("insert");
        store.enqueue(id, "ocr").expect("queued");
        store
            .work_failed(id, "ocr", "temporary", 500)
            .expect("failed");
        assert!(
            store
                .take_pending("ocr", 100, 5)
                .expect("not yet")
                .is_empty(),
            "not before its time"
        );
        assert_eq!(
            store.take_pending("ocr", 500, 5).expect("already"),
            vec![id]
        );
    }

    #[test]
    fn deleted_items_drop_out_of_the_queue() {
        let store = Store::in_memory().expect("schema");
        let id = store
            .insert_text("uuid-out", "something", 1)
            .expect("insert");
        store.enqueue(id, "ocr").expect("queued");
        store.mark_deleted(id, 2).expect("removed");
        assert!(
            store
                .take_pending("ocr", 10, 5)
                .expect("pending")
                .is_empty(),
            "what the user deleted does not get enriched"
        );
    }

    #[test]
    fn metadata_is_kept_per_key_and_replaced_not_duplicated() {
        let store = Store::in_memory().expect("schema");
        let id = store
            .insert_text("uuid-meta", "a video", 1)
            .expect("insert");
        store.set_meta(id, "duration", "227").expect("set");
        store.set_meta(id, "width", "1920").expect("set");
        store.set_meta(id, "duration", "228").expect("corrected");
        assert_eq!(
            store.meta(id, "duration").expect("read").as_deref(),
            Some("228")
        );
        assert_eq!(store.all_meta(id).expect("everything").len(), 2);
        assert!(store.meta(id, "artist").expect("read").is_none());
    }

    #[test]
    fn metadata_goes_away_with_the_item() {
        let store = Store::in_memory().expect("schema");
        let id = store
            .insert_text("uuid-meta", "something", 1)
            .expect("insert");
        store.set_meta(id, "artist", "someone").expect("set");
        store.mark_deleted(id, 2).expect("removed");
        assert!(
            store.all_meta(id).expect("everything").is_empty(),
            "derived data belongs to the user just like the content does"
        );
    }

    fn a_little_history() -> Store {
        let store = Store::in_memory().expect("schema");
        let rows = [
            ("uuid-1", "first note", Kind::Text, 10),
            ("uuid-2", "someone@example.test", Kind::Email, 20),
            ("uuid-3", "#FF8800", Kind::Color, 30),
            ("uuid-4", "second note", Kind::Text, 40),
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
        let listed = store
            .list(&Filter::default(), 10, None)
            .expect("listed")
            .rows;
        assert_eq!(listed.len(), 4, "with no term the history comes back");
        assert_eq!(
            listed.first().map(|one| one.preview.as_str()),
            Some("second note"),
            "the most recent one comes first"
        );
    }

    #[test]
    fn the_list_can_be_filtered_by_kind() {
        let store = a_little_history();
        let filter = Filter {
            kinds: vec![Kind::Text],
            ..Default::default()
        };
        let listed = store.list(&filter, 10, None).expect("listed").rows;
        assert_eq!(listed.len(), 2);
        assert!(listed.iter().all(|one| one.kind == Some(Kind::Text)));
    }

    #[test]
    fn several_kinds_can_be_asked_for_at_once() {
        let store = a_little_history();
        let filter = Filter {
            kinds: vec![Kind::Email, Kind::Color],
            ..Default::default()
        };
        assert_eq!(store.list(&filter, 10, None).expect("listed").rows.len(), 2);
    }

    #[test]
    fn filtering_and_searching_work_together() {
        let store = a_little_history();
        let filter = Filter {
            query: Some("note".into()),
            kinds: vec![Kind::Text],
            ..Default::default()
        };
        assert_eq!(store.list(&filter, 10, None).expect("listed").rows.len(), 2);

        let narrower = Filter {
            query: Some("note".into()),
            kinds: vec![Kind::Email],
            ..Default::default()
        };
        assert!(
            store
                .list(&narrower, 10, None)
                .expect("listed")
                .rows
                .is_empty(),
            "both the filter and the term get applied"
        );
    }

    #[test]
    fn only_pinned_can_be_asked_for() {
        let store = a_little_history();
        let listed = store
            .list(&Filter::default(), 10, None)
            .expect("listed")
            .rows;
        let id = listed.first().expect("there is one").id;
        store.set_pinned(id, true, 0).expect("pinned");
        let filter = Filter {
            pinned_only: true,
            ..Default::default()
        };
        let pinned = store.list(&filter, 10, None).expect("listed").rows;
        assert_eq!(pinned.len(), 1);
        assert!(pinned[0].pinned);
    }

    #[test]
    fn the_list_hands_out_a_cursor_only_while_there_is_more() {
        let store = a_little_history();
        let first = store.list(&Filter::default(), 2, None).expect("a page");
        assert_eq!(first.rows.len(), 2);
        let cursor = first.next.expect("two more remain");
        let second = store
            .list(&Filter::default(), 2, Some(cursor))
            .expect("next");
        assert_eq!(second.rows.len(), 2);
        assert!(second.rows.iter().all(|one| !first.rows.contains(one)));
        assert_eq!(second.next, None, "the last page promises no other");
    }

    #[test]
    fn a_page_that_ends_exactly_at_the_last_row_promises_nothing_more() {
        let store = a_little_history();
        let whole = store.list(&Filter::default(), 4, None).expect("a page");
        assert_eq!(whole.rows.len(), 4);
        assert_eq!(whole.next, None);
    }

    #[test]
    fn a_search_with_nothing_usable_returns_nothing_not_everything() {
        let store = a_little_history();
        let filter = Filter {
            query: Some("!!!".into()),
            ..Default::default()
        };
        assert!(
            store
                .list(&filter, 10, None)
                .expect("listed")
                .rows
                .is_empty(),
            "asking to search for something impossible cannot return the whole history"
        );
    }

    #[test]
    fn deleted_items_never_show_up_in_the_list() {
        let store = a_little_history();
        let listed = store
            .list(&Filter::default(), 10, None)
            .expect("listed")
            .rows;
        store.mark_deleted(listed[0].id, 99).expect("removed");
        assert_eq!(
            store
                .list(&Filter::default(), 10, None)
                .expect("listed")
                .rows
                .len(),
            3
        );
    }

    #[test]
    fn retention_takes_the_old_and_leaves_what_was_pinned() {
        let store = a_little_history();
        let listed = store
            .list(&Filter::default(), 10, None)
            .expect("listed")
            .rows;
        let oldest = listed.last().expect("there is one").id;
        store
            .set_pinned(oldest, true, 0)
            .expect("pins the oldest one");

        let removed = store.clear_older_than(35).expect("retention");
        assert_eq!(
            removed, 2,
            "what is from before the cutoff and not pinned goes away"
        );
        let left = store
            .list(&Filter::default(), 10, None)
            .expect("listed")
            .rows;
        assert_eq!(left.len(), 2);
        assert!(
            left.iter().any(|one| one.id == oldest),
            "a pinned item does not get removed by cleanup"
        );
    }

    #[test]
    fn clearing_everything_still_respects_what_was_pinned() {
        let store = a_little_history();
        let listed = store
            .list(&Filter::default(), 10, None)
            .expect("listed")
            .rows;
        store.set_pinned(listed[0].id, true, 0).expect("pinned");
        let removed = store.clear_all_unpinned(100).expect("emptied");
        assert_eq!(removed, 3);
        assert_eq!(store.count().expect("counted"), 1);
    }

    #[test]
    fn retention_with_nothing_old_enough_removes_nothing() {
        let store = a_little_history();
        assert_eq!(store.clear_older_than(0).expect("retention"), 0);
        assert_eq!(store.count().expect("counted"), 4);
    }

    #[test]
    fn a_word_that_is_not_there_finds_nothing() {
        let store = seeded();
        assert!(search(&store, "berlin").is_empty());
    }
}

#[cfg(test)]
mod identity {
    use super::*;

    #[test]
    fn what_was_captured_is_found_again() {
        let store = Store::in_memory().expect("opened");
        let item = captured("hello");
        store
            .insert_item("uuid-1", &item, "hello", 1)
            .expect("inserted");
        assert_eq!(
            store.find_by_hash(&item).expect("searched"),
            Some(1),
            "what insert_item stores, find_by_hash has to recognise"
        );
    }

    #[test]
    fn a_different_rendering_is_a_different_item() {
        let store = Store::in_memory().expect("opened");
        let plain = captured("hello");
        store
            .insert_item("uuid-1", &plain, "hello", 1)
            .expect("inserted");
        assert!(
            store
                .find_by_hash(&Item::plain("**hello**"))
                .expect("searched")
                .is_none()
        );
    }

    #[test]
    fn a_synthetic_text_is_not_a_captured_one() {
        assert_ne!(
            Item::plain("hello").fingerprint(),
            captured("hello").fingerprint()
        );
    }

    #[test]
    fn copying_the_same_thing_twice_from_google_reactivates_instead_of_duplicating() {
        use cp_core::item::Format;
        let store = Store::in_memory().expect("schema");
        let docs = |guid: &str| Item {
            kind: Some(Kind::Text),
            formats: vec![
                Format {
                    id: "public.html".into(),
                    payload: Payload::Inline(
                        format!("<b id=\"docs-internal-guid-{guid}\"><span>hello</span></b>")
                            .into_bytes(),
                    ),
                },
                Format {
                    id: "public.utf8-plain-text".into(),
                    payload: Payload::Inline(b"hello".to_vec()),
                },
            ],
        };
        let id = store
            .insert_item("uuid-docs", &docs("4a1e6b2f-7fff-1d3e"), "hello", 1)
            .expect("inserted");
        assert_eq!(
            store
                .find_by_hash(&docs("0c9d8e7f-7fff-aaaa"))
                .expect("searched"),
            Some(id),
            "a different GUID, the same item"
        );
        assert_eq!(
            store.find_by_hash(&Item::plain("hello")).expect("searched"),
            None,
            "plain text copied from where it was pasted is a different item"
        );
    }
}

#[cfg(test)]
mod listing {
    use super::*;
    use cp_core::item::Format;

    fn text_item(text: &str, kind: Kind) -> Item {
        Item {
            kind: Some(kind),
            formats: vec![Format {
                id: "public.utf8-plain-text".into(),
                payload: Payload::Inline(text.as_bytes().to_vec()),
            }],
        }
    }

    fn history() -> Store {
        let store = Store::in_memory().expect("schema");
        let rows = [
            ("uuid-1", "first note", Kind::Text, 10, "Safari"),
            ("uuid-2", "someone@example.test", Kind::Email, 20, "Slack"),
            ("uuid-3", "#FF8800", Kind::Color, 30, "Slack"),
            ("uuid-4", "second note", Kind::Text, 40, "Code"),
            ("uuid-5", "fn main() {}", Kind::Code, 50, "Code"),
        ];
        for (uuid, text, kind, at, app) in rows {
            let id = store
                .insert_item(uuid, &text_item(text, kind), text, at)
                .expect("insert");
            store.set_source(id, app, at).expect("sourced");
        }
        store
    }

    fn all(store: &Store, filter: &Filter) -> Vec<Listed> {
        store.list(filter, 100, None).expect("listed").rows
    }

    fn previews(rows: &[Listed]) -> Vec<&str> {
        rows.iter().map(|one| one.preview.as_str()).collect()
    }

    #[test]
    fn the_card_gets_everything_the_row_knows() {
        let store = history();
        let id = all(&store, &Filter::default())[0].id;
        store.set_label(id, Some("Startup"), 60).expect("labelled");
        store.set_color(id, 5, 61).expect("color");
        store.record_paste(id, 62).expect("pasted");
        store.set_pinned(id, true, 0).expect("pinned");
        let card = all(&store, &Filter::default())
            .into_iter()
            .find(|one| one.id == id)
            .expect("is there");
        assert_eq!(card.preview, "fn main() {}");
        assert_eq!(card.kind, Some(Kind::Code));
        assert_eq!(card.app.as_deref(), Some("Code"));
        assert_eq!(card.label.as_deref(), Some("Startup"));
        assert_eq!(card.color, 5);
        assert_eq!(card.paste_count, 1);
        assert_eq!(card.last_used_at, Some(62));
        assert_eq!(card.created_at, 50);
        assert_eq!(card.modified_at, 50, "pasting does not move it");
        assert!(card.pinned);
        assert_eq!(card.broken_since, None);
        assert_eq!(card.thumb_path, None);
        assert_eq!(card.snippet, None, "with no term there is no snippet");
    }

    #[test]
    fn searching_marks_the_fragment_that_matched() {
        let store = history();
        let filter = Filter {
            query: Some("sec".into()),
            ..Default::default()
        };
        let rows = all(&store, &filter);
        assert_eq!(rows.len(), 1);
        let snippet = rows[0].snippet.as_ref().expect("a snippet");
        assert_eq!(snippet.found_in, FoundIn::Text);
        let marked: Vec<&str> = snippet
            .excerpt
            .segments
            .iter()
            .filter(|one| one.matched)
            .map(|one| one.text.as_str())
            .collect();
        assert_eq!(marked, vec!["sec"]);
        assert_eq!(snippet.excerpt.plain(), "second note");
    }

    #[test]
    fn a_hit_on_the_label_says_so() {
        let store = history();
        let id = all(&store, &Filter::default())[0].id;
        store
            .set_label(id, Some("Invoice may"), 60)
            .expect("labelled");
        let filter = Filter {
            query: Some("invoice".into()),
            ..Default::default()
        };
        let rows = all(&store, &filter);
        assert_eq!(rows.len(), 1);
        let snippet = rows[0].snippet.as_ref().expect("a snippet");
        assert_eq!(snippet.found_in, FoundIn::Label);
        assert_eq!(snippet.excerpt.plain(), "Invoice may");
    }

    #[test]
    fn a_hit_on_what_was_read_inside_an_image_says_so() {
        let store = Store::in_memory().expect("schema");
        let image = Item {
            kind: Some(Kind::Image),
            formats: vec![Format {
                id: "public.png".into(),
                payload: Payload::Inline(vec![1]),
            }],
        };
        let id = store
            .insert_item("uuid-img", &image, "", 1)
            .expect("insert");
        store
            .set_ocr_text(id, "Order AB-4417 delivery", 2)
            .expect("ocr");
        let filter = Filter {
            query: Some("ab-4417".into()),
            ..Default::default()
        };
        let rows = all(&store, &filter);
        assert_eq!(rows.len(), 1);
        let snippet = rows[0].snippet.as_ref().expect("a snippet");
        assert_eq!(snippet.found_in, FoundIn::Ocr);
        assert_eq!(
            snippet.excerpt.plain(),
            "Order AB-4417 delivery",
            "the snippet shows what was read exactly as is, not folded"
        );
    }

    #[test]
    fn an_old_folded_ocr_is_still_shown_until_it_is_read_again() {
        let store = Store::in_memory().expect("schema");
        let image = Item {
            kind: Some(cp_core::kind::Kind::Image),
            formats: vec![Format {
                id: "public.png".into(),
                payload: Payload::Inline(vec![1]),
            }],
        };
        let id = store
            .insert_item("uuid-old", &image, "", 1)
            .expect("insert");
        store
            .raw()
            .execute(
                "UPDATE items SET search_ocr = 'folded order' WHERE id = ?1",
                [id],
            )
            .expect("the way version 3 left it");
        assert_eq!(
            store.ocr_text(id).expect("read").as_deref(),
            Some("folded order")
        );
        let rows = all(
            &store,
            &Filter {
                query: Some("folded".into()),
                ..Default::default()
            },
        );
        let snippet = rows[0].snippet.as_ref().expect("a snippet");
        assert_eq!(snippet.found_in, FoundIn::Ocr);
        assert_eq!(snippet.excerpt.plain(), "folded order");
    }

    #[test]
    fn a_hit_on_the_source_application_says_so() {
        let store = history();
        let filter = Filter {
            query: Some("slack".into()),
            ..Default::default()
        };
        let rows = all(&store, &filter);
        assert_eq!(rows.len(), 2);
        assert!(
            rows.iter()
                .all(|one| one.snippet.as_ref().map(|s| s.found_in) == Some(FoundIn::App))
        );
    }

    #[test]
    fn a_class_can_be_left_out() {
        let store = history();
        let filter = Filter {
            exclude_kinds: vec![Kind::Text, Kind::Code],
            ..Default::default()
        };
        assert_eq!(
            previews(&all(&store, &filter)),
            vec!["#FF8800", "someone@example.test"]
        );
    }

    #[test]
    fn the_source_application_filters_by_equality_not_by_search() {
        let store = history();
        let id = all(&store, &Filter::default())[0].id;
        store
            .set_label(id, Some("paste into slack"), 60)
            .expect("labelled");
        let filter = Filter {
            apps: vec!["slack".into()],
            ..Default::default()
        };
        let rows = all(&store, &filter);
        assert_eq!(rows.len(), 2, "only what was copied from Slack, case aside");
        assert!(rows.iter().all(|one| one.app.as_deref() == Some("Slack")));
    }

    #[test]
    fn several_applications_and_a_negated_one() {
        let store = history();
        let either = Filter {
            apps: vec!["Safari".into(), "Code".into()],
            ..Default::default()
        };
        assert_eq!(all(&store, &either).len(), 3);
        let not_code = Filter {
            exclude_apps: vec!["code".into()],
            ..Default::default()
        };
        assert_eq!(all(&store, &not_code).len(), 3);
    }

    #[test]
    fn since_keeps_what_was_copied_from_that_moment_on() {
        let store = history();
        let filter = Filter {
            since: Some(30),
            ..Default::default()
        };
        assert_eq!(all(&store, &filter).len(), 3, "the 30 is included");
    }

    #[test]
    fn since_counts_a_recopy_as_copied_again() {
        let store = history();
        let oldest = all(&store, &Filter::default())
            .last()
            .expect("there is one")
            .id;
        store.reactivate(oldest, 100).expect("copied again");
        let filter = Filter {
            since: Some(100),
            ..Default::default()
        };
        assert_eq!(
            previews(&all(&store, &filter)),
            vec!["first note"],
            "what gets copied again today belongs to today"
        );
    }

    #[test]
    fn broken_items_are_hidden_unless_asked_for() {
        let store = history();
        let id = all(&store, &Filter::default())[0].id;
        store.mark_broken(id, 99).expect("broken");
        assert_eq!(all(&store, &Filter::default()).len(), 4);
        let shown = Filter {
            broken: Broken::Shown,
            ..Default::default()
        };
        assert_eq!(all(&store, &shown).len(), 5);
        let only = Filter {
            broken: Broken::Only,
            ..Default::default()
        };
        let rows = all(&store, &only);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].broken_since, Some(99));
    }

    #[test]
    fn a_scoped_label_query_does_not_match_the_content() {
        let store = history();
        let rows = all(&store, &Filter::default());
        store
            .set_label(rows[1].id, Some("note"), 60)
            .expect("labelled");
        let by_label = Filter {
            label_query: Some("note".into()),
            ..Default::default()
        };
        let found = all(&store, &by_label);
        assert_eq!(found.len(), 1, "«note» is in two contents and one label");
        assert_eq!(found[0].id, rows[1].id);
    }

    #[test]
    fn a_label_query_and_a_text_query_both_apply() {
        let store = history();
        let rows = all(&store, &Filter::default());
        store
            .set_label(rows[0].id, Some("startup"), 60)
            .expect("labelled");
        store
            .set_label(rows[1].id, Some("startup"), 61)
            .expect("labelled");
        let filter = Filter {
            query: Some("main".into()),
            label_query: Some("startup".into()),
            ..Default::default()
        };
        assert_eq!(previews(&all(&store, &filter)), vec!["fn main() {}"]);
    }

    #[test]
    fn a_label_query_with_nothing_usable_finds_nothing() {
        let store = history();
        let filter = Filter {
            label_query: Some("!!!".into()),
            ..Default::default()
        };
        assert!(all(&store, &filter).is_empty());
    }

    #[test]
    fn most_pasted_comes_first_and_ties_break_the_same_way_every_time() {
        let store = history();
        let rows = all(&store, &Filter::default());
        for _ in 0..3 {
            store.record_paste(rows[4].id, 70).expect("pasted");
        }
        store.record_paste(rows[2].id, 71).expect("pasted");
        let filter = Filter {
            order: Order::MostPasted,
            ..Default::default()
        };
        let ordered = all(&store, &filter);
        assert_eq!(ordered[0].id, rows[4].id);
        assert_eq!(ordered[1].id, rows[2].id);
        assert_eq!(
            ordered[2..].iter().map(|one| one.id).collect::<Vec<_>>(),
            vec![rows[0].id, rows[1].id, rows[3].id],
            "at an equal count, the newest one first"
        );
    }

    #[test]
    fn last_used_puts_what_was_never_pasted_at_the_end() {
        let store = history();
        let rows = all(&store, &Filter::default());
        store.record_paste(rows[3].id, 80).expect("pasted");
        store.record_paste(rows[1].id, 90).expect("pasted");
        let filter = Filter {
            order: Order::LastUsed,
            ..Default::default()
        };
        let ordered = all(&store, &filter);
        assert_eq!(ordered[0].id, rows[1].id);
        assert_eq!(ordered[1].id, rows[3].id);
        assert!(ordered[2..].iter().all(|one| one.last_used_at.is_none()));
    }

    fn walk(store: &Store, filter: &Filter, page: usize) -> Vec<i64> {
        let mut seen = Vec::new();
        let mut after = None;
        loop {
            let got = store.list(filter, page, after).expect("a page");
            seen.extend(got.rows.iter().map(|one| one.id));
            match got.next {
                Some(cursor) => after = Some(cursor),
                None => return seen,
            }
        }
    }

    #[test]
    fn every_order_pages_without_repeating_or_skipping_even_with_ties() {
        let store = Store::in_memory().expect("schema");
        for at in 0..23 {
            let id = store
                .insert_item(
                    &format!("uuid-{at}"),
                    &text_item(&format!("note {at}"), Kind::Text),
                    &format!("note {at}"),
                    at % 4,
                )
                .expect("insert");
            for _ in 0..(at % 3) {
                store.record_paste(id, at % 5).expect("pasted");
            }
        }
        for order in [Order::Recent, Order::MostPasted, Order::LastUsed] {
            let filter = Filter {
                order,
                ..Default::default()
            };
            let mut ids = walk(&store, &filter, 4);
            assert_eq!(ids.len(), 23, "{order:?} skipped rows");
            ids.sort_unstable();
            ids.dedup();
            assert_eq!(ids.len(), 23, "{order:?} repeated rows");
        }
    }

    #[test]
    fn the_tabs_count_only_the_classes_that_exist_within_the_search() {
        let store = history();
        let facets = store.facets(&Filter::default()).expect("facets");
        assert_eq!(
            facets,
            vec![
                Facet {
                    kind: Kind::Text,
                    count: 2
                },
                Facet {
                    kind: Kind::Code,
                    count: 1
                },
                Facet {
                    kind: Kind::Color,
                    count: 1
                },
                Facet {
                    kind: Kind::Email,
                    count: 1
                },
            ],
            "by count, and at an equal count by name"
        );
        let within = Filter {
            apps: vec!["Slack".into()],
            kinds: vec![Kind::Text],
            ..Default::default()
        };
        let facets = store.facets(&within).expect("facets");
        assert_eq!(
            facets.iter().map(|one| one.kind).collect::<Vec<_>>(),
            vec![Kind::Color, Kind::Email],
            "the chosen tab does not narrow the others; the app and the term do"
        );
    }

    #[test]
    fn an_item_without_a_class_has_no_tab() {
        let store = Store::in_memory().expect("schema");
        store
            .insert_item(
                "uuid-none",
                &Item {
                    kind: None,
                    formats: vec![],
                },
                "",
                1,
            )
            .expect("insert");
        assert!(store.facets(&Filter::default()).expect("facets").is_empty());
    }

    #[test]
    fn an_impossible_search_has_no_tabs_either() {
        let store = history();
        let filter = Filter {
            query: Some("!!!".into()),
            ..Default::default()
        };
        assert!(store.facets(&filter).expect("facets").is_empty());
    }

    #[test]
    fn the_applications_come_with_their_counts_most_used_first() {
        let store = history();
        let apps = store.distinct_apps(&Filter::default()).expect("apps");
        assert_eq!(
            apps,
            vec![
                AppCount {
                    app: "Code".into(),
                    count: 2
                },
                AppCount {
                    app: "Slack".into(),
                    count: 2
                },
                AppCount {
                    app: "Safari".into(),
                    count: 1
                },
            ]
        );
    }

    #[test]
    fn two_spellings_of_one_application_are_one_entry() {
        let store = history();
        let id = all(&store, &Filter::default())[0].id;
        store.set_source(id, "slack", 60).expect("sourced");
        let apps = store.distinct_apps(&Filter::default()).expect("apps");
        let slack = apps
            .iter()
            .find(|one| one.app == "Slack")
            .expect("is there");
        assert_eq!(slack.count, 3);
        assert!(apps.iter().all(|one| one.app != "slack"));
    }

    #[test]
    fn deleted_items_count_for_nothing() {
        let store = history();
        let id = all(&store, &Filter::default())[0].id;
        store.mark_deleted(id, 99).expect("removed");
        let facets = store.facets(&Filter::default()).expect("facets");
        assert!(facets.iter().all(|one| one.kind != Kind::Code));
        let apps = store.distinct_apps(&Filter::default()).expect("apps");
        assert_eq!(
            apps.iter()
                .find(|one| one.app == "Code")
                .map(|one| one.count),
            Some(1)
        );
    }

    #[test]
    fn no_order_sorts_the_history_in_memory() {
        let store = history();
        for order in Order::ALL {
            let filter = Filter {
                order,
                ..Default::default()
            };
            let clauses =
                Clauses::of(&filter, true, true).expect("with no term there are still clauses");
            let sql = page_sql(&clauses, order.key(), false);
            let plan: Vec<String> = store
                .db
                .prepare(&format!("EXPLAIN QUERY PLAN {sql}"))
                .expect("prepared")
                .query_map([50i64], |row| row.get::<_, String>(3))
                .expect("a plan")
                .map(|row| row.expect("a row"))
                .collect();
            assert!(
                !plan.iter().any(|step| step.contains("TEMP B-TREE")),
                "{order:?} sorts in memory: {plan:?}"
            );
        }
    }

    #[test]
    fn leaving_a_class_out_keeps_what_has_no_class() {
        let store = history();
        store
            .insert_item(
                "uuid-none",
                &Item {
                    kind: None,
                    formats: vec![],
                },
                "no class",
                60,
            )
            .expect("insert");
        let filter = Filter {
            exclude_kinds: vec![Kind::Image],
            ..Default::default()
        };
        assert_eq!(
            all(&store, &filter).len(),
            6,
            "excluding images cannot hide what is nothing at all"
        );
    }

    #[test]
    fn an_excluded_class_has_no_tab() {
        let store = history();
        let filter = Filter {
            exclude_kinds: vec![Kind::Text],
            ..Default::default()
        };
        let facets = store.facets(&filter).expect("facets");
        assert!(facets.iter().all(|one| one.kind != Kind::Text));
        assert_eq!(facets.len(), 3);
    }

    #[test]
    fn asking_for_no_rows_is_an_empty_page_not_a_panic() {
        let store = history();
        let page = store.list(&Filter::default(), 0, None).expect("a page");
        assert!(page.rows.is_empty());
        assert_eq!(page.next, None);
        let huge = store
            .list(&Filter::default(), usize::MAX, None)
            .expect("a page");
        assert_eq!(huge.rows.len(), 5);
        assert_eq!(huge.next, None);
    }

    #[test]
    fn a_broken_item_can_be_found_again() {
        let store = history();
        let id = all(&store, &Filter::default())[0].id;
        store.mark_broken(id, 99).expect("broken");
        assert_eq!(all(&store, &Filter::default()).len(), 4);
        store.mark_present(id).expect("came back");
        let rows = all(&store, &Filter::default());
        assert_eq!(rows.len(), 5, "the volume got remounted");
        assert_eq!(rows[0].broken_since, None);
        assert_eq!(
            store.purge_broken_before(i64::MAX).expect("purged"),
            0,
            "and it is no longer within anyone's deadline"
        );
    }

    #[test]
    fn the_footer_count_matches_what_the_list_shows() {
        let store = history();
        let id = all(&store, &Filter::default())[0].id;
        store.mark_broken(id, 99).expect("broken");
        assert_eq!(
            store.count().expect("total"),
            5,
            "the total keeps counting broken ones"
        );
        assert_eq!(
            store.count_matching(&Filter::default()).expect("counted"),
            4,
            "what the footer shows is what the list shows"
        );
        let filter = Filter {
            query: Some("note".into()),
            apps: vec!["safari".into()],
            ..Default::default()
        };
        assert_eq!(store.count_matching(&filter).expect("counted"), 1);
        let impossible = Filter {
            query: Some("!!!".into()),
            ..Default::default()
        };
        assert_eq!(store.count_matching(&impossible).expect("counted"), 0);
        let facets: i64 = store
            .facets(&Filter::default())
            .expect("facets")
            .iter()
            .map(|one| one.count)
            .sum();
        assert_eq!(facets, 4, "and the tabs add up to the same");
        assert_eq!(
            store
                .distinct_apps(&Filter::default())
                .expect("apps")
                .iter()
                .map(|one| one.count)
                .sum::<i64>(),
            4,
            "apps do not count broken ones either"
        );
    }

    #[test]
    fn every_order_has_a_stable_name_that_comes_back() {
        for order in Order::ALL {
            assert_eq!(Order::from_name(order.as_str()), Some(order));
        }
        let mut names: Vec<&str> = Order::ALL.iter().map(|order| order.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 3);
        assert_eq!(Order::from_name("Recent"), None, "the name is exact");
    }

    #[test]
    fn a_cursor_from_another_order_is_refused_not_misread() {
        let store = history();
        let recent = store.list(&Filter::default(), 2, None).expect("a page");
        let cursor = recent.next.expect("there is more");
        let pasted = Filter {
            order: Order::MostPasted,
            ..Default::default()
        };
        assert!(matches!(
            store.list(&pasted, 2, Some(cursor)),
            Err(Error::WrongCursor { .. })
        ));
        let text = cursor.encode();
        assert_eq!(
            Cursor::decode(&text),
            Some(cursor),
            "it goes out and comes back as text"
        );
        assert_eq!(Cursor::decode("recent:1"), None);
        assert_eq!(Cursor::decode("sideways:1:2"), None);
        assert_eq!(Cursor::decode("recent:1:2:3"), None);
        assert_eq!(Cursor::decode("recent:x:2"), None);
    }

    #[test]
    fn the_preview_is_capped_but_the_excerpt_still_sees_the_whole_text() {
        let store = Store::in_memory().expect("schema");
        let text = format!("{}needle", "hay ".repeat(1_000));
        store.insert_text("uuid-long", &text, 1).expect("insert");
        let rows = store
            .list(&Filter::default(), 10, None)
            .expect("listed")
            .rows;
        assert_eq!(rows[0].preview.chars().count(), PREVIEW_CHARS);
        let filter = Filter {
            query: Some("needle".into()),
            ..Default::default()
        };
        let rows = store.list(&filter, 10, None).expect("listed").rows;
        let snippet = rows[0].snippet.as_ref().expect("a snippet");
        assert!(
            snippet
                .excerpt
                .segments
                .iter()
                .any(|one| one.matched && one.text == "needle"),
            "the needle is past the cap on the preview"
        );
    }

    #[test]
    fn the_applications_follow_the_search_but_not_their_own_filter() {
        let store = history();
        let within = Filter {
            query: Some("note".into()),
            apps: vec!["Safari".into()],
            ..Default::default()
        };
        let apps = store.distinct_apps(&within).expect("apps");
        assert_eq!(
            apps.iter().map(|one| one.app.as_str()).collect::<Vec<_>>(),
            vec!["Code", "Safari"],
            "both apps with a note, even though the filter asks for only Safari"
        );
        let impossible = Filter {
            query: Some("!!!".into()),
            ..Default::default()
        };
        assert!(store.distinct_apps(&impossible).expect("apps").is_empty());
    }

    #[test]
    fn pinning_can_be_undone_and_moves_the_version() {
        let store = history();
        let id = all(&store, &Filter::default())[0].id;
        store.set_pinned(id, true, 70).expect("pinned");
        assert!(all(&store, &Filter::default())[0].pinned);
        assert!(
            store
                .changed_since(60)
                .expect("changes")
                .contains(&"uuid-5".to_string())
        );
        store.set_pinned(id, false, 71).expect("unpinned");
        assert!(!all(&store, &Filter::default())[0].pinned);
    }

    #[test]
    fn a_thumbnail_path_can_be_set_and_shows_on_the_card() {
        let store = history();
        let id = all(&store, &Filter::default())[0].id;
        store
            .set_thumb(id, Some("thumbs/5.png"), 70)
            .expect("a thumbnail");
        assert_eq!(
            all(&store, &Filter::default())[0].thumb_path.as_deref(),
            Some("thumbs/5.png")
        );
        store.set_thumb(id, None, 71).expect("no thumbnail");
        assert_eq!(all(&store, &Filter::default())[0].thumb_path, None);
    }

    #[test]
    fn an_item_comes_back_with_the_formats_it_was_stored_with() {
        let (_dir, store) = on_disk();
        let big = Item {
            kind: Some(Kind::Image),
            formats: vec![
                Format {
                    id: "public.png".into(),
                    payload: Payload::Blob(vec![7; 200_000]),
                },
                Format {
                    id: "public.tiff".into(),
                    payload: Payload::Announced { size: Some(4_000) },
                },
                Format {
                    id: "com.apple.icns".into(),
                    payload: Payload::Absent,
                },
            ],
        };
        let id = store.insert_item("uuid-item", &big, "", 1).expect("insert");
        let back = store.item(id).expect("read").expect("is there");
        assert_eq!(back.kind, Some(Kind::Image));
        assert_eq!(back.formats.len(), 3);
        assert!(
            matches!(back.format("public.png").expect("png").payload, Payload::Blob(ref b) if b.len() == 200_000)
        );
        assert_eq!(
            back.format("public.tiff").expect("tiff").payload,
            Payload::Announced { size: Some(4_000) }
        );
        assert_eq!(
            back.format("com.apple.icns").expect("icns").payload,
            Payload::Absent
        );
        assert_eq!(store.item(404).expect("read"), None);
        store.mark_deleted(id, 2).expect("removed");
        assert_eq!(store.item(id).expect("read"), None, "lo borrado no vuelve");
    }

    #[test]
    fn a_text_inserted_directly_is_classified_like_a_capture() {
        let store = Store::in_memory().expect("schema");
        store.insert_text("uuid-c", "#FF8800", 1).expect("insert");
        store.insert_text("uuid-t", "una nota", 2).expect("insert");
        let rows = store
            .list(&Filter::default(), 10, None)
            .expect("listed")
            .rows;
        assert_eq!(rows[0].kind, Some(Kind::Text));
        assert_eq!(rows[1].kind, Some(Kind::Color));
    }

    #[test]
    fn every_filter_at_once_narrows_to_the_one_row() {
        let store = history();
        let rows = all(&store, &Filter::default());
        let target = rows
            .iter()
            .find(|one| one.preview == "second note")
            .expect("is there");
        store
            .set_label(target.id, Some("key"), 60)
            .expect("labelled");
        store.set_color(target.id, 2, 61).expect("color");
        store.set_pinned(target.id, true, 62).expect("pinned");
        let filter = Filter {
            query: Some("note".into()),
            label_query: Some("key".into()),
            kinds: vec![Kind::Text, Kind::Code],
            exclude_kinds: vec![Kind::Email],
            apps: vec!["Code".into(), "Safari".into()],
            exclude_apps: vec!["Slack".into()],
            colors: vec![2],
            pinned_only: true,
            since: Some(20),
            broken: Broken::Shown,
            order: Order::MostPasted,
        };
        let found = all(&store, &filter);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, target.id);
        assert_eq!(store.count_matching(&filter).expect("counted"), 1);
        assert_eq!(
            store.facets(&filter).expect("facets"),
            vec![Facet {
                kind: Kind::Text,
                count: 1
            }]
        );
    }

    #[test]
    fn editing_a_pinned_item_keeps_its_pin_label_colour_and_app() {
        let store = history();
        let id = all(&store, &Filter::default())[0].id;
        store.set_label(id, Some("fixed"), 60).expect("labelled");
        store.set_color(id, 3, 61).expect("color");
        store.set_pinned(id, true, 62).expect("pinned");
        store
            .update_text(id, "different content", 63)
            .expect("edited");
        let card = all(&store, &Filter::default())
            .into_iter()
            .find(|one| one.id == id)
            .expect("is there");
        assert!(card.pinned);
        assert_eq!(card.label.as_deref(), Some("fixed"));
        assert_eq!(card.color, 3);
        assert_eq!(card.app.as_deref(), Some("Code"));
    }

    #[test]
    fn a_row_with_a_class_nobody_knows_lists_as_no_class() {
        let store = history();
        store
            .db
            .execute(
                "UPDATE items SET kind = 'hologram' WHERE uuid = 'uuid-5'",
                [],
            )
            .expect("a newer database");
        let rows = all(&store, &Filter::default());
        assert_eq!(rows[0].kind, None);
        assert!(store.facets(&Filter::default()).expect("facets").len() == 3);
    }

    #[test]
    fn walking_the_pages_reads_the_same_order_as_one_big_page() {
        let store = Store::in_memory().expect("schema");
        for at in 0..23 {
            let id = store
                .insert_item(
                    &format!("uuid-{at}"),
                    &text_item(&format!("note {at}"), Kind::Text),
                    &format!("note {at}"),
                    at % 4,
                )
                .expect("insert");
            for _ in 0..(at % 3) {
                store.record_paste(id, at % 5).expect("pasted");
            }
        }
        for order in Order::ALL {
            let filter = Filter {
                order,
                ..Default::default()
            };
            let whole: Vec<i64> = all(&store, &filter).iter().map(|one| one.id).collect();
            assert_eq!(walk(&store, &filter, 4), whole, "{order:?}");
        }
    }

    #[test]
    fn nothing_a_person_can_type_as_an_application_breaks_the_query() {
        let store = history();
        for app in ["'; DROP TABLE items; --", "\"", "%", "Straße"] {
            let filter = Filter {
                apps: vec![app.into()],
                ..Default::default()
            };
            store
                .list(&filter, 10, None)
                .unwrap_or_else(|why| panic!("«{app}» broke the listing: {why}"));
        }
        assert_eq!(
            store.count().expect("counted"),
            5,
            "the table is still there"
        );
    }
}

#[cfg(test)]
mod housekeeping {
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
}
