use tauri::{AppHandle, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub fn bind<R: Runtime>(app: &AppHandle<R>, said: &str) -> Result<(), String> {
    let wanted: Shortcut = said
        .parse()
        .map_err(|_| format!("«{said}» no es una combinación que el sistema entienda"))?;
    let keys = app.global_shortcut();
    let _ = keys.unregister_all();
    let handle = app.clone();
    keys.on_shortcut(wanted, move |_app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            crate::panel::show(&handle);
        }
    })
    .map_err(|why| format!("el sistema no cedió «{said}»: {why}"))
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
}
