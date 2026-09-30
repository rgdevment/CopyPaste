use crate::here;
use crate::note::note;
use cp_core::capture::Captured;
use cp_core::item::Item;
use cp_core::kind::Kind;
use cp_store::Store;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct Engine {
    watching: here::Watching,
    stop: Arc<AtomicBool>,
    errands: Option<std::thread::JoinHandle<()>>,
}

impl Engine {
    pub fn start(db: &Path, fresh: impl Fn(i64) + Send + 'static) -> Result<Self, cp_store::Error> {
        let store = Store::open(db)?;
        let watching = here::Watching::start(move || {
            if let Some(id) = kept(&store) {
                fresh(id);
            }
        });
        let stop = Arc::new(AtomicBool::new(false));
        let errands = errands(db, stop.clone());
        Ok(Self {
            watching,
            stop,
            errands,
        })
    }

    pub fn ours(&self) -> bool {
        self.watching.ours()
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.errands.take() {
            let _ = thread.join();
        }
    }
}

const SIDE: i32 = cp_core::thumbnail::MAX_SIDE as i32;
const NAP: std::time::Duration = std::time::Duration::from_millis(400);
const LATER: i64 = 60_000;
const SWEEPS_EVERY: std::time::Duration = std::time::Duration::from_secs(3_600);

fn errands(db: &Path, stop: Arc<AtomicBool>) -> Option<std::thread::JoinHandle<()>> {
    let store = match Store::open(db) {
        Ok(store) => store,
        Err(why) => {
            note(&format!("nobody enriches what was copied: {why}"));
            return None;
        }
    };
    let Some(thumbs) = here::thumbs_dir() else {
        note("there is nowhere to leave the thumbnails");
        return None;
    };
    Some(std::thread::spawn(move || {
        let mut swept = std::time::Instant::now() - SWEEPS_EVERY;
        while !stop.load(Ordering::Relaxed) {
            if swept.elapsed() >= SWEEPS_EVERY {
                sweep(&store);
                swept = std::time::Instant::now();
            }
            if !errand(&store, &thumbs) {
                std::thread::sleep(NAP);
            }
        }
    }))
}

fn sweep(store: &Store) {
    let Some(dir) = here::data_dir() else {
        return;
    };
    let path = cp_config::at(&dir);
    if !path.exists() {
        return;
    }
    let kept = match cp_config::read(&path) {
        Ok(kept) => kept,
        Err(why) => {
            note(&format!("what to keep could not be read: {why}"));
            return;
        }
    };
    let policy = policy_of(&kept);
    if policy == cp_store::Policy::default() {
        return;
    }
    match store.sweep(&policy, crate::app::now_ms()) {
        Ok(swept) => {
            if swept.expired + swept.over_bytes + swept.orphans > 0 {
                note(&format!(
                    "{} went by age, {} by room and {} were left loose",
                    swept.expired, swept.over_bytes, swept.orphans
                ));
            }
        }
        Err(why) => note(&format!("room could not be made: {why}")),
    }
}

fn policy_of(kept: &cp_config::Config) -> cp_store::Policy {
    cp_store::Policy::keeping(kept.keeps_days, kept.images_quota_mb)
}

fn errand(store: &Store, thumbs: &Path) -> bool {
    let at = crate::app::now_ms();
    if let Some(id) = first_waiting(store, "thumb", at) {
        thumbed(store, id, at, thumbs);
        return true;
    }
    if let Some(id) = first_waiting(store, "ocr", at) {
        read_out(store, id, at);
        return true;
    }
    false
}

fn first_waiting(store: &Store, job: &str, at: i64) -> Option<i64> {
    match store.take_pending(job, at, 1) {
        Ok(waiting) => waiting.into_iter().next(),
        Err(why) => {
            note(&format!("the {job} queue could not be looked at: {why}"));
            None
        }
    }
}

fn thumbed(store: &Store, id: i64, at: i64, thumbs: &Path) {
    let Some(png) = store.item(id).ok().flatten().as_ref().and_then(thumb_of) else {
        give_up(store, id, "thumb", "the thumbnail could not be drawn", at);
        return;
    };
    let Some(landed) = written(thumbs, id, &png) else {
        give_up(store, id, "thumb", "the thumbnail could not be stored", at);
        return;
    };
    if let Err(why) = store.set_thumb(id, Some(&landed), at) {
        note(&format!("{id} has a thumbnail nobody wrote down: {why}"));
    }
    done(store, id, "thumb");
}

