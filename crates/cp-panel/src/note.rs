use cp_core::trouble::Trouble;
use std::path::Path;

pub fn where_to() -> std::path::PathBuf {
    folder().join("cp-panel.log")
}

fn real_folder() -> std::path::PathBuf {
    crate::here::data_dir().map_or_else(std::env::temp_dir, |dir| dir.join("logs"))
}

#[cfg(not(test))]
fn folder() -> std::path::PathBuf {
    real_folder()
}

#[cfg(test)]
fn folder() -> std::path::PathBuf {
    std::env::temp_dir().join("cp-panel-tests")
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

pub fn trouble(trouble: Trouble, why: &str) {
    note(why);
    tell(&format!("trouble {}", trouble.key()));
}

pub fn well(trouble: Trouble) {
    tell(&format!("well {}", trouble.key()));
}

pub const ERRANDS: &str = "errands";
pub const COUNTER: &str = "counter";

pub fn spared(thread: Option<&str>) -> bool {
    thread.is_some_and(|name| [ERRANDS, COUNTER].contains(&name))
}

pub fn catch_panics() {
    cp_core::note::catch_panics(note);
}

pub fn end_on_panic(end: fn(&str)) {
    let before = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        before(info);
        if spared(std::thread::current().name()) {
            return;
        }
        end("a panic leaves the panel unsure of itself, so it ends to be started again");
    }));
}

#[cfg(test)]
#[path = "note_test.rs"]
mod tests;
