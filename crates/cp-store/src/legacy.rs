use crate::{Error, Result, Store};
use cp_core::item::{Format, Item, Payload, SYNTHETIC_IMAGE, SYNTHETIC_TEXT};
use cp_core::kind::Kind;
use rusqlite::{Connection, OpenFlags};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Former {
    pub items: i64,
    pub pictures: i64,
    pub pictures_gone: i64,
    pub pinned: i64,
    pub labelled: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Brought {
    pub added: i64,
    pub already: i64,
    pub refused: i64,
    pub without_their_picture: i64,
}

pub fn kind_of(said: i64) -> Option<Kind> {
    match said {
        0 => Some(Kind::Text),
        1 => Some(Kind::Image),
        2 => Some(Kind::File),
        3 => Some(Kind::Folder),
        4 => Some(Kind::Link),
        5 => Some(Kind::Audio),
        6 => Some(Kind::Video),
        7 => Some(Kind::Email),
        8 => Some(Kind::Phone),
        9 => Some(Kind::Color),
        10 => Some(Kind::Ip),
        11 => Some(Kind::Uuid),
        12 => Some(Kind::Json),
        _ => Some(Kind::Text),
    }
}

pub const fn holds_a_path(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::Image | Kind::File | Kind::Folder | Kind::Audio | Kind::Video
    )
}

struct Row {
    uuid: String,
    content: String,
    kind: Kind,
    created_at: i64,
    modified_at: i64,
    app: Option<String>,
    pinned: bool,
    label: Option<String>,
    colour: i64,
    meta: Option<String>,
    thumb: Option<String>,
    broken: bool,
}

fn opened(from: &Path) -> Result<Connection> {
    let db = Connection::open_with_flags(from, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let found: i64 = db
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'clipboard_items'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if found == 0 {
        return Err(Error::NotTheFormerOne);
    }
    Ok(db)
}

pub fn look(from: &Path) -> Result<Former> {
    let db = opened(from)?;
    let mut stmt = db.prepare("SELECT type, content, is_pinned, label FROM clipboard_items")?;
    let mut rows = stmt.query([])?;
    let mut former = Former::default();
    while let Some(row) = rows.next()? {
        let kind = kind_of(row.get::<_, i64>(0).unwrap_or(0)).unwrap_or(Kind::Text);
        let content: String = row.get(1).unwrap_or_default();
        former.items += 1;
        if row.get::<_, Option<i64>>(2).ok().flatten().unwrap_or(0) != 0 {
            former.pinned += 1;
        }
        if row
            .get::<_, Option<String>>(3)
            .ok()
            .flatten()
            .is_some_and(|one| !one.is_empty())
        {
            former.labelled += 1;
        }
        if kind == Kind::Image {
            former.pictures += 1;
            if !Path::new(&content).exists() {
                former.pictures_gone += 1;
            }
        }
    }
    Ok(former)
}

pub fn bring(from: &Path, into: &Store, at: i64) -> Result<Brought> {
    let db = opened(from)?;
    let mut stmt = db.prepare(
        "SELECT id, content, type, created_at, modified_at, app_source, is_pinned, label,
                card_color, metadata, thumb_path, broken_since
         FROM clipboard_items
         ORDER BY created_at",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Row {
            uuid: row.get::<_, Option<String>>(0)?.unwrap_or_default(),
            content: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            kind: kind_of(row.get::<_, Option<i64>>(2)?.unwrap_or(0)).unwrap_or(Kind::Text),
            created_at: row.get::<_, Option<i64>>(3)?.unwrap_or(0),
            modified_at: row.get::<_, Option<i64>>(4)?.unwrap_or(0),
            app: row
                .get::<_, Option<String>>(5)?
                .filter(|one| !one.is_empty()),
            pinned: row.get::<_, Option<i64>>(6)?.unwrap_or(0) != 0,
            label: row
                .get::<_, Option<String>>(7)?
                .filter(|one| !one.is_empty()),
            colour: row.get::<_, Option<i64>>(8)?.unwrap_or(0),
            meta: row
                .get::<_, Option<String>>(9)?
                .filter(|one| !one.is_empty()),
            thumb: row
                .get::<_, Option<String>>(10)?
                .filter(|one| !one.is_empty()),
            broken: row.get::<_, Option<i64>>(11)?.is_some(),
        })
    })?;

    let mut brought = Brought::default();
    for row in rows {
        let Ok(row) = row else {
            brought.refused += 1;
            continue;
        };
        match carry(into, &row, at) {
            Ok(Landed::Added { without_picture }) => {
                brought.added += 1;
                if without_picture {
                    brought.without_their_picture += 1;
                }
            }
            Ok(Landed::Already) => brought.already += 1,
            Err(_) => brought.refused += 1,
        }
    }
    Ok(brought)
}

