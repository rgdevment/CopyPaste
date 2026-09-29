use super::*;

#[test]
fn the_clock_reads_as_hours_minutes_and_seconds() {
    let stamp = clock_now();
    assert_eq!(stamp.len(), 8);
    assert_eq!(stamp.matches(':').count(), 2);
    assert!(
        stamp.chars().all(|one| one.is_ascii_digit() || one == ':'),
        "{stamp}"
    );
}

#[test]
fn what_the_log_holds_is_only_readable_by_whoever_copied_it() {
    note("any line at all");
    let path = where_to();
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
}

#[test]
fn the_log_sits_where_the_privacy_note_says_it_does() {
    let where_it_is = where_to();
    assert!(where_it_is.starts_with(folder()));
    assert_eq!(
        where_it_is.file_name().and_then(|name| name.to_str()),
        Some("cp-panel.log")
    );
}
