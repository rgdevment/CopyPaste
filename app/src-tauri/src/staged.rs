use crate::update::{self, Attempt, HERE, Kept};
use std::path::PathBuf;
use tauri::Manager;

const PACKAGE: &str = "update-package";
const AT_START: std::time::Duration = std::time::Duration::from_secs(10);
const NETWORK_AT_START: std::time::Duration = std::time::Duration::from_secs(20);
const ONLINE_AGAIN: std::time::Duration = std::time::Duration::from_secs(2);
const RETRY_AFTER: u64 = 24 * 60 * 60;
const OFFLINE_STARTS: u32 = 3;

#[derive(Debug, PartialEq, Eq)]
pub enum Missed {
    Offline(String),
    Gone,
    Failed(String),
}

pub fn wanted() -> bool {
    crate::settings::settings()
        .ok()
        .and_then(|one| one.updates_itself)
        == Some(true)
}

fn package() -> Option<PathBuf> {
    Some(crate::settings::folder()?.join(PACKAGE))
}

pub fn discard() {
    if let Some(path) = package() {
        let _ = std::fs::remove_file(path);
    }
}

pub fn worth_staging(held: &Kept, here: &str, now: u64) -> Option<String> {
    let found = held
        .found
        .clone()
        .filter(|_| held.from.as_deref() == Some(here))?;
    let fresh = held.staged.as_deref() != Some(found.as_str())
        && held.tried.as_deref() != Some(found.as_str());
    let failed_lately = held
        .attempt
        .as_ref()
        .is_some_and(|one| one.version == found && one.at <= now && now - one.at < RETRY_AFTER);
    (fresh && !failed_lately && update::allowed(&found, here)).then_some(found)
}

pub const fn unreachable(why: &tauri_plugin_updater::Error) -> bool {
    matches!(
        why,
        tauri_plugin_updater::Error::Reqwest(_) | tauri_plugin_updater::Error::Network(_)
    )
}

pub fn gives_up_after(offline_starts: Option<u32>) -> bool {
    offline_starts.unwrap_or(0) >= OFFLINE_STARTS
}

pub fn to_apply(held: &Kept, here: &str, wanted: bool, replaces_itself: bool) -> Option<String> {
    let want = held
        .staged
        .clone()
        .filter(|_| held.from.as_deref() == Some(here))?;
    let untried = held.tried.as_deref() != Some(want.as_str());
    (wanted && replaces_itself && untried && update::allowed(&want, here)).then_some(want)
}

pub fn stage_soon(app: &tauri::AppHandle) {
    if !wanted() || !update::replaces_itself() {
        return;
    }
    let Some(want) = worth_staging(&update::kept(), HERE, update::now()) else {
        return;
    };
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let alone = app.state::<update::Installing>();
        let Some(_busy) = alone.inner().claim() else {
            return;
        };
        match staged(&app, &want).await {
            Ok(()) => crate::note::note(&format!(
                "{want} is downloaded and installs on the next start"
            )),
            Err(why) => {
                crate::note::note(&format!("{want} could not be downloaded ahead: {why}"));
                let mut held = update::kept();
                held.attempt = Some(Attempt {
                    version: want,
                    at: update::now(),
                });
                update::keep(&held);
            }
        }
    });
}

async fn staged(app: &tauri::AppHandle, want: &str) -> Result<(), String> {
    let feeds = update::feeds_of(want)?;
    let update = update::checked(app, &feeds, want, None, update::CHECKING)
        .await
        .map_err(|why| why.to_string())?
        .ok_or_else(|| "the feed does not offer it yet".to_owned())?;
    let (_, bytes) = update::downloaded(app, update, &feeds, want).await?;
    let path = package().ok_or_else(crate::settings::nowhere)?;
    update::laid_down(&path, &bytes).map_err(|why| why.to_string())?;
    let mut held = update::kept();
    if held.found.as_deref() != Some(want) || held.from.as_deref() != Some(HERE) {
        discard();
        return Err("a newer look moved on while it downloaded".to_owned());
    }
    held.staged = Some(want.to_owned());
    held.offline_starts = None;
    update::keep(&held);
    Ok(())
}

pub fn apply_at_start(app: &tauri::AppHandle) {
    let held = update::kept();
    if held.staged.is_none() {
        return;
    }
    let Some(want) = to_apply(&held, HERE, wanted(), update::replaces_itself()) else {
        drop_staged(held);
        return;
    };
    let Some(bytes) = package().and_then(|path| std::fs::read(path).ok()) else {
        drop_staged(held);
        return;
    };
    match tauri::async_runtime::block_on(installed(app, &want, &bytes)) {
        Ok(()) => {
            update::forget();
            app.restart();
        }
        Err(Missed::Offline(why)) => {
            let mut held = update::kept();
            held.offline_starts = Some(held.offline_starts.unwrap_or(0) + 1);
            if gives_up_after(held.offline_starts) {
                crate::note::note(&format!(
                    "{want} found no connection on {OFFLINE_STARTS} starts, so it is left to the offer: {why}"
                ));
                held.tried = Some(want);
                drop_staged(held);
                return;
            }
            crate::note::note(&format!(
                "{want} waits for a start with a connection: {why}"
            ));
            update::keep(&held);
        }
        Err(Missed::Gone) => {
            crate::note::note(&format!(
                "{want} is no longer offered, so it is not installed"
            ));
            drop_staged(update::kept());
        }
        Err(Missed::Failed(why)) => {
            crate::note::note(&format!("{want} did not install on start: {why}"));
            drop_staged(update::kept());
        }
    }
}

fn drop_staged(mut held: Kept) {
    discard();
    held.staged = None;
    update::keep(&held);
}

async fn installed(app: &tauri::AppHandle, want: &str, bytes: &[u8]) -> Result<(), Missed> {
    let feeds = update::feeds_of(want).map_err(Missed::Failed)?;
    let update = match checked_once_online(app, &feeds, want).await {
        Err(why) if unreachable(&why) => return Err(Missed::Offline(why.to_string())),
        Err(why) => return Err(Missed::Failed(why.to_string())),
        Ok(None) => return Err(Missed::Gone),
        Ok(Some(update)) => update::trusted(update, want).map_err(Missed::Failed)?,
    };
    let mut held = update::kept();
    held.tried = Some(want.to_owned());
    update::keep(&held);
    match update.install(bytes) {
        Ok(()) => Ok(()),
        Err(why) if update::worth_the_spare(&why) => {
            let spare = update::checked(app, &feeds, want, Some(update::SPARE), AT_START)
                .await
                .map_err(|why| Missed::Failed(why.to_string()))?
                .ok_or(Missed::Gone)?;
            let spare = update::trusted(spare, want).map_err(Missed::Failed)?;
            spare
                .install(bytes)
                .map_err(|why| Missed::Failed(why.to_string()))
        }
        Err(why) => Err(Missed::Failed(why.to_string())),
    }
}

async fn checked_once_online(
    app: &tauri::AppHandle,
    feeds: &[tauri::Url],
    want: &str,
) -> Result<Option<tauri_plugin_updater::Update>, tauri_plugin_updater::Error> {
    let since = std::time::Instant::now();
    loop {
        let left = NETWORK_AT_START.saturating_sub(since.elapsed());
        match update::checked(app, feeds, want, None, left.min(AT_START)).await {
            Err(why) if unreachable(&why) && since.elapsed() + ONLINE_AGAIN < NETWORK_AT_START => {
                std::thread::sleep(ONLINE_AGAIN);
            }
            other => return other,
        }
    }
}

#[cfg(test)]
#[path = "staged_test.rs"]
mod tests;
