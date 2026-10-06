use std::path::{Path, PathBuf};

use crate::{Error, Result, Store};

pub struct Opened {
    pub store: Store,
    pub set_aside: Option<PathBuf>,
}

pub fn open_or_set_aside(path: &Path, at_ms: i64) -> Result<Opened> {
    match Store::open(path) {
        Ok(store) => Ok(Opened {
            store,
            set_aside: None,
        }),
        Err(Error::Db(why)) if is_unreadable(&why) => {
            let kept = set_aside(path, at_ms)?;
            let store = Store::open(path)?;
            Ok(Opened {
                store,
                set_aside: Some(kept),
            })
        }
        Err(other) => Err(other),
    }
}

fn is_unreadable(why: &rusqlite::Error) -> bool {
    matches!(
        why.sqlite_error_code(),
        Some(rusqlite::ErrorCode::NotADatabase | rusqlite::ErrorCode::DatabaseCorrupt)
    )
}

fn set_aside(path: &Path, at_ms: i64) -> Result<PathBuf> {
    let stem = path
        .file_stem()
        .map_or_else(|| "history".into(), |name| name.to_string_lossy());
    let extension = path
        .extension()
        .map_or_else(|| "db".into(), |name| name.to_string_lossy());
    let renamed =
        |suffix: &str| path.with_file_name(format!("{stem}.corrupt-{at_ms}.{extension}{suffix}"));
    let kept = renamed("");
    let mut moves = Vec::new();
    for suffix in ["-wal", "-shm"] {
        let mut side = path.as_os_str().to_os_string();
        side.push(suffix);
        moves.push((PathBuf::from(side), renamed(suffix)));
    }
    if let Some(parent) = path.parent() {
        for folder in ["blobs", "thumbs"] {
            moves.push((
                parent.join(folder),
                parent.join(format!("{folder}.corrupt-{at_ms}")),
            ));
        }
    }
    moves.push((path.to_path_buf(), kept.clone()));
    let mut done: Vec<(PathBuf, PathBuf)> = Vec::new();
    for (from, to) in moves {
        if !from.exists() {
            continue;
        }
        if let Err(why) = std::fs::rename(&from, &to) {
            for (back, there) in done.iter().rev() {
                let _ = std::fs::rename(there, back);
            }
            return Err(why.into());
        }
        done.push((from, to));
    }
    Ok(kept)
}

#[cfg(test)]
#[path = "recover_test.rs"]
mod tests;
