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
    std::fs::rename(path, &kept)?;
    for suffix in ["-wal", "-shm"] {
        let mut side = path.as_os_str().to_os_string();
        side.push(suffix);
        let side = PathBuf::from(side);
        if side.exists() {
            std::fs::rename(&side, renamed(suffix))?;
        }
    }
    if let Some(parent) = path.parent() {
        let blobs = parent.join("blobs");
        if blobs.exists() {
            std::fs::rename(&blobs, parent.join(format!("blobs.corrupt-{at_ms}")))?;
        }
    }
    Ok(kept)
}

#[cfg(test)]
#[path = "recover_test.rs"]
mod tests;
