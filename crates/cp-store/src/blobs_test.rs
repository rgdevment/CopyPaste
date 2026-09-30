use super::*;

fn temporary() -> (tempfile::TempDir, Blobs) {
    let dir = tempfile::tempdir().expect("a folder");
    let blobs = Blobs::at(&dir.path().join("blobs")).expect("a store");
    (dir, blobs)
}

#[test]
fn what_goes_in_comes_out() {
    let (_dir, blobs) = temporary();
    let digest = blobs.put(b"some arbitrary bytes").expect("stored");
    assert_eq!(
        blobs.get(&digest).expect("read").as_deref(),
        Some(&b"some arbitrary bytes"[..])
    );
}

#[test]
fn the_same_content_is_stored_once() {
    let (_dir, blobs) = temporary();
    let first = blobs.put(b"repeated").expect("stored");
    let second = blobs.put(b"repeated").expect("stored again");
    assert_eq!(first, second, "the name is the content");
}

#[test]
fn different_content_never_shares_a_name() {
    let (_dir, blobs) = temporary();
    assert_ne!(
        blobs.put(b"one").expect("a"),
        blobs.put(b"other").expect("b")
    );
}

#[test]
fn an_io_error_that_is_not_a_missing_file_is_not_swallowed() {
    let (_dir, blobs) = temporary();
    let digest = "a".repeat(64);
    std::fs::create_dir_all(blobs.path_for(&digest)).expect("creates a folder");
    assert!(
        blobs.get(&digest).is_err(),
        "a genuine I/O error cannot come back as Ok(None)"
    );
}

#[test]
fn asking_for_something_that_is_not_there_is_not_an_error() {
    let (_dir, blobs) = temporary();
    let missing = "0".repeat(64);
    assert!(blobs.get(&missing).expect("read").is_none());
    assert!(!blobs.exists(&missing));
    blobs
        .remove(&missing)
        .expect("deleting what is not there does not fail");
}

#[test]
fn deleting_overwrites_before_unlinking() {
    let (dir, blobs) = temporary();
    let digest = blobs.put(b"the bank password").expect("stored");
    let twin = dir.path().join("twin");
    std::fs::hard_link(blobs.path_for(&digest), &twin).expect("a hard link");
    blobs.remove(&digest).expect("removed");
    assert!(!blobs.exists(&digest));
    assert!(blobs.get(&digest).expect("read").is_none());
    let bytes = std::fs::read(&twin).expect("the other name still stands");
    assert_eq!(bytes.len(), b"the bank password".len());
    assert!(
        bytes.iter().all(|b| *b == 0),
        "the blocks were overwritten with zeros before the name was let go"
    );
}

#[cfg(unix)]
fn symlink_file(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).expect("a link");
}

#[cfg(windows)]
fn symlink_file(target: &Path, link: &Path) {
    std::os::windows::fs::symlink_file(target, link).expect("a link");
}

#[test]
fn a_symlink_named_like_a_digest_is_unlinked_never_followed() {
    let (dir, blobs) = temporary();
    let victim = dir.path().join("not-ours.txt");
    std::fs::write(&victim, b"do not touch me").expect("a file");
    let digest = "c".repeat(64);
    let link = blobs.path_for(&digest);
    std::fs::create_dir_all(link.parent().expect("a parent")).expect("a folder");
    symlink_file(&victim, &link);
    aged(&victim);
    assert_eq!(blobs.sweep(&|_| false).expect("swept"), 1);
    assert!(!link.exists() && std::fs::symlink_metadata(&link).is_err());
    assert_eq!(
        std::fs::read(&victim).expect("still there"),
        b"do not touch me",
        "the link gets removed, never what is behind it"
    );
}

#[test]
fn a_directory_named_like_a_digest_is_left_alone() {
    let (_dir, blobs) = temporary();
    let digest = "d".repeat(64);
    let folder = blobs.path_for(&digest);
    std::fs::create_dir_all(&folder).expect("a folder");
    assert_eq!(blobs.sweep(&|_| false).expect("swept"), 0);
    assert!(folder.is_dir());
    blobs
        .remove(&digest)
        .expect("deleting a folder is not deleting a blob");
    assert!(folder.is_dir());
}

#[test]
fn an_empty_blob_is_still_a_blob() {
    let (_dir, blobs) = temporary();
    let digest = blobs.put(b"").expect("stored");
    assert_eq!(blobs.get(&digest).expect("read"), Some(Vec::new()));
}

#[test]
fn nothing_is_left_behind_when_writing_succeeds() {
    let (dir, blobs) = temporary();
    blobs.put(b"something").expect("stored");
    let partials = files_under(&dir.path().join("blobs"))
        .into_iter()
        .filter(|p| p.extension().is_some_and(|e| e == "partial"))
        .count();
    assert_eq!(
        partials, 0,
        "the temporary file gets renamed, not left behind"
    );
}

