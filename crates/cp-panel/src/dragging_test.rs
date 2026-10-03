use super::*;

const A_PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 1, 2, 3];

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
    assert!(files_for(&said, None, std::time::SystemTime::now()).is_empty());
}

#[test]
fn a_file_that_is_there_is_dragged_as_itself_and_never_copied() {
    let dir = std::env::temp_dir().join("cp-drag-itself");
    std::fs::create_dir_all(&dir).expect("the directory has to exist for this to mean anything");
    let at = dir.join("already-named.txt");
    std::fs::write(&at, b"dragged").expect("the file has to exist for this to mean anything");
    let said = vec![at.to_string_lossy().into_owned()];
    assert_eq!(
        files_for(&said, Some(A_PNG), std::time::SystemTime::now()),
        vec![at.clone()],
        "a card that already has a file on disk is dragged as that file, bytes and name and all"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_image_with_no_file_is_written_out_once_and_dragged_from_there() {
    let now = std::time::SystemTime::now();
    let first = files_for(&[], Some(A_PNG), now);
    let again = files_for(&[], Some(A_PNG), now);
    assert_eq!(first.len(), 1);
    assert_eq!(first, again, "the same bytes land on the same file");
    assert_eq!(
        first[0].extension().and_then(|one| one.to_str()),
        Some("png"),
        "the extension comes from the bytes, so whatever receives it knows what it got"
    );
    for one in first {
        let _ = std::fs::remove_file(one);
    }
}

#[test]
fn bytes_that_are_no_image_are_not_written_anywhere() {
    assert!(
        files_for(
            &[],
            Some(b"this is not an image"),
            std::time::SystemTime::now()
        )
        .is_empty(),
        "without a known extension nothing receiving the file would know what to do with it"
    );
}
