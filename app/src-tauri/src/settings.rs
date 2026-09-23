use cp_config::Config;
use std::path::PathBuf;

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

fn nowhere() -> String {
    "no se encontró la carpeta donde CopyPaste guarda lo suyo".to_owned()
}

#[tauri::command]
pub fn settings() -> Result<Config, String> {
    let dir = folder().ok_or_else(nowhere)?;
    cp_config::read(&cp_config::at(&dir)).map_err(|why| why.to_string())
}

#[tauri::command]
pub fn keep(config: Config) -> Result<Config, String> {
    let dir = folder().ok_or_else(nowhere)?;
    let path = cp_config::at(&dir);
    cp_config::write(&path, &config).map_err(|why| why.to_string())?;
    cp_config::read(&path).map_err(|why| why.to_string())
}

#[tauri::command]
pub fn where_it_lives() -> Result<String, String> {
    let dir = folder().ok_or_else(nowhere)?;
    Ok(dir.to_string_lossy().into_owned())
}

#[derive(serde::Serialize)]
pub struct Former {
    path: String,
    bytes: u64,
}

#[tauri::command]
pub fn former() -> Option<Former> {
    let db = folder()?.join("clipboard.db");
    let weighed = std::fs::metadata(&db).ok()?.len();
    Some(Former {
        path: db.to_string_lossy().into_owned(),
        bytes: weighed,
    })
}
