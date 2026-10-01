use std::path::{Path, PathBuf};

pub fn where_to() -> PathBuf {
    crate::settings::folder()
        .map_or_else(std::env::temp_dir, |dir| dir.join("logs"))
        .join("cp-gui.log")
}

fn guard(at: &Path, mode: u32) {
    let _ = cp_store::restrict(at, mode);
}

pub fn note(what: &str) {
    cp_core::note::note_to(&where_to(), what, &guard);
}

pub fn catch_panics() {
    cp_core::note::catch_panics(note);
}

#[cfg(test)]
#[path = "note_test.rs"]
mod tests;
