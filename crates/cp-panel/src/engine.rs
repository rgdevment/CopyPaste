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
const A_DAY: i64 = 24 * 60 * 60 * 1_000;

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
    cp_store::Policy {
        keep_for: kept
            .keeps_days
            .filter(|days| *days > 0)
            .map(|days| i64::from(days) * A_DAY),
        keep_at_most: None,
        bytes_at_most: kept
            .images_quota_mb
            .filter(|mb| *mb > 0)
            .map(|mb| i64::from(mb) * 1024 * 1024),
        broken_for: None,
    }
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
            note("the clipboard stayed busy however much we insisted");
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
mod tests {
    use super::*;
    use cp_core::item::{Format, Payload, SYNTHETIC_IMAGE, SYNTHETIC_TEXT};

    fn text(what: &str) -> Item {
        Item {
            kind: Some(Kind::Text),
            formats: vec![Format {
                id: SYNTHETIC_TEXT.into(),
                payload: Payload::Inline(what.as_bytes().to_vec()),
            }],
        }
    }

    fn image() -> Item {
        Item {
            kind: Some(Kind::Image),
            formats: vec![Format {
                id: SYNTHETIC_IMAGE.into(),
                payload: Payload::Inline(vec![0x89, b'P', b'N', b'G']),
            }],
        }
    }

    fn drawn(side: u32) -> Item {
        let square = image::RgbaImage::from_pixel(side, side, image::Rgba([12, 200, 140, 255]));
        let mut out = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(square)
            .write_to(&mut out, image::ImageFormat::Png)
            .expect("drawn");
        Item {
            kind: Some(Kind::Image),
            formats: vec![Format {
                id: SYNTHETIC_IMAGE.into(),
                payload: Payload::Inline(out.into_inner()),
            }],
        }
    }

    #[cfg(target_os = "windows")]
    fn files(paths: &[&str]) -> Item {
        Item {
            kind: Some(Kind::File),
            formats: vec![Format {
                id: "CF_HDROP".into(),
                payload: Payload::Inline(cp_win::drop::drop_of(paths)),
            }],
        }
    }

    #[cfg(target_os = "macos")]
    fn files(paths: &[&str]) -> Item {
        let urls = paths
            .iter()
            .map(|path| format!("file://{path}"))
            .collect::<Vec<_>>()
            .join("\n");
        Item {
            kind: Some(Kind::File),
            formats: vec![Format {
                id: "public.file-url".into(),
                payload: Payload::Inline(urls.into_bytes()),
            }],
        }
    }

    #[cfg(target_os = "windows")]
    const SOMEWHERE: [&str; 2] = ["C:\\uno.txt", "C:\\dos.txt"];
    #[cfg(target_os = "macos")]
    const SOMEWHERE: [&str; 2] = ["/tmp/uno.txt", "/tmp/dos.txt"];

    #[test]
    fn the_preview_of_text_is_the_text_itself() {
        assert_eq!(preview_of(&text("hello world")), "hello world");
    }

    #[test]
    fn an_image_has_no_preview_until_the_reading_gives_it_one() {
        assert_eq!(preview_of(&image()), "");
    }

    #[test]
    fn files_preview_as_their_paths_one_per_line() {
        let said = preview_of(&files(&SOMEWHERE));
        assert_eq!(said, SOMEWHERE.join("\n"));
    }

    #[test]
    fn the_name_never_gives_away_what_was_copied() {
        let secret = text("a password");
        let one = name_for(1_000, &secret);
        let other = name_for(1_000, &secret);
        assert_ne!(one, other, "two identical copies never share a name");
        let fingerprint = format!("{:016x}", secret.fingerprint());
        assert!(
            !one.contains(&fingerprint),
            "the name gives away what was copied"
        );
    }

    #[test]
    fn only_what_can_be_enriched_is_queued() {
        assert_eq!(jobs_for(&image()), ["thumb", "ocr"]);
        assert!(jobs_for(&text("nothing to do here")).is_empty());
    }

    #[test]
    fn a_file_is_only_queued_where_its_thumbnail_can_be_drawn() {
        let queued = jobs_for(&files(&SOMEWHERE));
        if here::THUMBNAILS_FILES {
            assert_eq!(queued, ["thumb"]);
        } else {
            assert!(
                queued.is_empty(),
                "queueing what nobody will draw is no use"
            );
        }
    }

