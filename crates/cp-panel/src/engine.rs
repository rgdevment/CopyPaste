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
    errands: std::sync::Mutex<Option<std::thread::JoinHandle<()>>>,
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
            errands: std::sync::Mutex::new(errands),
        })
    }

    pub fn close(&self) {
        if !self.watching.close() {
            note("the clipboard watcher would not stop and was left behind");
        }
        self.stop.store(true, Ordering::Relaxed);
        let Ok(mut held) = self.errands.lock() else {
            note("the errands thread could not be reached to stop it");
            return;
        };
        if let Some(thread) = held.take()
            && !cp_core::closing::join_soon(thread)
        {
            note("the errands thread would not stop and was left behind");
        }
    }

    pub fn writing(&self) -> bool {
        self.watching.writing()
    }

    pub fn ours(&self) -> bool {
        self.watching.ours()
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.close();
    }
}

const SIDE: i32 = cp_core::thumbnail::MAX_SIDE as i32;
const WAVE_WIDE: u32 = 384;
const WAVE_HIGH: u32 = 64;
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
        catch_up(&store);
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

const CATCH_UP: usize = 500;

fn catch_up(store: &Store) {
    for (kinds, key, job) in [
        (&["image"][..], crate::media::WIDTH, "thumb"),
        (&["video", "audio"][..], crate::media::DURATION, "media"),
    ] {
        match store.missing_meta(kinds, key, CATCH_UP) {
            Ok(waiting) => {
                for id in waiting {
                    if let Err(why) = store.enqueue(id, job) {
                        note(&format!("{id} was left without {job} queued: {why}"));
                        break;
                    }
                }
            }
            Err(why) => note(&format!("what is unmeasured could not be looked at: {why}")),
        }
    }
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
    if let Some(id) = first_waiting(store, "media", at) {
        measured(store, id, at);
        return true;
    }
    if let Some(id) = first_waiting(store, "folder", at) {
        walked(store, id, at);
        return true;
    }
    grouped_some(store)
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
    let item = store.item(id).ok().flatten();
    if let Some(sides) = item.as_ref().and_then(|one| {
        here::content_of(one, None)
            .image
            .and_then(cp_core::thumbnail::size_of)
    }) {
        measured_sides(store, id, sides.width, sides.height);
    }
    let Some(png) = item.as_ref().and_then(thumb_of) else {
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

fn measured(store: &Store, id: i64, at: i64) {
    let Some(path) = store
        .item(id)
        .ok()
        .flatten()
        .as_ref()
        .and_then(crate::media::first_path_of)
    else {
        done(store, id, "media");
        return;
    };
    let said = here::media_of(std::path::Path::new(&path));
    if said.is_empty() {
        give_up(store, id, "media", "the shell knows nothing about it", at);
        return;
    }
    for (key, value) in said {
        if let Err(why) = store.set_meta(id, key, &value) {
            note(&format!("{id} has a {key} nobody wrote down: {why}"));
        }
    }
    done(store, id, "media");
}

fn grouped_some(store: &Store) -> bool {
    let waiting = match store.ungrouped(crate::group::AT_A_TIME) {
        Ok(waiting) => waiting,
        Err(why) => {
            note(&format!("what has no group could not be looked at: {why}"));
            return false;
        }
    };
    if waiting.is_empty() {
        return false;
    }
    for (id, kind, preview) in &waiting {
        let key = crate::group::key_of(*kind, preview);
        let key = if key.is_empty() {
            crate::group::UNKNOWN
        } else {
            key.as_str()
        };
        if let Err(why) = store.set_group(*id, key) {
            note(&format!("{id} was left without a group: {why}"));
            return false;
        }
    }
    true
}

fn walked(store: &Store, id: i64, at: i64) {
    let Some(path) = store
        .item(id)
        .ok()
        .flatten()
        .as_ref()
        .and_then(crate::media::first_path_of)
    else {
        done(store, id, "folder");
        return;
    };
    let counting =
        cp_core::reading::begin(move || crate::folder::counted_in(std::path::Path::new(&path)));
    match counting.waited(crate::folder::PATIENCE) {
        cp_core::reading::Waited::Answered(Some(seen)) => {
            if let Err(why) = store.set_meta(id, crate::folder::ENTRIES, &seen.to_string()) {
                note(&format!("{id} was counted and nobody wrote it down: {why}"));
            }
            done(store, id, "folder");
        }
        cp_core::reading::Waited::Answered(None) => {
            give_up(store, id, "folder", "the folder could not be read", at);
        }
        cp_core::reading::Waited::StillRunning => {
            give_up(store, id, "folder", "the folder did not answer in time", at);
        }
        cp_core::reading::Waited::Gone => {
            give_up(
                store,
                id,
                "folder",
                "counting the folder did not survive",
                at,
            );
        }
    }
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
    if item.kind == Some(Kind::Audio) {
        let bars = crate::wave::bars_of(Path::new(first))?;
        return cp_core::thumbnail::of_wave(&bars, crate::wave::TALLEST, WAVE_WIDE, WAVE_HIGH);
    }
    here::thumb_of_file(Path::new(first), SIDE)
}

fn measured_sides(store: &Store, id: i64, width: u32, height: u32) {
    for (key, value) in [(crate::media::WIDTH, width), (crate::media::HEIGHT, height)] {
        if value > 0
            && let Err(why) = store.set_meta(id, key, &value.to_string())
        {
            note(&format!("{id} has a {key} nobody wrote down: {why}"));
        }
    }
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
    let key = crate::group::key_of(item.kind, &preview_of(item));
    if !key.is_empty()
        && let Err(why) = store.set_group(id, &key)
    {
        note(&format!("{id} was left without a group: {why}"));
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
        Some(Kind::Image) => &["thumb", "ocr", "media"],
        Some(Kind::Video) => &["thumb", "media"],
        Some(Kind::Audio) => &["thumb", "media"],
        Some(Kind::Folder) if here::THUMBNAILS_FILES => &["thumb", "folder"],
        Some(Kind::Folder) => &["folder"],
        Some(Kind::File) if here::THUMBNAILS_FILES => &["thumb"],
        _ => &[],
    }
}

#[cfg(test)]
#[path = "engine_test.rs"]
mod tests;
