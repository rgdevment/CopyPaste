use std::path::Path;

pub fn where_to() -> std::path::PathBuf {
    folder().join("cp-panel.log")
}

fn folder() -> std::path::PathBuf {
    crate::here::data_dir().map_or_else(std::env::temp_dir, |dir| dir.join("logs"))
}

fn guard(at: &Path, mode: u32) {
    let _ = cp_store::restrict(at, mode);
}

pub fn note(what: &str) {
    cp_core::note::note_to(&where_to(), what, &guard);
}

pub fn tell(what: &str) {
    cp_core::note::said_to(&mut std::io::stdout().lock(), what);
}

pub fn trouble(what: &str) {
    note(what);
    tell(&format!("trouble {what}"));
}

pub const ERRANDS: &str = "errands";
pub const COUNTER: &str = "counter";

pub fn spared(thread: Option<&str>) -> bool {
    thread.is_some_and(|name| [ERRANDS, COUNTER].contains(&name))
}

pub fn catch_panics() {
    cp_core::note::catch_panics(note);
    let before = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        before(info);
        if spared(std::thread::current().name()) {
            return;
        }
        note("a panic leaves the panel unsure of itself, so it ends to be started again");
        std::process::abort();
    }));
}

#[cfg(test)]
#[path = "note_test.rs"]
mod tests;
