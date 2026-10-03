use crate::here;
use crate::note::note;
use cp_core::kind::Kind;
use cp_store::Store;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Doing {
    Nothing,
    Open,
}

pub const PATIENCE: std::time::Duration = std::time::Duration::from_millis(400);

const _: () = assert!(PATIENCE.as_millis() <= 500);

pub fn doing_for(kind: Option<Kind>, paths: &[String]) -> Doing {
    let Some(said) = first_of(paths) else {
        return Doing::Nothing;
    };
    if !looks_like_a_path(said) {
        return Doing::Nothing;
    }
    match kind {
        Some(Kind::File | Kind::Folder | Kind::Image | Kind::Video | Kind::Audio) => Doing::Open,
        _ => Doing::Nothing,
    }
}

pub fn can_open_link(kind: Option<Kind>, preview: &str) -> bool {
    kind == Some(Kind::Link)
        && preview
            .lines()
            .next()
            .is_some_and(|one| a_link_worth_opening(one).is_some())
}

pub fn can_open(kind: Option<Kind>, paths: &[String]) -> bool {
    matches!(kind, Some(Kind::Image)) || doing_for(kind, paths) == Doing::Open
}

pub fn a_link_worth_opening(said: &str) -> Option<&str> {
    let said = said.trim();
    if said.chars().any(char::is_whitespace) {
        return None;
    }
    let lower = said.to_ascii_lowercase();
    (lower.starts_with("http://") || lower.starts_with("https://"))
        .then_some(said)
        .filter(|one| one.len() > 8)
}

pub fn extension_of(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        return Some("png");
    }
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some("jpg");
    }
    if bytes.starts_with(b"GIF8") {
        return Some("gif");
    }
    if bytes.starts_with(b"BM") {
        return Some("bmp");
    }
    if bytes.starts_with(b"RIFF") && bytes.len() > 12 && &bytes[8..12] == b"WEBP" {
        return Some("webp");
    }
    None
}

pub const SEEN_KEPT_FOR: std::time::Duration = std::time::Duration::from_secs(60 * 60);

pub fn named_for(bytes: &[u8]) -> Option<String> {
    Some(format!(
        "{:016x}.{}",
        cp_core::hash::content_hash(bytes),
        extension_of(bytes)?
    ))
}

pub fn spilled(dir: &std::path::Path, bytes: &[u8]) -> Option<std::path::PathBuf> {
    let at = dir.join(named_for(bytes)?);
    if std::fs::metadata(&at).is_ok_and(|one| one.len() == bytes.len() as u64) {
        return Some(at);
    }
    std::fs::create_dir_all(dir).ok()?;
    let _ = cp_store::restrict(dir, 0o700);
    std::fs::write(&at, bytes).ok()?;
    let _ = cp_store::restrict(&at, 0o600);
    Some(at)
}

pub fn sweep_seen(dir: &std::path::Path, now: std::time::SystemTime) -> usize {
    let Ok(read) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut gone = 0;
    for one in read.flatten() {
        let old = one
            .metadata()
            .and_then(|meta| meta.modified())
            .ok()
            .and_then(|at| now.duration_since(at).ok())
            .is_some_and(|since| since > SEEN_KEPT_FOR);
        if old && std::fs::remove_file(one.path()).is_ok() {
            gone += 1;
        }
    }
    gone
}

pub fn where_previews_go() -> std::path::PathBuf {
    std::env::temp_dir().join("CopyPaste").join("seen")
}

pub fn looks_like_a_path(said: &str) -> bool {
    if said.starts_with(['\\', '/']) {
        return true;
    }
    let drive = said.as_bytes();
    if drive.len() >= 3
        && drive[0].is_ascii_alphabetic()
        && drive[1] == b':'
        && (drive[2] == b'\\' || drive[2] == b'/')
    {
        return true;
    }
    said.contains(['\\', '/'])
}

pub fn first_of(paths: &[String]) -> Option<&str> {
    paths
        .iter()
        .map(|one| one.trim())
        .find(|one| !one.is_empty())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reached {
    Opened,
    Working,
    Refused,
    NoLink,
    Missing,
}

pub fn reach_for(store: &Store, id: i64) -> Reached {
    let Ok(Some(item)) = store.item(id) else {
        note(&format!("{id} could not be looked up to open it"));
        return Reached::Refused;
    };
    let content = here::content_of(&item, None);
    if let Some(said) = content.text.as_deref()
        && let Some(url) = crate::opening::a_link_worth_opening(said.lines().next().unwrap_or(""))
    {
        let url = url.to_owned();
        let asking = cp_core::reading::begin(move || here::open_link(&url));
        return match answered(asking.waited(crate::opening::PATIENCE)) {
            Reached::Refused => Reached::NoLink,
            other => other,
        };
    }
    let said_path = crate::opening::first_of(&content.paths)
        .map(str::to_owned)
        .or_else(|| {
            content
                .text
                .as_deref()
                .map(|said| said.lines().next().unwrap_or("").trim().to_owned())
                .filter(|said| crate::opening::looks_like_a_path(said))
        });
    let on_disk = said_path.is_some();
    let path = match said_path {
        Some(said) => std::path::PathBuf::from(said),
        None => {
            let Some(bytes) = content.image else {
                note(&format!("{id} has nothing a viewer could be given"));
                return Reached::Refused;
            };
            let seen = crate::opening::where_previews_go();
            crate::opening::sweep_seen(&seen, std::time::SystemTime::now());
            let Some(spilled) = crate::opening::spilled(&seen, bytes) else {
                note(&format!("{id} could not be written out to be seen"));
                return Reached::Refused;
            };
            spilled
        }
    };
    let asking = cp_core::reading::begin(move || {
        let opened = here::open_path(&path);
        landing_of(opened, on_disk, path.exists())
    });
    match asking.waited(crate::opening::PATIENCE) {
        cp_core::reading::Waited::Answered(Landing::Opened) => Reached::Opened,
        cp_core::reading::Waited::Answered(Landing::Gone) => Reached::Missing,
        cp_core::reading::Waited::Answered(Landing::Refused) => Reached::Refused,
        cp_core::reading::Waited::StillRunning => Reached::Working,
        cp_core::reading::Waited::Gone => Reached::Refused,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Landing {
    Opened,
    Refused,
    Gone,
}

fn landing_of(opened: bool, on_disk: bool, still_there: bool) -> Landing {
    if opened {
        return Landing::Opened;
    }
    if on_disk && !still_there {
        return Landing::Gone;
    }
    Landing::Refused
}

fn answered(said: cp_core::reading::Waited<bool>) -> Reached {
    match said {
        cp_core::reading::Waited::Answered(true) => Reached::Opened,
        cp_core::reading::Waited::Answered(false) => Reached::Refused,
        cp_core::reading::Waited::StillRunning => Reached::Working,
        cp_core::reading::Waited::Gone => Reached::Refused,
    }
}

#[cfg(test)]
#[path = "opening_test.rs"]
mod tests;
