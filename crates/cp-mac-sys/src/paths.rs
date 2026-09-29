use std::path::PathBuf;

pub fn data_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("CopyPaste"),
    )
}

pub fn database() -> Option<PathBuf> {
    Some(data_dir()?.join("history.db"))
}

pub fn legacy_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("com.rgdevment.copypaste")
            .join("CopyPaste"),
    )
}

pub fn legacy_database() -> Option<PathBuf> {
    Some(legacy_dir()?.join("clipboard.db"))
}

pub fn blobs_dir() -> Option<PathBuf> {
    Some(data_dir()?.join("blobs"))
}

pub fn thumbs_dir() -> Option<PathBuf> {
    Some(data_dir()?.join("thumbs"))
}

pub fn pastes_dir() -> PathBuf {
    std::env::temp_dir().join("CopyPaste")
}

#[cfg(test)]
#[path = "paths_test.rs"]
mod tests;
