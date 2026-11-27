use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const APART: u64 = 24 * 60 * 60;

const FROM: [&str; 2] = ["github.com", "objects.githubusercontent.com"];
const RELEASES: [&str; 2] = [
    "/rgdevment/CopyPaste/releases/download/",
    "/rgdevment/CoPanda/releases/download/",
];
pub const SPARE: &str = include_str!("../updater-spare.pub").trim_ascii();
const ROOM: usize = 64 * 1024;
const DOWNLOAD: std::time::Duration = std::time::Duration::from_secs(600);

const MANIFEST: &str =
    "https://raw.githubusercontent.com/rgdevment/CopyPaste/manifest/release-manifest.json";
const LATEST: &str = "https://raw.githubusercontent.com/rgdevment/CopyPaste/manifest/latest.json";
const CANDIDATE: &str =
    "https://raw.githubusercontent.com/rgdevment/CopyPaste/manifest/candidate.json";
static TURN: AtomicU64 = AtomicU64::new(0);

const PATIENCE: std::time::Duration = std::time::Duration::from_secs(10);
pub const CHECKING: std::time::Duration = std::time::Duration::from_secs(30);
const FIRST_LOOK: std::time::Duration = std::time::Duration::from_secs(20);
const LOOKS_AGAIN: std::time::Duration = std::time::Duration::from_secs(60 * 60);

pub const HERE: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Installing(AtomicBool);

pub struct Alone<'a>(&'a AtomicBool);

