use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

const UP_TO: u64 = 1024 * 1024;

pub fn where_to() -> std::path::PathBuf {
    folder().join("cp-panel.log")
}

fn folder() -> std::path::PathBuf {
    crate::here::data_dir().map_or_else(std::env::temp_dir, |dir| dir.join("logs"))
}

pub fn note(what: &str) {
    let clock = clock_now();
    let path = where_to();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
        let _ = cp_store::restrict(dir, 0o700);
    }
    if std::fs::metadata(&path).map_or(0, |it| it.len()) > UP_TO {
        let _ = std::fs::write(&path, b"");
    }
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = cp_store::restrict(&path, 0o600);
        let _ = writeln!(file, "{clock} {what}");
    }
}

pub fn tell(what: &str) {
    use std::io::Write;
    let out = std::io::stdout();
    let mut out = out.lock();
    let _ = writeln!(out, "{what}");
    let _ = out.flush();
}

pub fn trouble(what: &str) {
    note(what);
    tell(&format!("trouble {what}"));
}

pub fn catch_panics() {
    let before = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        match info.location() {
            Some(at) => note(&format!("panic at {}:{}", at.file(), at.line())),
            None => note("a panic somewhere that could not be placed"),
        }
        before(info);
    }));
}

fn clock_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_secs())
        .unwrap_or(0);
    format!(
        "{:02}:{:02}:{:02}",
        (secs / 3_600) % 24,
        (secs / 60) % 60,
        secs % 60
    )
}

#[cfg(test)]
#[path = "note_test.rs"]
mod tests;
