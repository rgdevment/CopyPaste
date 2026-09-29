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
fn the_clock_reads_as_hours_minutes_and_seconds() {
    let said = clock();
    assert_eq!(said.len(), 8);
    assert!(said.chars().filter(|one| *one == ':').count() == 2);
}
