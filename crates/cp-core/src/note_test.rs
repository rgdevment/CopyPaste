use super::*;

fn aside(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("cp-note-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir.join("some.log")
}

fn nothing(_at: &Path, _mode: u32) {}

#[test]
fn every_line_carries_the_day_and_who_wrote_it() {
    let path = aside("shape");
    note_to(&path, "a line worth placing in time", &nothing);
    let held = std::fs::read_to_string(&path).expect("is here");
    let mut parts = held.split_whitespace();
    assert_eq!(
        parts.next().expect("a date").matches('-').count(),
        2,
        "{held}"
    );
    assert_eq!(
        parts.next().expect("a clock").matches(':').count(),
        2,
        "{held}"
    );
    assert_eq!(
        parts.next().and_then(|whose| whose.parse::<u32>().ok()),
        Some(std::process::id()),
        "{held}"
    );
    assert!(held.contains("a line worth placing in time"), "{held}");
    let _ = std::fs::remove_dir_all(path.parent().expect("has a folder"));
}

#[test]
fn the_permissions_are_asked_for_the_folder_and_for_the_file() {
    use std::sync::Mutex;
    let asked: Mutex<Vec<u32>> = Mutex::new(Vec::new());
    let path = aside("guard");
    note_to(&path, "one line", &|_at, mode| {
        asked.lock().expect("nobody poisoned it").push(mode);
    });
    assert_eq!(
        asked.into_inner().expect("nobody poisoned it"),
        vec![0o700, 0o600],
        "the folder first, then the file"
    );
    let _ = std::fs::remove_dir_all(path.parent().expect("has a folder"));
}

#[test]
fn a_log_that_grew_too_much_is_set_aside_not_thrown_away() {
    let path = aside("rotation");
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).expect("a folder to write in");
    }
    std::fs::write(&path, vec![b'x'; (UP_TO + 1) as usize]).expect("a log that is too big");
    note_to(&path, "the line that follows the rotation", &nothing);
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
fn a_log_under_the_ceiling_is_left_where_it_is() {
    let path = aside("no-rotation");
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).expect("a folder to write in");
    }
    std::fs::write(&path, b"short enough").expect("a small log");
    assert!(!rotate(&path, UP_TO));
    assert!(!path.with_extension("log.1").exists());
    let _ = std::fs::remove_dir_all(path.parent().expect("has a folder"));
}

#[test]
fn a_log_that_is_not_there_yet_is_not_rotated() {
    let path = aside("missing");
    assert!(!rotate(&path, UP_TO));
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

#[test]
fn a_panic_is_written_with_the_place_when_there_is_one() {
    assert_eq!(
        panic_line(Some(("src/here.rs", 42)), "it broke"),
        "panic at src/here.rs:42: it broke"
    );
    assert_eq!(
        panic_line(None, "it broke"),
        "a panic that could not be placed: it broke"
    );
}

#[test]
fn a_line_is_one_line_however_long_what_it_says_is() {
    let said = line_of("a message");
    assert_eq!(said.lines().count(), 1, "{said}");
    assert!(said.ends_with("a message"), "{said}");
}

#[test]
fn what_is_told_goes_out_as_one_line_and_is_flushed() {
    let mut out: Vec<u8> = Vec::new();
    said_to(&mut out, "the panel is talking");
    assert_eq!(
        String::from_utf8_lossy(&out),
        "the panel is talking
"
    );
}

#[test]
fn telling_twice_keeps_both_lines_in_order() {
    let mut out: Vec<u8> = Vec::new();
    said_to(&mut out, "first");
    said_to(&mut out, "second");
    assert_eq!(
        String::from_utf8_lossy(&out),
        "first
second
"
    );
}
