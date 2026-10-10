use super::*;

#[test]
fn the_home_folder_is_hidden_however_its_slashes_or_case_are_written() {
    let said = redact(
        "a copy written to C:\\Users\\Mario\\Desktop\\x.copypaste\nopened c:/users/mario/AppData",
        Some("C:\\Users\\Mario"),
        None,
    );
    assert_eq!(
        said,
        "a copy written to ~\\Desktop\\x.copypaste\nopened ~/AppData"
    );
}

#[test]
fn the_user_name_is_hidden_only_as_a_whole_word() {
    let said = redact(
        "D:\\mario\\backup and MARIO~1 but not marionette",
        Some("C:\\Users\\Mario"),
        Some("mario"),
    );
    assert_eq!(said, "D:\\<user>\\backup and <user>~1 but not marionette");
}

#[test]
fn a_name_too_short_to_hide_safely_is_left_alone() {
    let said = redact("a b c", Some("/"), Some("a"));
    assert_eq!(said, "a b c");
}

#[test]
fn the_tail_starts_on_a_whole_line_and_keeps_the_end() {
    let dir = std::env::temp_dir().join("cp-report-tests");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("tail.log");
    std::fs::write(&path, "first line\nsecond line\nthird\n").expect("written");

    assert_eq!(
        tail_of(&path, 12).as_deref(),
        Some("third\n"),
        "a line cut in half would read as a different message"
    );
    assert_eq!(
        tail_of(&path, 1024).as_deref(),
        Some("first line\nsecond line\nthird\n")
    );
    assert_eq!(tail_of(&dir.join("missing.log"), 1024), None);
}

#[test]
fn the_report_is_named_after_the_day_it_was_saved() {
    assert_eq!(
        named("2026-10-10 12:00:00Z"),
        "copypaste-report-2026-10-10.txt"
    );
}

#[test]
fn a_line_said_over_and_over_is_written_once_with_its_count() {
    let log = [
        "2026-10-10 10:00:00Z 7 started",
        "2026-10-10 10:00:01Z 7 the panel said: Slint chain",
        "2026-10-10 10:00:02Z 7 the panel said: Slint chain",
        "2026-10-10 10:00:03Z 7 the panel said: Slint chain",
        "2026-10-10 10:00:04Z 7 done",
    ]
    .join("\n");
    let folded = [
        "2026-10-10 10:00:00Z 7 started",
        "2026-10-10 10:00:01Z 7 the panel said: Slint chain",
        "  (the line above, 2 more times)",
        "2026-10-10 10:00:04Z 7 done",
        "",
    ]
    .join("\n");
    assert_eq!(fold(&log, 100), folded);
}

#[test]
fn only_the_last_lines_are_kept() {
    assert_eq!(fold("a 1 1 x\nb 1 1 y\nc 1 1 z\n", 2), "b 1 1 y\nc 1 1 z\n");
}
