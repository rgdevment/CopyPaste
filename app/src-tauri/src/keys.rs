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
        Ok(()) => note(&format!("el atajo «{said}» responde")),
        Err(why) => note(&format!("sin atajo del panel: {why}")),
    }
}

pub fn bind<R: Runtime>(app: &AppHandle<R>, said: &str) -> Result<(), String> {
    let wanted: Shortcut = said
        .parse()
        .map_err(|_| format!("«{said}» no es una combinación que el sistema entienda"))?;
    let keys = app.global_shortcut();
    let _ = keys.unregister_all();
    remember(app, None);
    let handle = app.clone();
    keys.on_shortcut(wanted, move |_app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            crate::panel::show(&handle);
        }
    })
    .map_err(|why| format!("el sistema no cedió «{said}»: {why}"))?;
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
mod tests {
    use super::*;

    #[test]
    fn what_the_settings_file_ships_with_is_a_combination_the_system_knows() {
        assert!(cp_config::SHORTCUT.parse::<Shortcut>().is_ok());
    }

    #[test]
    fn a_combination_nobody_could_press_is_refused_before_the_system_sees_it() {
        assert!("".parse::<Shortcut>().is_err());
        assert!("Ctrl+".parse::<Shortcut>().is_err());
    }

    #[test]
    fn nothing_is_bound_until_the_system_says_yes() {
        let bound = Bound::default();
        assert!(bound.0.lock().expect("sin envenenar").is_none());
    }
}
