use std::path::PathBuf;

#[derive(serde::Serialize)]
pub struct Kept {
    path: String,
    items: i64,
    bytes: u64,
}

#[derive(serde::Serialize)]
pub struct Brought {
    added: i64,
    already: i64,
}

fn nowhere() -> String {
    "no se encontró la carpeta donde CopyPaste guarda lo suyo".to_owned()
}

fn history() -> Result<PathBuf, String> {
    Ok(crate::settings::folder()
        .ok_or_else(nowhere)?
        .join("history.db"))
}

#[tauri::command]
pub fn save_backup(path: String, at: i64) -> Result<Kept, String> {
    let store = cp_store::Store::open(&history()?).map_err(|why| why.to_string())?;
    let to = PathBuf::from(&path);
    let made = cp_store::backup::write(&store, &to, at).map_err(|why| why.to_string())?;
    crate::note::note(&format!(
        "copia escrita en {path}: {} elementos",
        made.items
    ));
    Ok(Kept {
        path,
        items: made.items,
        bytes: made.bytes,
    })
}

#[tauri::command]
pub fn peek_backup(path: String) -> Result<Kept, String> {
    let taken = cp_store::backup::read(&PathBuf::from(&path)).map_err(|why| why.to_string())?;
    Ok(Kept {
        path,
        items: taken.items,
        bytes: taken.bytes,
    })
}

#[tauri::command]
pub fn load_backup(app: tauri::AppHandle, path: String, at: i64) -> Result<Brought, String> {
    let from = PathBuf::from(&path);
    cp_store::backup::read(&from).map_err(|why| why.to_string())?;
    crate::panel::quit(&app);
    let landed = bring(&from, at);
    if let Err(why) = crate::panel::relight(&app) {
        crate::note::note(&format!("el panel no volvió tras la copia: {why}"));
    }
    let brought = landed?;
    crate::note::note(&format!(
        "de {path} llegaron {} elementos y {} ya estaban",
        brought.added, brought.already
    ));
    Ok(Brought {
        added: brought.added,
        already: brought.already,
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
        let dir = crate::settings::folder().expect("carpeta");
        assert!(db.starts_with(&dir));
        assert_eq!(
            db.file_name().and_then(|it| it.to_str()),
            Some("history.db")
        );
    }
}
