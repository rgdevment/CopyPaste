use cp_config::Config;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

const NEWS: &str = include_str!("../../src/news.json");

pub const LABEL: &str = "welcome";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Greeting {
    Tour { former: bool },
    News { versions: Vec<String> },
    Keys,
    Nothing,
}

#[derive(Default)]
pub struct Now(Mutex<Option<Greeting>>);

#[derive(serde::Deserialize)]
struct Told {
    version: String,
}

pub fn told() -> Vec<String> {
    serde_json::from_str::<Vec<Told>>(NEWS)
        .map(|all| all.into_iter().map(|one| one.version).collect())
        .unwrap_or_default()
}

pub fn decide(
    fresh: bool,
    former: bool,
    welcomed: Option<&str>,
    running: &str,
    told: &[String],
) -> Greeting {
    let since = welcomed.and_then(|one| semver::Version::parse(one).ok());
    let Some(since) = since.filter(|_| !fresh) else {
        return Greeting::Tour { former };
    };
    let Ok(now) = semver::Version::parse(running) else {
        return Greeting::Nothing;
    };
    if since >= now {
        return Greeting::Nothing;
    }
    let mut fresh_news: Vec<(semver::Version, &String)> = told
        .iter()
        .filter_map(|said| semver::Version::parse(said).ok().map(|one| (one, said)))
        .filter(|(one, _)| *one <= now && *one > since)
        .collect();
    fresh_news.sort_by(|a, b| b.0.cmp(&a.0));
    fresh_news.dedup_by(|a, b| a.0 == b.0);
    if fresh_news.is_empty() {
        return Greeting::Nothing;
    }
    Greeting::News {
        versions: fresh_news
            .into_iter()
            .map(|(_, said)| said.clone())
            .collect(),
    }
}

pub fn behind(welcomed: Option<&str>, running: &str) -> bool {
    let Ok(now) = semver::Version::parse(running) else {
        return false;
    };
    welcomed
        .and_then(|one| semver::Version::parse(one).ok())
        .is_none_or(|since| since < now)
}

pub fn raise<R: Runtime>(app: &AppHandle<R>, fresh: bool) {
    let kept = crate::settings::settings().ok();
    if kept.is_none() && !fresh {
        app.manage(Now(Mutex::new(Some(Greeting::Nothing))));
        return;
    }
    let welcomed = kept.as_ref().and_then(|one| one.welcomed.clone());
    let running = app.package_info().version.to_string();
    let former =
        crate::settings::former_folder().is_some_and(|dir| dir.join("clipboard.db").exists());
    let asked = decide(fresh, former, welcomed.as_deref(), &running, &told());
    let quiet = asked == Greeting::Nothing;
    app.manage(Now(Mutex::new(Some(asked))));
    if !quiet {
        open(app);
        return;
    }
    if behind(welcomed.as_deref(), &running) {
        mark(app);
    }
}

fn mark<R: Runtime>(app: &AppHandle<R>) {
    let Some(dir) = crate::settings::folder() else {
        return;
    };
    let path = cp_config::at(&dir);
    let Ok(kept) = cp_config::read(&path) else {
        return;
    };
    let running = app.package_info().version.to_string();
    if !behind(kept.welcomed.as_deref(), &running) {
        return;
    }
    let written = cp_config::write(
        &path,
        &Config {
            welcomed: Some(running),
            ..kept
        },
    );
    if let Err(why) = written {
        crate::note::note(&format!("the welcome could not be written down: {why}"));
    }
}

pub fn open<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        return;
    }
    let built = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html#welcome".into()))
        .title("CopyPaste")
        .inner_size(600.0, 460.0)
        .resizable(false)
        .maximizable(false)
        .decorations(false)
        .center()
        .build();
    match built {
        Ok(window) => {
            let handle = app.clone();
            window.on_window_event(move |event| {
                if matches!(event, tauri::WindowEvent::Destroyed) && !only_keys(&handle) {
                    mark(&handle);
                }
            });
            let _ = window.set_focus();
        }
        Err(why) => crate::note::note(&format!("the welcome would not open: {why}")),
    }
}

fn only_keys<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.try_state::<Now>()
        .and_then(|state| state.0.lock().ok().map(|held| held.clone()))
        .is_some_and(|held| held == Some(Greeting::Keys))
}

pub fn reopened<R: Runtime>(app: &AppHandle<R>) {
    if app.get_webview_window(LABEL).is_none()
        && let Some(state) = app.try_state::<Now>()
        && let Ok(mut held) = state.0.lock()
    {
        *held = Some(Greeting::Keys);
    }
    open(app);
}

#[tauri::command]
pub fn greeting(state: tauri::State<'_, Now>) -> Greeting {
    state
        .0
        .lock()
        .ok()
        .and_then(|held| held.clone())
        .unwrap_or(Greeting::Nothing)
}

#[tauri::command]
pub async fn tour(app: AppHandle) {
    let wanted = Greeting::Tour { former: false };
    if let Some(state) = app.try_state::<Now>()
        && let Ok(mut held) = state.0.lock()
    {
        *held = Some(wanted.clone());
    }
    if app.get_webview_window(LABEL).is_some() {
        let _ = app.emit_to(LABEL, "greeting", wanted);
    }
    open(&app);
}

#[tauri::command]
pub async fn open_settings(app: AppHandle, rail: String) {
    crate::tray::surface_at(&app, Some(&rail));
}

#[cfg(test)]
#[path = "welcome_test.rs"]
mod tests;