fn read_out(store: &Store, id: i64, at: i64) {
    if !here::ocr_available() {
        done(store, id, "ocr");
        return;
    }
    let item = match store.item(id) {
        Ok(Some(item)) => item,
        Ok(None) => {
            done(store, id, "ocr");
            return;
        }
        Err(why) => {
            give_up(
                store,
                id,
                "ocr",
                &format!("it could not be read in order to read it: {why}"),
                at,
            );
            return;
        }
    };
    let found = here::content_of(&item, None).image.and_then(here::text_in);
    if let Some(text) = found
        && let Err(why) = store.set_ocr_text(id, &text, at)
    {
        note(&format!("{id} was read and nobody wrote it down: {why}"));
    }
    done(store, id, "ocr");
}

fn thumb_of(item: &Item) -> Option<Vec<u8>> {
    let content = here::content_of(item, None);
    if let Some(image) = content.image {
        return cp_core::thumbnail::of_image(image, cp_core::thumbnail::MAX_SIDE);
    }
    let first = content.paths.first()?;
    here::thumb_of_file(Path::new(first), SIDE)
}

fn written(dir: &Path, id: i64, png: &[u8]) -> Option<String> {
    std::fs::create_dir_all(dir).ok()?;
    let _ = cp_store::restrict(dir, 0o700);
    let landed = dir.join(format!("{id}.png"));
    std::fs::write(&landed, png).ok()?;
    let _ = cp_store::restrict(&landed, 0o600);
    Some(landed.to_string_lossy().into_owned())
}

fn done(store: &Store, id: i64, job: &str) {
    if let Err(why) = store.work_done(id, job) {
        note(&format!("{id} is still in the {job} queue: {why}"));
    }
}

fn give_up(store: &Store, id: i64, job: &str, why: &str, at: i64) {
    match store.work_failed(id, job, why, at + LATER) {
        Ok(true) => {}
        Ok(false) => note(&format!("{id} goes without {job} for good: {why}")),
        Err(trouble) => note(&format!(
            "the {job} failure of {id} went unwritten: {trouble}"
        )),
    }
}

fn kept(store: &Store) -> Option<i64> {
    let item = match here::capture_insisting() {
        Captured::Kept(item) => item,
        Captured::Refused(_) => {
            note("a copy was dropped because the app it came from asked for that");
            return None;
        }
        Captured::TooSlow => {
            note("a read of the clipboard never came back, however much we insisted");
            return None;
        }
        Captured::Busy => {
            note("the clipboard was held by another program every time we asked");
            return None;
        }
        Captured::Nothing | Captured::Superseded => return None,
    };
    let from = here::in_front();
    keep(store, &item, crate::app::now_ms(), from.as_deref())
}

fn keep(store: &Store, item: &Item, at: i64, from: Option<&str>) -> Option<i64> {
    match store.find_by_hash(item) {
        Ok(Some(id)) => {
            if let Err(why) = store.reactivate(id, at) {
                note(&format!("the repeat of {id} did not rise: {why}"));
            }
            return Some(id);
        }
        Ok(None) => {}
        Err(why) => note(&format!(
            "whether it was already here could not be looked up: {why}"
        )),
    }
    let id = match store.insert_item(&name_for(at, item), item, &preview_of(item), at) {
        Ok(id) => id,
        Err(why) => {
            note(&format!("what was copied could not be stored: {why}"));
            return None;
        }
    };
    if let Some(from) = from
        && let Err(why) = store.set_source(id, from, at)
    {
        note(&format!(
            "{id} was left not knowing where it came from: {why}"
        ));
    }
    for job in jobs_for(item) {
        if let Err(why) = store.enqueue(id, job) {
            note(&format!("{id} was left without {job} queued: {why}"));
        }
    }
    Some(id)
}

fn preview_of(item: &Item) -> String {
    let content = here::content_of(item, None);
    if let Some(text) = content.text {
        return text.into_owned();
    }
    if !content.paths.is_empty() {
        return content.paths.join("\n");
    }
    String::new()
}

fn name_for(at: i64, _item: &Item) -> String {
    static TURN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let turn = TURN.fetch_add(1, Ordering::Relaxed);
    let since = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |it| it.subsec_nanos());
    format!("{at:x}-{since:08x}{turn:08x}")
}

fn jobs_for(item: &Item) -> &'static [&'static str] {
    match item.kind {
        Some(Kind::Image) => &["thumb", "ocr"],
        Some(Kind::File) | Some(Kind::Folder) if here::THUMBNAILS_FILES => &["thumb"],
        _ => &[],
    }
}

#[cfg(test)]
#[path = "engine_test.rs"]
mod tests;
