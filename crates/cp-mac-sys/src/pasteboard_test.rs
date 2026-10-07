use super::*;

fn reference_of(path: &std::path::Path) -> String {
    NSURL::fileURLWithPath(&NSString::from_str(&path.display().to_string()))
        .fileReferenceURL()
        .and_then(|url| url.absoluteString())
        .map(|url| url.to_string())
        .expect("a file that exists has a reference")
}

#[test]
fn a_finder_reference_becomes_the_path_it_points_at() {
    let dir = std::env::temp_dir().join(format!("cp-ref-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("profile picture.png");
    std::fs::write(&file, b"png").unwrap();
    let reference = reference_of(&file);
    assert!(reference.starts_with("file:///.file/id="), "{reference}");
    let resolved = resolved_file_url(&reference).unwrap();
    std::fs::remove_dir_all(&dir).ok();
    assert!(resolved.starts_with("file:///"), "{resolved}");
    assert!(resolved.ends_with("/profile%20picture.png"), "{resolved}");
    assert!(!resolved.contains(".file/id="));
}

#[test]
fn a_path_url_comes_back_as_it_was() {
    assert_eq!(
        resolved_file_url("file:///tmp/one.txt").as_deref(),
        Some("file:///tmp/one.txt")
    );
    assert_eq!(
        resolved_file_url("file:///tmp/with%20space.txt").as_deref(),
        Some("file:///tmp/with%20space.txt")
    );
}

#[test]
fn what_is_not_a_file_has_no_path() {
    assert_eq!(resolved_file_url("https://example.com/x.png"), None);
    assert_eq!(resolved_file_url("not a url"), None);
    assert_eq!(resolved_file_url(""), None);
}

#[test]
fn a_dead_reference_has_no_path_either() {
    let file = std::env::temp_dir().join(format!("cp-dead-{}.txt", std::process::id()));
    std::fs::write(&file, b"ephemeral").unwrap();
    let reference = reference_of(&file);
    std::fs::remove_file(&file).unwrap();
    assert_eq!(resolved_file_url(&reference), None);
}

#[test]
fn the_system_codes_become_the_access_they_name() {
    assert_eq!(Access::from_raw(0), Some(Access::Default));
    assert_eq!(Access::from_raw(1), Some(Access::Ask));
    assert_eq!(Access::from_raw(2), Some(Access::AlwaysAllow));
    assert_eq!(Access::from_raw(3), Some(Access::AlwaysDeny));
    assert_eq!(Access::from_raw(4), None);
    assert_eq!(Access::from_raw(-1), None);
}

#[test]
fn asking_for_the_access_never_fails_even_on_a_mac_that_lacks_it() {
    let _ = access_from_any_thread();
}
