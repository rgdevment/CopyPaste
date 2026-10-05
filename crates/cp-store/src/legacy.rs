use crate::{Error, More, Result, Store};
use cp_core::item::{Format, Item, Payload, Placement, SYNTHETIC_IMAGE, SYNTHETIC_TEXT, placement};
use cp_core::kind::Kind;
use rusqlite::{Connection, OpenFlags};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Former {
    pub items: i64,
    pub pictures: i64,
    pub pictures_gone: i64,
    pub pinned: i64,
    pub labelled: i64,
    pub with_styles: i64,
    pub beyond_keep: i64,
}

pub const THEIR_MARK: &str = "2x-";
pub const PAST_THEIR_MARK: &str = "2x.";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Came {
    pub count: i64,
    pub still: i64,
    pub when: Option<i64>,
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

#[cfg(target_os = "windows")]
pub const RICH_HTML: &str = "HTML Format";
#[cfg(not(target_os = "windows"))]
pub const RICH_HTML: &str = "public.html";

#[cfg(target_os = "windows")]
pub const RICH_RTF: &str = "Rich Text Format";
#[cfg(not(target_os = "windows"))]
pub const RICH_RTF: &str = "public.rtf";

pub const RICH_KEYS: [&str; 2] = ["html", "rtf"];

const A_SECOND: i64 = 1_000;

pub const fn in_millis(seconds: i64) -> i64 {
    seconds.saturating_mul(A_SECOND)
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
    pastes: i64,
    broken: Option<i64>,
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

pub fn look(from: &Path, at: i64, keep_for: Option<i64>) -> Result<Former> {
    let db = opened(from)?;
    let root = root_of(from);
    let mut stmt = db.prepare(
        "SELECT type, content, is_pinned, label, metadata, created_at, modified_at
         FROM clipboard_items",
    )?;
    let mut rows = stmt.query([])?;
    let mut former = Former::default();
    while let Some(row) = rows.next()? {
        let kind = kind_of(row.get::<_, i64>(0).unwrap_or(0)).unwrap_or(Kind::Text);
        let content: String = row.get(1).unwrap_or_default();
        let pinned = row.get::<_, Option<i64>>(2).ok().flatten().unwrap_or(0) != 0;
        former.items += 1;
        if pinned {
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
        if styled_of(row.get::<_, Option<String>>(4).ok().flatten().as_deref()) {
            former.with_styles += 1;
        }
        let created = row.get::<_, Option<i64>>(5).ok().flatten().unwrap_or(0);
        let modified = row.get::<_, Option<i64>>(6).ok().flatten().unwrap_or(0);
        if let Some(age) = keep_for
            && !pinned
            && lands_at(created, modified, at) < at - age
        {
            former.beyond_keep += 1;
        }
        if kind == Kind::Image {
            former.pictures += 1;
            if picture_at(root.as_deref(), &content).is_none() {
                former.pictures_gone += 1;
            }
        }
    }
    Ok(former)
}

fn lands_at(created: i64, modified: i64, at: i64) -> i64 {
    let when = if created > 0 { in_millis(created) } else { at };
    if modified > created {
        in_millis(modified).max(when)
    } else {
        when
    }
}

fn root_of(from: &Path) -> Option<PathBuf> {
    std::fs::canonicalize(from.parent()?).ok()
}

fn picture_at(root: Option<&Path>, said: &str) -> Option<PathBuf> {
    if said.is_empty() {
        return None;
    }
    let root = root?;
    let real = std::fs::canonicalize(said)
        .or_else(|_| {
            let name = said
                .rsplit(['\\', '/'])
                .next()
                .filter(|name| !name.is_empty() && *name != "." && *name != "..")
                .ok_or(std::io::ErrorKind::NotFound)?;
            std::fs::canonicalize(root.join("images").join(name))
        })
        .ok()?;
    if !real.starts_with(root) {
        return None;
    }
    let weighed = std::fs::symlink_metadata(&real).ok()?;
    if !weighed.is_file() || weighed.len() == 0 {
        return None;
    }
    let size = usize::try_from(weighed.len()).ok()?;
    (placement(size) != Placement::Refused).then_some(real)
}

pub fn bring(from: &Path, into: &Store, at: i64) -> Result<Brought> {
    bring_telling(from, into, at, &|_, _| {})
}

const AT_A_TIME: usize = 500;

pub fn bring_telling(
    from: &Path,
    into: &Store,
    at: i64,
    telling: &dyn Fn(i64, i64),
) -> Result<Brought> {
    let db = opened(from)?;
    let root = root_of(from);
    let total: i64 = db
        .query_row("SELECT count(*) FROM clipboard_items", [], |row| row.get(0))
        .unwrap_or(0);
    let mut stmt = db.prepare(
        "SELECT id, content, type, created_at, modified_at, app_source, is_pinned, label,
                card_color, metadata, paste_count, broken_since
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
            pastes: row.get::<_, Option<i64>>(10)?.unwrap_or(0),
            broken: row.get::<_, Option<i64>>(11)?,
        })
    })?;

    let mut brought = Brought::default();
    let mut rows = rows;
    let mut done = 0i64;
    loop {
        let mut carried = 0;
        into.all_or_nothing(|| {
            for _ in 0..AT_A_TIME {
                let Some(row) = rows.next() else {
                    break;
                };
                carried += 1;
                let Ok(row) = row else {
                    brought.refused += 1;
                    continue;
                };
                match carry(into, &row, at, root.as_deref()) {
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
            Ok(())
        })?;
        if carried == 0 {
            break;
        }
        done += carried as i64;
        telling(done, total);
    }
    Ok(brought)
}

enum Landed {
    Added { without_picture: bool },
    Already,
}

fn carry(into: &Store, row: &Row, at: i64, root: Option<&Path>) -> Result<Landed> {
    let (item, without_picture) = made_of(row, root);
    if into.find_by_hash(&item)?.is_some() {
        return Ok(Landed::Already);
    }
    let name = named(row, at);
    if into.knows_name(&name)? {
        return Ok(Landed::Already);
    }
    let when = if row.created_at > 0 {
        in_millis(row.created_at)
    } else {
        at
    };
    let meta = meta_in(row.meta.as_deref());
    let more = More {
        came_at: Some(at),
        modified_at: (row.modified_at > row.created_at).then(|| in_millis(row.modified_at)),
        touched_at: Some(at),
        used_at: None,
        app: row.app.as_deref(),
        label: row.label.as_deref(),
        color: row.colour,
        pinned: row.pinned,
        pastes: row.pastes,
        broken: row.broken.map(in_millis),
        meta: &meta,
        jobs: jobs_for(row.kind, !without_picture),
    };
    into.insert_full(&name, &item, &preview_of(row, without_picture), when, &more)?;
    Ok(Landed::Added { without_picture })
}

fn made_of(row: &Row, root: Option<&Path>) -> (Item, bool) {
    let (mut item, without_picture) = body_of(row, root);
    for (id, bytes) in rich_of(row.meta.as_deref()) {
        item.formats.push(Format {
            id: id.into(),
            payload: Payload::stored(bytes),
        });
    }
    (item, without_picture)
}

fn body_of(row: &Row, root: Option<&Path>) -> (Item, bool) {
    if row.kind == Kind::Image {
        let read = picture_at(root, &row.content).and_then(|at| std::fs::read(at).ok());
        return match read {
            Some(bytes) if !bytes.is_empty() => (
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

pub fn jobs_for(kind: Kind, with_its_picture: bool) -> &'static [&'static str] {
    match kind {
        Kind::Image if with_its_picture => &["thumb", "ocr"],
        Kind::File | Kind::Folder if cp_core::thumbnail::THUMBNAILS_FILES => &["thumb"],
        _ => &[],
    }
}

fn preview_of(row: &Row, without_picture: bool) -> String {
    if row.kind == Kind::Image && !without_picture {
        return String::new();
    }
    row.content.clone()
}

fn named(row: &Row, at: i64) -> String {
    if row.uuid.is_empty() {
        format!("{at:x}-{:016x}", row.modified_at)
    } else {
        format!("{THEIR_MARK}{}", row.uuid)
    }
}

pub fn meta_in(said: Option<&str>) -> Vec<(String, String)> {
    let Some(read) = read_object(said) else {
        return Vec::new();
    };
    read.into_iter()
        .filter(|(key, _)| !RICH_KEYS.contains(&key.as_str()))
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

fn read_object(said: Option<&str>) -> Option<serde_json::Map<String, serde_json::Value>> {
    match serde_json::from_str(said?) {
        Ok(serde_json::Value::Object(read)) => Some(read),
        _ => None,
    }
}

pub fn rich_of(said: Option<&str>) -> Vec<(&'static str, Vec<u8>)> {
    let Some(read) = read_object(said) else {
        return Vec::new();
    };
    let decoded = |key: &str| {
        read.get(key)
            .and_then(serde_json::Value::as_str)
            .and_then(un_base64)
    };
    let mut carried = Vec::new();
    if let Some(bytes) = decoded("html")
        .filter(|bytes| html_ok(bytes))
        .and_then(html_for)
    {
        carried.push((RICH_HTML, bytes));
    }
    if let Some(bytes) = decoded("rtf").filter(|bytes| rtf_ok(bytes)) {
        carried.push((RICH_RTF, bytes));
    }
    carried
}

const A_HEAD: usize = THE_WINDOWS_HEADER.len();

fn styled_of(said: Option<&str>) -> bool {
    let Some(read) = read_object(said) else {
        return false;
    };
    let head = |key: &str| {
        read.get(key)
            .and_then(serde_json::Value::as_str)
            .and_then(|one| un_base64_upto(one, A_HEAD))
            .unwrap_or_default()
    };
    html_ok(&head("html")) || rtf_ok(&head("rtf"))
}

fn html_ok(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    !cfg!(target_os = "windows") || bytes.starts_with(THE_WINDOWS_HEADER)
}

fn rtf_ok(bytes: &[u8]) -> bool {
    bytes.starts_with(br"{\rtf")
}

const THE_WINDOWS_HEADER: &[u8] = b"Version:";

fn html_for(bytes: Vec<u8>) -> Option<Vec<u8>> {
    if cfg!(target_os = "windows") || !bytes.starts_with(THE_WINDOWS_HEADER) {
        return Some(bytes);
    }
    unwrapped(&bytes)
}

fn unwrapped(bytes: &[u8]) -> Option<Vec<u8>> {
    let at = said_at(bytes, b"StartHTML:")
        .filter(|at| bytes.get(*at) == Some(&b'<'))
        .or_else(|| bytes.iter().position(|one| *one == b'<'))?;
    Some(bytes[at..].to_vec())
}

fn said_at(bytes: &[u8], key: &[u8]) -> Option<usize> {
    let at = bytes
        .windows(key.len())
        .position(|one| one == key)?
        .checked_add(key.len())?;
    let digits: String = bytes[at..]
        .iter()
        .take_while(|one| one.is_ascii_digit())
        .map(|one| *one as char)
        .collect();
    digits.parse().ok()
}

fn un_base64(said: &str) -> Option<Vec<u8>> {
    un_base64_upto(said, usize::MAX)
}

fn un_base64_upto(said: &str, most: usize) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity((said.len() / 4 * 3).min(most));
    let mut held: u32 = 0;
    let mut bits: u32 = 0;
    for byte in said.bytes() {
        let six = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            b'\r' | b'\n' => continue,
            _ => return None,
        };
        held = (held << 6) | u32::from(six);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(u8::try_from((held >> bits) & 0xFF).ok()?);
            if out.len() >= most {
                break;
            }
        }
    }
    (!out.is_empty()).then_some(out)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Swept {
    pub files: i64,
    pub bytes: u64,
}

pub const OURS_ALONE: [&str; 5] = [
    "images",
    "config",
    ".initialized",
    "crash.log",
    "last_cleanup.txt",
];

pub const THEIR_LOGS: &str = "copypaste_";

pub const THEIR_SPARE: &str = ".pre-restore-";

pub fn drop_former(dir: &Path) -> Result<Swept> {
    let mut swept = Swept::default();
    let db = dir.join("clipboard.db");
    for one in [db.clone(), sidecar(&db, "-wal"), sidecar(&db, "-shm")] {
        swept = taken(&one, swept)?;
    }
    for one in OURS_ALONE {
        swept = gone(&dir.join(one), swept)?;
    }
    for one in named_like(dir, THEIR_SPARE) {
        swept = gone(&one, swept)?;
    }
    swept = their_logs(&dir.join("logs"), swept)?;
    Ok(swept)
}

fn named_like(dir: &Path, head: &str) -> Vec<PathBuf> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    read.filter_map(std::result::Result::ok)
        .map(|one| one.path())
        .filter(|at| {
            at.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(head))
        })
        .collect()
}

fn gone(at: &Path, mut swept: Swept) -> Result<Swept> {
    let Ok(kind) = std::fs::symlink_metadata(at) else {
        return Ok(swept);
    };
    if kind.is_dir() {
        swept = emptied(at, swept)?;
        let _ = std::fs::remove_dir_all(at);
        return Ok(swept);
    }
    taken(at, swept)
}

fn their_logs(dir: &Path, mut swept: Swept) -> Result<Swept> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Ok(swept);
    };
    for one in read.filter_map(std::result::Result::ok) {
        let at = one.path();
        let theirs = at
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(THEIR_LOGS));
        if theirs {
            swept = taken(&at, swept)?;
        }
    }
    let _ = std::fs::remove_dir(dir);
    Ok(swept)
}

fn emptied(dir: &Path, mut swept: Swept) -> Result<Swept> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Ok(swept);
    };
    for one in read.filter_map(std::result::Result::ok) {
        swept = gone(&one.path(), swept)?;
    }
    Ok(swept)
}

fn taken(at: &Path, mut swept: Swept) -> Result<Swept> {
    let Ok(weighed) = std::fs::symlink_metadata(at) else {
        return Ok(swept);
    };
    if weighed.file_type().is_symlink() {
        std::fs::remove_file(at).map_err(Error::Io)?;
        swept.files += 1;
        return Ok(swept);
    }
    if !weighed.is_file() {
        return Ok(swept);
    }
    swept.files += 1;
    swept.bytes += weighed.len();
    crate::blobs::remove_at(at)?;
    Ok(swept)
}

fn sidecar(path: &Path, tail: &str) -> PathBuf {
    let mut said = path.as_os_str().to_os_string();
    said.push(tail);
    PathBuf::from(said)
}

#[cfg(test)]
#[path = "legacy_test.rs"]
mod tests;
