use super::*;

const AT_MS: i64 = 1_700_000_000_000;

fn damaged(dir: &Path) -> PathBuf {
    let path = dir.join("history.db");
    std::fs::write(&path, vec![0x5a; 8192]).expect("written");
    path
}

#[test]
fn a_healthy_history_opens_and_nothing_is_set_aside() {
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().join("history.db");
    let opened = open_or_set_aside(&path, AT_MS).expect("opened");
    assert!(opened.set_aside.is_none());
    assert_eq!(opened.store.count().expect("counted"), 0);
}

#[test]
fn a_file_that_is_not_a_database_is_kept_aside_and_a_fresh_one_starts() {
    let dir = tempfile::tempdir().expect("dir");
    let path = damaged(dir.path());
    let opened = open_or_set_aside(&path, AT_MS).expect("opened");
    let kept = opened.set_aside.expect("set aside");
    assert_eq!(
        kept.file_name().and_then(|name| name.to_str()),
        Some("history.corrupt-1700000000000.db")
    );
    assert_eq!(std::fs::read(&kept).expect("kept"), vec![0x5a; 8192]);
    opened
        .store
        .insert_text("uuid-a", "fresh", 1)
        .expect("usable");
    assert_eq!(opened.store.count().expect("counted"), 1);
}

#[test]
fn what_sat_next_to_the_damaged_file_goes_aside_with_it() {
    let dir = tempfile::tempdir().expect("dir");
    let path = damaged(dir.path());
    std::fs::write(dir.path().join("history.db-wal"), b"wal").expect("wal");
    std::fs::write(dir.path().join("history.db-shm"), b"shm").expect("shm");
    std::fs::create_dir(dir.path().join("blobs")).expect("blobs");
    std::fs::write(dir.path().join("blobs").join("one"), b"bytes").expect("blob");
    set_aside(&path, AT_MS).expect("set aside");
    let aside = |name: &str| dir.path().join(name);
    assert_eq!(
        std::fs::read(aside("history.corrupt-1700000000000.db-wal")).expect("wal kept"),
        b"wal"
    );
    assert!(aside("history.corrupt-1700000000000.db-shm").exists());
    assert_eq!(
        std::fs::read(aside("blobs.corrupt-1700000000000").join("one")).expect("blob kept"),
        b"bytes"
    );
    assert!(!aside("blobs").join("one").exists());
}

#[test]
fn a_history_from_the_future_is_not_touched() {
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().join("history.db");
    let future = crate::SCHEMA_VERSION + 1;
    {
        let db = rusqlite::Connection::open(&path).expect("created");
        db.execute_batch(&format!(
            "PRAGMA user_version = {future}; CREATE TABLE t(x);"
        ))
        .expect("written");
    }
    let before = std::fs::read(&path).expect("read");
    let failed = open_or_set_aside(&path, AT_MS).err().expect("refused");
    assert!(matches!(failed, Error::FromTheFuture { found, .. } if found == future));
    assert_eq!(std::fs::read(&path).expect("still there"), before);
    let others = std::fs::read_dir(dir.path()).expect("listed").count();
    assert!(others <= 1 + 2, "nothing was moved: {others}");
    assert!(!dir.path().join("history.corrupt-1700000000000.db").exists());
}

#[test]
fn a_path_that_cannot_be_opened_at_all_is_an_error_and_moves_nothing() {
    let dir = tempfile::tempdir().expect("dir");
    let blocker = dir.path().join("file");
    std::fs::write(&blocker, b"x").expect("written");
    let path = blocker.join("history.db");
    assert!(open_or_set_aside(&path, AT_MS).is_err());
    assert_eq!(std::fs::read(&blocker).expect("same"), b"x");
}

#[test]
fn the_thumbnails_go_aside_with_the_history_they_belong_to() {
    let dir = tempfile::tempdir().expect("dir");
    let path = damaged(dir.path());
    std::fs::create_dir(dir.path().join("thumbs")).expect("thumbs");
    std::fs::write(dir.path().join("thumbs").join("one.png"), b"png").expect("thumb");
    set_aside(&path, AT_MS).expect("set aside");
    assert_eq!(
        std::fs::read(
            dir.path()
                .join("thumbs.corrupt-1700000000000")
                .join("one.png")
        )
        .expect("thumb kept"),
        b"png"
    );
}

#[test]
fn a_move_that_fails_halfway_puts_back_what_had_moved_and_leaves_the_history_in_place() {
    let dir = tempfile::tempdir().expect("dir");
    let path = damaged(dir.path());
    std::fs::write(dir.path().join("history.db-wal"), b"wal").expect("wal");
    std::fs::create_dir(dir.path().join("blobs")).expect("blobs");
    std::fs::write(dir.path().join("blobs").join("one"), b"bytes").expect("blob");
    std::fs::create_dir(dir.path().join("thumbs")).expect("thumbs");
    std::fs::create_dir(dir.path().join("thumbs.corrupt-1700000000000")).expect("in the way");
    std::fs::write(
        dir.path()
            .join("thumbs.corrupt-1700000000000")
            .join("taken"),
        b"x",
    )
    .expect("a full folder blocks the move");
    assert!(set_aside(&path, AT_MS).is_err());
    assert!(path.exists(), "the history stays where it was");
    assert_eq!(
        std::fs::read(dir.path().join("history.db-wal")).expect("wal back"),
        b"wal"
    );
    assert_eq!(
        std::fs::read(dir.path().join("blobs").join("one")).expect("blob back"),
        b"bytes",
        "the pictures are back beside the history that names them"
    );
}
