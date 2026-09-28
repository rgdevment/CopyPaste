use std::path::PathBuf;

#[derive(serde::Serialize)]
pub struct Saved {
    path: String,
    items: i64,
    missing: i64,
    bytes: u64,
}

#[derive(serde::Serialize)]
pub struct Brought {
    added: i64,
    already: i64,
    refused: i64,
    #[serde(rename = "fromElsewhere")]
    from_elsewhere: bool,
}

fn history() -> Result<PathBuf, String> {
    Ok(crate::settings::folder()
        .ok_or_else(crate::settings::nowhere)?
        .join("history.db"))
}

#[tauri::command(async)]
pub fn save_backup(path: String, at: i64) -> Result<Saved, String> {
    let store = cp_store::Store::open(&history()?).map_err(|why| why.to_string())?;
    let to = PathBuf::from(&path);
    let made = cp_store::backup::write(&store, &to, at).map_err(|why| why.to_string())?;
    crate::note::note(&format!("a copy written to {path}: {} items", made.items));
    Ok(Saved {
        path,
        items: made.items,
        missing: made.missing,
        bytes: made.bytes,
    })
}

#[tauri::command(async)]
pub fn peek_backup(path: String) -> Result<Saved, String> {
    let taken = cp_store::backup::read(&PathBuf::from(&path)).map_err(|why| why.to_string())?;
    Ok(Saved {
        path,
        items: taken.items,
        missing: 0,
        bytes: taken.bytes,
    })
}

#[tauri::command(async)]
pub fn load_backup(app: tauri::AppHandle, path: String, at: i64) -> Result<Brought, String> {
    let from = PathBuf::from(&path);
    cp_store::backup::read(&from).map_err(|why| why.to_string())?;
    crate::panel::quit(&app);
    let landed = bring(&from, at);
    if let Err(why) = crate::panel::relight(&app) {
        crate::note::note(&format!(
            "the panel did not come back after the copy: {why}"
        ));
    }
    let brought = landed?;
    crate::note::note(&format!(
        "{} items came from {path} and {} were already here",
        brought.added, brought.already
    ));
    Ok(Brought {
        added: brought.added,
        already: brought.already,
        refused: brought.refused,
        from_elsewhere: brought.from_elsewhere,
    })
}

fn bring(from: &std::path::Path, at: i64) -> Result<cp_store::Brought, String> {
    let store = cp_store::Store::open(&history()?).map_err(|why| why.to_string())?;
    cp_store::backup::bring(from, &store, at).map_err(|why| why.to_string())
}

#[cfg(test)]
mod tests {
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
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Crossed {
    added: i64,
    already: i64,
    refused: i64,
    without_their_picture: i64,
}

#[tauri::command(async)]
pub fn bring_former(app: tauri::AppHandle, at: i64) -> Result<Crossed, String> {
    let former = crate::settings::folder()
        .ok_or_else(crate::settings::nowhere)?
        .join("clipboard.db");
    if !former.exists() {
        return Err("there is no CopyPaste 2 history here".to_owned());
    }
    crate::panel::quit(&app);
    let landed = crossed(&former, at);
    if let Err(why) = crate::panel::relight(&app) {
        crate::note::note(&format!("the panel did not come back: {why}"));
    }
    let brought = landed?;
    crate::note::note(&format!(
        "from CopyPaste 2: {} brought in, {} were already here",
        brought.added, brought.already
    ));
    Ok(Crossed {
        added: brought.added,
        already: brought.already,
        refused: brought.refused,
        without_their_picture: brought.without_their_picture,
    })
}

fn crossed(from: &std::path::Path, at: i64) -> Result<cp_store::legacy::Brought, String> {
    let store = cp_store::Store::open(&history()?).map_err(|why| why.to_string())?;
    cp_store::legacy::bring(from, &store, at).map_err(|why| why.to_string())
}

#[derive(serde::Serialize)]
pub struct Swept {
    files: i64,
    bytes: u64,
}

#[tauri::command(async)]
pub fn drop_former() -> Result<Swept, String> {
    let dir = crate::settings::folder().ok_or_else(crate::settings::nowhere)?;
    let swept = cp_store::legacy::drop_former(&dir).map_err(|why| why.to_string())?;
    crate::note::note(&format!(
        "CopyPaste 2 is gone from this machine: {} files",
        swept.files
    ));
    Ok(Swept {
        files: swept.files,
        bytes: swept.bytes,
    })
}
