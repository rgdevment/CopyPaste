use super::*;

fn aside(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("cp-note-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir.join("cp-panel.log")
}

#[test]
fn what_the_log_holds_is_only_readable_by_whoever_copied_it() {
    let path = aside("permissions");
    note_to(&path, "any line at all");
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
    let _ = std::fs::remove_dir_all(path.parent().expect("has a folder"));
}

#[test]
fn every_line_carries_the_day_and_who_wrote_it() {
    let path = aside("shape");
    note_to(&path, "a line worth placing in time");
    let held = std::fs::read_to_string(&path).expect("is here");
    let mut parts = held.split_whitespace();
    let day = parts.next().expect("a date");
    assert_eq!(day.matches('-').count(), 2, "{held}");
    let clock = parts.next().expect("a clock");
    assert_eq!(clock.matches(':').count(), 2, "{held}");
    let whose = parts.next().expect("a pid");
    assert_eq!(
        whose.parse::<u32>().ok(),
        Some(std::process::id()),
        "{held}"
    );
    assert!(held.contains("a line worth placing in time"), "{held}");
    let _ = std::fs::remove_dir_all(path.parent().expect("has a folder"));
}

#[test]
fn a_log_that_grew_too_much_is_set_aside_not_thrown_away() {
    let path = aside("rotation");
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).expect("a folder to write in");
    }
    std::fs::write(&path, vec![b'x'; (UP_TO + 1) as usize]).expect("a log that is too big");
    note_to(&path, "the line that follows the rotation");
    let kept = path.with_extension("log.1");
    assert!(kept.exists(), "what was there has to survive somewhere");
    assert_eq!(
        std::fs::metadata(&kept).expect("is here").len(),
        UP_TO + 1,
        "and survive whole"
    );
    let held = std::fs::read_to_string(&path).expect("is here");
    assert!(
        held.contains("the line that follows the rotation"),
        "{held}"
    );
    assert!(held.len() < 200, "the new log starts empty");
    let _ = std::fs::remove_dir_all(path.parent().expect("has a folder"));
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

#[test]
fn the_message_of_a_panic_reaches_the_log_whatever_shape_it_has() {
    assert_eq!(
        said_in(&"what std writes for a literal"),
        "what std writes for a literal"
    );
    assert_eq!(
        said_in(&String::from("what it writes when there is formatting")),
        "what it writes when there is formatting"
    );
    assert_eq!(said_in(&7_u8), "with nothing said");
}
