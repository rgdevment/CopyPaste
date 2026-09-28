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

pub fn bring(from: &Path, into: &Store, at: i64) -> Result<Brought> {
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
                Err(_) => brought.refused += 1,
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
    let id = into.insert_item(
        &named(source, row.id, at),
        &item,
        &row.preview,
        row.created_at,
    )?;
    if let Some(app) = row.app.as_deref() {
        into.set_source(id, app, at)?;
    }
    if let Some(label) = row.label.as_deref() {
        into.set_label(id, Some(label), at)?;
    }
    if row.color != 0 {
        into.set_color(id, row.color, at)?;
    }
    if row.pinned {
        into.set_pinned(id, true, at)?;
    }
    if let Some(text) = source.ocr_text(row.id)? {
        into.set_ocr_text(id, &text, at)?;
    }
    Ok(true)
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
        db.prepare("SELECT DISTINCT blob_path FROM item_formats WHERE blob_path IS NOT NULL")?;
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
mod tests {
    use super::*;
    use cp_core::item::{Format, Item, Payload, SYNTHETIC_IMAGE, SYNTHETIC_TEXT};
    use cp_core::kind::Kind;

    fn text(what: &str) -> Item {
        Item {
            kind: Some(Kind::Text),
            formats: vec![Format {
                id: SYNTHETIC_TEXT.into(),
                payload: Payload::Inline(what.as_bytes().to_vec()),
            }],
        }
    }

    fn heavy() -> Item {
        Item {
            kind: Some(Kind::Image),
            formats: vec![Format {
                id: SYNTHETIC_IMAGE.into(),
                payload: Payload::Blob(vec![7u8; cp_core::item::INLINE_UP_TO + 64]),
            }],
        }
    }

    #[cfg(target_os = "windows")]
    fn files(paths: &[&str]) -> Item {
        Item {
            kind: Some(Kind::File),
            formats: vec![Format {
                id: "CF_HDROP".into(),
                payload: Payload::Inline(paths.join("\0").into_bytes()),
            }],
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn files(paths: &[&str]) -> Item {
        let urls = paths
            .iter()
            .map(|path| format!("file://{path}"))
            .collect::<Vec<_>>()
            .join("\n");
        Item {
            kind: Some(Kind::File),
            formats: vec![Format {
                id: "public.file-url".into(),
                payload: Payload::Inline(urls.into_bytes()),
            }],
        }
    }

    fn somewhere(name: &str) -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().expect("a folder");
        let store = Store::open(&dir.path().join(name)).expect("opened");
        (dir, store)
    }

    #[test]
    fn what_was_kept_travels_whole_from_one_machine_to_another() {
        let (there, store) = somewhere("history.db");
        store
            .insert_item("one", &text("the first thing"), "the first thing", 1_000)
            .expect("stored");
        let heavy_id = store
            .insert_item("two", &heavy(), "", 2_000)
            .expect("stored");
        store.set_pinned(heavy_id, true, 2_000).expect("pinned");
        let backup = there.path().join("mine.cpbackup");
        let made = write(&store, &backup, 3_000).expect("exported");
        assert_eq!(made.items, 2);
        assert_eq!(made.blobs, 1, "the blob had to travel along");
        assert!(made.bytes > 0);
        drop(store);

        let taken = read(&backup).expect("read");
        assert_eq!(taken.format, FORMAT);
        assert_eq!(taken.items, 2);
        assert_eq!(taken.written_at, 3_000);

        let (_here, landed) = somewhere("history.db");
        let brought = bring(&backup, &landed, 4_000).expect("imported");
        assert_eq!(brought.added, 2);
        assert_eq!(brought.already, 0);
        assert_eq!(landed.count().expect("counted"), 2);
        let page = landed
            .list(&crate::Filter::default(), 10, None)
            .expect("listed");
        let picture = page
            .rows
            .iter()
            .find(|one| one.kind == Some(Kind::Image))
            .expect("the image arrived");
        assert!(picture.pinned, "what was pinned is still pinned");
        let payload = landed
            .payload_of(picture.id, SYNTHETIC_IMAGE)
            .expect("read")
            .expect("the blob is there");
        assert_eq!(payload.len(), cp_core::item::INLINE_UP_TO + 64);
    }

    #[test]
    fn importing_adds_and_never_takes_away_what_was_already_there() {
        let (there, store) = somewhere("history.db");
        store
            .insert_item("one", &text("from the backup"), "from the backup", 1_000)
            .expect("stored");
        let backup = there.path().join("mine.cpbackup");
        write(&store, &backup, 1_000).expect("exported");
        drop(store);

        let (_here, landed) = somewhere("history.db");
        for (at, what) in [(1, "one"), (2, "two"), (3, "three")] {
            landed
                .insert_item(&format!("v{at}"), &text(what), what, at)
                .expect("stored");
        }
        let brought = bring(&backup, &landed, 5_000).expect("imported");
        assert_eq!(brought.added, 1);
        assert_eq!(
            landed.count().expect("counted"),
            4,
            "none of what was already there went away"
        );
    }

    #[test]
    fn the_same_copy_brought_twice_does_not_double_anything() {
        let (there, store) = somewhere("history.db");
        store
            .insert_item("one", &text("something"), "something", 1_000)
            .expect("stored");
        store
            .insert_item("two", &text("something else"), "something else", 2_000)
            .expect("stored");
        let backup = there.path().join("mine.cpbackup");
        write(&store, &backup, 1_000).expect("exported");
        drop(store);

        let (_here, landed) = somewhere("history.db");
        let first = bring(&backup, &landed, 3_000).expect("imported");
        let again = bring(&backup, &landed, 4_000).expect("imported again");
        assert_eq!(first.added, 2);
        assert_eq!(again.added, 0);
        assert_eq!(again.already, 2);
        assert_eq!(landed.count().expect("counted"), 2);
    }

    #[test]
    fn a_copy_refuses_to_be_written_over_the_history_it_is_copying() {
        let (there, store) = somewhere("history.db");
        store
            .insert_item("one", &text("what is there"), "what is there", 1_000)
            .expect("stored");
        let live = there.path().join("history.db");
        assert!(
            matches!(write(&store, &live, 2_000), Err(Error::OntoItself)),
            "the history cannot be its own copy"
        );
        assert_eq!(store.count().expect("counted"), 1, "and it is still whole");
        drop(store);
        assert_eq!(
            Store::open(&live)
                .expect("reopened")
                .count()
                .expect("counted"),
            1,
            "reopening it from disk confirms it"
        );
    }

    #[test]
    fn a_file_that_can_no_longer_be_found_still_travels_and_still_arrives() {
        let (there, store) = somewhere("history.db");
        let id = store
            .insert_item("one", &files(&["/tmp/gone.txt"]), "/tmp/gone.txt", 1_000)
            .expect("stored");
        store.mark_broken(id, 2_000).expect("marked broken");
        let backup = there.path().join("mine.cpbackup");
        assert_eq!(write(&store, &backup, 3_000).expect("exported").items, 1);
        drop(store);

        let (_here, landed) = somewhere("history.db");
        let brought = bring(&backup, &landed, 4_000).expect("imported");
        assert_eq!(
            brought.added, 1,
            "what is broken does not get left out silently"
        );
        assert_eq!(landed.count().expect("counted"), 1);
    }

    #[test]
    fn a_backup_that_cannot_be_written_does_not_take_the_previous_one_with_it() {
        let (there, store) = somewhere("history.db");
        store
            .insert_item(
                "one",
                &text("what was already there"),
                "what was already there",
                1_000,
            )
            .expect("stored");
        let backup = there.path().join("mine.cpbackup");
        write(&store, &backup, 1_000).expect("exported");
        let before = std::fs::read(&backup).expect("read");

        let nowhere = there.path().join("dead-end");
        std::fs::write(&nowhere, b"I am not a folder").expect("written");
        assert!(
            write(&store, &nowhere.join("another.cpbackup"), 2_000).is_err(),
            "writing inside a file cannot end well"
        );
        assert_eq!(
            std::fs::read(&backup).expect("read"),
            before,
            "the previous copy is still whole"
        );
    }

    #[test]
    fn a_half_written_backup_never_takes_the_name_of_the_good_one() {
        let (there, store) = somewhere("history.db");
        store
            .insert_item("one", &text("something"), "something", 1_000)
            .expect("stored");
        let backup = there.path().join("mine.cpbackup");
        write(&store, &backup, 1_000).expect("exported");
        assert!(backup.exists());
        assert!(
            !there.path().join("mine.cpbackup.part").exists(),
            "the half-written file does not stick around"
        );
        assert!(read(&backup).is_ok(), "what is left behind is readable");
    }

    #[test]
    fn a_backup_says_where_it_was_made_so_the_other_system_can_warn() {
        let (there, store) = somewhere("history.db");
        store
            .insert_item("one", &text("something"), "something", 1_000)
            .expect("stored");
        let backup = there.path().join("mine.cpbackup");
        write(&store, &backup, 1_000).expect("exported");
        assert_eq!(
            read(&backup).expect("read").platform.as_deref(),
            Some(WHERE_IT_WAS_MADE)
        );

        let (_here, landed) = somewhere("history.db");
        let brought = bring(&backup, &landed, 2_000).expect("imported");
        assert!(!brought.from_elsewhere, "it comes from this very platform");
    }

    #[test]
    fn a_blob_the_sweep_took_before_it_could_travel_is_counted_and_not_hidden() {
        let (there, store) = somewhere("history.db");
        store
            .insert_item("one", &heavy(), "", 1_000)
            .expect("stored");
        for digest in std::fs::read_dir(there.path().join("blobs"))
            .expect("blobs")
            .filter_map(|one| one.ok())
        {
            let _ = std::fs::remove_dir_all(digest.path());
        }
        let backup = there.path().join("mine.cpbackup");
        let made = write(&store, &backup, 2_000).expect("exported");
        assert_eq!(made.blobs, 0);
        assert_eq!(
            made.missing, 1,
            "what went missing gets counted, not hidden"
        );
    }

    #[test]
    fn what_was_opened_to_read_a_backup_is_swept_when_it_is_done() {
        let (there, store) = somewhere("history.db");
        store
            .insert_item("one", &text("something"), "something", 1_000)
            .expect("stored");
        let backup = there.path().join("mine.cpbackup");
        write(&store, &backup, 1_000).expect("exported");
        drop(store);

        let dir = {
            let opened = Opened::of(&backup).expect("opened");
            assert!(
                opened.db.exists(),
                "the copy gets materialised so it can be read"
            );
            opened.dir.clone()
        };
        assert!(!dir.exists(), "the opened copy stayed on disk");
    }

    #[test]
    fn what_is_brought_in_is_a_history_and_not_a_backup() {
        let (there, store) = somewhere("history.db");
        store
            .insert_item("one", &text("something"), "something", 1_000)
            .expect("stored");
        let backup = there.path().join("mine.cpbackup");
        write(&store, &backup, 1_000).expect("exported");
        drop(store);

        let (_here, landed) = somewhere("history.db");
        bring(&backup, &landed, 2_000).expect("imported");
        let left: i64 = landed
            .raw()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE name LIKE 'backup_%'",
                [],
                |row| row.get(0),
            )
            .expect("counted");
        assert_eq!(left, 0);
    }

    #[test]
    fn a_file_that_is_not_a_backup_is_refused_before_anything_is_touched() {
        let dir = tempfile::tempdir().expect("a folder");
        let stranger = dir.path().join("anything.cpbackup");
        std::fs::write(&stranger, b"I am not a database").expect("written");
        assert!(read(&stranger).is_err());

        let plain = dir.path().join("plain.db");
        let store = Store::open(&plain).expect("opened");
        store
            .insert_item("one", &text("something"), "something", 1_000)
            .expect("stored");
        drop(store);
        assert!(
            matches!(read(&plain), Err(Error::NotABackup)),
            "a loose database is not a backup"
        );

        let (_here, landed) = somewhere("history.db");
        landed
            .insert_item("mine", &text("mine"), "mine", 1_000)
            .expect("stored");
        assert!(bring(&plain, &landed, 2_000).is_err());
        assert_eq!(
            landed.count().expect("counted"),
            1,
            "what was theirs is still there"
        );
    }

    #[test]
    fn exporting_twice_over_the_same_file_simply_writes_it_again() {
        let (there, store) = somewhere("history.db");
        store
            .insert_item("one", &text("something"), "something", 1_000)
            .expect("stored");
        let backup = there.path().join("mine.cpbackup");
        let first = write(&store, &backup, 1_000).expect("exported");
        store
            .insert_item("two", &text("something else"), "something else", 2_000)
            .expect("stored");
        let again = write(&store, &backup, 2_000).expect("exported again");
        assert_eq!(first.items, 1);
        assert_eq!(again.items, 2);
        assert_eq!(read(&backup).expect("read").items, 2);
    }

    #[test]
    fn an_empty_history_still_makes_a_backup_that_can_be_brought_back() {
        let (there, store) = somewhere("history.db");
        let backup = there.path().join("empty.cpbackup");
        let made = write(&store, &backup, 1_000).expect("exported");
        assert_eq!(made.items, 0);
        assert_eq!(made.blobs, 0);
        drop(store);

        let (_here, landed) = somewhere("history.db");
        let brought = bring(&backup, &landed, 2_000).expect("imported");
        assert_eq!(brought.added, 0);
        assert_eq!(landed.count().expect("counted"), 0);
    }
}
