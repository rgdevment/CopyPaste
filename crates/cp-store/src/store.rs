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
    pub group: String,
    pub snippet: Option<Snippet>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cursor {
    order: Order,
    key: String,
    id: i64,
}

impl Cursor {
    pub fn encode(&self) -> String {
        format!("{}:{}:{}", self.order.as_str(), self.key, self.id)
    }

    pub fn decode(text: &str) -> Option<Cursor> {
        let (name, rest) = text.split_once(':')?;
        let order = Order::from_name(name)?;
        let (key, id) = rest.rsplit_once(':')?;
        if order != Order::ByGroup && key.parse::<i64>().is_err() {
            return None;
        }
        Some(Cursor {
            order,
            key: key.to_owned(),
            id: id.parse().ok()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Page {
    pub rows: Vec<Listed>,
    pub next: Option<Cursor>,
}

pub const TOUCHED: &str = "MAX(items.modified_at, COALESCE(items.last_used_at, 0))";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Order {
    #[default]
    Recent,
    MostPasted,
    LastUsed,
    ByGroup,
}

impl Order {
    pub const ALL: [Order; 4] = [
        Order::Recent,
        Order::MostPasted,
        Order::LastUsed,
        Order::ByGroup,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Order::Recent => "recent",
            Order::MostPasted => "most-pasted",
            Order::LastUsed => "last-used",
            Order::ByGroup => "by-group",
        }
    }

    pub fn from_name(name: &str) -> Option<Order> {
        Order::ALL.into_iter().find(|order| order.as_str() == name)
    }

    fn key(self) -> &'static str {
        match self {
            Order::Recent => TOUCHED,
            Order::MostPasted => "items.paste_count",
            Order::LastUsed => "COALESCE(items.last_used_at, -1)",
            Order::ByGroup => "items.group_key",
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Gone {
    thumbs: Vec<String>,
    digests: Vec<String>,
}

impl Gone {
    fn and(&mut self, other: Gone) {
        self.thumbs.extend(other.thumbs);
        self.digests.extend(other.digests);
    }
}

pub struct Store {
    db: Connection,
    blobs: Option<crate::Blobs>,
    exposure: Restricted,
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

    pub fn set_label(&self, id: i64, label: Option<&str>, at: i64) -> Result<bool> {
        let rows = self.db.execute(
            "UPDATE items SET label = ?2, search_label = ?3, updated_at = ?4 WHERE id = ?1",
            params![id, label, label.map(fold).unwrap_or_default(), at],
        )?;
        Ok(rows > 0)
    }

    pub fn set_source(&self, id: i64, app: &str, at: i64) -> Result<()> {
        self.db.execute(
            "UPDATE items SET app_source = ?2, search_app = ?3, updated_at = ?4 WHERE id = ?1",
            params![id, app, fold(app), at],
        )?;
        Ok(())
    }

    pub fn mark_deleted(&self, id: i64, at: i64) -> Result<()> {
        let gone = self.erase(id, at)?;
        self.forget(&gone)?;
        self.checkpoint_briefly()?;
        Ok(())
    }

    fn erase(&self, id: i64, at: i64) -> Result<Gone> {
        let mut gone = Gone::default();
        gone.thumbs.extend(self.thumb_of(id)?);
        self.db.execute(
            "UPDATE items
             SET deleted_at = ?2, updated_at = ?2,
                 preview_text = '', search_text = '', search_label = '',
                 search_app = '', search_ocr = '', ocr_text = NULL, label = NULL,
                 app_source = NULL, thumb_path = NULL, content_hash = 0
             WHERE id = ?1",
            params![id, at],
        )?;
        gone.and(self.release(id)?);
        Ok(gone)
    }

    fn thumb_of(&self, id: i64) -> Result<Option<String>> {
        Ok(self
            .db
            .query_row("SELECT thumb_path FROM items WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .optional()?
            .flatten())
    }

    fn release(&self, id: i64) -> Result<Gone> {
        let gone = Gone {
            thumbs: Vec::new(),
            digests: self.blobs_of(id)?,
        };
        self.db
            .execute("DELETE FROM item_formats WHERE item_id = ?1", [id])?;
        self.db
            .execute("DELETE FROM item_meta WHERE item_id = ?1", [id])?;
        self.db
            .execute("DELETE FROM pending_work WHERE item_id = ?1", [id])?;
        Ok(gone)
    }

    fn forget(&self, gone: &Gone) -> Result<()> {
        for path in &gone.thumbs {
            let _ = crate::blobs::remove_at(std::path::Path::new(path));
        }
        let Some(blobs) = &self.blobs else {
            return Ok(());
        };
        for digest in &gone.digests {
            if self.blob_is_referenced(digest)? {
                continue;
            }
            blobs.remove_if_settled(digest)?;
        }
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
        let mut gone = Gone::default();
        let transaction = self.db.unchecked_transaction()?;
        for id in ids {
            gone.and(self.erase(*id, at)?);
        }
        transaction.commit()?;
        self.forget(&gone)?;
        Ok(ids.len())
    }

    fn blobs_of(&self, id: i64) -> Result<Vec<String>> {
        let mut stmt = self
            .db
            .prepare("SELECT digest FROM item_formats WHERE item_id = ?1 AND digest IS NOT NULL")?;
        let rows = stmt.query_map([id], |row| row.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    fn blob_is_referenced(&self, digest: &str) -> Result<bool> {
        let count: i64 = self.db.query_row(
            "SELECT COUNT(*) FROM item_formats WHERE digest = ?1",
            [digest],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn payload_of(&self, id: i64, format: &str) -> Result<Option<Vec<u8>>> {
        let found: Option<(Option<Vec<u8>>, Option<String>)> = self
            .db
            .query_row(
                "SELECT inline_data, digest FROM item_formats
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

    pub fn insert_full(
        &self,
        uuid: &str,
        item: &Item,
        preview: &str,
        created_at: i64,
        more: &More<'_>,
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
        let modified = more.modified_at.unwrap_or(created_at).max(created_at);
        let touched = more.touched_at.unwrap_or(modified);
        let point = Point::open(&self.db, "carry")?;
        self.db.execute(
            "INSERT INTO items (uuid, kind, preview_text, created_at, modified_at, updated_at,
                                content_hash, search_text, app_source, search_app,
                                label, search_label, card_color, pinned, paste_count,
                                broken_since, last_used_at, came_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?16, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?17,
                     ?18)",
            params![
                uuid,
                kind,
                head_of(preview),
                created_at,
                modified,
                hash,
                fold(preview),
                more.app,
                more.app.map(fold).unwrap_or_default(),
                more.label,
                more.label.map(fold).unwrap_or_default(),
                more.color,
                i64::from(more.pinned),
                more.pastes.max(0),
                more.broken,
                touched,
                more.used_at,
                more.came_at
            ],
        )?;
        let id = self.db.last_insert_rowid();
        self.write_rows(id, &rows)?;
        for (key, value) in more.meta {
            self.set_meta(id, key, value)?;
        }
        for job in more.jobs {
            self.enqueue(id, job)?;
        }
        point.keep()?;
        Ok(id)
    }

    pub fn all_or_nothing<T>(&self, work: impl FnOnce() -> Result<T>) -> Result<T> {
        let point = Point::open(&self.db, "batch")?;
        let out = work()?;
        point.keep()?;
        Ok(out)
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
                "INSERT INTO item_formats (item_id, format, size_bytes, inline_data, digest)
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
        let previous = Gone {
            thumbs: Vec::new(),
            digests: self.blobs_of(id)?,
        };
        self.db
            .execute("DELETE FROM item_formats WHERE item_id = ?1", [id])?;
        self.db
            .execute("DELETE FROM item_meta WHERE item_id = ?1", [id])?;
        self.db
            .execute("DELETE FROM pending_work WHERE item_id = ?1", [id])?;
        self.write_rows(id, &rows)?;
        transaction.commit()?;
        self.forget(&previous)?;
        self.checkpoint_briefly()?;
        Ok(())
    }

    pub fn find_by_hash(&self, item: &Item) -> Result<Option<i64>> {
        if !item.is_comparable() {
            return Ok(None);
        }
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
            "SELECT format, size_bytes, inline_data, digest FROM item_formats
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
        let mut gone = Gone::default();
        let transaction = self.db.unchecked_transaction()?;
        for id in &doomed {
            gone.and(self.release(*id)?);
            self.db.execute("DELETE FROM items WHERE id = ?1", [id])?;
        }
        transaction.commit()?;
        self.forget(&gone)?;
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

    pub fn knows_name(&self, uuid: &str) -> Result<bool> {
        Ok(self
            .db
            .query_row("SELECT 1 FROM items WHERE uuid = ?1", [uuid], |_| Ok(()))
            .optional()?
            .is_some())
    }

    pub fn came_from_the_former(&self) -> Result<crate::legacy::Came> {
        let (count, still, when) = self.db.query_row(
            "SELECT COUNT(*), SUM(deleted_at IS NULL), MIN(came_at) FROM items
             WHERE came_at IS NOT NULL",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get::<_, Option<i64>>(1)?.unwrap_or(0),
                    row.get::<_, Option<i64>>(2)?,
                ))
            },
        )?;
        Ok(crate::legacy::Came { count, still, when })
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
            clauses.bound.push(bound_key(filter.order, &cursor.key));
            clauses.bound.push(bound_key(filter.order, &cursor.key));
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
                group: row.get(16)?,
                snippet: None,
            };
            listed.snippet = snippet_of(&listed, &whole, &ocr, &terms);
            let key = match filter.order {
                Order::ByGroup => row.get::<_, String>(14)?,
                Order::Recent | Order::MostPasted | Order::LastUsed => {
                    row.get::<_, i64>(14)?.to_string()
                }
            };
            Ok((listed, key))
        })?;
        let mut keyed: Vec<(Listed, String)> = rows.collect::<rusqlite::Result<_>>()?;
        let more = keyed.len() > limit;
        keyed.truncate(limit);
        let next = match keyed.last() {
            Some((last, key)) if more => Some(Cursor {
                order: filter.order,
                key: key.clone(),
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
                 SELECT DISTINCT digest, size_bytes FROM item_formats
                 WHERE digest IS NOT NULL)",
            [],
            |row| row.get(0),
        )?;
        Ok(Usage {
            items,
            bytes: inline + blobs,
        })
    }

    pub const PAGE: usize = 100;
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

#[derive(Debug, Default)]
pub struct More<'a> {
    pub came_at: Option<i64>,
    pub modified_at: Option<i64>,
    pub touched_at: Option<i64>,
    pub used_at: Option<i64>,
    pub app: Option<&'a str>,
    pub label: Option<&'a str>,
    pub color: i64,
    pub pinned: bool,
    pub pastes: i64,
    pub broken: Option<i64>,
    pub meta: &'a [(String, String)],
    pub jobs: &'a [&'a str],
}

struct Point<'a> {
    db: &'a Connection,
    name: &'static str,
    open: bool,
}

impl<'a> Point<'a> {
    fn open(db: &'a Connection, name: &'static str) -> Result<Self> {
        db.execute_batch(&format!("SAVEPOINT {name}"))?;
        Ok(Self {
            db,
            name,
            open: true,
        })
    }

    fn keep(mut self) -> Result<()> {
        self.open = false;
        self.db
            .execute_batch(&format!("RELEASE {}", self.name))
            .map_err(Error::from)
    }
}

impl Drop for Point<'_> {
    fn drop(&mut self) {
        if self.open {
            let _ = self
                .db
                .execute_batch(&format!("ROLLBACK TO {0}; RELEASE {0}", self.name));
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Usage {
    pub items: i64,
    pub bytes: i64,
}

pub const PREVIEW_CHARS: usize = 2_000;

fn bound_key(order: Order, key: &str) -> Box<dyn ToSql> {
    match order {
        Order::ByGroup => Box::new(key.to_owned()),
        Order::Recent | Order::MostPasted | Order::LastUsed => {
            Box::new(key.parse::<i64>().unwrap_or(i64::MIN))
        }
    }
}

fn page_sql(clauses: &Clauses, key: &str, with_query: bool) -> String {
    format!(
        "SELECT items.id, items.modified_at, items.created_at, items.kind,
                SUBSTR(items.preview_text, 1, {preview}), items.app_source, items.label,
                items.card_color, items.thumb_path, items.paste_count, items.last_used_at,
                items.broken_since, items.pinned, {ocr}, page.key, {whole},
                items.group_key
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
fn big_image(byte: u8) -> Item {
    Item {
        kind: Some(Kind::Image),
        formats: vec![cp_core::item::Format {
            id: "public.png".into(),
            payload: Payload::Blob(vec![byte; 200_000]),
        }],
    }
}

#[cfg(test)]
fn aged(path: &std::path::Path) {
    let file = std::fs::File::options()
        .write(true)
        .open(path)
        .expect("opened");
    file.set_modified(std::time::UNIX_EPOCH).expect("aged");
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

#[path = "store_housekeeping.rs"]
mod upkeep;

pub use upkeep::{A_DAY, Policy, Swept};

#[cfg(test)]
#[path = "store_test.rs"]
mod tests;

#[cfg(test)]
#[path = "store_identity_test.rs"]
mod identity;

#[cfg(test)]
#[path = "store_listing_test.rs"]
mod listing;

#[cfg(test)]
#[path = "store_housekeeping_test.rs"]
mod housekeeping;
