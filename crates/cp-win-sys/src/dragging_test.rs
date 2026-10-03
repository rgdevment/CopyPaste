use super::*;

#[test]
fn nothing_to_drag_never_reaches_the_shell() {
    assert_eq!(from_window(0, &[]), Dragged::Nothing);
}

#[test]
fn a_path_that_is_not_there_yields_no_item_to_drag() {
    let _ole = Ole::enter();
    assert!(
        data_object_of(&[Path::new(r"Z:\cp-drag\nothing-is-here.txt")]).is_none(),
        "the shell cannot name what does not exist, and a drag of nothing is not a drag"
    );
}

#[test]
fn a_path_that_is_there_becomes_something_the_shell_can_hand_over() {
    let _ole = Ole::enter();
    let at = std::env::temp_dir().join("cp-drag-real.txt");
    std::fs::write(&at, b"dragged")
        .expect("the temporary file has to exist for this to mean anything");
    assert!(data_object_of(&[at.as_path()]).is_some());
    let _ = std::fs::remove_file(&at);
}
