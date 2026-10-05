use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const APART: u64 = 24 * 60 * 60;

const FROM: [&str; 2] = ["github.com", "objects.githubusercontent.com"];

const MANIFEST: &str =
    "https://raw.githubusercontent.com/rgdevment/CopyPaste/manifest/release-manifest.json";
const LATEST: &str = "https://raw.githubusercontent.com/rgdevment/CopyPaste/manifest/latest.json";
const CANDIDATE: &str =
    "https://raw.githubusercontent.com/rgdevment/CopyPaste/manifest/candidate.json";
const PATIENCE: std::time::Duration = std::time::Duration::from_secs(10);

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

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    latest: String,
    #[serde(default)]
    latest_prerelease: Option<String>,
}

pub fn tracks_candidates(here: &str) -> bool {
    here.parse::<semver::Version>()
        .is_ok_and(|version| !version.pre.is_empty())
}

pub fn newer(here: &str, manifest: &str) -> Option<String> {
    let running: semver::Version = here.parse().ok()?;
    let read: Manifest = serde_json::from_str(manifest).ok()?;
    let candidates = tracks_candidates(here);
    let mut best = read
        .latest
        .parse::<semver::Version>()
        .ok()
        .filter(|stable| candidates || stable.pre.is_empty());
    if candidates
        && let Some(said) = read.latest_prerelease.as_deref()
        && let Ok(candidate) = said.parse::<semver::Version>()
        && best.as_ref().is_none_or(|stable| candidate > *stable)
    {
        best = Some(candidate);
    }
    best.filter(|best| *best > running)
        .map(|best| best.to_string())
}

pub fn feeds_for(version: &str) -> Vec<&'static str> {
    if tracks_candidates(version) {
        vec![CANDIDATE, LATEST]
    } else {
        vec![LATEST]
    }
}

async fn fetched() -> Result<String, String> {
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }
    let answer = reqwest::Client::builder()
        .timeout(PATIENCE)
        .user_agent(concat!("copypaste/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|why| why.to_string())?
        .get(MANIFEST)
        .send()
        .await
        .map_err(|why| why.to_string())?;
    if !answer.status().is_success() {
        return Err(format!("the release manifest answered {}", answer.status()));
    }
    answer.text().await.map_err(|why| why.to_string())
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
pub async fn update_ready(now_please: Option<bool>) -> Result<Looked, String> {
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

    let manifest = fetched().await?;
    let seen = newer(env!("CARGO_PKG_VERSION"), &manifest);
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
    let feeds = feeds_for(&want)
        .into_iter()
        .map(|one| one.parse::<tauri::Url>().map_err(|why| why.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let update = app
        .updater_builder()
        .endpoints(feeds)
        .map_err(|why| why.to_string())?
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
#[path = "update_test.rs"]
mod tests;
