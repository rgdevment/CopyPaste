use super::there::{agent_for, ours, program_in};
use std::path::Path;

#[test]
fn the_agent_names_the_binary_that_wrote_it() {
    let exe = Path::new("/Applications/CopyPaste.app/Contents/MacOS/CopyPaste");
    let written = agent_for(exe);
    assert_eq!(program_in(&written).as_deref(), exe.to_str());
    assert!(written.contains("<key>RunAtLoad</key>"));
    assert!(written.starts_with("<?xml"));
}

#[test]
fn an_entry_that_names_another_program_is_not_ours() {
    let exe = Path::new("/Applications/CopyPaste.app/Contents/MacOS/CopyPaste");
    assert!(ours(
        "/Applications/CopyPaste.app/Contents/MacOS/CopyPaste",
        exe
    ));
    assert!(!ours(
        "/Applications/Otro.app/Contents/MacOS/CopyPaste",
        exe
    ));
    assert!(!ours("", exe));
}

#[test]
fn a_path_the_xml_had_to_escape_comes_back_whole() {
    let exe = Path::new("/Users/alguien/Rock & Roll/CopyPaste.app/Contents/MacOS/CopyPaste");
    let written = agent_for(exe);
    assert!(written.contains("&amp;"), "a raw & breaks the plist");
    assert_eq!(program_in(&written).as_deref(), exe.to_str());
    assert!(ours(&program_in(&written).expect("a path"), exe));
}

#[test]
fn an_agent_pointing_at_a_copy_that_moved_reads_as_off_so_it_can_be_written_again() {
    let moved = Path::new("/Users/quien/Downloads/CopyPaste.app/Contents/MacOS/CopyPaste");
    let now = Path::new("/Applications/CopyPaste.app/Contents/MacOS/CopyPaste");
    let written = agent_for(moved);
    let said = program_in(&written).expect("a path");
    assert!(ours(&said, moved));
    assert!(
        !ours(&said, now),
        "once it moves, the startup entry no longer points at this copy"
    );
}

#[test]
fn a_plist_without_program_arguments_says_nothing() {
    assert_eq!(program_in("<plist><dict></dict></plist>"), None);
    assert_eq!(program_in(""), None);
}
