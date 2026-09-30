use super::*;

#[test]
fn paste_is_command_v_and_nothing_else() {
    assert!(is_paste_shortcut("v", 0));
    assert!(is_paste_shortcut("V", 0));
    assert!(
        !is_paste_shortcut("v", 1),
        "⇧⌘V is «paste with the same style»"
    );
    assert!(!is_paste_shortcut("v", 2), "⌥⌘V is something else");
    assert!(!is_paste_shortcut("c", 0));
    assert!(!is_paste_shortcut("", 0));
}

#[test]
fn a_process_that_does_not_exist_has_no_menu_bar() {
    assert_eq!(press_paste(i32::MAX), Err(MenuFailure::NoMenuBar));
}
