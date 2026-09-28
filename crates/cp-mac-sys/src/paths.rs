use std::path::PathBuf;

pub fn data_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("CopyPaste"),
    )
}

pub fn database() -> Option<PathBuf> {
    Some(data_dir()?.join("history.db"))
}

pub fn legacy_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("com.rgdevment.copypaste")
            .join("CopyPaste"),
    )
}

pub fn legacy_database() -> Option<PathBuf> {
    Some(legacy_dir()?.join("clipboard.db"))
}

pub fn blobs_dir() -> Option<PathBuf> {
    Some(data_dir()?.join("blobs"))
}

pub fn thumbs_dir() -> Option<PathBuf> {
    Some(data_dir()?.join("thumbs"))
}

pub fn pastes_dir() -> PathBuf {
    std::env::temp_dir().join("CopyPaste")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn everything_hangs_from_one_folder() {
        let root = data_dir().expect("HOME");
        for path in [
            database().expect("base"),
            blobs_dir().expect("blobs"),
            thumbs_dir().expect("thumbnails"),
        ] {
            assert!(path.starts_with(&root), "{path:?} strayed from {root:?}");
        }
    }

    #[test]
    fn the_2x_kept_its_own_folder_under_the_bundle_id() {
        let ours = data_dir().expect("HOME");
        let theirs = legacy_dir().expect("HOME");
        assert_ne!(
            ours, theirs,
            "Flutter put it under the bundle id, not beside us"
        );
        assert!(
            theirs.ends_with("com.rgdevment.copypaste/CopyPaste"),
            "{theirs:?}"
        );
        assert!(
            !legacy_database().expect("theirs").starts_with(&ours),
            "so nothing of the 2.x lives inside our folder on a Mac"
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
            assert!(!pastes.starts_with(&root), "{pastes:?} is not history");
        }
        assert_eq!(
            pastes.file_name().and_then(|n| n.to_str()),
            Some("CopyPaste")
        );
    }

    #[test]
    fn the_folder_is_the_one_2x_already_uses() {
        let root = data_dir().expect("HOME");
        assert!(
            root.ends_with("Library/Application Support/CopyPaste"),
            "{root:?}"
        );
    }
}
