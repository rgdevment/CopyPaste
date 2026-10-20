use crate::note::note;
use cp_config::Config;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Role {
    #[default]
    Panel,
    Plain,
    Forms,
}

impl Role {
    pub const ALL: [Role; 3] = [Role::Panel, Role::Plain, Role::Forms];

    pub fn of(self, config: &Config) -> &str {
        match self {
            Role::Panel => &config.shortcut,
            Role::Plain => &config.plain_shortcut,
            Role::Forms => &config.forms_shortcut,
        }
    }

    pub fn set(self, config: &mut Config, said: String) {
        match self {
            Role::Panel => config.shortcut = said,
            Role::Plain => config.plain_shortcut = said,
            Role::Forms => config.forms_shortcut = said,
        }
    }

    pub fn fallback(self) -> &'static str {
        match self {
            Role::Panel => cp_config::SHORTCUT,
            Role::Plain => cp_config::PLAIN_SHORTCUT,
            Role::Forms => cp_config::FORMS_SHORTCUT,
        }
    }

    fn spare(self) -> &'static [&'static str] {
        match self {
            Role::Panel => SPARE,
            Role::Plain => PLAIN_SPARE,
            Role::Forms => FORMS_SPARE,
        }
    }

    fn answer<R: Runtime>(self, app: &AppHandle<R>) {
        match self {
            Role::Panel => crate::panel::show(app),
            Role::Plain => crate::panel::paste_plain(app),
            Role::Forms => crate::panel::choose(app),
        }
    }

    fn named(self) -> &'static str {
        match self {
            Role::Panel => "the panel",
            Role::Plain => "pasting plain",
            Role::Forms => "pasting as",
        }
    }
}

#[derive(Default)]
pub struct Bound(Mutex<HashMap<Role, String>>);

#[derive(serde::Serialize)]
pub struct Keys {
    wanted: String,
    bound: bool,
}

pub fn raise<R: Runtime>(app: &AppHandle<R>, config: &Config) {
    app.manage(Bound::default());
    for role in Role::ALL {
        let said = role.of(config);
        if said.is_empty() {
            continue;
        }
        match bind(app, role, said) {
            Ok(()) => note(&format!(
                "the «{said}» shortcut answers for {}",
                role.named()
            )),
            Err(why) => note(&format!("no shortcut for {}: {why}", role.named())),
        }
    }
}

pub fn release<R: Runtime>(app: &AppHandle<R>, role: Role) {
    if let Some(one) = held(app, role).and_then(|said| said.parse::<Shortcut>().ok())
        && let Err(why) = app.global_shortcut().unregister(one)
    {
        note(&format!(
            "«{one:?}» stayed registered for {}: {why}",
            role.named()
        ));
    }
    remember(app, role, None);
}

pub fn bind<R: Runtime>(app: &AppHandle<R>, role: Role, said: &str) -> Result<(), String> {
    release(app, role);
    if said.is_empty() {
        return Ok(());
    }
    let wanted: Shortcut = said
        .parse()
        .map_err(|_| format!("«{said}» is not a combination the system understands"))?;
    let handle = app.clone();
    app.global_shortcut()
        .on_shortcut(wanted, move |_app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                role.answer(&handle);
            }
        })
        .map_err(|why| format!("the system would not give up «{said}»: {why}"))?;
    remember(app, role, Some(said));
    Ok(())
}

fn held<R: Runtime>(app: &AppHandle<R>, role: Role) -> Option<String> {
    app.try_state::<Bound>().and_then(|state| {
        state
            .0
            .lock()
            .ok()
            .and_then(|held| held.get(&role).cloned())
    })
}

fn remember<R: Runtime>(app: &AppHandle<R>, role: Role, said: Option<&str>) {
    if let Some(state) = app.try_state::<Bound>()
        && let Ok(mut held) = state.0.lock()
    {
        match said {
            Some(said) => held.insert(role, said.to_owned()),
            None => held.remove(&role),
        };
    }
}

#[cfg(target_os = "macos")]
pub const SPARE: &[&str] = &["Ctrl+Alt+V", "Ctrl+Cmd+V", "Ctrl+Shift+V"];
#[cfg(not(target_os = "macos"))]
pub const SPARE: &[&str] = &["Ctrl+Alt+V", "Alt+Shift+V", "Ctrl+Alt+Shift+V"];

#[cfg(target_os = "macos")]
pub const PLAIN_SPARE: &[&str] = &["Ctrl+Shift+V", "Ctrl+Cmd+V", "Ctrl+Alt+B"];
#[cfg(not(target_os = "macos"))]
pub const PLAIN_SPARE: &[&str] = &["Alt+Shift+V", "Ctrl+Alt+Shift+V", "Ctrl+Alt+B"];

#[cfg(target_os = "macos")]
pub const FORMS_SPARE: &[&str] = &["Ctrl+Alt+P", "Ctrl+Cmd+P", "Ctrl+Shift+P"];
#[cfg(not(target_os = "macos"))]
pub const FORMS_SPARE: &[&str] = &["Ctrl+Alt+P", "Alt+Shift+P", "Ctrl+Alt+Shift+P"];

fn grantable<R: Runtime>(app: &AppHandle<R>, one: Shortcut) -> bool {
    let keys = app.global_shortcut();
    if keys.is_registered(one) {
        return false;
    }
    if keys.on_shortcut(one, |_app, _shortcut, _event| {}).is_err() {
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
pub fn spare(app: tauri::AppHandle, taken: String, role: Option<Role>) -> Vec<String> {
    let held = taken.parse::<Shortcut>().ok();
    role.unwrap_or_default()
        .spare()
        .iter()
        .filter_map(|said| said.parse::<Shortcut>().ok().map(|one| (said, one)))
        .filter(|(_, one)| held != Some(*one))
        .filter(|(_, one)| grantable(&app, *one))
        .map(|(said, _)| (*said).to_owned())
        .collect()
}

#[tauri::command]
pub fn keys(app: tauri::AppHandle, role: Option<Role>) -> Keys {
    let role = role.unwrap_or_default();
    let wanted = crate::settings::settings()
        .map(|one| role.of(&one).to_owned())
        .unwrap_or_else(|_| role.fallback().to_owned());
    let bound = held(&app, role).is_some();
    Keys { wanted, bound }
}

#[cfg(test)]
#[path = "keys_test.rs"]
mod tests;
