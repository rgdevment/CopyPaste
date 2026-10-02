use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::Emitter;

#[derive(Default)]
pub struct Crossing(AtomicBool);

pub struct Alone<'a>(&'a AtomicBool);

impl Crossing {
    fn claim(&self) -> Option<Alone<'_>> {
        self.0
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
            .then(|| Alone(&self.0))
    }

    pub fn underway(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

impl Drop for Alone<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

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
pub fn load_backup(
    app: tauri::AppHandle,
    alone: tauri::State<'_, Crossing>,
    path: String,
    at: i64,
) -> Result<Brought, String> {
    let _busy = alone
        .inner()
        .claim()
        .ok_or_else(|| "the history is already being written to".to_owned())?;
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
    cp_store::backup::bring(from, &store, at, &|what| crate::note::note(what))
        .map_err(|why| why.to_string())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Crossed {
    added: i64,
    already: i64,
    refused: i64,
    without_their_picture: i64,
    swept: i64,
    crowded: i64,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Underway {
    done: i64,
    total: i64,
}

#[tauri::command(async)]
pub fn bring_former(
    app: tauri::AppHandle,
    alone: tauri::State<'_, Crossing>,
    at: i64,
) -> Result<Crossed, String> {
    let _busy = alone
        .inner()
        .claim()
        .ok_or_else(|| "the CopyPaste 2 history is already being brought over".to_owned())?;
    let former = crate::settings::former_folder()
        .ok_or_else(crate::settings::nowhere)?
        .join("clipboard.db");
    if !former.exists() {
        return Err("there is no CopyPaste 2 history here".to_owned());
    }
    crate::panel::quit(&app);
    let landed = crossed(&app, &former, at);
    if let Err(why) = crate::panel::relight(&app) {
        crate::note::note(&format!("the panel did not come back: {why}"));
    }
    let _ = app.emit("crossed", ());
    let (brought, swept) = landed?;
    crate::note::note(&format!(
        "from CopyPaste 2: {} brought in, {} were already here, {} went by the kept time and {} for room",
        brought.added, brought.already, swept.expired, swept.over_bytes
    ));
    Ok(Crossed {
        added: brought.added,
        already: brought.already,
        refused: brought.refused,
        without_their_picture: brought.without_their_picture,
        swept: swept.expired as i64,
        crowded: swept.over_bytes as i64,
    })
}

fn crossed(
    app: &tauri::AppHandle,
    from: &std::path::Path,
    at: i64,
) -> Result<(cp_store::legacy::Brought, cp_store::Swept), String> {
    let store = cp_store::Store::open(&history()?).map_err(|why| why.to_string())?;
    let policy = crate::settings::policy();
    let swept = |why: &str| match store.sweep(&policy, at) {
        Ok(swept) => swept,
        Err(trouble) => {
            crate::note::note(&format!(
                "what is kept could not be applied {why}: {trouble}"
            ));
            cp_store::Swept::default()
        }
    };
    swept("before");
    let telling = app.clone();
    let brought = cp_store::legacy::bring_telling(from, &store, at, &move |done, total| {
        let _ = telling.emit("crossing", Underway { done, total });
    })
    .map_err(|why| why.to_string())?;
    Ok((brought, swept("after")))
}

#[derive(serde::Serialize)]
pub struct Swept {
    files: i64,
    bytes: u64,
}

#[tauri::command(async)]
pub fn drop_former() -> Result<Swept, String> {
    let dir = crate::settings::former_folder().ok_or_else(crate::settings::nowhere)?;
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

#[cfg(test)]
#[path = "backup_test.rs"]
mod tests;
