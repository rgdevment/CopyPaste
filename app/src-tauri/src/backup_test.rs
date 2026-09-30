use super::*;

#[test]
fn the_history_a_backup_speaks_for_is_the_one_the_panel_writes() {
    let Ok(db) = history() else {
        return;
    };
    let dir = crate::settings::folder().expect("a folder");
    assert!(db.starts_with(&dir));
    assert_eq!(
        db.file_name().and_then(|it| it.to_str()),
        Some("history.db")
    );
}
