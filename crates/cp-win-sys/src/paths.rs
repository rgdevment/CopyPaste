use std::path::PathBuf;

pub fn data_dir() -> Option<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA")?;
    Some(PathBuf::from(local).join("CopyPaste"))
}

pub fn database() -> Option<PathBuf> {
    Some(data_dir()?.join("history.db"))
}

const STORE_FAMILY: &str = "rgdevment.CopyPaste-ClipboardManager_kdjgfdc2rb3gc";

pub fn legacy_dir() -> Option<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA")?;
    Some(legacy_in(std::path::Path::new(&local)))
}

pub fn legacy_in(local: &std::path::Path) -> PathBuf {
    let plain = local.join("CopyPaste");
    if plain.join("clipboard.db").exists() {
        return plain;
    }
    let packaged = local
        .join("Packages")
        .join(STORE_FAMILY)
        .join("LocalCache")
        .join("Local")
        .join("CopyPaste");
    if packaged.join("clipboard.db").exists() {
        return packaged;
    }
    plain
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
