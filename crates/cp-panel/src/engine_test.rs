const A_DAY: i64 = cp_store::A_DAY;

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

fn of_kind(kind: Kind) -> Item {
    Item {
        kind: Some(kind),
        formats: Vec::new(),
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
    assert_eq!(
        jobs_for(&image()),
        ["thumb", "ocr", "media"],
        "a picture gets a thumbnail, its text read, and its sides measured"
    );
    assert_eq!(jobs_for(&of_kind(Kind::Video)), ["thumb", "media"]);
    assert_eq!(
        jobs_for(&of_kind(Kind::Audio)),
        ["thumb", "media"],
        "sound has no cover, so its thumbnail is its own waveform"
    );
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
    let side = cp_core::thumbnail::size_of(&std::fs::read(made).expect("read")).expect("a size");
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
