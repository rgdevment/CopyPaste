use std::path::PathBuf;

pub fn data_dir() -> Option<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA")?;
    Some(PathBuf::from(local).join("CopyPaste"))
}

pub fn database() -> Option<PathBuf> {
    Some(data_dir()?.join("history.db"))
}

pub fn legacy_dir() -> Option<PathBuf> {
    data_dir()
}

pub fn legacy_database() -> Option<PathBuf> {
    Some(legacy_dir()?.join("clipboard.db"))
}

pub fn thumbs_dir() -> Option<PathBuf> {
    Some(data_dir()?.join("thumbs"))
}

pub fn blobs_dir() -> Option<PathBuf> {
    Some(data_dir()?.join("blobs"))
}

pub fn pastes_dir() -> PathBuf {
    std::env::temp_dir().join("CopyPaste")
}

#[cfg(test)]
#[path = "paths_test.rs"]
mod tests;