fn aged(path: &Path) {
    let file = std::fs::File::options()
        .write(true)
        .open(path)
        .expect("opened");
    file.set_modified(std::time::UNIX_EPOCH).expect("aged");
}

#[test]
fn a_blob_nobody_references_is_removed_once_it_is_old_enough() {
    let (_dir, blobs) = temporary();
    let digest = blobs.put(b"orphaned").expect("stored");
    assert_eq!(
        blobs.sweep(&|_| false).expect("swept"),
        0,
        "freshly written: it might belong to an item that is only half stored"
    );
    aged(&blobs.path_for(&digest));
    assert_eq!(blobs.sweep(&|_| false).expect("swept"), 1);
    assert!(!blobs.exists(&digest));
}

#[test]
fn a_referenced_blob_survives_the_sweep_however_old() {
    let (_dir, blobs) = temporary();
    let digest = blobs.put(b"has an owner").expect("stored");
    aged(&blobs.path_for(&digest));
    let keep = digest.clone();
    assert_eq!(blobs.sweep(&|name| name == keep).expect("swept"), 0);
    assert!(blobs.exists(&digest));
}

#[test]
fn a_partial_file_left_by_a_crash_goes_too() {
    let (_dir, blobs) = temporary();
    let digest = "b".repeat(64);
    let leftover = blobs.path_for(&digest).with_extension("partial");
    std::fs::create_dir_all(leftover.parent().expect("a parent")).expect("a folder");
    std::fs::write(&leftover, b"halfway done").expect("written");
    aged(&leftover);
    assert_eq!(blobs.sweep(&|_| true).expect("swept"), 1);
    assert!(!leftover.exists(), "even though its digest is referenced");
}

#[test]
fn a_fresh_blob_is_not_removed_by_a_release_only_by_the_sweep_later() {
    let (_dir, blobs) = temporary();
    let digest = blobs.put(b"freshly written").expect("stored");
    assert!(
        !blobs.remove_if_settled(&digest).expect("does not touch it"),
        "another connection might be about to reference it"
    );
    assert!(blobs.exists(&digest));
    aged(&blobs.path_for(&digest));
    assert!(blobs.remove_if_settled(&digest).expect("now it does"));
    assert!(!blobs.exists(&digest));
    assert!(
        !blobs.remove_if_settled(&digest).expect("it is gone now"),
        "deleting twice is not an error"
    );
}

#[test]
fn two_writers_never_share_a_temporary_file() {
    let (dir, blobs) = temporary();
    let a = blobs.put(b"one").expect("a");
    let b = blobs.put(b"two").expect("b");
    assert_ne!(a, b);
    let leftovers = files_under(&dir.path().join("blobs"))
        .into_iter()
        .filter(|p| p.to_string_lossy().contains(".partial"))
        .count();
    assert_eq!(leftovers, 0);
}

#[test]
fn sweeping_an_empty_store_is_nothing() {
    let (_dir, blobs) = temporary();
    assert_eq!(blobs.sweep(&|_| false).expect("swept"), 0);
}

#[test]
fn a_file_that_is_not_a_blob_is_neither_touched_nor_counted() {
    let (dir, blobs) = temporary();
    let root = dir.path().join("blobs");
    let strays = [
        root.join(".DS_Store"),
        root.join("x"),
        root.join("façade.txt"),
        root.join("ab").join("cd").join("notes.partial"),
    ];
    for stray in &strays {
        std::fs::create_dir_all(stray.parent().expect("a parent")).expect("a folder");
        std::fs::write(stray, b"not ours").expect("written");
        aged(stray);
    }
    assert_eq!(blobs.sweep(&|_| false).expect("swept"), 0);
    for stray in &strays {
        assert!(stray.exists(), "{} was not ours", stray.display());
    }
}

#[test]
fn only_a_digest_shaped_name_is_a_blob() {
    let digest = "0123456789abcdef".repeat(4);
    assert_eq!(digest_of(&digest), Some((digest.as_str(), false)));
    let partial = format!("{digest}.partial");
    assert_eq!(digest_of(&partial), Some((digest.as_str(), true)));
    let private = format!("{digest}.4242-7.partial");
    assert_eq!(digest_of(&private), Some((digest.as_str(), true)));
    assert_eq!(digest_of(".DS_Store"), None);
    assert_eq!(digest_of(&"g".repeat(64)), None, "is not hexadecimal");
    assert_eq!(digest_of(&"a".repeat(63)), None, "it is one short");
    assert_eq!(digest_of("x.partial"), None);
}

#[test]
fn reclaiming_a_blob_that_already_exists_makes_it_fresh_again() {
    let (_dir, blobs) = temporary();
    let digest = blobs.put(b"reclaimed").expect("stored");
    aged(&blobs.path_for(&digest));
    blobs.put(b"reclaimed").expect("again");
    assert_eq!(
        blobs.sweep(&|_| false).expect("swept"),
        0,
        "whoever just reclaimed it has not written its row yet"
    );
    assert!(blobs.exists(&digest));
}