    fn somewhere() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().expect("a folder");
        let store = Store::open(&dir.path().join("history.db")).expect("opened");
        (dir, store)
    }

    #[test]
    fn without_a_window_in_front_the_card_simply_has_no_app() {
        let (_dir, store) = somewhere();
        let id = keep(&store, &text("from nowhere at all"), 1_000, None).expect("stored");
        let page = store
            .list(&cp_store::Filter::default(), 10, None)
            .expect("listed");
        let mine = page.rows.iter().find(|one| one.id == id).expect("is here");
        assert_eq!(mine.app, None);
    }

    #[test]
    fn what_is_copied_lands_with_its_preview_and_its_kind() {
        let (_dir, store) = somewhere();
        let id = keep(&store, &text("the first thing"), 1_000, None).expect("stored");
        let kept = store.item(id).expect("read").expect("still there");
        assert_eq!(kept.kind, Some(Kind::Text));
        assert_eq!(store.count().expect("counted"), 1);
    }

    #[test]
    fn the_same_thing_copied_twice_is_one_row_that_rises() {
        let (_dir, store) = somewhere();
        let first = keep(&store, &text("the same"), 1_000, None).expect("stored");
        let again = keep(&store, &text("the same"), 5_000, None).expect("recognised");
        assert_eq!(first, again);
        assert_eq!(store.count().expect("counted"), 1);
    }

    #[test]
    fn two_different_copies_are_two_rows() {
        let (_dir, store) = somewhere();
        keep(&store, &text("one"), 1_000, None).expect("stored");
        keep(&store, &text("another"), 2_000, None).expect("stored");
        assert_eq!(store.count().expect("counted"), 2);
    }

    #[test]
    fn an_image_leaves_its_reading_and_its_thumbnail_pending() {
        let (_dir, store) = somewhere();
        let id = keep(&store, &image(), 1_000, None).expect("stored");
        let mut waiting = store.take_pending("ocr", 2_000, 10).expect("the queue");
        waiting.extend(store.take_pending("thumb", 2_000, 10).expect("the queue"));
        assert_eq!(waiting, [id, id]);
    }

    #[test]
    fn what_was_copied_remembers_the_app_it_came_from() {
        let (_dir, store) = somewhere();
        let id = keep(&store, &text("from the browser"), 1_000, Some("chrome")).expect("stored");
        let page = store
            .list(&cp_store::Filter::default(), 10, None)
            .expect("listed");
        let mine = page.rows.iter().find(|one| one.id == id).expect("is here");
        assert_eq!(mine.app.as_deref(), Some("chrome"));
    }

    #[test]
    fn a_picture_ends_up_with_a_thumbnail_it_can_show() {
        let (dir, store) = somewhere();
        let thumbs = dir.path().join("thumbs");
        let id = keep(&store, &drawn(600), 1_000, None).expect("stored");
        assert!(errand(&store, &thumbs), "there was work to do");
        let page = store
            .list(&cp_store::Filter::default(), 10, None)
            .expect("listed");
        let mine = page.rows.iter().find(|one| one.id == id).expect("is here");
        let made = mine.thumb_path.as_deref().expect("it has a thumbnail");
        assert!(
            std::path::Path::new(made).exists(),
            "{made} was never written"
        );
        let side =
            cp_core::thumbnail::size_of(&std::fs::read(made).expect("read")).expect("a size");
        assert!(side.width <= cp_core::thumbnail::MAX_SIDE);
        assert!(
            store
                .take_pending("thumb", 2_000, 10)
                .expect("the queue")
                .is_empty()
        );
    }

    #[test]
    fn what_the_settings_say_becomes_what_the_store_sweeps() {
        let kept = cp_config::Config {
            keeps_days: Some(30),
            images_quota_mb: Some(512),
            ..cp_config::Config::default()
        };
        let policy = policy_of(&kept);
        assert_eq!(policy.keep_for, Some(30 * A_DAY));
        assert_eq!(policy.bytes_at_most, Some(512 * 1024 * 1024));
    }

    #[test]
    fn keeping_things_forever_and_without_a_limit_sweeps_nothing() {
        let kept = cp_config::Config {
            keeps_days: None,
            images_quota_mb: None,
            ..cp_config::Config::default()
        };
        assert_eq!(policy_of(&kept), cp_store::Policy::default());
        let zeroed = cp_config::Config {
            keeps_days: Some(0),
            images_quota_mb: Some(0),
            ..cp_config::Config::default()
        };
        assert_eq!(policy_of(&zeroed), cp_store::Policy::default());
    }

    #[test]
    fn what_is_older_than_the_setting_goes_and_what_is_pinned_stays() {
        let (_dir, store) = somewhere();
        let now = 100 * A_DAY;
        let old = keep(&store, &text("from long ago"), now - 40 * A_DAY, None).expect("stored");
        let recent = keep(&store, &text("from yesterday"), now - A_DAY, None).expect("stored");
        let pinned = keep(&store, &text("pinned and old"), now - 40 * A_DAY, None).expect("stored");
        store.set_pinned(pinned, true, now).expect("pinned");
        let kept = cp_config::Config {
            keeps_days: Some(30),
            ..cp_config::Config::default()
        };
        store.sweep(&policy_of(&kept), now).expect("swept");
        let page = store
            .list(&cp_store::Filter::default(), 10, None)
            .expect("listed");
        let left: Vec<i64> = page.rows.iter().map(|one| one.id).collect();
        assert!(!left.contains(&old), "what was old had to go");
        assert!(left.contains(&recent), "what is recent stays");
        assert!(left.contains(&pinned), "what is pinned never expires");
    }

    #[test]
    fn nothing_waiting_means_nothing_to_do() {
        let (dir, store) = somewhere();
        assert!(!errand(&store, &dir.path().join("thumbs")));
    }

    #[test]
    fn something_that_cannot_be_drawn_waits_instead_of_spinning() {
        let (dir, store) = somewhere();
        let thumbs = dir.path().join("thumbs");
        let id = keep(&store, &image(), 1_000, None).expect("stored");
        assert!(errand(&store, &thumbs), "it gave it a try");
        let now = crate::app::now_ms();
        assert!(
            store
                .take_pending("thumb", now, 10)
                .expect("the queue")
                .is_empty(),
            "it is not retried straight away"
        );
        let later = store
            .take_pending("thumb", now + LATER + 1_000, 10)
            .expect("the queue");
        assert_eq!(later, [id], "its turn comes round again later");
    }

    #[test]
    fn plain_text_asks_nobody_for_anything() {
        let (_dir, store) = somewhere();
        keep(&store, &text("with no trimmings"), 1_000, None).expect("stored");
        assert!(
            store
                .take_pending("thumb", 2_000, 10)
                .expect("the queue")
                .is_empty()
        );
        assert!(
            store
                .take_pending("ocr", 2_000, 10)
                .expect("the queue")
                .is_empty()
        );
    }
}
