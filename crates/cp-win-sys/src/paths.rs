use std::path::PathBuf;

pub fn data_dir() -> Option<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA")?;
    Some(PathBuf::from(local).join("CopyPaste"))
}

pub fn database() -> Option<PathBuf> {
    Some(data_dir()?.join("history.db"))
}

pub fn legacy_database() -> Option<PathBuf> {
    Some(data_dir()?.join("clipboard.db"))
}

pub fn thumbs_dir() -> Option<PathBuf> {
    Some(data_dir()?.join("thumbs"))
}

pub fn blobs_dir() -> Option<PathBuf> {
    Some(data_dir()?.join("blobs"))
}

pub fn pastes_dir() -> PathBuf {
    std::env::temp_dir().join("CopyPaste")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn everything_hangs_from_one_folder() {
        let root = data_dir().expect("LOCALAPPDATA");
        for path in [
            database().expect("base"),
            legacy_database().expect("base vieja"),
            blobs_dir().expect("blobs"),
            thumbs_dir().expect("miniaturas"),
        ] {
            assert!(path.starts_with(&root), "{path:?} se salió de {root:?}");
        }
    }

    #[test]
    fn the_data_folder_is_local_and_never_roaming() {
        let root = data_dir().expect("LOCALAPPDATA");
        let text = root.to_string_lossy().to_ascii_lowercase();
        assert!(text.contains("local"), "{text}");
        assert!(
            !text.contains("roaming"),
            "un perfil itinerante subiría el historial a la red: {text}"
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
}
