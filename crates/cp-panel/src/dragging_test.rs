use super::*;

fn a_png(tag: u8) -> Vec<u8> {
    vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, tag]
}

fn now() -> SystemTime {
    SystemTime::now()
}

#[test]
fn only_what_another_application_could_receive_as_a_file_can_be_dragged() {
    let path = vec!["/tmp/cp-drag/one.pdf".to_owned()];
    assert!(can_drag(Some(Kind::File), &path));
    assert!(can_drag(Some(Kind::Folder), &path));
    assert!(can_drag(Some(Kind::Video), &path));
    assert!(can_drag(Some(Kind::Audio), &path));
    assert!(
        can_drag(Some(Kind::Image), &[]),
        "an image copied from a browser has no path and is still the thing people drag most"
    );
    assert!(!can_drag(Some(Kind::Link), &path));
    assert!(!can_drag(Some(Kind::Json), &path));
    assert!(!can_drag(Some(Kind::Color), &path));
    assert!(!can_drag(None, &path));
}

#[test]
fn a_file_card_whose_file_is_gone_has_nothing_to_hand_over() {
    let said = vec!["/tmp/cp-drag/was-here-yesterday.pdf".to_owned()];
    assert!(
        !can_drag(Some(Kind::File), &[]),
        "a card with no path at all is not a file card"
    );
    assert!(files_for(Some(Kind::File), &said, None, None, now()).is_empty());
}

