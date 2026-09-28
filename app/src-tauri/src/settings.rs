use cp_config::Config;
use std::path::PathBuf;

pub fn former_folder() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        cp_win_sys::paths::legacy_dir()
    }
    #[cfg(target_os = "macos")]
    {
        cp_mac_sys::paths::legacy_dir()
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        None
    }
}

pub fn folder() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        cp_win_sys::paths::data_dir()
    }
    #[cfg(target_os = "macos")]
    {
        cp_mac_sys::paths::data_dir()
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        None
    }
}

pub fn nowhere() -> String {
    "the folder where CopyPaste keeps its own was not found".to_owned()
}

pub fn settle() {
    let Some(dir) = folder() else {
        return;
    };
    let path = cp_config::at(&dir);
    if path.exists() {
        return;
    }
    if let Err(why) = cp_config::write(&path, &Config::default()) {
        crate::note::note(&format!("the settings could not be started off: {why}"));
    }
}

#[tauri::command]
pub fn settings() -> Result<Config, String> {
    let dir = folder().ok_or_else(nowhere)?;
    cp_config::read(&cp_config::at(&dir)).map_err(|why| why.to_string())
}

#[tauri::command]
pub fn keep(app: tauri::AppHandle, config: Config) -> Result<Config, String> {
    let dir = folder().ok_or_else(nowhere)?;
    let path = cp_config::at(&dir);
    let before = cp_config::read(&path).ok().map(|one| one.shortcut);
    cp_config::write(&path, &config).map_err(|why| why.to_string())?;
    let landed = cp_config::read(&path).map_err(|why| why.to_string())?;
    if before.as_deref() == Some(landed.shortcut.as_str()) {
        return Ok(landed);
    }
    let Err(why) = crate::keys::bind(&app, &landed.shortcut) else {
        return Ok(landed);
    };
    crate::note::note(&format!("the new shortcut could not be taken: {why}"));
    let was = before.unwrap_or_else(|| cp_config::SHORTCUT.to_owned());
    let back = Config {
        shortcut: was.clone(),
        ..landed
    };
    let _ = cp_config::write(&path, &back);
    if let Err(twice) = crate::keys::bind(&app, &was) {
        crate::note::note(&format!(
            "and the one before it did not come back either: {twice}"
        ));
    }
    Err(why)
}

#[tauri::command]
pub fn where_it_lives() -> Result<String, String> {
    let dir = folder().ok_or_else(nowhere)?;
    Ok(dir.to_string_lossy().into_owned())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Former {
    path: String,
    bytes: u64,
    items: i64,
    pictures: i64,
    pictures_gone: i64,
    pinned: i64,
    labelled: i64,
    with_styles: i64,
    unreadable: Option<String>,
}

#[tauri::command]
pub fn former() -> Result<Option<Former>, String> {
    let db = former_folder().ok_or_else(nowhere)?.join("clipboard.db");
    let weighed = match std::fs::metadata(&db) {
        Ok(weighed) => weighed,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(why) => return Err(why.to_string()),
    };
    let mut former = Former {
        path: db.to_string_lossy().into_owned(),
        bytes: weighed.len(),
        items: 0,
        pictures: 0,
        pictures_gone: 0,
        pinned: 0,
        labelled: 0,
        with_styles: 0,
        unreadable: None,
    };
    match cp_store::legacy::look(&db) {
        Ok(looked) => {
            former.items = looked.items;
            former.pictures = looked.pictures;
            former.pictures_gone = looked.pictures_gone;
            former.pinned = looked.pinned;
            former.labelled = looked.labelled;
            former.with_styles = looked.with_styles;
        }
        Err(why) => former.unreadable = Some(why.to_string()),
    }
    Ok(Some(former))
}
