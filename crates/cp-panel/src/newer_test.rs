use super::*;

const HERE: &str = "3.0.3";

#[test]
fn a_newer_version_found_by_this_copy_is_waiting() {
    let found = r#"{"checked_at":1,"found":"3.0.4","from":"3.0.3"}"#;
    assert_eq!(waiting(found, HERE, None).as_deref(), Some("3.0.4"));
}

#[test]
fn what_an_older_copy_found_is_not_this_one_s_news() {
    let found = r#"{"checked_at":1,"found":"3.0.3","from":"3.0.2"}"#;
    assert_eq!(waiting(found, HERE, None), None);
    let stale = r#"{"checked_at":1,"found":"3.0.4","from":"3.0.2"}"#;
    assert_eq!(waiting(stale, HERE, None), None);
}

#[test]
fn nothing_found_broken_or_this_same_version_says_nothing() {
    assert_eq!(
        waiting(
            r#"{"checked_at":1,"found":null,"from":"3.0.3"}"#,
            HERE,
            None
        ),
        None
    );
    assert_eq!(
        waiting(r#"{"found":"  ","from":"3.0.3"}"#, HERE, None),
        None
    );
    assert_eq!(
        waiting(r#"{"found":"3.0.3","from":"3.0.3"}"#, HERE, None),
        None
    );
    assert_eq!(waiting("not json", HERE, None), None);
    assert_eq!(waiting(r#"{"found":"3.0.4"}"#, HERE, None), None);
}

#[test]
fn a_version_put_away_stays_away_until_a_newer_one_arrives() {
    let found = r#"{"found":"3.0.4","from":"3.0.3"}"#;
    assert_eq!(waiting(found, HERE, Some("3.0.4\n")), None);
    let newer = r#"{"found":"3.0.5","from":"3.0.3"}"#;
    assert_eq!(
        waiting(newer, HERE, Some("3.0.4")).as_deref(),
        Some("3.0.5")
    );
}

#[test]
fn the_files_on_disk_are_read_and_putting_away_is_remembered() {
    let dir = tempfile::tempdir().expect("a folder");
    assert_eq!(waiting_in(dir.path(), HERE), None);
    std::fs::write(
        dir.path().join(FOUND),
        r#"{"found":"3.0.4","from":"3.0.3"}"#,
    )
    .expect("writes");
    assert_eq!(waiting_in(dir.path(), HERE).as_deref(), Some("3.0.4"));
    put_away_in(dir.path(), "3.0.4").expect("puts away");
    assert_eq!(waiting_in(dir.path(), HERE), None);
}

#[test]
fn the_line_names_the_version_in_either_language() {
    assert_eq!(line_of("3.0.4", false), "CopyPaste 3.0.4 está lista");
    assert_eq!(line_of("3.0.4", true), "CopyPaste 3.0.4 is ready");
}
