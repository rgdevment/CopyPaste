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
