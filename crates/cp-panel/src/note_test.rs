use super::*;

#[test]
fn the_log_sits_where_the_privacy_note_says_it_does() {
    let where_it_is = where_to();
    assert!(where_it_is.starts_with(folder()));
    assert_eq!(
        where_it_is.file_name().and_then(|name| name.to_str()),
        Some("cp-panel.log")
    );
}

#[test]
fn what_the_log_holds_is_only_readable_by_whoever_copied_it() {
    let dir = std::env::temp_dir().join(format!("cp-panel-guard-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a folder to write in");
    let path = dir.join("cp-panel.log");
    std::fs::write(&path, b"a line\n").expect("a log");
    guard(&path, 0o600);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path)
            .expect("is here")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600, "the log is nobody else's business");
    }
    #[cfg(not(unix))]
    assert!(path.exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn only_the_helpers_may_fall_without_taking_the_panel_down() {
    assert!(spared(Some(ERRANDS)));
    assert!(spared(Some(COUNTER)));
    assert!(
        !spared(Some("main")),
        "the window cannot go on without its loop"
    );
    assert!(
        !spared(None),
        "the clipboard watcher and the orders reader have no name, and the panel is useless without them"
    );
}

static ENDED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn count_the_end(_: &str) {
    ENDED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

#[test]
fn a_panic_ends_the_panel_unless_a_helper_had_it() {
    let before = std::panic::take_hook();
    end_on_panic(count_the_end);
    let _ = std::thread::spawn(|| panic!("the clipboard watcher broke")).join();
    let after_the_watcher = ENDED.load(std::sync::atomic::Ordering::SeqCst);
    let _ = std::thread::Builder::new()
        .name(ERRANDS.to_owned())
        .spawn(|| panic!("a thumbnail broke"))
        .expect("a thread to break")
        .join();
    let after_the_errands = ENDED.load(std::sync::atomic::Ordering::SeqCst);
    std::panic::set_hook(before);
    assert_eq!(after_the_watcher, 1, "a watcher that broke ends the panel");
    assert_eq!(
        after_the_errands, 1,
        "a thumbnail that broke only takes its own thread"
    );
}