impl Installing {
    pub fn claim(&self) -> Option<Alone<'_>> {
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
    Download,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ready {
    pub version: String,
    pub installs: bool,
    pub coming: Option<Coming>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Coming {
    Queued,
    Paused,
    NotYet,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Looked {
    pub route: Route,
    pub looked: bool,
    pub ready: Option<Ready>,
    pub not_sold: bool,
    pub replaces_itself: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Kept {
    pub checked_at: Option<u64>,
    pub found: Option<String>,
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shop: Option<crate::shop::Memory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staged: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tried: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt: Option<Attempt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offline_starts: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Attempt {
    pub version: String,
    pub at: u64,
}

pub const fn self_installs(route: Route) -> bool {
    matches!(route, Route::Download)
}

pub fn ours(url: &str) -> bool {
    url.parse::<tauri::Url>().is_ok_and(|at| {
        at.scheme() == "https"
            && at.host_str() == Some(FROM[0])
            && RELEASES.iter().any(|one| at.path().starts_with(one))
    }) || url
        .parse::<tauri::Url>()
        .is_ok_and(|at| at.scheme() == "https" && at.host_str() == Some(FROM[1]))
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
    schema: u32,
    latest: String,
    #[serde(default)]
    latest_prerelease: Option<String>,
}

pub const OFFLINE: &str = "offline";
pub const UNREADABLE: &str = "unreadable";
pub const BUSY: &str = "busy";
pub const MOUNTED: &str = "mounted";
pub const NOTHING: &str = "nothing";
pub const PUBLISHING: &str = "publishing";
pub const GONE: &str = "gone";
pub const FOREIGN: &str = "foreign";
pub const FAILED: &str = "failed";
pub const STOPPED: &str = "stopped";
pub const UNANSWERED: &str = "unanswered";

fn read(manifest: &str) -> Result<Manifest, &'static str> {
    let read: Manifest = serde_json::from_str(manifest).map_err(|_| UNREADABLE)?;
    if read.schema != 1 || read.latest.parse::<semver::Version>().is_err() {
        return Err(UNREADABLE);
    }
    Ok(read)
}

pub fn allowed(found: &str, here: &str) -> bool {
    worth_offering(found, here)
        && (tracks_candidates(here)
            || found
                .parse::<semver::Version>()
                .is_ok_and(|one| one.pre.is_empty()))
}

pub fn tracks_candidates(here: &str) -> bool {
    here.parse::<semver::Version>()
        .is_ok_and(|version| !version.pre.is_empty())
}

pub fn newer(here: &str, manifest: &str) -> Result<Option<String>, &'static str> {
    let read = read(manifest)?;
    Ok(newer_in(here, &read))
}

pub fn newer_stable(here: &str, manifest: &str) -> Result<Option<String>, &'static str> {
    let read = read(manifest)?;
    let (Ok(stable), Ok(running)) = (
        read.latest.parse::<semver::Version>(),
        here.parse::<semver::Version>(),
    ) else {
        return Ok(None);
    };
    Ok((stable.pre.is_empty() && stable > running).then(|| stable.to_string()))
}

fn newer_in(here: &str, read: &Manifest) -> Option<String> {
    let running: semver::Version = here.parse().ok()?;
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

async fn looked(
    app: &tauri::AppHandle,
    feeds: Vec<tauri::Url>,
    asked: String,
    target: Option<&'static str>,
    key: Option<&'static str>,
    patience: std::time::Duration,
) -> Result<Option<tauri_plugin_updater::Update>, tauri_plugin_updater::Error> {
    use tauri_plugin_updater::UpdaterExt;
    let mut builder = app.updater_builder();
    if let Some(target) = target {
        builder = builder.target(target);
    }
    if let Some(key) = key {
        builder = builder.pubkey(key);
    }
    builder
        .endpoints(feeds)?
        .version_comparator(move |_, release| release.version.to_string() == asked)
        .timeout(patience)
        .build()?
        .check()
        .await
}

pub async fn checked(
    app: &tauri::AppHandle,
    feeds: &[tauri::Url],
    want: &str,
    key: Option<&'static str>,
    patience: std::time::Duration,
) -> Result<Option<tauri_plugin_updater::Update>, tauri_plugin_updater::Error> {
    let native = native();
    match looked(app, feeds.to_vec(), want.to_owned(), native, key, patience).await {
        Err(tauri_plugin_updater::Error::TargetNotFound(_)) if native.is_some() => {
            looked(app, feeds.to_vec(), want.to_owned(), None, key, patience).await
        }
        other => other,
    }
}

pub fn failed(want: &str, why: &dyn std::fmt::Display) -> String {
    crate::note::note(&format!("the update to {want} did not go through: {why}"));
    FAILED.to_owned()
}

pub const fn worth_the_spare(why: &tauri_plugin_updater::Error) -> bool {
    matches!(why, tauri_plugin_updater::Error::Minisign(_))
}

pub fn trusted(
    mut update: tauri_plugin_updater::Update,
    want: &str,
) -> Result<tauri_plugin_updater::Update, String> {
    if !ours(update.download_url.as_str()) {
        crate::note::note(&format!("the feed sends {want} to {}", update.download_url));
        return Err(FOREIGN.to_owned());
    }
    update.timeout = Some(DOWNLOAD);
    Ok(update)
}

pub async fn downloaded(
    app: &tauri::AppHandle,
    update: tauri_plugin_updater::Update,
    feeds: &[tauri::Url],
    want: &str,
) -> Result<(tauri_plugin_updater::Update, Vec<u8>), String> {
    let update = trusted(update, want)?;
    match update.download(|_, _| {}, || {}).await {
        Ok(bytes) => Ok((update, bytes)),
        Err(why) if worth_the_spare(&why) => {
            crate::note::note(&format!(
                "{want} does not verify against the primary updater key, so the spare is tried: {why}"
            ));
            let spare = checked(app, feeds, want, Some(SPARE), CHECKING)
                .await
                .map_err(|why| failed(want, &why))?
                .ok_or_else(|| failed(want, &"the feed stopped offering it"))?;
            let spare = trusted(spare, want)?;
            let bytes = spare
                .download(|_, _| {}, || {})
                .await
                .map_err(|why| failed(want, &why))?;
            Ok((spare, bytes))
        }
        Err(why) => Err(failed(want, &why)),
    }
}

pub fn target_for(arch: &str, translated: bool) -> Option<&'static str> {
    (arch == "x86_64" && translated).then_some("darwin-aarch64")
}

fn native() -> Option<&'static str> {
    target_for(std::env::consts::ARCH, translated())
}

fn translated() -> bool {
    #[cfg(target_os = "macos")]
    {
        cp_mac_sys::translation::translated()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

pub fn feeds_of(version: &str) -> Result<Vec<tauri::Url>, String> {
    feeds_for(version)
        .into_iter()
        .map(|one| one.parse::<tauri::Url>().map_err(|why| why.to_string()))
        .collect()
}

pub fn feeds_for(version: &str) -> Vec<&'static str> {
    if tracks_candidates(version) {
        vec![CANDIDATE, LATEST]
    } else {
        vec![LATEST]
    }
}

pub async fn fetched() -> Result<String, String> {
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }
    let offline = |why: &dyn std::fmt::Display| {
        crate::note::note(&format!("the release manifest could not be read: {why}"));
        OFFLINE.to_owned()
    };
    let answer = reqwest::Client::builder()
        .timeout(PATIENCE)
        .user_agent(concat!("copypaste/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|why| offline(&why))?
        .get(MANIFEST)
        .header("Cache-Control", "no-cache")
        .send()
        .await
        .map_err(|why| offline(&why))?;
    if !answer.status().is_success() {
        return Err(offline(&answer.status()));
    }
    let bytes = answer.bytes().await.map_err(|why| offline(&why))?;
    if bytes.len() > ROOM {
        return Err(UNREADABLE.to_owned());
    }
    String::from_utf8(bytes.to_vec()).map_err(|_| UNREADABLE.to_owned())
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

pub fn chosen(running: Option<&Path>) -> Route {
    let store =
        running.is_some_and(|at| parted(at).any(|part| part.eq_ignore_ascii_case("WindowsApps")));
    if store { Route::Store } else { Route::Download }
}

pub fn route() -> Route {
    chosen(std::env::current_exe().ok().as_deref())
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

fn at() -> Option<PathBuf> {
    Some(crate::settings::folder()?.join("update.json"))
}

pub fn kept() -> Kept {
    at().and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|said| serde_json::from_str(&said).ok())
        .unwrap_or_default()
}

pub fn keep(one: &Kept) {
    let Some(path) = at() else {
        return;
    };
    let Ok(said) = serde_json::to_string(one) else {
        return;
    };
    let _ = laid_down(&path, said.as_bytes());
}

pub fn laid_down(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let turn = TURN.fetch_add(1, Ordering::Relaxed);
    let landing = path.with_extension(format!("{}.{turn}.part", std::process::id()));
    let moved = poured(&landing, bytes).and_then(|()| std::fs::rename(&landing, path));
    if moved.is_err() {
        let _ = std::fs::remove_file(&landing);
    }
    moved
}

fn poured(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut file = std::fs::File::create(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

pub fn replaces_itself() -> bool {
    self_installs(route()) && !mounted(std::env::current_exe().ok().as_deref())
}

fn offered(version: &str, route: Route) -> Option<Ready> {
    allowed(version, HERE).then(|| Ready {
        version: version.to_owned(),
        installs: self_installs(route) && replaces_itself(),
        coming: None,
    })
}

#[tauri::command]
pub fn install_route() -> Route {
    route()
}

#[tauri::command(async)]
pub async fn update_ready(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    now_please: Option<bool>,
) -> Result<Looked, String> {
    let asked = now_please.unwrap_or(false);
    if route() == Route::Store {
        return crate::shop::looked(crate::shop::owner(&window), asked).await;
    }
    let looked = looked_for_download(asked).await?;
    if looked.ready.as_ref().is_some_and(|one| one.installs) {
        crate::staged::stage_soon(&app);
    }
    Ok(looked)
}

async fn looked_for_download(asked: bool) -> Result<Looked, String> {
    let route = Route::Download;
    let held = kept();
    let fresh = held.from.as_deref() == Some(HERE);
    if !asked && fresh && !due(held.checked_at, now()) {
        return Ok(Looked {
            route,
            looked: held.checked_at.is_some(),
            ready: held.found.as_deref().and_then(|one| offered(one, route)),
            not_sold: false,
            replaces_itself: replaces_itself(),
        });
    }

    let manifest = fetched().await?;
    let seen = newer(HERE, &manifest).map_err(str::to_owned)?;
    let latest = kept();
    keep(&Kept {
        checked_at: Some(now()),
        found: seen.clone(),
        from: Some(HERE.to_owned()),
        shop: None,
        staged: latest.staged.filter(|_| fresh),
        tried: latest.tried.filter(|_| fresh),
        attempt: latest.attempt.filter(|_| fresh),
        offline_starts: latest.offline_starts.filter(|_| fresh),
    });
    Ok(Looked {
        route,
        looked: true,
        ready: seen.as_deref().and_then(|one| offered(one, route)),
        not_sold: false,
        replaces_itself: replaces_itself(),
    })
}

#[tauri::command(async)]
pub async fn update_install(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    alone: tauri::State<'_, Installing>,
) -> Result<(), String> {
    let _busy = alone.inner().claim().ok_or_else(|| BUSY.to_owned())?;

    let route = route();
    if route == Route::Store {
        return crate::shop::install(&app, crate::shop::owner(&window)).await;
    }
    if mounted(std::env::current_exe().ok().as_deref()) {
        return Err(MOUNTED.to_owned());
    }
    let held = kept();
    let want = held
        .found
        .filter(|_| held.from.as_deref() == Some(HERE))
        .ok_or_else(|| NOTHING.to_owned())?;
    if !allowed(&want, HERE) {
        forget();
        return Err(NOTHING.to_owned());
    }

    let feeds = feeds_of(&want)?;
    let update = checked(&app, &feeds, &want, None, CHECKING)
        .await
        .map_err(|why| failed(&want, &why))?;

    let Some(update) = update else {
        let still = fetched()
            .await
            .ok()
            .and_then(|manifest| newer(HERE, &manifest).ok().flatten());
        if still.as_deref() == Some(want.as_str()) {
            return Err(PUBLISHING.to_owned());
        }
        forget();
        return Err(GONE.to_owned());
    };
    let (update, bytes) = downloaded(&app, update, &feeds, &want).await?;

    let waiting = app.clone();
    let _ = tauri::async_runtime::spawn_blocking(move || crate::panel::quit(&waiting)).await;
    if let Err(why) = update.install(bytes) {
        if let Err(back) = crate::panel::relight(&app) {
            crate::note::note(&format!("the panel did not come back: {back}"));
        }
        return Err(failed(&want, &why));
    }

    forget();
    let handle = app.clone();
    app.run_on_main_thread(move || handle.restart())
        .map_err(|why| why.to_string())
}

pub fn keep_looking(app: &tauri::AppHandle) {
    if route() == Route::Store {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(FIRST_LOOK);
        loop {
            match tauri::async_runtime::block_on(looked_for_download(false)) {
                Ok(looked) if looked.ready.as_ref().is_some_and(|one| one.installs) => {
                    crate::staged::stage_soon(&app);
                }
                Ok(_) => {}
                Err(why) => {
                    crate::note::note(&format!("looking for an update on its own failed: {why}"));
                }
            }
            std::thread::sleep(LOOKS_AGAIN);
        }
    });
}

pub fn forget() {
    keep(&Kept::default());
    crate::staged::discard();
}

#[cfg(test)]
#[path = "update_test.rs"]
mod tests;
