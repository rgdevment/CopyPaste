use crate::Result;
use crate::store::Store;
use cp_core::kind::Kind;
use rusqlite::types::ToSql;
use rusqlite::{OptionalExtension, params, params_from_iter};

pub type MetaByItem = std::collections::HashMap<i64, std::collections::HashMap<String, String>>;

impl Store {
    pub fn set_group(&self, id: i64, key: &str) -> Result<()> {
        self.raw().execute(
            "UPDATE items SET group_key = ?2 WHERE id = ?1",
            params![id, key],
        )?;
        Ok(())
    }

    pub fn ungrouped(&self, limit: usize) -> Result<Vec<(i64, Option<Kind>, String)>> {
        let mut stmt = self.raw().prepare(
            "SELECT id, kind, SUBSTR(preview_text, 1, 400) FROM items
             WHERE group_key = '' AND deleted_at IS NULL
               AND kind IN ('link', 'folder', 'file')
             ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map([i64::try_from(limit).unwrap_or(i64::MAX)], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        Ok(rows
            .collect::<rusqlite::Result<Vec<_>>>()?
            .into_iter()
            .map(|(id, kind, preview)| (id, kind.as_deref().and_then(Kind::from_name), preview))
            .collect())
    }

    pub fn missing_meta(&self, kinds: &[&str], key: &str, limit: usize) -> Result<Vec<i64>> {
        if kinds.is_empty() {
            return Ok(Vec::new());
        }
        let marks = vec!["?"; kinds.len()].join(", ");
        let sql = format!(
            "SELECT id FROM items
             WHERE deleted_at IS NULL AND kind IN ({marks})
               AND NOT EXISTS (
                   SELECT 1 FROM item_meta
                   WHERE item_id = items.id AND key = ?
               )
             ORDER BY modified_at DESC LIMIT ?"
        );
        let mut bound: Vec<Box<dyn ToSql>> = Vec::with_capacity(kinds.len() + 2);
        for kind in kinds {
            bound.push(Box::new((*kind).to_owned()));
        }
        bound.push(Box::new(key.to_owned()));
        bound.push(Box::new(i64::try_from(limit).unwrap_or(i64::MAX)));
        let mut stmt = self.raw().prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(bound.iter()), |row| row.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn set_meta(&self, id: i64, key: &str, value: &str) -> Result<()> {
        self.raw().execute(
            "INSERT INTO item_meta (item_id, key, value) VALUES (?1, ?2, ?3)
             ON CONFLICT(item_id, key) DO UPDATE SET value = excluded.value",
            params![id, key, value],
        )?;
        Ok(())
    }

    pub fn meta(&self, id: i64, key: &str) -> Result<Option<String>> {
        Ok(self
            .raw()
            .query_row(
                "SELECT value FROM item_meta WHERE item_id = ?1 AND key = ?2",
                params![id, key],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub fn all_meta(&self, id: i64) -> Result<Vec<(String, String)>> {
        let mut stmt = self
            .raw()
            .prepare("SELECT key, value FROM item_meta WHERE item_id = ?1 ORDER BY key")?;
        let rows = stmt.query_map([id], |row| Ok((row.get(0)?, row.get(1)?)))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn meta_for(&self, ids: &[i64], keys: &[&str]) -> Result<MetaByItem> {
        if ids.is_empty() || keys.is_empty() {
            return Ok(MetaByItem::new());
        }
        let marks = |how_many: usize| vec!["?"; how_many].join(", ");
        let sql = format!(
            "SELECT item_id, key, value FROM item_meta
             WHERE item_id IN ({}) AND key IN ({})",
            marks(ids.len()),
            marks(keys.len())
        );
        let mut bound: Vec<Box<dyn ToSql>> = Vec::with_capacity(ids.len() + keys.len());
        for id in ids {
            bound.push(Box::new(*id));
        }
        for key in keys {
            bound.push(Box::new((*key).to_owned()));
        }
        let mut stmt = self.raw().prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(bound.iter()), |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        let mut found = MetaByItem::new();
        for row in rows {
            let (id, key, value) = row?;
            found.entry(id).or_default().insert(key, value);
        }
        Ok(found)
    }

    pub fn enqueue(&self, id: i64, job: &str) -> Result<()> {
        self.raw().execute(
            "INSERT OR IGNORE INTO pending_work (item_id, job) VALUES (?1, ?2)",
            params![id, job],
        )?;
        Ok(())
    }

    pub fn take_pending(&self, job: &str, now: i64, limit: usize) -> Result<Vec<i64>> {
        let mut stmt = self.raw().prepare(
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
        self.raw().execute(
            "DELETE FROM pending_work WHERE item_id = ?1 AND job = ?2",
            params![id, job],
        )?;
        Ok(())
    }

    pub const MAX_ATTEMPTS: i64 = 3;

    pub fn work_failed(&self, id: i64, job: &str, why: &str, retry_at: i64) -> Result<bool> {
        self.raw().execute(
            "UPDATE pending_work
             SET attempts = attempts + 1, last_error = ?3, not_before = ?4
             WHERE item_id = ?1 AND job = ?2",
            params![id, job, why, retry_at],
        )?;
        let attempts: i64 = self
            .raw()
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
}
