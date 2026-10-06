use crate::note::note;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[derive(Default)]
pub struct Bound(Mutex<Option<String>>);

#[derive(serde::Serialize)]
pub struct Keys {
    wanted: String,
    bound: bool,
}

pub fn raise<R: Runtime>(app: &AppHandle<R>, said: &str) {
    app.manage(Bound::default());
    match bind(app, said) {
        Ok(()) => note(&format!("the «{said}» shortcut answers")),
        Err(why) => note(&format!("no shortcut for the panel: {why}")),
    }
}

pub fn bind<R: Runtime>(app: &AppHandle<R>, said: &str) -> Result<(), String> {
    let wanted: Shortcut = said
        .parse()
        .map_err(|_| format!("«{said}» is not a combination the system understands"))?;
    let keys = app.global_shortcut();
    let _ = keys.unregister_all();
    remember(app, None);
    let handle = app.clone();
    keys.on_shortcut(wanted, move |_app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            crate::panel::show(&handle);
        }
    })
    .map_err(|why| format!("the system would not give up «{said}»: {why}"))?;
    remember(app, Some(said));
    Ok(())
}

fn remember<R: Runtime>(app: &AppHandle<R>, said: Option<&str>) {
    if let Some(state) = app.try_state::<Bound>()
        && let Ok(mut held) = state.0.lock()
    {
        *held = said.map(str::to_owned);
    }
}

#[cfg(target_os = "macos")]
pub const SPARE: &[&str] = &[
    "Ctrl+Alt+V",
    "Shift+Cmd+V",
    "Ctrl+Shift+V",
    "Shift+Cmd+Space",
];
#[cfg(not(target_os = "macos"))]
pub const SPARE: &[&str] = &[
    "Ctrl+Alt+V",
    "Ctrl+Shift+V",
    "Ctrl+Alt+C",
    "Ctrl+Shift+Space",
    "Alt+Shift+V",
    "Ctrl+Alt+Space",
];

fn grantable<R: Runtime>(app: &AppHandle<R>, one: Shortcut) -> bool {
    let keys = app.global_shortcut();
    if keys.is_registered(one) {
        return false;
    }
    let handle = app.clone();
    if keys
        .on_shortcut(one, move |_app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                crate::panel::show(&handle);
            }
        })
        .is_err()
    {
        return false;
    }
    if let Err(why) = keys.unregister(one) {
        note(&format!(
            "«{one:?}» stayed registered after the probe: {why}"
        ));
    }
    true
}

#[tauri::command]
pub fn spare(app: tauri::AppHandle, taken: String) -> Vec<String> {
    let held = taken.parse::<Shortcut>().ok();
    SPARE
        .iter()
        .filter_map(|said| said.parse::<Shortcut>().ok().map(|one| (said, one)))
        .filter(|(_, one)| held != Some(*one))
        .filter(|(_, one)| grantable(&app, *one))
        .map(|(said, _)| (*said).to_owned())
        .collect()
}

#[tauri::command]
pub fn keys(app: tauri::AppHandle) -> Keys {
    let wanted = crate::settings::settings()
        .map(|one| one.shortcut)
        .unwrap_or_else(|_| cp_config::SHORTCUT.to_owned());
    let bound = app
        .try_state::<Bound>()
        .and_then(|state| state.0.lock().ok().map(|held| held.is_some()))
        .unwrap_or(false);
    Keys { wanted, bound }
}

#[cfg(test)]
#[path = "keys_test.rs"]
mod tests;
