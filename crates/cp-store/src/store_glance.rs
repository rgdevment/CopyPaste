use super::Store;
use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Glance {
    pub items: i64,
    pub today: i64,
    pub pinned: i64,
    pub bytes: i64,
}

impl Store {
    pub fn glance(&self, since: i64) -> Result<Glance> {
        let usage = self.usage()?;
        let (today, pinned): (i64, i64) = self.db.query_row(
            "SELECT COALESCE(SUM(modified_at >= ?1), 0), COALESCE(SUM(pinned), 0)
             FROM items WHERE deleted_at IS NULL",
            [since],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        Ok(Glance {
            items: usage.items,
            today,
            pinned,
            bytes: usage.bytes,
        })
    }
}
