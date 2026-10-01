use std::io::Write;
use std::path::Path;

pub const UP_TO: u64 = 1024 * 1024;

pub fn rotate(path: &Path, up_to: u64) -> bool {
    if std::fs::metadata(path).map_or(0, |it| it.len()) <= up_to {
        return false;
    }
    std::fs::rename(path, path.with_extension("log.1")).is_ok()
}

pub fn line_of(what: &str) -> String {
    format!("{} {} {what}", crate::stamp::now(), std::process::id())
}

pub fn note_to(path: &Path, what: &str, guard: &dyn Fn(&Path, u32)) {
    let line = line_of(what);
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
        guard(dir, 0o700);
    }
    rotate(path, UP_TO);
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        guard(path, 0o600);
        let _ = writeln!(file, "{line}");
    }
}

pub fn said_to(out: &mut impl Write, what: &str) {
    let _ = writeln!(out, "{what}");
    let _ = out.flush();
}

pub fn catch_panics(note: impl Fn(&str) + Send + Sync + 'static) {
    let before = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let at = info.location().map(|at| (at.file(), at.line()));
        note(&panic_line(at, &said_in(info.payload())));
        before(info);
    }));
}

pub fn panic_line(at: Option<(&str, u32)>, said: &str) -> String {
    match at {
        Some((file, line)) => format!("panic at {file}:{line}: {said}"),
        None => format!("a panic that could not be placed: {said}"),
    }
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
