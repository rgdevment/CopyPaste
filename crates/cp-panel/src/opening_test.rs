use super::*;

fn one(path: &str) -> Vec<String> {
    vec![path.to_owned()]
}

#[test]
fn what_lives_in_a_file_can_be_opened() {
    for kind in [
        Kind::File,
        Kind::Folder,
        Kind::Image,
        Kind::Video,
        Kind::Audio,
    ] {
        assert_eq!(
            doing_for(Some(kind), &one(r"D:\Mario\one.pdf")),
            Doing::Open,
            "{kind:?}"
        );
    }
}

#[test]
fn what_is_only_text_has_nothing_to_open() {
    for kind in [
        Kind::Text,
        Kind::Code,
        Kind::Json,
        Kind::Link,
        Kind::Email,
        Kind::Phone,
        Kind::Color,
        Kind::Ip,
        Kind::Uuid,
        Kind::Token,
    ] {
        assert_eq!(
            doing_for(Some(kind), &one(r"D:\Mario\one.pdf")),
            Doing::Nothing,
            "{kind:?} is not something the system knows how to open"
        );
    }
    assert_eq!(doing_for(None, &one(r"D:\one.pdf")), Doing::Nothing);
}

#[test]
fn an_image_pasted_from_the_clipboard_has_no_path_and_no_button() {
    assert_eq!(doing_for(Some(Kind::Image), &[]), Doing::Nothing);
    assert_eq!(
        doing_for(Some(Kind::Image), &[String::new(), "   ".to_owned()]),
        Doing::Nothing,
        "a card must not offer a button that cannot do anything"
    );
}

#[test]
fn the_first_path_worth_having_is_the_one_that_opens() {
    assert_eq!(first_of(&[]), None);
    assert_eq!(first_of(&[String::new()]), None);
    assert_eq!(
        first_of(&[String::new(), "  ".to_owned(), " D:/two.txt ".to_owned()]),
        Some("D:/two.txt"),
        "the blanks are skipped and what is left is trimmed"
    );
    assert_eq!(
        first_of(&["D:/one.txt".to_owned(), "D:/two.txt".to_owned()]),
        Some("D:/one.txt"),
        "several files were copied at once; the first is the one the card stands for"
    );
}

#[test]
fn the_wait_is_short_enough_that_a_click_does_not_feel_stuck() {
    assert!(
        PATIENCE <= std::time::Duration::from_millis(500),
        "the path may live on a drive that is not answering, and the panel cannot freeze on it"
    );
}

#[test]
fn what_is_not_a_path_is_not_offered_as_one() {
    for said in [
        "copypaste",
        "just some words",
        "#FF8800",
        "C:",
        "a:b",
        "{ \"key\": 1 }",
    ] {
        assert!(!looks_like_a_path(said), "«{said}»");
        assert_eq!(
            doing_for(Some(Kind::File), &one(said)),
            Doing::Nothing,
            "«{said}»"
        );
    }
}

#[test]
fn a_path_is_a_path_in_either_direction_and_on_a_share() {
    for said in [
        r"D:\Mario\one.pdf",
        "D:/Mario/one.pdf",
        "/home/mario/one.pdf",
        r"\server\share\one.pdf",
        "C:/",
        "Mario/one.pdf",
    ] {
        assert!(looks_like_a_path(said), "«{said}»");
    }
}

#[test]
fn the_bytes_say_which_viewer_will_know_what_to_do() {
    assert_eq!(extension_of(&[0x89, b'P', b'N', b'G', 13, 10]), Some("png"));
    assert_eq!(extension_of(&[0xFF, 0xD8, 0xFF, 0xE0]), Some("jpg"));
    assert_eq!(extension_of(b"GIF89a....."), Some("gif"));
    assert_eq!(extension_of(b"BM......"), Some("bmp"));
    assert_eq!(extension_of(b"RIFF\0\0\0\0WEBPVP8 "), Some("webp"));
}