enum Landed {
    Added { without_picture: bool },
    Already,
}

fn carry(into: &Store, row: &Row, at: i64) -> Result<Landed> {
    let (item, without_picture) = made_of(row);
    if into.find_by_hash(&item)?.is_some() {
        return Ok(Landed::Already);
    }
    let when = if row.created_at > 0 {
        row.created_at
    } else {
        at
    };
    let id = into.insert_item(&named(row, at), &item, &preview_of(row), when)?;

    if let Some(app) = row.app.as_deref() {
        into.set_source(id, app, at)?;
    }
    if let Some(label) = row.label.as_deref() {
        into.set_label(id, Some(label), at)?;
    }
    if row.colour != 0 {
        into.set_color(id, row.colour, at)?;
    }
    if row.pinned {
        into.set_pinned(id, true, at)?;
    }
    if let Some(thumb) = row.thumb.as_deref()
        && Path::new(thumb).exists()
    {
        into.set_thumb(id, Some(thumb), at)?;
    }
    if row.broken {
        into.mark_broken(id, at)?;
    }
    for (key, value) in meta_in(row.meta.as_deref()) {
        into.set_meta(id, &key, &value)?;
    }
    Ok(Landed::Added { without_picture })
}

fn made_of(row: &Row) -> (Item, bool) {
    if row.kind == Kind::Image {
        return match std::fs::read(&row.content) {
            Ok(bytes) if !bytes.is_empty() => (
                Item {
                    kind: Some(Kind::Image),
                    formats: vec![Format {
                        id: SYNTHETIC_IMAGE.into(),
                        payload: Payload::Blob(bytes),
                    }],
                },
                false,
            ),
            _ => (text_of(row, Kind::Image), true),
        };
    }
    (text_of(row, row.kind), false)
}

fn text_of(row: &Row, kind: Kind) -> Item {
    Item {
        kind: Some(kind),
        formats: vec![Format {
            id: SYNTHETIC_TEXT.into(),
            payload: Payload::Inline(row.content.as_bytes().to_vec()),
        }],
    }
}

fn preview_of(row: &Row) -> String {
    row.content.clone()
}

fn named(row: &Row, at: i64) -> String {
    if row.uuid.is_empty() {
        format!("{at:x}-{:016x}", row.modified_at)
    } else {
        format!("2x-{}", row.uuid)
    }
}

