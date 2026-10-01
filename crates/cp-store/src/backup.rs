use crate::{Blobs, Error, Result, Store};
use rusqlite::Connection;
use std::path::Path;

pub const FORMAT: u32 = 1;
pub const EXTENSION: &str = "cpbackup";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Made {
    pub items: i64,
    pub blobs: i64,
    pub missing: i64,
    pub bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Brought {
    pub added: i64,
    pub already: i64,
    pub refused: i64,
    pub blobs: i64,
    pub from_elsewhere: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Taken {
    pub format: u32,
    pub schema: u32,
    pub items: i64,
    pub written_at: i64,
    pub bytes: u64,
    pub platform: Option<String>,
}

pub fn write(store: &Store, to: &Path, at: i64) -> Result<Made> {
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent).map_err(Error::Io)?;
    }
    if lives_at(store.raw()).is_some_and(|live| same_file(&live, to)) {
        return Err(Error::OntoItself);
    }
    let landing = sidecar(to, ".part");
    for path in [
        landing.clone(),
        sidecar(&landing, "-wal"),
        sidecar(&landing, "-shm"),
    ] {
        remove(&path)?;
    }
    let made = written_at(store, &landing, at);
    if made.is_err() {
        for path in [
            landing.clone(),
            sidecar(&landing, "-wal"),
            sidecar(&landing, "-shm"),
        ] {
            let _ = remove(&path);
        }
        return made;
    }
    std::fs::rename(&landing, to).map_err(Error::Io)?;
    crate::store::restrict(to, 0o600)?;
    made.map(|made| Made {
        bytes: weighed(to),
        ..made
    })
}

fn written_at(store: &Store, to: &Path, at: i64) -> Result<Made> {
    store
        .raw()
        .execute("VACUUM INTO ?1", [&to.to_string_lossy()])?;
    crate::store::restrict(to, 0o600)?;

    let copy = Connection::open(to)?;
    copy.execute_batch(
        "CREATE TABLE backup_blobs (digest TEXT PRIMARY KEY, bytes BLOB NOT NULL);
         CREATE TABLE backup_note (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
    )?;

    let blobs = store.blobs();
    let mut kept = 0;
    let mut missing = 0;
    if let Some(blobs) = blobs {
        let digests = wanted(&copy)?;
        let mut insert =
            copy.prepare("INSERT OR IGNORE INTO backup_blobs (digest, bytes) VALUES (?1, ?2)")?;
        for digest in digests {
            let Some(bytes) = blobs.get(&digest)? else {
                missing += 1;
                continue;
            };
            insert.execute(rusqlite::params![digest, bytes])?;
            kept += 1;
        }
    }

    let items: i64 = copy.query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0))?;
    for (key, value) in [
        ("format", FORMAT.to_string()),
        ("schema", crate::SCHEMA_VERSION.to_string()),
        ("written_at", at.to_string()),
        ("items", items.to_string()),
        ("platform", WHERE_IT_WAS_MADE.to_owned()),
    ] {
        copy.execute(
            "INSERT OR REPLACE INTO backup_note (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, value],
        )?;
    }
    copy.execute_batch("PRAGMA journal_mode = DELETE; VACUUM;")?;
    drop(copy);
    for side in [sidecar(to, "-wal"), sidecar(to, "-shm")] {
        remove(&side)?;
    }

    Ok(Made {
        items,
        blobs: kept,
        missing,
        bytes: weighed(to),
    })
}

#[cfg(target_os = "windows")]
pub const WHERE_IT_WAS_MADE: &str = "windows";
#[cfg(target_os = "macos")]
pub const WHERE_IT_WAS_MADE: &str = "macos";
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub const WHERE_IT_WAS_MADE: &str = "elsewhere";