#[test]
fn what_no_viewer_would_recognise_is_not_written_out_at_all() {
    for bytes in [
        &b""[..],
        b"not an image",
        b"RIFF\0\0\0\0WAVE",
        &[0x89, b'P'][..],
    ] {
        assert_eq!(extension_of(bytes), None, "{bytes:?}");
    }
}

#[test]
fn a_capture_is_written_once_and_found_again() {
    let dir = std::env::temp_dir().join(format!("cp-seen-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let bytes = [0x89, b'P', b'N', b'G', 1, 2, 3, 4];
    let first = spilled(&dir, &bytes).expect("written");
    assert_eq!(
        first.extension().and_then(|one| one.to_str()),
        Some("png"),
        "the viewer picks its application by the extension"
    );
    assert_eq!(std::fs::read(&first).expect("read"), bytes);
    let again = spilled(&dir, &bytes).expect("found");
    assert_eq!(again, first, "the same capture is not written twice");
    assert_eq!(spilled(&dir, b"not an image"), None);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_capture_with_no_file_of_its_own_can_still_be_opened() {
    assert!(
        can_open(Some(Kind::Image), &[]),
        "a screenshot has no path, and is still the thing a viewer is for"
    );
    assert!(!can_open(Some(Kind::Text), &[]));
    assert!(can_open(Some(Kind::Video), &one(r"D:\one.mp4")));
    assert!(
        !can_open(Some(Kind::Video), &[]),
        "a video without a file is nothing a viewer could be handed"
    );
}

#[test]
fn only_the_web_is_handed_to_a_browser() {
    for said in [
        "https://example.com/one",
        "http://example.com",
        "HTTPS://EXAMPLE.COM/x",
        "  https://example.com/two  ",
    ] {
        assert!(a_link_worth_opening(said).is_some(), "«{said}»");
    }
    for said in [
        "file:///C:/Windows/System32",
        "javascript:alert(1)",
        "ftp://example.com",
        "mailto:someone@example.com",
        "https://",
        "example.com",
        "https://example.com one",
        "",
    ] {
        assert_eq!(
            a_link_worth_opening(said),
            None,
            "«{said}» is not something a card should hand to a browser"
        );
    }
}

#[test]
fn a_link_card_offers_to_open_and_a_plain_one_does_not() {
    assert!(can_open_link(Some(Kind::Link), "https://example.com/one"));
    assert!(can_open_link(
        Some(Kind::Link),
        "https://example.com/one\nand a second line"
    ));
    assert!(!can_open_link(Some(Kind::Link), "not a url at all"));
    assert!(!can_open_link(Some(Kind::Link), "file:///C:/secret.txt"));
    assert!(!can_open_link(Some(Kind::Text), "https://example.com/one"));
}

#[test]
fn two_pictures_that_share_an_id_do_not_share_a_preview() {
    let dir = tempfile::tempdir().expect("a folder");
    let one = [b"\x89PNG\r\n\x1a\n".to_vec(), vec![1u8; 40]].concat();
    let other = [b"\x89PNG\r\n\x1a\n".to_vec(), vec![2u8; 40]].concat();
    assert_eq!(one.len(), other.len());
    let first = spilled(dir.path(), &one).expect("written");
    let second = spilled(dir.path(), &other).expect("written");
    assert_ne!(
        first, second,
        "ids come back around when rows are deleted, so naming by id and length would show the \
         picture somebody already deleted"
    );
    assert_eq!(std::fs::read(&first).expect("read"), one);
    assert_eq!(std::fs::read(&second).expect("read"), other);
}

#[test]
fn what_was_spilled_long_ago_is_swept_and_what_is_fresh_stays() {
    let dir = tempfile::tempdir().expect("a folder");
    let bytes = [b"\x89PNG\r\n\x1a\n".to_vec(), vec![3u8; 16]].concat();
    let at = spilled(dir.path(), &bytes).expect("written");
    assert_eq!(
        super::sweep_seen(dir.path(), std::time::SystemTime::now()),
        0
    );
    assert!(at.exists(), "what was just opened is still being looked at");

    let later =
        std::time::SystemTime::now() + super::SEEN_KEPT_FOR + std::time::Duration::from_secs(60);
    assert_eq!(super::sweep_seen(dir.path(), later), 1);
    assert!(
        !at.exists(),
        "clipboard content written out in the clear must not outlive the look at it"
    );
}

#[test]
fn spilling_the_same_picture_twice_writes_it_once() {
    let dir = tempfile::tempdir().expect("a folder");
    let bytes = [b"\x89PNG\r\n\x1a\n".to_vec(), vec![9u8; 24]].concat();
    let at = spilled(dir.path(), &bytes).expect("written");
    let first = std::fs::metadata(&at)
        .expect("there")
        .modified()
        .expect("when");
    std::thread::sleep(std::time::Duration::from_millis(40));
    let again = spilled(dir.path(), &bytes).expect("found");
    assert_eq!(at, again);
    assert_eq!(
        std::fs::metadata(&again)
            .expect("there")
            .modified()
            .expect("when"),
        first,
        "a picture already written out is handed over as it is, not written again"
    );
}

#[test]
fn where_previews_go_is_a_folder_of_ours_inside_the_temporary_one() {
    let at = super::where_previews_go();
    assert!(at.starts_with(std::env::temp_dir()));
    assert!(at.ends_with("seen"));
    assert!(
        at.parent().is_some_and(|one| one.ends_with("CopyPaste")),
        "what is written out in the clear lives under our own name: {at:?}"
    );
}

#[test]
fn what_is_and_is_not_a_path_is_decided_by_the_separator_and_the_drive() {
    assert!(super::looks_like_a_path(r"C:\Users\Mario"));
    assert!(super::looks_like_a_path(r"c:/Users/Mario"));
    assert!(super::looks_like_a_path(r"\\servidor\share"));
    assert!(super::looks_like_a_path(r"/home/mario"));
    assert!(super::looks_like_a_path(r"carpeta/archivo.txt"));
    assert!(!super::looks_like_a_path(r"C:"));
    assert!(!super::looks_like_a_path(r"a:b"));
    assert!(
        super::looks_like_a_path(r"1:\no-es-unidad"),
        "a separator is enough: what is not a drive may still be a relative path"
    );
    assert!(!super::looks_like_a_path("solo texto"));
    assert!(!super::looks_like_a_path(""));
}

#[test]
fn only_a_file_that_is_really_gone_is_marked_as_gone() {
    assert_eq!(landing_of(true, true, true), Landing::Opened);
    assert_eq!(landing_of(false, true, false), Landing::Gone);
    assert_eq!(
        landing_of(false, true, true),
        Landing::Refused,
        "the system refuses to open plenty of files that are right there: no application for the \
         extension, a share violation, SmartScreen. Marking those as gone hides them and the \
         housekeeping deletes them later"
    );
    assert_eq!(
        landing_of(false, false, false),
        Landing::Refused,
        "what was written out to be looked at has no path of its own to be missing from"
    );
}

#[test]
fn sweeping_previews_that_were_never_written_takes_nothing_and_says_so() {
    let nowhere = std::env::temp_dir().join("cp-seen-never-written-here");
    let _ = std::fs::remove_dir_all(&nowhere);
    assert_eq!(sweep_seen(&nowhere, std::time::SystemTime::now()), 0);
}

#[test]
fn what_the_system_said_about_opening_it_is_read_the_same_way_every_time() {
    use cp_core::reading::Waited;
    assert_eq!(answered(Waited::Answered(true)), Reached::Opened);
    assert_eq!(answered(Waited::Answered(false)), Reached::Refused);
    assert_eq!(
        answered(Waited::StillRunning),
        Reached::Working,
        "a viewer that is slow to open has not refused, and the panel waits for it"
    );
    assert_eq!(answered(Waited::Gone), Reached::Refused);
}
