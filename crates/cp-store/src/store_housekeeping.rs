use super::{Gone, Store};
use crate::Result;

impl Store {
    pub fn sweep(&self, policy: &Policy, now: i64) -> Result<Swept> {
        let mut swept = self.enforce(policy, now)?;
        swept.orphans = self.collect_orphans()?;
        swept.truncated = self.checkpoint()?;
        self.vacuum_step(VACUUM_PAGES)?;
        Ok(swept)
    }

    pub fn sweep_thumbs(&self, dir: &std::path::Path) -> Result<usize> {
        let kept = self.referenced_thumbs()?;
        let mut removed = 0;
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Ok(0);
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !entry.file_type().is_ok_and(|kind| kind.is_file()) {
                continue;
            }
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if kept.contains(name) || crate::blobs::is_fresh(&path) {
                continue;
            }
            crate::blobs::remove_at(&path)?;
            removed += 1;
        }
        Ok(removed)
    }

    fn referenced_thumbs(&self) -> Result<std::collections::HashSet<String>> {
        let mut stmt = self.db.prepare(
            "SELECT thumb_path FROM items WHERE thumb_path IS NOT NULL AND deleted_at IS NULL",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut kept = std::collections::HashSet::new();
        for said in rows {
            let said = said?;
            if let Some(name) = std::path::Path::new(&said)
                .file_name()
                .and_then(|name| name.to_str())
            {
                kept.insert(name.to_owned());
            }
        }
        Ok(kept)
    }

    fn collect_orphans(&self) -> Result<usize> {
        let Some(blobs) = &self.blobs else {
            return Ok(0);
        };
        let referenced = self.referenced_blobs()?;
        blobs.sweep(&|digest| referenced.contains(digest))
    }

    fn enforce(&self, policy: &Policy, now: i64) -> Result<Swept> {
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
            let mut gone = Gone::default();
            let transaction = self.db.unchecked_transaction()?;
            for (id, bytes) in candidates {
                gone.and(self.erase(id, at)?);
                evicted += 1;
                freed += bytes;
                if freed >= usage - limit {
                    break;
                }
            }
            transaction.commit()?;
            self.forget(&gone)?;
        }
    }

    fn eviction_candidates(&self) -> Result<Vec<(i64, i64)>> {
        let mut stmt = self.db.prepare(
            "SELECT items.id, COALESCE(SUM(COALESCE(LENGTH(f.inline_data), f.size_bytes, 0)), 0)
             FROM items LEFT JOIN item_formats f
               ON f.item_id = items.id AND (f.inline_data IS NOT NULL OR f.digest IS NOT NULL)
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
            .prepare("SELECT DISTINCT digest FROM item_formats WHERE digest IS NOT NULL")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Policy {
    pub keep_for: Option<i64>,
    pub keep_at_most: Option<i64>,
    pub bytes_at_most: Option<i64>,
    pub broken_for: Option<i64>,
}

pub const A_DAY: i64 = 24 * 60 * 60 * 1_000;

const VACUUM_PAGES: u32 = 256;

const _: () = assert!(VACUUM_PAGES > 0);

impl Policy {
    pub fn keeping(days: Option<u16>, quota_mb: Option<u32>) -> Self {
        Self {
            keep_for: days
                .filter(|days| *days > 0)
                .map(|days| i64::from(days) * A_DAY),
            keep_at_most: None,
            bytes_at_most: quota_mb
                .filter(|mb| *mb > 0)
                .map(|mb| i64::from(mb) * 1024 * 1024),
            broken_for: None,
        }
    }
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