#[test]
fn a_file_that_is_there_is_dragged_as_itself_and_never_copied() {
    let dir = std::env::temp_dir().join("cp-drag-itself");
    std::fs::create_dir_all(&dir).expect("the directory has to exist for this to mean anything");
    let at = dir.join("already-named.txt");
    std::fs::write(&at, b"dragged").expect("the file has to exist for this to mean anything");
    let said = vec![at.to_string_lossy().into_owned()];
    assert_eq!(
        files_for(
            Some(Kind::File),
            &said,
            Some(&a_png(1)),
            Some("whatever"),
            now()
        ),
        vec![at.clone()],
        "a card that already has a file on disk is dragged as that file, bytes and name and all"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_name_of_the_card_becomes_the_name_of_the_file() {
    assert_eq!(
        named_as(Some("logo final"), &a_png(2)).as_deref(),
        Some("logo final.png")
    );
}

#[test]
fn a_card_with_no_name_falls_back_to_what_its_bytes_are() {
    let bytes = a_png(3);
    let said = named_as(None, &bytes).expect("png bytes always yield a name");
    assert!(said.ends_with(".png"));
    assert_eq!(
        Some(said),
        crate::opening::named_for(&bytes),
        "with nothing to go on, the content hash is still unique, which is the half that matters"
    );
}

#[test]
fn bytes_that_are_no_image_are_named_by_nobody() {
    assert!(named_as(Some("logo final"), b"this is not an image").is_none());
    assert!(
        files_for(
            Some(Kind::Image),
            &[],
            Some(b"this is not an image"),
            None,
            now()
        )
        .is_empty(),
        "without a known extension nothing receiving the file would know what to do with it"
    );
}

#[test]
fn what_a_file_system_would_refuse_never_reaches_it() {
    assert_eq!(
        fit_for_a_file("the 2024/2025 report").as_deref(),
        Some("the 20242025 report")
    );
    assert_eq!(
        fit_for_a_file("C:\\Windows\\win.ini").as_deref(),
        Some("CWindowswin.ini")
    );
    assert_eq!(
        fit_for_a_file("  spaced   out  ").as_deref(),
        Some("spaced out")
    );
    assert_eq!(
        fit_for_a_file("trailing dots...").as_deref(),
        Some("trailing dots")
    );
    assert_eq!(fit_for_a_file("a\u{0}b\u{7}c").as_deref(), Some("abc"));
    assert_eq!(fit_for_a_file("////").as_deref(), None);
    assert_eq!(fit_for_a_file("   ").as_deref(), None);
    assert_eq!(fit_for_a_file("").as_deref(), None);
}

#[test]
fn the_names_windows_keeps_for_its_devices_are_left_alone() {
    for one in ["CON", "con", "NUL", "com1", "LPT9", "aux"] {
        assert_eq!(
            fit_for_a_file(one).as_deref(),
            None,
            "«{one}» names a device on Windows, and a file called that cannot be written"
        );
    }
    assert_eq!(fit_for_a_file("console").as_deref(), Some("console"));
    assert_eq!(fit_for_a_file("com10").as_deref(), Some("com10"));
}

#[test]
fn a_name_longer_than_a_person_would_type_is_cut_without_breaking_a_letter() {
    let long = "ñ".repeat(NAME_ROOM * 2);
    let said = fit_for_a_file(&long).expect("a long name is still a name");
    assert_eq!(said.chars().count(), NAME_ROOM);
    assert!(
        said.chars().all(|one| one == 'ñ'),
        "a cut must not leave half a letter behind"
    );
}

#[test]
fn two_images_sharing_a_name_never_share_a_file() {
    let same = "logo final";
    let (mine, yours) = (a_png(10), a_png(11));
    let one = files_for(Some(Kind::Image), &[], Some(&mine), Some(same), now());
    let two = files_for(Some(Kind::Image), &[], Some(&yours), Some(same), now());
    assert_eq!(one.len(), 1);
    assert_eq!(two.len(), 1);
    assert_eq!(
        one[0].file_name(),
        two[0].file_name(),
        "both keep the name the card was given"
    );
    assert_ne!(
        one[0], two[0],
        "and each lives under its own content, so neither can hand over the other's bytes"
    );
    assert_eq!(std::fs::read(&one[0]).expect("it was written"), mine);
    assert_eq!(std::fs::read(&two[0]).expect("it was written"), yours);
    for at in [one[0].as_path(), two[0].as_path()] {
        let _ = std::fs::remove_dir_all(at.parent().expect("it lives under its content"));
    }
}

#[test]
fn the_same_image_is_written_once_and_handed_over_again() {
    let at = now();
    let bytes = a_png(20);
    let first = files_for(Some(Kind::Image), &[], Some(&bytes), Some("once"), at);
    let again = files_for(Some(Kind::Image), &[], Some(&bytes), Some("once"), at);
    assert_eq!(first, again);
    for one in first {
        let _ = std::fs::remove_dir_all(one.parent().expect("it lives under its content"));
    }
}

#[test]
fn what_was_dragged_an_hour_ago_is_swept_and_what_was_dragged_now_is_not() {
    let dir = std::env::temp_dir().join("cp-drag-sweep");
    let old = dir.join("0000000000000000");
    std::fs::create_dir_all(&old).expect("the directory has to exist for this to mean anything");
    std::fs::write(old.join("old.png"), a_png(30)).expect("the file has to exist");
    let long_ago = SystemTime::now() + KEPT_FOR + Duration::from_secs(60);
    assert_eq!(sweep_dragged(&dir, long_ago), 1);
    assert!(
        !old.exists(),
        "a folder past its hour goes whole, with what it held"
    );

    let fresh = dir.join("1111111111111111");
    std::fs::create_dir_all(&fresh).expect("the directory has to exist");
    assert_eq!(sweep_dragged(&dir, SystemTime::now()), 0);
    assert!(fresh.exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_name_no_file_system_would_hold_is_cut_to_one_that_fits() {
    let family = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}";
    assert_eq!(family.len(), 25, "one grapheme, twenty five bytes of it");
    let said = fit_for_a_file(&family.repeat(NAME_ROOM)).expect("emoji are a name too");
    assert!(
        said.len() <= BYTES_ROOM,
        "{} bytes: APFS and NTFS both stop at 255 per component, and the extension still has to fit",
        said.len()
    );
    assert!(
        said.len().is_multiple_of(family.len()),
        "the cut falls between graphemes, never inside one"
    );
}

#[test]
fn a_card_whose_kind_is_not_draggable_is_refused_on_the_real_item_too() {
    let dir = std::env::temp_dir().join("cp-drag-kind");
    std::fs::create_dir_all(&dir).expect("the directory has to exist");
    let at = dir.join("looks-like-a-path.txt");
    std::fs::write(&at, b"text").expect("the file has to exist");
    let said = vec![at.to_string_lossy().into_owned()];
    assert!(
        files_for(Some(Kind::Text), &said, None, None, now()).is_empty(),
        "the view gates on a preview string; the handler has the item, and has to agree with it"
    );
    assert_eq!(
        files_for(Some(Kind::File), &said, None, None, now()),
        vec![at.clone()]
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn sweeping_a_folder_that_was_never_made_takes_nothing_and_says_so() {
    let nowhere = std::env::temp_dir().join("cp-drag-never-made-this-one");
    let _ = std::fs::remove_dir_all(&nowhere);
    assert_eq!(sweep_dragged(&nowhere, now()), 0);
}

#[test]
fn a_folder_whose_file_was_used_lately_outlives_the_hour_since_it_was_made() {
    let dir = std::env::temp_dir().join("cp-drag-touched");
    let _ = std::fs::remove_dir_all(&dir);
    let kept = dir.join("2222222222222222");
    std::fs::create_dir_all(&kept).expect("the directory has to exist");
    let file = kept.join("kept.png");
    std::fs::write(&file, a_png(40)).expect("the file has to exist");
    let later = SystemTime::now() + KEPT_FOR + Duration::from_secs(60);
    std::fs::File::options()
        .append(true)
        .open(&file)
        .and_then(|one| one.set_modified(later))
        .expect("the file takes a later date");
    assert_eq!(
        sweep_dragged(&dir, later + Duration::from_secs(60)),
        0,
        "a path still pasted somewhere points at a file used a minute ago, whatever the folder's age"
    );
    assert!(file.exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn handing_the_same_image_over_again_renews_its_hour() {
    let bytes = a_png(21);
    let first = files_for(Some(Kind::Image), &[], Some(&bytes), None, now());
    let later = SystemTime::now() + Duration::from_secs(1800);
    let again = files_for(Some(Kind::Image), &[], Some(&bytes), None, later);
    assert_eq!(first, again);
    let touched = std::fs::metadata(&again[0])
        .and_then(|meta| meta.modified())
        .expect("the file is there");
    assert!(
        touched >= later,
        "reusing the file moves its date to the last use"
    );
    let _ = std::fs::remove_dir_all(again[0].parent().expect("it lives under its content"));
}
