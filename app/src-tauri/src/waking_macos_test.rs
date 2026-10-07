use super::there::{legacy_in, retire};
use std::path::Path;

#[test]
fn the_old_hand_written_agent_lives_where_the_old_version_put_it() {
    assert_eq!(
        legacy_in(Path::new("/Users/alguien")),
        Path::new("/Users/alguien/Library/LaunchAgents/com.rgdevment.copypaste.plist")
    );
}

#[test]
fn retiring_the_old_agent_deletes_it_and_says_so_only_once() {
    let plist = std::env::temp_dir().join(format!("cp-legacy-{}.plist", std::process::id()));
    std::fs::write(&plist, "<plist/>").expect("write");
    assert!(retire(&plist).expect("first"));
    assert!(!plist.exists());
    assert!(!retire(&plist).expect("second"));
}

#[test]
fn an_agent_that_cannot_be_removed_is_an_error_not_a_silent_success() {
    let folder = std::env::temp_dir();
    assert!(retire(&folder).is_err());
}