pub fn meta_in(said: Option<&str>) -> Vec<(String, String)> {
    let Some(said) = said else {
        return Vec::new();
    };
    let Ok(serde_json::Value::Object(read)) = serde_json::from_str::<serde_json::Value>(said)
    else {
        return Vec::new();
    };
    read.into_iter()
        .filter_map(|(key, value)| {
            let said = match value {
                serde_json::Value::String(one) => one,
                serde_json::Value::Null => return None,
                other => other.to_string(),
            };
            (!said.is_empty()).then_some((key, said))
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Swept {
    pub files: i64,
    pub bytes: u64,
}

pub const OURS_ALONE: [&str; 3] = ["images", "config", ".initialized"];

pub fn drop_former(dir: &Path) -> Result<Swept> {
    let mut swept = Swept::default();
    let db = dir.join("clipboard.db");
    for one in [db.clone(), sidecar(&db, "-wal"), sidecar(&db, "-shm")] {
        swept = taken(&one, swept)?;
    }
    for one in OURS_ALONE {
        let at = dir.join(one);
        if at.is_dir() {
            swept = emptied(&at, swept)?;
            let _ = std::fs::remove_dir_all(&at);
        } else {
            swept = taken(&at, swept)?;
        }
    }
    Ok(swept)
}

fn emptied(dir: &Path, mut swept: Swept) -> Result<Swept> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Ok(swept);
    };
    for one in read.filter_map(std::result::Result::ok) {
        let at = one.path();
        if at.is_dir() {
            swept = emptied(&at, swept)?;
        } else {
            swept = taken(&at, swept)?;
        }
    }
    Ok(swept)
}

fn taken(at: &Path, mut swept: Swept) -> Result<Swept> {
    let Ok(weighed) = std::fs::metadata(at) else {
        return Ok(swept);
    };
    if !weighed.is_file() {
        return Ok(swept);
    }
    swept.files += 1;
    swept.bytes += weighed.len();
    crate::blobs::remove_at(at)?;
    Ok(swept)
}

fn sidecar(path: &Path, tail: &str) -> std::path::PathBuf {
    let mut said = path.as_os_str().to_os_string();
    said.push(tail);
    std::path::PathBuf::from(said)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_numbers_the_2x_stores_are_not_the_positions_of_its_enum() {
        assert_eq!(kind_of(0), Some(Kind::Text));
        assert_eq!(kind_of(1), Some(Kind::Image));
        assert_eq!(kind_of(12), Some(Kind::Json));
        assert_eq!(
            kind_of(-1),
            Some(Kind::Text),
            "what the 2.x could not tell apart arrives as text"
        );
        assert_eq!(kind_of(99), Some(Kind::Text));
    }

    #[test]
    fn what_carries_a_path_instead_of_its_content_is_known() {
        assert!(holds_a_path(Kind::Image));
        assert!(holds_a_path(Kind::File));
        assert!(holds_a_path(Kind::Video));
        assert!(!holds_a_path(Kind::Text));
        assert!(!holds_a_path(Kind::Link));
    }

    #[test]
    fn metadata_that_is_not_an_object_brings_nothing_and_does_not_fail() {
        assert!(meta_in(None).is_empty());
        assert!(meta_in(Some("")).is_empty());
        assert!(meta_in(Some("not json at all")).is_empty());
        assert!(meta_in(Some("[1, 2, 3]")).is_empty());
    }

    #[test]
    fn metadata_comes_across_as_pairs_whatever_the_json_held() {
        let mut pairs = meta_in(Some(
            r#"{"width": 1920, "artist": "alguien", "empty": "", "nothing": null}"#,
        ));
        pairs.sort();
        assert_eq!(
            pairs,
            vec![
                ("artist".to_owned(), "alguien".to_owned()),
                ("width".to_owned(), "1920".to_owned())
            ],
            "numbers keep their value, and what says nothing is left out"
        );
    }

    #[test]
    fn a_name_from_the_2x_says_where_it_came_from() {
        let row = Row {
            uuid: "abc-123".to_owned(),
            content: String::new(),
            kind: Kind::Text,
            created_at: 1,
            modified_at: 1,
            app: None,
            pinned: false,
            label: None,
            colour: 0,
            meta: None,
            thumb: None,
            broken: false,
        };
        assert_eq!(named(&row, 9), "2x-abc-123");
        let nameless = Row {
            uuid: String::new(),
            ..row
        };
        assert!(named(&nameless, 9).starts_with('9'));
    }

    const FORMER: &str = "CREATE TABLE clipboard_items (
            id TEXT NOT NULL PRIMARY KEY,
            content TEXT NOT NULL,
            type INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            modified_at INTEGER NOT NULL,
            app_source TEXT,
            is_pinned INTEGER NOT NULL DEFAULT 0,
            label TEXT,
            card_color INTEGER NOT NULL DEFAULT 0,
            metadata TEXT,
            paste_count INTEGER NOT NULL DEFAULT 0,
            content_hash TEXT,
            thumb_path TEXT,
            broken_since INTEGER);";

    fn a_former_history(dir: &Path, picture: &Path) -> std::path::PathBuf {
        let at = dir.join("clipboard.db");
        let db = Connection::open(&at).expect("opened");
        db.execute_batch(FORMER).expect("made");
        let mut put = db
            .prepare(
                "INSERT INTO clipboard_items
                 (id, content, type, created_at, modified_at, app_source, is_pinned, label,
                  card_color, metadata, paste_count, content_hash, thumb_path, broken_since)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
            )
            .expect("prepared");
        put.execute(rusqlite::params![
            "one",
            "just some text",
            0,
            1_000,
            1_000,
            "Mail",
            0,
            None::<String>,
            0,
            None::<String>,
            3,
            "h1",
            None::<String>,
            None::<i64>
        ])
        .expect("stored");
        put.execute(rusqlite::params![
            "two",
            picture.to_string_lossy(),
            1,
            2_000,
            2_000,
            "Preview",
            1,
            "the receipt",
            2,
            r#"{"width": 800}"#,
            0,
            "h2",
            None::<String>,
            None::<i64>
        ])
        .expect("stored");
        put.execute(rusqlite::params![
            "three",
            "/gone/for/good.png",
            1,
            3_000,
            3_000,
            None::<String>,
            0,
            None::<String>,
            0,
            None::<String>,
            0,
            "h3",
            None::<String>,
            None::<i64>
        ])
        .expect("stored");
        put.execute(rusqlite::params![
            "four",
            "https://example.com",
            4,
            4_000,
            4_000,
            None::<String>,
            0,
            None::<String>,
            0,
            None::<String>,
            0,
            "h4",
            None::<String>,
            9_000
        ])
        .expect("stored");
        put.execute(rusqlite::params![
            "five",
            "{\"a\": 1}",
            12,
            5_000,
            5_000,
            None::<String>,
            0,
            None::<String>,
            0,
            None::<String>,
            0,
            "h5",
            None::<String>,
            None::<i64>
        ])
        .expect("stored");
        drop(put);
        drop(db);
        at
    }

    #[test]
    fn a_history_from_the_2x_crosses_with_what_the_3_0_can_hold() {
        let there = tempfile::tempdir().expect("a folder");
        let picture = there.path().join("shot.png");
        std::fs::write(&picture, b"\x89PNG-pretend").expect("written");
        let former = a_former_history(there.path(), &picture);

        let looked = look(&former).expect("looked");
        assert_eq!(looked.items, 5);
        assert_eq!(looked.pictures, 2);
        assert_eq!(looked.pictures_gone, 1, "one picture is no longer on disk");
        assert_eq!(looked.pinned, 1);
        assert_eq!(looked.labelled, 1);

        let here = tempfile::tempdir().expect("a folder");
        let into = Store::open(&here.path().join("history.db")).expect("opened");
        let brought = bring(&former, &into, 9_999).expect("brought");
        assert_eq!(brought.added, 5);
        assert_eq!(brought.already, 0);
        assert_eq!(brought.refused, 0);
        assert_eq!(
            brought.without_their_picture, 1,
            "the one whose file is gone arrives as the path it pointed at"
        );

        let page = into
            .list(
                &crate::Filter {
                    broken: crate::Broken::Shown,
                    ..Default::default()
                },
                20,
                None,
            )
            .expect("listed");
        assert_eq!(page.rows.len(), 5);

        let picture_row = page
            .rows
            .iter()
            .find(|one| one.label.as_deref() == Some("the receipt"))
            .expect("the labelled one is here");
        assert_eq!(picture_row.kind, Some(Kind::Image));
        assert!(picture_row.pinned, "what was pinned is still pinned");
        assert_eq!(picture_row.app.as_deref(), Some("Preview"));
        assert_eq!(picture_row.color, 2, "and its colour crossed too");
        let bytes = into
            .payload_of(picture_row.id, SYNTHETIC_IMAGE)
            .expect("read")
            .expect("the picture itself, not its path");
        assert_eq!(bytes, b"\x89PNG-pretend");
        let meta = into.all_meta(picture_row.id).expect("read");
        assert_eq!(meta, vec![("width".to_owned(), "800".to_owned())]);

        let link = page
            .rows
            .iter()
            .find(|one| one.kind == Some(Kind::Link))
            .expect("the link is here");
        assert!(link.broken_since.is_some(), "what was broken stays broken");

        assert!(
            page.rows.iter().any(|one| one.kind == Some(Kind::Json)),
            "the 2.x told json apart and so does this"
        );
    }

    #[test]
    fn bringing_the_same_history_twice_adds_nothing_the_second_time() {
        let there = tempfile::tempdir().expect("a folder");
        let picture = there.path().join("shot.png");
        std::fs::write(&picture, b"png").expect("written");
        let former = a_former_history(there.path(), &picture);

        let here = tempfile::tempdir().expect("a folder");
        let into = Store::open(&here.path().join("history.db")).expect("opened");
        let first = bring(&former, &into, 1).expect("brought");
        let again = bring(&former, &into, 2).expect("brought again");
        assert_eq!(first.added, 5);
        assert_eq!(again.added, 0);
        assert_eq!(again.already, 5);
        assert_eq!(into.count().expect("counted"), 5);
    }

    #[test]
    fn what_the_2x_left_behind_is_never_written_to() {
        let there = tempfile::tempdir().expect("a folder");
        let picture = there.path().join("shot.png");
        std::fs::write(&picture, b"png").expect("written");
        let former = a_former_history(there.path(), &picture);
        let before = std::fs::metadata(&former).expect("there").len();

        let here = tempfile::tempdir().expect("a folder");
        let into = Store::open(&here.path().join("history.db")).expect("opened");
        bring(&former, &into, 1).expect("brought");

        assert_eq!(
            std::fs::metadata(&former).expect("still there").len(),
            before,
            "the former history is read, never touched"
        );
        assert!(picture.exists(), "and neither are the files it points at");
    }

    #[test]
    fn dropping_the_2x_takes_its_own_and_leaves_everything_else() {
        let dir = tempfile::tempdir().expect("a folder");
        let at = dir.path();
        std::fs::write(at.join("clipboard.db"), b"the former history").expect("written");
        std::fs::write(at.join("clipboard.db-wal"), b"its log").expect("written");
        std::fs::create_dir_all(at.join("images")).expect("made");
        std::fs::write(at.join("images/one.png"), b"a picture").expect("written");
        std::fs::create_dir_all(at.join("config")).expect("made");
        std::fs::write(at.join("config/app.json"), b"its settings").expect("written");
        std::fs::write(at.join(".initialized"), b"").expect("written");

        std::fs::write(at.join("history.db"), b"what the 3.0 keeps").expect("written");
        std::fs::create_dir_all(at.join("blobs")).expect("made");
        std::fs::write(at.join("blobs/kept"), b"ours").expect("written");
        std::fs::write(at.join("config.toml"), b"ours too").expect("written");

        let swept = drop_former(at).expect("swept");
        assert_eq!(
            swept.files, 5,
            "database, log, picture, settings and the flag"
        );
        assert!(swept.bytes > 0);

        for gone in [
            "clipboard.db",
            "clipboard.db-wal",
            "images",
            "config",
            ".initialized",
        ] {
            assert!(
                !at.join(gone).exists(),
                "{gone} was the 2.x's and had to go"
            );
        }
        for kept in ["history.db", "blobs/kept", "config.toml"] {
            assert!(at.join(kept).exists(), "{kept} is the 3.0's and stays");
        }
    }

    #[test]
    fn dropping_the_2x_never_reaches_the_files_it_only_pointed_at() {
        let dir = tempfile::tempdir().expect("a folder");
        let elsewhere = tempfile::tempdir().expect("another folder");
        let theirs = elsewhere.path().join("the report they copied.pdf");
        std::fs::write(&theirs, b"a document of their own").expect("written");
        std::fs::write(dir.path().join("clipboard.db"), b"history").expect("written");

        drop_former(dir.path()).expect("swept");
        assert!(
            theirs.exists(),
            "a file the 2.x listed is the person's, not CopyPaste's"
        );
    }

    #[test]
    fn dropping_a_folder_that_holds_nothing_of_the_2x_does_nothing_and_says_so() {
        let dir = tempfile::tempdir().expect("a folder");
        std::fs::write(dir.path().join("history.db"), b"only the 3.0 here").expect("written");
        let swept = drop_former(dir.path()).expect("swept");
        assert_eq!(swept.files, 0);
        assert!(dir.path().join("history.db").exists());
    }

    #[test]
    fn a_database_that_is_not_the_former_one_is_refused() {
        let dir = tempfile::tempdir().expect("a folder");
        let stranger = dir.path().join("whatever.db");
        let db = Connection::open(&stranger).expect("opened");
        db.execute_batch("CREATE TABLE something (id INTEGER);")
            .expect("made");
        drop(db);
        assert!(matches!(look(&stranger), Err(Error::NotTheFormerOne)));
    }
}
