use cp_config::Config;
use std::path::PathBuf;

pub const NOTICES: &str = include_str!("../../../THIRD-PARTY-BUNDLED.md");

pub const LICENCES: &str = include_str!("../../../THIRD-PARTY-LICENSES.md");

#[tauri::command]
pub fn notices() -> &'static str {
    NOTICES
}

#[tauri::command]
pub fn licences() -> &'static str {
    LICENCES
}

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

pub fn policy() -> cp_store::Policy {
    let Some(path) = folder()
        .map(|dir| cp_config::at(&dir))
        .filter(|at| at.exists())
    else {
        return cp_store::Policy::default();
    };
    let Ok(kept) = cp_config::read(&path) else {
        return cp_store::Policy::default();
    };
    cp_store::Policy::keeping(kept.keeps_days, kept.images_quota_mb)
}

pub fn nowhere() -> String {
    "the folder where CopyPaste keeps its own was not found".to_owned()
}

pub fn settle() -> bool {
    let Some(dir) = folder() else {
        return false;
    };
    let path = cp_config::at(&dir);
    if path.exists() {
        return false;
    }
    match cp_config::write(&path, &Config::default()) {
        Ok(()) => true,
        Err(why) => {
            crate::note::note(&format!("the settings could not be started off: {why}"));
            false
        }
    }
}

#[tauri::command]
pub fn settings() -> Result<Config, String> {
    let dir = folder().ok_or_else(nowhere)?;
    read_or_reset(&cp_config::at(&dir)).map_err(|why| why.to_string())
}

fn read_or_reset(path: &std::path::Path) -> Result<Config, cp_config::Error> {
    let at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_millis() as i64);
    cp_config::read_or_reset(path, at_ms)
}

#[tauri::command]
pub fn looks() -> cp_config::look::Choices {
    cp_config::look::choices_here()
}

#[tauri::command]
pub fn keep(app: tauri::AppHandle, config: Config) -> Result<Config, String> {
    let said = written(&app, config);
    crate::looks::wear(&app);
    if let Ok(now) = settings() {
        let _ = tauri::Emitter::emit(&app, "kept", now);
    }
    said
}

fn written(app: &tauri::AppHandle, config: Config) -> Result<Config, String> {
    let dir = folder().ok_or_else(nowhere)?;
    let path = cp_config::at(&dir);
    let was = read_or_reset(&path).ok();
    let before = was.as_ref().map(|one| one.shortcut.clone());
    let config = Config {
        welcomed: was.and_then(|one| one.welcomed),
        ..config
    };
    cp_config::write(&path, &config).map_err(|why| why.to_string())?;
    let landed = cp_config::read(&path).map_err(|why| why.to_string())?;
    if before.as_deref() == Some(landed.shortcut.as_str()) {
        return Ok(landed);
    }
    let Err(why) = crate::keys::bind(app, &landed.shortcut) else {
        return Ok(landed);
    };
    crate::note::note(&format!("the new shortcut could not be taken: {why}"));
    let was = before.unwrap_or_else(|| cp_config::SHORTCUT.to_owned());
    let back = Config {
        shortcut: was.clone(),
        ..landed
    };
    let _ = cp_config::write(&path, &back);
    if let Err(twice) = crate::keys::bind(app, &was) {
        crate::note::note(&format!(
            "and the one before it did not come back either: {twice}"
        ));
    }
    Err(why)
}

#[tauri::command(async)]
pub fn storage_used() -> Result<i64, String> {
    let at = folder().ok_or_else(nowhere)?.join("history.db");
    if !at.exists() {
        return Ok(0);
    }
    let store = cp_store::Store::open(&at).map_err(|why| why.to_string())?;
    Ok(store.usage().map_err(|why| why.to_string())?.bytes)
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
    beyond_keep: i64,
    came: Option<i64>,
    came_still: i64,
    came_at: Option<i64>,
    unreadable: Option<String>,
}

fn came_over() -> Option<cp_store::legacy::Came> {
    let at = folder()?.join("history.db");
    for wait in [0u64, 150, 400] {
        if wait > 0 {
            std::thread::sleep(std::time::Duration::from_millis(wait));
        }
        if let Ok(store) = cp_store::Store::open(&at)
            && let Ok(came) = store.came_from_the_former()
        {
            return Some(came);
        }
    }
    crate::note::note("the history was busy, so whether the 2.x ever crossed is unknown");
    None
}

#[tauri::command(async)]
pub fn former(at: i64) -> Result<Option<Former>, String> {
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
        beyond_keep: 0,
        came: None,
        came_still: 0,
        came_at: None,
        unreadable: None,
    };
    if let Some(came) = came_over() {
        former.came = Some(came.count);
        former.came_still = came.still;
        former.came_at = came.when;
    }
    match cp_store::legacy::look(&db, at, policy().keep_for) {
        Ok(looked) => {
            former.items = looked.items;
            former.pictures = looked.pictures;
            former.pictures_gone = looked.pictures_gone;
            former.pinned = looked.pinned;
            former.labelled = looked.labelled;
            former.with_styles = looked.with_styles;
            former.beyond_keep = looked.beyond_keep;
        }
        Err(why) => former.unreadable = Some(why.to_string()),
    }
    Ok(Some(former))
}
