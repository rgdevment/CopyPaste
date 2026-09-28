use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const APART: u64 = 24 * 60 * 60;
const GETS: Duration = Duration::from_secs(600);

const FROM: [&str; 2] = ["github.com", "objects.githubusercontent.com"];

const CASKS: [&str; 1] = ["copypaste"];
const PREFIXES: [&str; 2] = ["/opt/homebrew", "/usr/local"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Route {
    Store,
    Brew,
    Download,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ready {
    pub version: String,
    pub notes: Option<String>,
    pub route: Route,
    pub installs: bool,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Kept {
    checked_at: Option<u64>,
    found: Option<String>,
}

pub const fn self_installs(route: Route) -> bool {
    matches!(route, Route::Download)
}

pub fn ours(url: &str) -> bool {
    url.parse::<url::Url>().is_ok_and(|at| {
        at.scheme() == "https" && at.host_str().is_some_and(|host| FROM.contains(&host))
    })
}

pub fn due(last: Option<u64>, now: u64) -> bool {
    last.is_none_or(|at| at > now || now.saturating_sub(at) >= APART)
}

pub fn worth_offering(found: &str, here: &str) -> bool {
    let (Ok(found), Ok(here)) = (
        found.parse::<semver::Version>(),
        here.parse::<semver::Version>(),
    ) else {
        return false;
    };
    found > here
}

pub fn mounted(running: Option<&Path>) -> bool {
    cfg!(target_os = "macos")
        && running.is_some_and(|at| at.starts_with("/Volumes/") || at.starts_with("/private/tmp/"))
}

fn chosen(running: Option<&Path>, there: impl Fn(&Path) -> bool) -> Route {
    let packaged = running.is_some_and(|at| {
        at.to_string_lossy()
            .split(['/', '\\'])
            .any(|part| part.eq_ignore_ascii_case("WindowsApps"))
    });
    if packaged {
        return Route::Store;
    }
    let brewed = CASKS.iter().any(|cask| {
        PREFIXES
            .iter()
            .any(|root| there(Path::new(&format!("{root}/Caskroom/{cask}"))))
    });
    if brewed {
        return Route::Brew;
    }
    Route::Download
}

pub fn route() -> Route {
    chosen(std::env::current_exe().ok().as_deref(), |at| at.is_dir())
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

fn at() -> Option<PathBuf> {
    Some(crate::settings::folder()?.join("update.json"))
}

fn kept() -> Kept {
    at().and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|said| serde_json::from_str(&said).ok())
        .unwrap_or_default()
}

fn keep(one: &Kept) {
    let Some(path) = at() else {
        return;
    };
    let Ok(said) = serde_json::to_string(one) else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(&path, said);
}

fn offered(version: String, notes: Option<String>, route: Route) -> Option<Ready> {
    worth_offering(&version, env!("CARGO_PKG_VERSION")).then(|| Ready {
        version,
        notes,
        route,
        installs: self_installs(route) && !mounted(std::env::current_exe().ok().as_deref()),
    })
}

#[tauri::command(async)]
pub async fn update_ready(
    app: tauri::AppHandle,
    now_please: Option<bool>,
) -> Result<Option<Ready>, String> {
    let route = route();
    if route == Route::Store {
        return Ok(None);
    }
    let asked = now_please.unwrap_or(false);
    let held = kept();
    if !asked && !due(held.checked_at, now()) {
        return Ok(held.found.and_then(|version| offered(version, None, route)));
    }

    use tauri_plugin_updater::UpdaterExt;
    let found = app
        .updater()
        .map_err(|why| why.to_string())?
        .check()
        .await
        .map_err(|why| why.to_string())?;

    let seen = found.map(|one| (one.version.clone(), one.body.clone()));
    keep(&Kept {
        checked_at: Some(now()),
        found: seen.as_ref().map(|(version, _)| version.clone()),
    });
    Ok(seen.and_then(|(version, notes)| offered(version, notes, route)))
}

#[tauri::command(async)]
pub async fn update_install(app: tauri::AppHandle) -> Result<(), String> {
    let route = route();
    if !self_installs(route) {
        return Err("this copy is updated by whoever installed it".to_owned());
    }
    if mounted(std::env::current_exe().ok().as_deref()) {
        return Err("a copy running from the disk image cannot replace itself".to_owned());
    }
    let want = kept()
        .found
        .ok_or_else(|| "there is nothing waiting to be installed".to_owned())?;

    use tauri_plugin_updater::UpdaterExt;
    let asked = want.clone();
    let update = app
        .updater_builder()
        .version_comparator(move |_, release| release.version.to_string() == asked)
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|why| why.to_string())?
        .check()
        .await
        .map_err(|why| why.to_string())?;

    let Some(mut update) = update else {
        keep(&Kept {
            checked_at: None,
            found: None,
        });
        return Err(format!("{want} is not on the feed any more"));
    };
    if !ours(update.download_url.as_str()) {
        return Err("the feed points the download somewhere that is not ours".to_owned());
    }
    update.timeout = Some(GETS);

    crate::panel::quit(&app);
    let landed = update.download_and_install(|_, _| {}, || {}).await;
    if landed.is_err()
        && let Err(why) = crate::panel::relight(&app)
    {
        crate::note::note(&format!("the panel did not come back: {why}"));
    }
    landed.map_err(|why| why.to_string())?;

    keep(&Kept {
        checked_at: None,
        found: None,
    });
    let handle = app.clone();
    app.run_on_main_thread(move || handle.restart())
        .map_err(|why| why.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_copy_the_store_keeps_is_updated_by_the_store() {
        let there = |_: &Path| false;
        assert_eq!(
            chosen(
                Some(Path::new(
                    r"C:\Program Files\WindowsApps\CopyPaste\cp-gui.exe"
                )),
                there
            ),
            Route::Store
        );
        assert!(!self_installs(Route::Store));
    }

    #[test]
    fn a_copy_brew_keeps_is_updated_by_brew() {
        let there = |at: &Path| at == Path::new("/opt/homebrew/Caskroom/copypaste");
        assert_eq!(
            chosen(
                Some(Path::new(
                    "/Applications/CopyPaste.app/Contents/MacOS/CopyPaste"
                )),
                there
            ),
            Route::Brew
        );
        assert!(!self_installs(Route::Brew));
    }

    #[test]
    fn anything_else_installs_its_own_update() {
        let there = |_: &Path| false;
        assert_eq!(
            chosen(
                Some(Path::new(
                    "/Applications/CopyPaste.app/Contents/MacOS/CopyPaste"
                )),
                there
            ),
            Route::Download
        );
        assert_eq!(chosen(None, there), Route::Download);
        assert!(self_installs(Route::Download));
    }

    #[test]
    fn a_copy_running_from_the_disk_image_knows_it_cannot_replace_itself() {
        if cfg!(target_os = "macos") {
            assert!(mounted(Some(Path::new("/Volumes/CopyPaste/CopyPaste.app"))));
            assert!(!mounted(Some(Path::new("/Applications/CopyPaste.app"))));
        }
        assert!(!mounted(None));
    }

    #[test]
    fn a_download_that_does_not_come_from_our_releases_is_refused() {
        assert!(ours(
            "https://github.com/rgdevment/CopyPaste/releases/download/v3.0.1/copypaste.exe"
        ));
        assert!(ours("https://objects.githubusercontent.com/whatever"));
        assert!(!ours("https://evil.example.com/copypaste.exe"));
        assert!(!ours("http://github.com/rgdevment/CopyPaste"));
        assert!(!ours("github.com/rgdevment"));
        assert!(!ours(""));
    }

    #[test]
    fn a_clock_put_back_does_not_stop_the_next_look() {
        let now = 1_000_000;
        assert!(due(None, now));
        assert!(due(Some(now - APART), now));
        assert!(!due(Some(now - APART + 1), now));
        assert!(due(Some(now + 5_000), now), "a look in the future is stale");
    }

    #[test]
    fn only_a_higher_version_is_worth_offering() {
        assert!(worth_offering("3.0.1", "3.0.0"));
        assert!(worth_offering("3.1.0", "3.0.9"));
        assert!(!worth_offering("3.0.0", "3.0.0"));
        assert!(!worth_offering("2.9.9", "3.0.0"));
        assert!(!worth_offering("not a version", "3.0.0"));
        assert!(!worth_offering("3.0.1", "neither is this"));
    }

    #[test]
    fn a_candidate_is_not_offered_over_the_stable_it_came_from() {
        assert!(!worth_offering("3.0.0-rc.1", "3.0.0"));
        assert!(worth_offering("3.0.0", "3.0.0-rc.1"));
    }

    #[test]
    fn what_is_kept_survives_being_written_and_read_again() {
        let one = Kept {
            checked_at: Some(1_700_000_000),
            found: Some("3.0.1".to_owned()),
        };
        let said = serde_json::to_string(&one).expect("written");
        let back: Kept = serde_json::from_str(&said).expect("read");
        assert_eq!(back.checked_at, one.checked_at);
        assert_eq!(back.found, one.found);

        let empty: Kept = serde_json::from_str("{}").expect("an empty one still reads");
        assert!(empty.checked_at.is_none() && empty.found.is_none());
    }
}
