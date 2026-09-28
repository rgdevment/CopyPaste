use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const APART: u64 = 24 * 60 * 60;

const FROM: [&str; 2] = ["github.com", "objects.githubusercontent.com"];

#[derive(Default)]
pub struct Installing(AtomicBool);

struct Alone<'a>(&'a AtomicBool);

impl Installing {
    fn claim(&self) -> Option<Alone<'_>> {
        self.0
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
            .then(|| Alone(&self.0))
    }
}

impl Drop for Alone<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

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
    pub installs: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Looked {
    pub route: Route,
    pub looked: bool,
    pub ready: Option<Ready>,
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
    url.parse::<tauri::Url>().is_ok_and(|at| {
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
        && running.is_some_and(|at| {
            at.starts_with("/Volumes/")
                || at.starts_with("/private/tmp/")
                || parted(at).any(|part| part == "AppTranslocation")
        })
}

fn parted(at: &Path) -> impl Iterator<Item = &str> {
    at.as_os_str()
        .to_str()
        .unwrap_or_default()
        .split(['/', '\\'])
}

const CASKROOMS: [&str; 2] = [
    "/opt/homebrew/Caskroom/copypaste",
    "/usr/local/Caskroom/copypaste",
];
const APPDIR: &str = "/Applications/";

fn chosen(running: Option<&Path>, there: impl Fn(&Path) -> bool) -> Route {
    let named = |what: &str| {
        running.is_some_and(|at| parted(at).any(|part| part.eq_ignore_ascii_case(what)))
    };
    if named("WindowsApps") {
        return Route::Store;
    }
    if named("Caskroom") {
        return Route::Brew;
    }
    let where_brew_puts_it = running.is_some_and(|at| at.starts_with(APPDIR));
    if cfg!(target_os = "macos")
        && where_brew_puts_it
        && CASKROOMS.iter().any(|one| there(Path::new(one)))
    {
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
    let landing = path.with_extension(format!("{}.part", std::process::id()));
    if poured(&landing, said.as_bytes()).is_err() {
        let _ = std::fs::remove_file(&landing);
        return;
    }
    if std::fs::rename(&landing, &path).is_err() {
        let _ = std::fs::remove_file(&landing);
    }
}

fn poured(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut file = std::fs::File::create(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

fn offered(version: &str, route: Route) -> Option<Ready> {
    worth_offering(version, env!("CARGO_PKG_VERSION")).then(|| Ready {
        version: version.to_owned(),
        installs: self_installs(route) && !mounted(std::env::current_exe().ok().as_deref()),
    })
}

#[tauri::command(async)]
pub async fn update_ready(
    app: tauri::AppHandle,
    now_please: Option<bool>,
) -> Result<Looked, String> {
    let route = route();
    if route == Route::Store {
        return Ok(Looked {
            route,
            looked: false,
            ready: None,
        });
    }
    let asked = now_please.unwrap_or(false);
    let held = kept();
    if !asked && !due(held.checked_at, now()) {
        return Ok(Looked {
            route,
            looked: held.checked_at.is_some(),
            ready: held.found.as_deref().and_then(|one| offered(one, route)),
        });
    }

    use tauri_plugin_updater::UpdaterExt;
    let found = app
        .updater_builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|why| why.to_string())?
        .check()
        .await
        .map_err(|why| why.to_string())?;

    let seen = found.map(|one| one.version.clone());
    keep(&Kept {
        checked_at: Some(now()),
        found: seen.clone(),
    });
    Ok(Looked {
        route,
        looked: true,
        ready: seen.as_deref().and_then(|one| offered(one, route)),
    })
}

#[tauri::command(async)]
pub async fn update_install(
    app: tauri::AppHandle,
    alone: tauri::State<'_, Installing>,
) -> Result<(), String> {
    let _busy = alone
        .inner()
        .claim()
        .ok_or_else(|| "an update is already on its way".to_owned())?;

    let route = route();
    if !self_installs(route) {
        return Err("this copy is updated by whoever installed it".to_owned());
    }
    if mounted(std::env::current_exe().ok().as_deref()) {
        return Err("a copy running from a read-only place cannot replace itself".to_owned());
    }
    let want = kept()
        .found
        .ok_or_else(|| "there is nothing waiting to be installed".to_owned())?;
    if !worth_offering(&want, env!("CARGO_PKG_VERSION")) {
        forget();
        return Err(format!("{want} is not newer than what is running"));
    }

    use tauri_plugin_updater::UpdaterExt;
    let asked = want.clone();
    let update = app
        .updater_builder()
        .version_comparator(move |_, release| release.version.to_string() == asked)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|why| why.to_string())?
        .check()
        .await
        .map_err(|why| why.to_string())?;

    let Some(update) = update else {
        forget();
        return Err(format!("{want} is not on the feed any more"));
    };
    if !ours(update.download_url.as_str()) {
        return Err("the feed points the download somewhere that is not ours".to_owned());
    }

    let bytes = update
        .download(|_, _| {}, || {})
        .await
        .map_err(|why| why.to_string())?;

    let waiting = app.clone();
    let _ = tauri::async_runtime::spawn_blocking(move || crate::panel::quit(&waiting)).await;
    if let Err(why) = update.install(bytes) {
        if let Err(back) = crate::panel::relight(&app) {
            crate::note::note(&format!("the panel did not come back: {back}"));
        }
        return Err(why.to_string());
    }

    forget();
    let handle = app.clone();
    app.run_on_main_thread(move || handle.restart())
        .map_err(|why| why.to_string())
}

fn forget() {
    keep(&Kept {
        checked_at: None,
        found: None,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_copy_the_store_keeps_is_updated_by_the_store() {
        assert_eq!(
            chosen(
                Some(Path::new(
                    r"C:\Program Files\WindowsApps\CopyPaste\cp-gui.exe"
                )),
                UNBREWED
            ),
            Route::Store
        );
        assert!(!self_installs(Route::Store));
    }

    const BREWED: fn(&Path) -> bool = |at| at == Path::new("/opt/homebrew/Caskroom/copypaste");
    const UNBREWED: fn(&Path) -> bool = |_| false;

    #[test]
    fn the_cask_moves_the_bundle_out_of_the_caskroom_and_the_receipt_still_tells() {
        let at = Path::new("/Applications/CopyPaste.app/Contents/MacOS/CopyPaste");
        if cfg!(target_os = "macos") {
            assert_eq!(
                chosen(Some(at), BREWED),
                Route::Brew,
                "the running path says Applications, the receipt says brew"
            );
        }
        assert_eq!(
            chosen(Some(at), UNBREWED),
            Route::Download,
            "no receipt, no brew"
        );
        assert!(!self_installs(Route::Brew));
    }

    #[test]
    fn a_copy_running_from_the_caskroom_itself_is_brew_too() {
        assert_eq!(
            chosen(
                Some(Path::new(
                    "/opt/homebrew/Caskroom/copypaste/3.0.0/CopyPaste.app/Contents/MacOS/CopyPaste"
                )),
                UNBREWED
            ),
            Route::Brew
        );
    }

    #[test]
    fn a_copy_somewhere_else_is_not_brew_however_many_casks_are_installed() {
        assert_eq!(
            chosen(
                Some(Path::new(
                    "/Users/quien/Downloads/CopyPaste.app/Contents/MacOS/CopyPaste"
                )),
                BREWED
            ),
            Route::Download,
            "another copy being brewed says nothing about the one running"
        );
        assert_eq!(chosen(None, BREWED), Route::Download);
        assert!(self_installs(Route::Download));
    }

    #[test]
    fn a_copy_running_from_somewhere_read_only_knows_it_cannot_replace_itself() {
        if cfg!(target_os = "macos") {
            assert!(mounted(Some(Path::new("/Volumes/CopyPaste/CopyPaste.app"))));
            assert!(
                mounted(Some(Path::new(
                    "/private/var/folders/xy/AppTranslocation/1E2/d/CopyPaste.app"
                ))),
                "Gatekeeper's copy is read only too"
            );
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
        assert!(!ours("https://github.com.evil.example.com/x"));
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
    fn only_one_install_can_be_under_way_at_a_time() {
        let alone = Installing::default();
        let first = alone.claim();
        assert!(first.is_some());
        assert!(alone.claim().is_none(), "a second one is turned away");
        drop(first);
        assert!(alone.claim().is_some(), "and the next one may go");
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

    #[test]
    fn a_store_copy_never_claims_to_have_looked() {
        let looked = Looked {
            route: Route::Store,
            looked: false,
            ready: None,
        };
        assert!(
            !looked.looked,
            "saying nothing is not saying it is up to date"
        );
    }
}
