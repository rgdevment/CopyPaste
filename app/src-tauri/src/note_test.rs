use super::*;

fn aside(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cp-gui-note-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir.join("cp-gui.log")
}

#[test]
fn the_log_sits_where_the_privacy_note_says_it_does() {
    let path = where_to();
    assert_eq!(
        path.file_name().and_then(|it| it.to_str()),
        Some("cp-gui.log")
    );
    if let Some(dir) = crate::settings::folder() {
        assert!(path.starts_with(dir));
    }
}

#[test]
fn every_line_carries_the_day_and_who_wrote_it() {
    let path = aside("shape");
    note_to(&path, "a line worth placing in time");
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
    let _ = std::fs::remove_dir_all(path.parent().expect("has a folder"));
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