pub fn read(from: &Path) -> Result<Taken> {
    let copy = Connection::open_with_flags(from, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let format = u32::try_from(noted(&copy, "format")?.ok_or(Error::NotABackup)?)
        .map_err(|_| Error::NotABackup)?;
    if format > FORMAT {
        return Err(Error::BackupFromTheFuture {
            found: format,
            supported: FORMAT,
        });
    }
    Ok(Taken {
        format,
        schema: u32::try_from(noted(&copy, "schema")?.unwrap_or(0)).unwrap_or(0),
        items: noted(&copy, "items")?.unwrap_or(0),
        written_at: noted(&copy, "written_at")?.unwrap_or(0),
        bytes: weighed(from),
        platform: said(&copy, "platform")?,
    })
}

pub fn bring(from: &Path, into: &Store, at: i64, say: &dyn Fn(&str)) -> Result<Brought> {
    let taken = read(from)?;
    if taken.schema > crate::SCHEMA_VERSION {
        return Err(Error::FromTheFuture {
            found: taken.schema,
            supported: crate::SCHEMA_VERSION,
        });
    }
    let opened = Opened::of(from)?;
    let source = Store::open(&opened.db)?;
    let mut brought = Brought {
        added: 0,
        already: 0,
        refused: 0,
        blobs: opened.blobs,
        from_elsewhere: taken
            .platform
            .as_deref()
            .is_some_and(|made| made != WHERE_IT_WAS_MADE),
    };
    let mut after = None;
    loop {
        let page = source.list(&everything(), Store::PAGE, after)?;
        if page.rows.is_empty() {
            break;
        }
        for row in &page.rows {
            match carry(&source, into, row, at) {
                Ok(true) => brought.added += 1,
                Ok(false) => brought.already += 1,
                Err(why) => {
                    say(&format!(
                        "what was copied on {} could not be brought over: {why}",
                        row.created_at
                    ));
                    brought.refused += 1;
                }
            }
        }
        after = page.next;
        if after.is_none() {
            break;
        }
    }
    Ok(brought)
}

fn carry(source: &Store, into: &Store, row: &crate::Listed, at: i64) -> Result<bool> {
    let Some(item) = source.item(row.id)? else {
        return Ok(false);
    };
    if into.find_by_hash(&item)?.is_some() {
        return Ok(false);
    }
    if !item.is_comparable() && taken(into, &named(source, row.id, at)) {
        return Ok(false);
    }
    let meta = source.all_meta(row.id)?;
    let nothing: &[&str] = &[];
    let more = crate::More {
        modified_at: Some(row.modified_at),
        touched_at: Some(at),
        used_at: row.last_used_at,
        app: row.app.as_deref(),
        label: row.label.as_deref(),
        color: row.color,
        pinned: row.pinned,
        pastes: row.paste_count,
        broken: row.broken_since,
        meta: &meta,
        jobs: row
            .kind
            .map_or(nothing, |kind| crate::legacy::jobs_for(kind, true)),
    };
    let id = into.insert_full(
        &free_name(source, into, row.id, at),
        &item,
        &row.preview,
        row.created_at,
        &more,
    )?;
    if !row.group.is_empty() {
        into.set_group(id, &row.group)?;
    }
    if let Some(text) = source.ocr_text(row.id)? {
        into.set_ocr_text(id, &text, at)?;
    }
    Ok(true)
}

fn free_name(source: &Store, into: &Store, id: i64, at: i64) -> String {
    let said = named(source, id, at);
    if !taken(into, &said) {
        return said;
    }
    let mut turn = 1;
    loop {
        let again = format!("{said}-{turn}");
        if !taken(into, &again) {
            return again;
        }
        turn += 1;
    }
}

fn taken(into: &Store, uuid: &str) -> bool {
    into.raw()
        .query_row("SELECT 1 FROM items WHERE uuid = ?1", [uuid], |_| Ok(()))
        .is_ok()
}

fn named(source: &Store, id: i64, at: i64) -> String {
    source
        .raw()
        .query_row("SELECT uuid FROM items WHERE id = ?1", [id], |row| {
            row.get::<_, String>(0)
        })
        .unwrap_or_else(|_| format!("{at:x}-{id:08x}"))
}

struct Opened {
    dir: std::path::PathBuf,
    db: std::path::PathBuf,
    blobs: i64,
}

impl Opened {
    fn of(from: &Path) -> Result<Self> {
        let dir = std::env::temp_dir().join(format!(
            "cp-backup-{}-{}-{:x}",
            std::process::id(),
            TURN.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |since| since.subsec_nanos())
        ));
        std::fs::create_dir_all(&dir).map_err(Error::Io)?;
        crate::store::restrict(&dir, 0o700)?;
        let mut opened = Self {
            db: dir.join("history.db"),
            blobs: 0,
            dir,
        };
        std::fs::copy(from, &opened.db).map_err(Error::Io)?;
        let blobs = Blobs::at(&opened.dir.join("blobs"))?;
        let coming = Connection::open(&opened.db)?;
        let mut kept = 0;
        {
            let mut stmt = coming.prepare("SELECT digest, bytes FROM backup_blobs")?;
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                let digest: String = row.get(0)?;
                let bytes: Vec<u8> = row.get(1)?;
                if blobs.put(&bytes)? == digest {
                    kept += 1;
                }
            }
        }
        coming.execute_batch(
            "DROP TABLE IF EXISTS backup_blobs;
             DROP TABLE IF EXISTS backup_note;",
        )?;
        drop(coming);
        opened.blobs = kept;
        Ok(opened)
    }
}

