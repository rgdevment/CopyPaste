use super::*;

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
fn the_tray_keeps_its_own_log_apart_from_the_panel() {
    let path = where_to();
    assert_ne!(
        path.file_name().and_then(|it| it.to_str()),
        Some("cp-panel.log"),
        "two processes writing the same file would interleave their lines"
    );
    if crate::settings::folder().is_some() {
        assert_eq!(
            path.parent().and_then(|dir| dir.file_name()),
            Some(std::ffi::OsStr::new("logs"))
        );
    }
}
