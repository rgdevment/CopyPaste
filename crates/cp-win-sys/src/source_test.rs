use super::*;

#[test]
fn the_name_is_what_a_person_recognises() {
    assert_eq!(
        stem_of(r"C:\Program Files\Google\Chrome\chrome.exe"),
        "chrome"
    );
    assert_eq!(stem_of(r"C:\Windows\explorer.exe"), "explorer");
    assert_eq!(stem_of("WINWORD.EXE"), "WINWORD");
}

#[test]
fn what_is_in_front_is_never_ourselves() {
    if let Some(named) = in_front() {
        assert!(!named.is_empty());
        assert_ne!(Some(named), name_of(std::process::id()));
    }
}

#[test]
fn a_name_without_a_folder_or_an_extension_still_works() {
    assert_eq!(stem_of("notepad"), "notepad");
    assert_eq!(stem_of(""), "");
}

#[test]
fn a_folder_with_a_dot_does_not_eat_the_name() {
    assert_eq!(stem_of(r"C:\apps\v1.2\editor.exe"), "editor");
    assert_eq!(stem_of(r"C:\apps\v1.2\editor"), "editor");
}

#[test]
fn both_separators_are_understood() {
    assert_eq!(stem_of("C:/Windows/System32/cmd.exe"), "cmd");
}

#[test]
fn this_very_process_can_be_named() {
    let mine = std::process::id();
    assert!(name_of(mine).is_some());
}
