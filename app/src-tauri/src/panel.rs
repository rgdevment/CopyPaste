use cp_core::trouble::Trouble as Reason;
use std::sync::{Condvar, Mutex};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};

#[derive(Default)]
pub struct Sidecar(Mutex<Option<CommandChild>>);

#[derive(Default)]
pub struct Trouble(Mutex<Option<Reason>>);

#[derive(Default)]
pub struct Relights(Mutex<Tries>);

#[derive(Default)]
pub struct Parting {
    turn: Mutex<Turn>,
    nudge: Condvar,
}

#[derive(Default)]
struct Turn {
    leaving: bool,
    coming_back: bool,
}

#[derive(Default)]
struct Tries {
    count: u32,
    last: Option<std::time::Instant>,
}

pub fn trouble<R: Runtime>(app: &AppHandle<R>) -> Option<Reason> {
    app.try_state::<Trouble>()
        .and_then(|state| state.0.lock().ok().and_then(|held| *held))
}

fn heard_from_panel<R: Runtime>(app: &AppHandle<R>, said: &str) {
    if said == "settings" {
        crate::tray::surface_at(app, Some("keys"));
        return;
    }
    if let Some(what) = said.strip_prefix("well ") {
        if let Some(reason) = Reason::from_key(what) {
            settled(app, reason);
        }
        return;
    }
    if said == "shown" {
        let _ = app.emit("panel-shown", ());
        return;
    }
    if said.split_whitespace().next() == Some("emptied") {
        let _ = app.emit("emptied", ());
        return;
    }
    let Some(what) = said.strip_prefix("trouble ") else {
        return;
    };
    crate::note::note(&format!("the panel says: {what}"));
    if let Some(reason) = Reason::from_key(what) {
        trouble_is(app, reason);
    }
}

fn settled<R: Runtime>(app: &AppHandle<R>, reason: Reason) {
    if let Some(state) = app.try_state::<Trouble>()
        && let Ok(mut held) = state.0.lock()
        && *held == Some(reason)
    {
        held.take();
        crate::tray::tell(app, None);
    }
}

fn all_is_well<R: Runtime>(app: &AppHandle<R>) {
    if let Some(state) = app.try_state::<Trouble>()
        && let Ok(mut held) = state.0.lock()
    {
        held.take();
    }
    crate::tray::tell(app, None);
}

pub fn raise<R: Runtime>(app: &AppHandle<R>) {
    app.manage(Sidecar::default());
    app.manage(Trouble::default());
    app.manage(Relights::default());
    app.manage(Parting::default());
    match light(app) {
        Ok(()) => crate::note::note("the panel waits in the background"),
        Err(why) => crate::note::note(&format!("the panel did not start: {why}")),
    }
}

pub fn show<R: Runtime>(app: &AppHandle<R>) {
    match shown(app) {
        Showing::Done | Showing::Waiting => {}
        Showing::GaveUp => {
            gave_up(app, Reason::Unsteady);
        }
        Showing::Refused => {
            crate::note::note("the panel would not show itself, not even freshly started");
            gave_up(app, Reason::Unanswering);
        }
    }
}

enum Showing {
    Done,
    Waiting,
    GaveUp,
    Refused,
}

fn shown<R: Runtime>(app: &AppHandle<R>) -> Showing {
    allow(app);
    if say(app, "show").is_ok() {
        return Showing::Done;
    }
    match asked_again(app) {
        crate::reviving::Verdict::Wait { .. } => return Showing::Waiting,
        crate::reviving::Verdict::Enough { .. } => return Showing::GaveUp,
        crate::reviving::Verdict::Light { .. } => {}
    }
    part(app, false);
    if light(app).is_err() {
        return Showing::Refused;
    }
    allow(app);
    if say(app, "show").is_ok() {
        Showing::Done
    } else {
        Showing::Refused
    }
}

fn asked_again<R: Runtime>(app: &AppHandle<R>) -> crate::reviving::Verdict {
    let granted = crate::reviving::Verdict::Light { tries: 1 };
    let Some(state) = app.try_state::<Relights>() else {
        return granted;
    };
    let Ok(mut tries) = state.0.lock() else {
        return granted;
    };
    let now = std::time::Instant::now();
    let since = tries.last.map(|then| now.duration_since(then));
    let verdict = crate::reviving::asked_again(tries.count, since);
    match verdict {
        crate::reviving::Verdict::Light { tries: count } => {
            tries.count = count;
            tries.last = Some(now);
            crate::note::note(&format!("the panel is being restarted, attempt {count}"));
        }
        crate::reviving::Verdict::Wait { .. } => {
            crate::note::note("the panel was just restarted, so this restart waits its turn");
        }
        crate::reviving::Verdict::Enough { .. } => {
            crate::note::note(&format!(
                "the panel was restarted {} times without holding, so it will not be again for {} seconds",
                crate::reviving::AT_MOST,
                crate::reviving::FORGETS_AFTER.as_secs()
            ));
        }
    }
    verdict
}

fn bring_back<R: Runtime>(app: &AppHandle<R>) {
    let Some(state) = app.try_state::<Parting>() else {
        return;
    };
    let Ok(mut turn) = state.turn.lock() else {
        return;
    };
    if turn.leaving || turn.coming_back {
        return;
    }
    turn.coming_back = true;
    drop(turn);
    let reviving = app.clone();
    std::thread::spawn(move || come_back(&reviving));
}

