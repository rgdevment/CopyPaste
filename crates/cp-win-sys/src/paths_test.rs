use super::*;

#[test]
fn everything_hangs_from_one_folder() {
    let root = data_dir().expect("LOCALAPPDATA");
    for path in [
        database().expect("base"),
        blobs_dir().expect("blobs"),
        thumbs_dir().expect("miniaturas"),
    ] {
        assert!(path.starts_with(&root), "{path:?} stepped outside {root:?}");
    }
}

#[test]
fn the_data_folder_is_local_and_never_roaming() {
    let root = data_dir().expect("LOCALAPPDATA");
    let text = root.to_string_lossy().to_ascii_lowercase();
    assert!(text.contains("local"), "{text}");
    assert!(
        !text.contains("roaming"),
        "a roaming profile would upload the history to the network: {text}"
    );
}

#[test]
fn the_new_database_never_touches_the_one_from_2x() {
    assert_ne!(database(), legacy_database());
    assert_eq!(
        database().and_then(|p| p.file_name().map(std::ffi::OsString::from)),
        Some("history.db".into())
    );
    assert_eq!(
        legacy_database().and_then(|p| p.file_name().map(std::ffi::OsString::from)),
        Some("clipboard.db".into())
    );
}

#[test]
fn what_is_pasted_out_of_the_history_lands_in_temp_not_in_the_data_folder() {
    let pastes = pastes_dir();
    assert!(pastes.starts_with(std::env::temp_dir()), "{pastes:?}");
    if let Some(root) = data_dir() {
        assert!(!pastes.starts_with(&root), "{pastes:?} no es historial");
    }
    assert_eq!(
        pastes.file_name().and_then(|n| n.to_str()),
        Some("CopyPaste")
    );
}

#[test]
fn the_folder_is_the_one_2x_already_uses() {
    let root = data_dir().expect("LOCALAPPDATA");
    assert_eq!(root.file_name().and_then(|n| n.to_str()), Some("CopyPaste"));
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cp-legacy-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch");
    dir
}

fn packaged_in(local: &std::path::Path) -> PathBuf {
    local
        .join("Packages")
        .join(STORE_FAMILY)
        .join("LocalCache")
        .join("Local")
        .join("CopyPaste")
}

#[test]
fn the_2x_history_is_looked_for_where_the_installer_left_it_first() {
    let local = scratch("plain");
    std::fs::create_dir_all(local.join("CopyPaste")).expect("plain");
    std::fs::write(local.join("CopyPaste").join("clipboard.db"), b"x").expect("db");
    std::fs::create_dir_all(packaged_in(&local)).expect("packaged");
    std::fs::write(packaged_in(&local).join("clipboard.db"), b"x").expect("db");
    assert_eq!(legacy_in(&local), local.join("CopyPaste"));
    let _ = std::fs::remove_dir_all(&local);
}

#[test]
fn a_2x_from_the_store_alone_is_found_inside_its_package() {
    let local = scratch("store");
    std::fs::create_dir_all(packaged_in(&local)).expect("packaged");
    std::fs::write(packaged_in(&local).join("clipboard.db"), b"x").expect("db");
    assert_eq!(legacy_in(&local), packaged_in(&local));
    let _ = std::fs::remove_dir_all(&local);
}

#[test]
fn with_no_2x_anywhere_the_plain_folder_is_the_answer() {
    let local = scratch("none");
    assert_eq!(legacy_in(&local), local.join("CopyPaste"));
    let _ = std::fs::remove_dir_all(&local);
}
