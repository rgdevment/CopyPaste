use cp_core::kind::Kind;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

pub const NAME_ROOM: usize = 64;

pub const BYTES_ROOM: usize = 200;

pub const KEPT_FOR: Duration = crate::opening::SEEN_KEPT_FOR;

const FORBIDDEN: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

const RESERVED: [&str; 22] = [
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

pub fn can_drag(kind: Option<Kind>, paths: &[String]) -> bool {
    match kind {
        Some(Kind::Image) => true,
        Some(Kind::File | Kind::Folder | Kind::Video | Kind::Audio) => {
            crate::opening::first_of(paths).is_some_and(crate::opening::looks_like_a_path)
        }
        _ => false,
    }
}

fn capped(said: &str) -> String {
    use unicode_segmentation::UnicodeSegmentation;
    let mut out = String::new();
    for one in said.graphemes(true).take(NAME_ROOM) {
        if out.len() + one.len() > BYTES_ROOM {
            break;
        }
        out.push_str(one);
    }
    out
}

pub fn fit_for_a_file(said: &str) -> Option<String> {
    let kept: String = said
        .chars()
        .filter(|one| !one.is_control() && !FORBIDDEN.contains(one))
        .collect();
    let squeezed = kept.split_whitespace().collect::<Vec<_>>().join(" ");
    let short = capped(&squeezed);
    let name = short.trim_end_matches(['.', ' ']).trim().to_owned();
    if name.is_empty() || RESERVED.contains(&name.to_lowercase().as_str()) {
        return None;
    }
    Some(name)
}

pub fn named_as(label: Option<&str>, bytes: &[u8]) -> Option<String> {
    let ext = crate::opening::extension_of(bytes)?;
    match label.and_then(fit_for_a_file) {
        Some(name) => Some(format!("{name}.{ext}")),
        None => crate::opening::named_for(bytes),
    }
}

pub fn where_dragged_go() -> PathBuf {
    std::env::temp_dir().join("CopyPaste").join("dragged")
}

pub fn sweep_dragged(dir: &Path, now: SystemTime) -> usize {
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
            .is_some_and(|since| since > KEPT_FOR);
        if old && std::fs::remove_dir_all(one.path()).is_ok() {
            gone += 1;
        }
    }
    gone
}

fn written(dir: &Path, bytes: &[u8], name: &str) -> Option<PathBuf> {
    let at = dir.join(name);
    if std::fs::metadata(&at).is_ok_and(|one| one.len() == bytes.len() as u64) {
        return Some(at);
    }
    std::fs::create_dir_all(dir).ok()?;
    let _ = cp_store::restrict(dir, 0o700);
    std::fs::write(&at, bytes).ok()?;
    let _ = cp_store::restrict(&at, 0o600);
    Some(at)
}

pub fn on_disk(paths: &[String]) -> Vec<PathBuf> {
    paths
        .iter()
        .map(|one| one.trim())
        .filter(|one| crate::opening::looks_like_a_path(one))
        .map(PathBuf::from)
        .filter(|one| one.exists())
        .collect()
}

pub fn files_for(
    kind: Option<Kind>,
    paths: &[String],
    image: Option<&[u8]>,
    label: Option<&str>,
    now: SystemTime,
) -> Vec<PathBuf> {
    if !can_drag(kind, paths) {
        return Vec::new();
    }
    let there = on_disk(paths);
    if !there.is_empty() {
        return there;
    }
    let Some(bytes) = image else {
        return Vec::new();
    };
    let Some(name) = named_as(label, bytes) else {
        return Vec::new();
    };
    let dragged = where_dragged_go();
    sweep_dragged(&dragged, now);
    let own = dragged.join(format!("{:016x}", cp_core::hash::content_hash(bytes)));
    written(&own, bytes, &name).into_iter().collect()
}

#[cfg(test)]
#[path = "dragging_test.rs"]
mod tests;