fn come_back<R: Runtime>(app: &AppHandle<R>) {
    let Some(state) = app.try_state::<Parting>() else {
        return;
    };
    let Ok(mut turn) = state.turn.lock() else {
        return;
    };
    let mut told = false;
    loop {
        if turn.leaving || still_there(app) {
            break;
        }
        let left = match asked_again(app) {
            crate::reviving::Verdict::Light { .. } => {
                back(app);
                break;
            }
            crate::reviving::Verdict::Wait { left } => left,
            crate::reviving::Verdict::Enough { left } => {
                if !told {
                    gave_up(app, Reason::Unsteady);
                    told = true;
                }
                left
            }
        };
        turn = match state.nudge.wait_timeout(turn, left) {
            Ok((turn, _)) => turn,
            Err(_) => return,
        };
    }
    turn.coming_back = false;
}

fn back<R: Runtime>(app: &AppHandle<R>) {
    match light(app) {
        Ok(()) => crate::note::note("the panel is back and watches the clipboard again"),
        Err(why) => {
            crate::note::note(&format!("the panel could not come back: {why}"));
            gave_up(app, Reason::Stopped);
        }
    }
}

fn gave_up<R: Runtime>(app: &AppHandle<R>, reason: Reason) {
    crate::note::note(reason.worded(false));
    trouble_is(app, reason);
    crate::tray::tell(app, Some(reason));
}

fn parting<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.try_state::<Parting>()
        .and_then(|state| state.turn.lock().ok().map(|turn| turn.leaving))
        .unwrap_or(false)
}

fn part<R: Runtime>(app: &AppHandle<R>, leaving: bool) {
    if let Some(state) = app.try_state::<Parting>()
        && let Ok(mut turn) = state.turn.lock()
    {
        turn.leaving = leaving;
        state.nudge.notify_all();
    }
}

fn forgive<R: Runtime>(app: &AppHandle<R>) {
    if let Some(state) = app.try_state::<Relights>()
        && let Ok(mut tries) = state.0.lock()
    {
        tries.count = 0;
        tries.last = None;
    }
}

fn trouble_is<R: Runtime>(app: &AppHandle<R>, reason: Reason) {
    if let Some(state) = app.try_state::<Trouble>()
        && let Ok(mut held) = state.0.lock()
    {
        *held = Some(reason);
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
    if let Err(Said::Broken) = say(app, "hide") {
        bring_back(app);
    }
}

pub fn empty<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    say(app, "empty").map_err(|said| {
        if let Said::Broken = said {
            bring_back(app);
        }
        "the panel is not listening".to_owned()
    })
}

const GOES_IN: std::time::Duration = std::time::Duration::from_millis(2_500);

const _: () = assert!(
    GOES_IN.as_millis()
        > cp_core::closing::PATIENCE.as_millis() + cp_core::closing::A_MOMENT.as_millis()
);
const LOOKS_EVERY: std::time::Duration = std::time::Duration::from_millis(15);

pub fn quit<R: Runtime>(app: &AppHandle<R>) {
    part(app, true);
    let _ = say(app, "quit");
    let until = std::time::Instant::now() + GOES_IN;
    while std::time::Instant::now() < until && still_there(app) {
        std::thread::sleep(LOOKS_EVERY);
    }
    if let Some(state) = app.try_state::<Sidecar>()
        && let Ok(mut held) = state.0.lock()
        && let Some(child) = held.take()
    {
        crate::note::note("the panel would not leave on its own and had to be closed");
        let _ = child.kill();
    }
}

fn still_there<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.try_state::<Sidecar>()
        .and_then(|state| state.0.lock().ok().map(|held| held.is_some()))
        .unwrap_or(false)
}

pub fn relight<R: Runtime>(app: &AppHandle<R>) -> Result<(), tauri_plugin_shell::Error> {
    forgive(app);
    part(app, false);
    light(app)
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
                CommandEvent::Stderr(line) => {
                    for said in String::from_utf8_lossy(&line).lines() {
                        let said = said.trim();
                        if !said.is_empty() {
                            crate::note::note(&format!("the panel said: {said}"));
                        }
                    }
                }
                CommandEvent::Terminated(how) => {
                    let leaving = parting(&handle);
                    let ours = forget(&handle, whose);
                    if ours && !leaving {
                        let fell = crate::reviving::fell(how.code, how.signal);
                        crate::note::note(&format!(
                            "the panel closed on its own, code {:?}, signal {:?}, fell {fell}",
                            how.code, how.signal
                        ));
                        if fell {
                            trouble_is(&handle, Reason::Stopped);
                            bring_back(&handle);
                        } else if trouble(&handle).is_none_or(Reason::still_keeping) {
                            trouble_is(&handle, Reason::Stopped);
                        }
                    }
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
        if let Some(child) = held.take() {
            crate::note::note("the panel stopped listening, so it was closed");
            let _ = child.kill();
        }
        return Err(Said::Broken);
    }
    Ok(())
}

fn forget<R: Runtime>(app: &AppHandle<R>, whose: u32) -> bool {
    if let Some(state) = app.try_state::<Sidecar>()
        && let Ok(mut held) = state.0.lock()
        && held.as_ref().is_some_and(|child| child.pid() == whose)
    {
        held.take();
        return true;
    }
    false
}

#[derive(Debug)]
pub enum Said {
    Gone,
    Broken,
}

#[cfg(test)]
#[path = "panel_test.rs"]
mod tests;