impl Drop for Opened {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

static TURN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn everything() -> crate::Filter {
    crate::Filter {
        broken: crate::Broken::Shown,
        ..Default::default()
    }
}

fn lives_at(db: &Connection) -> Option<std::path::PathBuf> {
    db.query_row(
        "SELECT file FROM pragma_database_list WHERE name = 'main'",
        [],
        |row| row.get::<_, String>(0),
    )
    .ok()
    .filter(|said| !said.is_empty())
    .map(std::path::PathBuf::from)
}

fn same_file(one: &Path, other: &Path) -> bool {
    let settled = |path: &Path| {
        path.canonicalize().unwrap_or_else(|_| {
            path.parent()
                .and_then(|parent| parent.canonicalize().ok())
                .map_or_else(
                    || path.to_path_buf(),
                    |parent| match path.file_name() {
                        Some(name) => parent.join(name),
                        None => parent,
                    },
                )
        })
    };
    settled(one) == settled(other)
}

fn wanted(db: &Connection) -> Result<Vec<String>> {
    let mut stmt =
        db.prepare("SELECT DISTINCT digest FROM item_formats WHERE digest IS NOT NULL")?;
    let found = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(found)
}

fn noted(db: &Connection, key: &str) -> Result<Option<i64>> {
    Ok(said(db, key)?.and_then(|said| said.parse().ok()))
}

fn said(db: &Connection, key: &str) -> Result<Option<String>> {
    let found: Option<String> = db
        .query_row(
            "SELECT value FROM backup_note WHERE key = ?1",
            [key],
            |row| row.get(0),
        )
        .or_else(|why| match why {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            rusqlite::Error::SqliteFailure(_, Some(ref said)) if said.contains("no such table") => {
                Ok(None)
            }
            other => Err(other),
        })?;
    Ok(found)
}

fn sidecar(path: &Path, tail: &str) -> std::path::PathBuf {
    let mut said = path.as_os_str().to_os_string();
    said.push(tail);
    std::path::PathBuf::from(said)
}

fn remove(path: &Path) -> Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(why) => Err(Error::Io(why)),
    }
}

fn weighed(path: &Path) -> u64 {
    std::fs::metadata(path).map_or(0, |it| it.len())
}

#[cfg(test)]
#[path = "backup_test.rs"]
mod tests;
