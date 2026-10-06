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

fn aside(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("cp-engine-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a folder to work in");
    dir
}

#[test]
fn an_engine_that_started_closes_and_closing_it_twice_is_no_error() {
    let dir = aside("closing");
    let engine = Engine::start(&dir.join("history.db"), |_| {}).expect("an engine");
    let started = std::time::Instant::now();
    engine.close();
    engine.close();
    assert!(
        started.elapsed() < cp_core::closing::PATIENCE * 3,
        "closing cannot take longer than the patience it was given"
    );
    drop(engine);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_engine_nobody_closed_is_still_closed_by_dropping_it() {
    let dir = aside("dropping");
    let engine = Engine::start(&dir.join("history.db"), |_| {}).expect("an engine");
    let started = std::time::Instant::now();
    drop(engine);
    assert!(
        started.elapsed() < cp_core::closing::PATIENCE * 3,
        "the drop has the same deadline as the deliberate close"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_loose_blob_is_collected_even_when_the_user_keeps_everything_for_ever() {
    let dir = aside("keeping-everything");
    let store = Store::open(&dir.join("history.db")).expect("a store");
    let blobs = store.blobs().expect("a blob store");
    let digest = blobs.put(b"nobody will claim me").expect("stored");
    let kept = cp_config::Config {
        keeps_days: None,
        images_quota_mb: None,
        ..Default::default()
    };
    let path = dir.join("config.toml");
    cp_config::write(&path, &kept).expect("written");
    assert_eq!(
        policy_of(&cp_config::read(&path).expect("read")),
        cp_store::Policy::default(),
        "«always» and «no limit» is the policy that asks for nothing"
    );
    let old = std::time::UNIX_EPOCH;
    let file = std::fs::File::options()
        .write(true)
        .open(blobs.where_it_is(&digest).expect("a real digest"))
        .expect("opened");
    file.set_modified(old).expect("aged");
    drop(file);

    sweep_as_kept(&store, &path, &dir.join("thumbs"));

    assert!(
        !blobs.exists(&digest),
        "a blob nobody references is nobody's, whatever the retention says"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_copy_with_nothing_readable_is_not_stored_as_an_empty_card() {
    let (_dir, store) = somewhere();
    let unread = [
        Payload::TooBig {
            size: 70 * 1024 * 1024,
        },
        Payload::Announced { size: None },
        Payload::Absent,
    ];
    for payload in unread {
        let item = Item {
            kind: None,
            formats: vec![Format {
                id: "public.tiff".into(),
                payload,
            }],
        };
        assert_eq!(keep(&store, &item, 1_000, None), None);
    }
    assert_eq!(keep(&store, &of_kind(Kind::Text), 1_000, None), None);
    assert_eq!(store.count().expect("counted"), 0);
}

#[test]
fn a_copy_with_one_readable_format_among_unread_ones_is_still_kept() {
    let (_dir, store) = somewhere();
    let item = Item {
        kind: Some(Kind::Image),
        formats: vec![
            Format {
                id: "public.tiff".into(),
                payload: Payload::TooBig {
                    size: 70 * 1024 * 1024,
                },
            },
            Format {
                id: SYNTHETIC_IMAGE.into(),
                payload: Payload::Inline(vec![0x89, b'P', b'N', b'G']),
            },
        ],
    };
    assert!(keep(&store, &item, 1_000, None).is_some());
    assert_eq!(store.count().expect("counted"), 1);
}

fn refuse_writes_to_the_queue(store: &Store) {
    store
        .raw()
        .execute_batch(
            "CREATE TRIGGER no_room_left BEFORE DELETE ON pending_work
             BEGIN SELECT RAISE(ABORT, 'disk full'); END;
             CREATE TRIGGER no_room_left_either BEFORE UPDATE ON pending_work
             BEGIN SELECT RAISE(ABORT, 'disk full'); END;",
        )
        .expect("trigger");
}

#[test]
fn an_errand_whose_outcome_cannot_be_written_backs_off_instead_of_spinning() {
    let (dir, store) = somewhere();
    let thumbs = dir.path().join("thumbs");
    keep(&store, &image(), 1_000, None).expect("stored");
    refuse_writes_to_the_queue(&store);
    assert!(
        !errand(&store, &thumbs),
        "the job is still waiting and the loop must nap"
    );
}

#[test]
fn done_and_giving_up_say_whether_they_were_written() {
    let (_dir, store) = somewhere();
    let id = keep(&store, &image(), 1_000, None).expect("stored");
    assert!(give_up(&store, id, "thumb", "no luck", 1_000));
    refuse_writes_to_the_queue(&store);
    assert!(!give_up(&store, id, "thumb", "no luck", 1_000));
    assert!(!done(&store, id, "thumb"));
}

fn waiting(store: &Store, job: &str) -> Vec<i64> {
    store
        .take_pending(job, crate::app::now_ms() + LATER * 1_000, 10)
        .expect("the queue")
}

#[test]
fn each_kind_of_errand_is_taken_and_leaves_its_queue() {
    let (dir, store) = somewhere();
    let thumbs = dir.path().join("thumbs");
    let id = keep(&store, &text("nothing to measure"), 1_000, None).expect("stored");
    for job in ["ocr", "media", "folder"] {
        store.enqueue(id, job).expect("queued");
        assert!(errand(&store, &thumbs), "the {job} errand was taken");
        assert!(waiting(&store, job).is_empty(), "the {job} errand is over");
    }
}

#[test]
fn a_real_folder_is_counted_and_a_missing_one_waits_for_later() {
    let (dir, store) = somewhere();
    let thumbs = dir.path().join("thumbs");
    let folder = dir.path().join("a folder");
    std::fs::create_dir(&folder).expect("folder");
    std::fs::write(folder.join("one"), b"1").expect("one");
    let here = folder.to_string_lossy().into_owned();
    let id = keep(&store, &files(&[&here]), 1_000, None).expect("stored");
    store.enqueue(id, "folder").expect("queued");
    assert!(errand(&store, &thumbs));
    assert!(waiting(&store, "folder").is_empty());
    std::fs::remove_dir_all(&folder).expect("gone");
    store.enqueue(id, "folder").expect("queued again");
    assert!(errand(&store, &thumbs), "it gave it a try");
    assert_eq!(
        waiting(&store, "folder"),
        [id],
        "a folder that is not there is tried later"
    );
}

#[test]
fn a_file_the_system_knows_nothing_about_is_measured_later() {
    let (dir, store) = somewhere();
    let thumbs = dir.path().join("thumbs");
    let plain = dir.path().join("notes.txt");
    std::fs::write(&plain, b"just words").expect("file");
    let here = plain.to_string_lossy().into_owned();
    let id = keep(&store, &files(&[&here]), 1_000, None).expect("stored");
    store.enqueue(id, "media").expect("queued");
    assert!(errand(&store, &thumbs));
}

#[test]
fn an_errand_that_keeps_failing_is_let_go_and_still_counts_as_written() {
    let (_dir, store) = somewhere();
    let id = keep(&store, &image(), 1_000, None).expect("stored");
    for _ in 0..12 {
        assert!(give_up(&store, id, "thumb", "never", 1_000));
    }
    assert!(
        waiting(&store, "thumb").is_empty(),
        "it is not tried for ever"
    );
}

#[test]
fn every_kind_of_errand_naps_when_its_outcome_cannot_be_written() {
    let (dir, store) = somewhere();
    let thumbs = dir.path().join("thumbs");
    let id = keep(&store, &text("nothing to measure"), 1_000, None).expect("stored");
    for job in ["ocr", "media", "folder"] {
        store.enqueue(id, job).expect("queued");
    }
    refuse_writes_to_the_queue(&store);
    assert!(
        !errand(&store, &thumbs),
        "the loop naps instead of spinning"
    );
}
