use std::io::Write;
use std::path::{Path, PathBuf};

const UP_TO: u64 = 1024 * 1024;

pub fn where_to() -> PathBuf {
    crate::settings::folder()
        .map_or_else(std::env::temp_dir, |dir| dir.join("logs"))
        .join("cp-gui.log")
}

fn rotate(path: &Path) {
    if std::fs::metadata(path).map_or(0, |it| it.len()) <= UP_TO {
        return;
    }
    let _ = std::fs::rename(path, path.with_extension("log.1"));
}

pub fn note(what: &str) {
    note_to(&where_to(), what);
}

pub fn note_to(path: &Path, what: &str) {
    let stamp = cp_core::stamp::now();
    let whose = std::process::id();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
        let _ = cp_store::restrict(dir, 0o700);
    }
    rotate(path);
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    else {
        return;
    };
    let _ = cp_store::restrict(path, 0o600);
    let _ = writeln!(file, "{stamp} {whose} {what}");
}

pub fn catch_panics() {
    let before = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let said = said_in(info.payload());
        match info.location() {
            Some(at) => note(&format!("panic at {}:{}: {said}", at.file(), at.line())),
            None => note(&format!("a panic that could not be placed: {said}")),
        }
        before(info);
    }));
}

pub fn said_in(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(said) = payload.downcast_ref::<&str>() {
        return (*said).to_owned();
    }
    payload
        .downcast_ref::<String>()
        .cloned()
        .unwrap_or_else(|| "with nothing said".to_owned())
}

#[cfg(test)]
#[path = "note_test.rs"]
mod tests;
