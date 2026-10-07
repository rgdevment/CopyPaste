use super::*;

#[test]
fn what_is_in_front_is_never_ourselves() {
    if let Some(named) = in_front() {
        assert!(!named.is_empty());
        assert_ne!(Some(named), app_name(our_pid()));
    }
    assert_ne!(ahead(), our_pid());
}

#[test]
fn nobody_in_front_reads_as_zero_and_never_as_a_pid() {
    let pid = ahead();
    assert!(pid >= 0);
    if pid != 0 {
        assert!(is_alive(pid));
    }
}

#[test]
fn missing_paths_reports_what_is_gone_and_only_that() {
    let dir = std::env::temp_dir().join(format!("cp-missing-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a folder");
    let present = dir.join("it is here.txt");
    std::fs::write(&present, b"x").expect("a file");
    let urls = format!(
        "file://{}\nfile://{}/does-not-exist.txt\n\n",
        present.display().to_string().replace(' ', "%20"),
        dir.display()
    );
    let missing = missing_paths(&urls);
    std::fs::remove_dir_all(&dir).ok();
    assert_eq!(missing.len(), 1);
    assert!(missing[0].ends_with("/does-not-exist.txt"));
    assert!(missing_paths("").is_empty());
}

#[test]
fn the_bundle_of_a_pid_is_the_one_its_app_answers_with() {
    assert_eq!(bundle_of(-1), None);
    if let Some((pid, bundle)) = frontmost() {
        assert_eq!(bundle_of(pid), bundle);
    }
}

#[test]
fn neither_the_panel_nor_its_host_is_ever_where_a_paste_goes() {
    assert_eq!(other_than(Some(10), &[10, 20]), 0);
    assert_eq!(
        other_than(Some(20), &[10, 20]),
        0,
        "the settings window of the host is ours too"
    );
    assert_eq!(other_than(Some(30), &[10, 20]), 30);
    assert_eq!(other_than(None, &[10, 20]), 0);
}

#[test]
fn being_in_front_and_having_someone_ahead_never_happen_together() {
    if is_ours_in_front() {
        assert_eq!(ahead(), 0);
    }
}
