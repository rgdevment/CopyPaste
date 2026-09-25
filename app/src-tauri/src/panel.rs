use std::sync::Mutex;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};

#[derive(Default)]
pub struct Sidecar(Mutex<Option<CommandChild>>);

#[derive(Default)]
pub struct Trouble(Mutex<Option<String>>);

pub fn trouble<R: Runtime>(app: &AppHandle<R>) -> Option<String> {
    app.try_state::<Trouble>()
        .and_then(|state| state.0.lock().ok().and_then(|held| held.clone()))
}

fn heard_from_panel<R: Runtime>(app: &AppHandle<R>, said: &str) {
    let Some(what) = said.strip_prefix("trouble ") else {
        return;
    };
    crate::note::note(&format!("el panel avisa: {what}"));
    if let Some(state) = app.try_state::<Trouble>()
        && let Ok(mut held) = state.0.lock()
    {
        *held = Some(what.to_owned());
    }
}

fn all_is_well<R: Runtime>(app: &AppHandle<R>) {
    if let Some(state) = app.try_state::<Trouble>()
        && let Ok(mut held) = state.0.lock()
    {
        held.take();
    }
}

pub fn raise<R: Runtime>(app: &AppHandle<R>) {
    #[cfg(target_os = "windows")]
    {
        app.manage(Sidecar::default());
        app.manage(Trouble::default());
        match light(app) {
            Ok(()) => crate::note::note("el panel queda esperando en segundo plano"),
            Err(why) => crate::note::note(&format!("el panel no arrancó: {why}")),
        }
    }
    #[cfg(not(target_os = "windows"))]
    let _ = app;
}

pub fn show<R: Runtime>(app: &AppHandle<R>) {
    allow(app);
    if say(app, "show").is_err() && light(app).is_ok() {
        allow(app);
        let _ = say(app, "show");
    }
}

fn allow<R: Runtime>(app: &AppHandle<R>) {
    #[cfg(target_os = "windows")]
    if let Some(state) = app.try_state::<Sidecar>()
        && let Ok(held) = state.0.lock()
        && let Some(child) = held.as_ref()
    {
        cp_win_sys::frontmost::let_it_come_forward(child.pid());
    }
    #[cfg(not(target_os = "windows"))]
    let _ = app;
}

pub fn hide<R: Runtime>(app: &AppHandle<R>) {
    let _ = say(app, "hide");
}

pub fn empty<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    say(app, "empty").map_err(|_| "el panel no está escuchando".to_owned())
}

pub fn quit<R: Runtime>(app: &AppHandle<R>) {
    let _ = say(app, "quit");
    if let Some(state) = app.try_state::<Sidecar>()
        && let Ok(mut held) = state.0.lock()
        && let Some(child) = held.take()
    {
        let _ = child.kill();
    }
}

fn light<R: Runtime>(app: &AppHandle<R>) -> Result<(), tauri_plugin_shell::Error> {
    let (mut heard, child) = app.shell().sidecar("cp-panel")?.args(["--serve"]).spawn()?;
    let whose = child.pid();
    if let Some(state) = app.try_state::<Sidecar>()
        && let Ok(mut held) = state.0.lock()
        && let Some(old) = held.replace(child)
    {
        let _ = old.kill();
    }
    all_is_well(app);
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = heard.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    heard_from_panel(&handle, String::from_utf8_lossy(&line).trim());
                }
                CommandEvent::Terminated(_) => {
                    forget(&handle, whose);
                    break;
                }
                _ => {}
            }
        }
    });
    Ok(())
}

fn say<R: Runtime>(app: &AppHandle<R>, what: &str) -> Result<(), Said> {
    let state = app.try_state::<Sidecar>().ok_or(Said::Gone)?;
    let mut held = state.0.lock().map_err(|_| Said::Gone)?;
    let child = held.as_mut().ok_or(Said::Gone)?;
    if child.write(format!("{what}\n").as_bytes()).is_err() {
        held.take();
        return Err(Said::Gone);
    }
    Ok(())
}

fn forget<R: Runtime>(app: &AppHandle<R>, whose: u32) {
    if let Some(state) = app.try_state::<Sidecar>()
        && let Ok(mut held) = state.0.lock()
        && held.as_ref().is_some_and(|child| child.pid() == whose)
    {
        held.take();
    }
}

#[derive(Debug)]
pub enum Said {
    Gone,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sidecar_starts_with_nobody_on_the_other_end() {
        let sidecar = Sidecar::default();
        assert!(sidecar.0.lock().expect("sin envenenar").is_none());
    }
}
