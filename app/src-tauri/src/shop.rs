#![cfg_attr(not(windows), allow(dead_code))]

use crate::update::{self, Coming, HERE, Kept, Looked, Ready, Route};
use tauri::Emitter;

const APART: u64 = 30 * 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shelf {
    Waiting,
    Landed(String),
    Queued,
    Paused,
    Current,
    Silent,
    NotSold,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ahead {
    Is(String),
    Nothing,
    Unheard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Said {
    Offered,
    Landed,
    Queued,
    Paused,
    NotYet,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Memory {
    pub version: Option<String>,
    pub said: Option<Said>,
}

impl Memory {
    const fn offered(&self) -> bool {
        matches!(self.said, Some(Said::Offered | Said::Landed))
    }

    const fn coming(&self) -> bool {
        matches!(self.said, Some(Said::Queued | Said::Paused | Said::NotYet))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    pub ready: Option<Ready>,
    pub memory: Memory,
    pub unanswered: bool,
}

impl Verdict {
    pub fn offered_version(&self) -> Option<String> {
        self.ready
            .as_ref()
            .filter(|one| one.installs && !one.version.is_empty())
            .map(|one| one.version.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trouble {
    Gone,
    Stopped,
    Failed(String),
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Underway {
    stage: &'static str,
    far: u64,
}

pub fn due(last: Option<u64>, now: u64) -> bool {
    last.is_none_or(|at| at > now || now.saturating_sub(at) >= APART)
}

pub fn waits(pressed: bool, ahead: &Ahead) -> bool {
    pressed && *ahead != Ahead::Nothing
}

pub fn ahead_of(here: &str, manifest: Result<String, String>) -> Ahead {
    match manifest.map(|one| update::newer_stable(here, &one)) {
        Ok(Ok(Some(version))) => Ahead::Is(version),
        Ok(Ok(None)) => Ahead::Nothing,
        Ok(Err(_)) | Err(_) => Ahead::Unheard,
    }
}

fn told(said: Said, version: Option<&str>) -> Ready {
    let version = version.unwrap_or_default().to_owned();
    let (installs, coming) = match said {
        Said::Offered => (true, None),
        Said::Landed => (false, None),
        Said::Queued => (false, Some(Coming::Queued)),
        Said::Paused => (false, Some(Coming::Paused)),
        Said::NotYet => (false, Some(Coming::NotYet)),
    };
    Ready {
        version,
        installs,
        coming,
    }
}

pub fn last_said(here: &str, was: &Memory) -> Option<Ready> {
    let version = was
        .version
        .as_deref()
        .filter(|one| update::worth_offering(one, here))?;
    Some(told(was.said?, Some(version)))
}

pub fn verdict(shelf: Shelf, ahead: Ahead, here: &str, was: Memory) -> Verdict {
    let number = match &ahead {
        Ahead::Is(version) => Some(version.clone()),
        Ahead::Nothing => None,
        Ahead::Unheard => was
            .version
            .clone()
            .filter(|one| update::worth_offering(one, here)),
    };
    let tell = |said: Said, version: Option<String>| Verdict {
        ready: Some(told(said, version.as_deref())),
        memory: Memory {
            version,
            said: Some(said),
        },
        unanswered: false,
    };
    let none = |unanswered: bool| Verdict {
        ready: None,
        memory: Memory::default(),
        unanswered,
    };
    match (shelf, ahead) {
        (Shelf::Waiting, _) => tell(Said::Offered, number),
        (Shelf::Landed(version), _) => tell(Said::Landed, Some(version)),
        (Shelf::Queued, _) => tell(Said::Queued, number),
        (Shelf::Paused, _) => tell(Said::Paused, number),
        (Shelf::NotSold, _) => none(false),
        (_, Ahead::Is(version)) => tell(Said::NotYet, Some(version)),
        (Shelf::Current, Ahead::Nothing) => none(false),
        (Shelf::Silent, Ahead::Nothing) if !was.offered() => none(true),
        (Shelf::Current, Ahead::Unheard) if !was.coming() => none(false),
        (shelf, _) => Verdict {
            ready: last_said(here, &was),
            memory: was,
            unanswered: shelf == Shelf::Silent,
        },
    }
}

const DOWNLOAD_IS_FOUR_FIFTHS_OF_THE_ERRAND: f64 = 0.8;

pub fn step(far: f64) -> (&'static str, u64) {
    if far >= DOWNLOAD_IS_FOUR_FIFTHS_OF_THE_ERRAND {
        return ("installing", 100);
    }
    (
        "getting",
        (far.max(0.0) / DOWNLOAD_IS_FOUR_FIFTHS_OF_THE_ERRAND * 100.0) as u64,
    )
}

#[cfg(windows)]
pub fn owner(window: &tauri::WebviewWindow) -> Option<isize> {
    window.hwnd().ok().map(|one| one.0 as isize)
}

#[cfg(not(windows))]
pub const fn owner(_window: &tauri::WebviewWindow) -> Option<isize> {
    None
}

pub fn sold() -> bool {
    there::sold_here()
}

pub async fn looked(owner: Option<isize>, pressed: bool) -> Result<Looked, String> {
    let held = update::kept();
    let fresh = held.from.as_deref() == Some(HERE);
    let was = held.shop.clone().filter(|_| fresh).unwrap_or_default();
    let not_sold = !sold();
    let window = owner.filter(|_| pressed || !fresh || due(held.checked_at, update::now()));
    let Some(window) = window else {
        return Ok(Looked {
            route: Route::Store,
            looked: fresh && held.checked_at.is_some(),
            ready: last_said(HERE, &was),
            not_sold,
            replaces_itself: false,
        });
    };

    let settled = tauri::async_runtime::spawn_blocking(there::settled)
        .await
        .ok()
        .flatten();
    let (shelf, ahead) = match settled {
        Some(shelf) => (shelf, Ahead::Unheard),
        None => {
            let ahead = ahead_of(HERE, update::fetched().await);
            let waits = waits(pressed, &ahead);
            let shelf = tauri::async_runtime::spawn_blocking(move || there::asked(window, waits))
                .await
                .unwrap_or(Shelf::Silent);
            (shelf, ahead)
        }
    };
    let verdict = verdict(shelf, ahead, HERE, was);
    update::keep(&Kept {
        checked_at: Some(update::now()),
        found: verdict.offered_version(),
        from: Some(HERE.to_owned()),
        shop: Some(verdict.memory.clone()).filter(|one| *one != Memory::default()),
        staged: None,
        tried: None,
        attempt: None,
        offline_starts: None,
    });
    if pressed && verdict.unanswered {
        return Err(update::UNANSWERED.to_owned());
    }
    Ok(Looked {
        route: Route::Store,
        looked: true,
        ready: verdict.ready,
        not_sold,
        replaces_itself: false,
    })
}

pub async fn install(app: &tauri::AppHandle, owner: Option<isize>) -> Result<(), String> {
    let Some(window) = owner else {
        return Err(update::failed(
            "the Store's version",
            &"the window has no handle to lend the Store",
        ));
    };
    let telling = app.clone();
    let taken = tauri::async_runtime::spawn_blocking(move || {
        let mut said = ("", 0);
        there::take(window, move |stage, far| {
            if said != (stage, far) {
                said = (stage, far);
                let _ = telling.emit("updating", Underway { stage, far });
            }
        })
    })
    .await
    .map_err(|why| why.to_string())?;

    match taken {
        Ok(()) => Ok(()),
        Err(Trouble::Gone) => {
            update::forget();
            Err(update::GONE.to_owned())
        }
        Err(Trouble::Stopped) => Err(update::STOPPED.to_owned()),
        Err(Trouble::Failed(why)) => Err(update::failed("the Store's version", &why)),
    }
}

#[cfg(windows)]
mod there {
    use super::{Shelf, Trouble};
    use crate::store::{self, Queue, Taken};

    const PATIENCE: std::time::Duration = std::time::Duration::from_secs(10);
    const AGAIN: std::time::Duration = std::time::Duration::from_secs(6);
    const UNTIL: std::time::Duration = std::time::Duration::from_secs(75);
    const AT_LENGTH: std::time::Duration = std::time::Duration::from_secs(30 * 60);

    fn apart<T: Send + 'static>(
        how_long: std::time::Duration,
        work: impl FnOnce() -> T + Send + 'static,
    ) -> Option<T> {
        let (tell, hear) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let _ = tell.send(work());
        });
        hear.recv_timeout(how_long).ok()
    }

    pub fn sold_here() -> bool {
        store::sold_here()
    }

    pub fn settled() -> Option<Shelf> {
        if !store::sold_here() {
            return Some(Shelf::NotSold);
        }
        store::waiting_beside_us().map(Shelf::Landed)
    }

    pub fn asked(window: isize, waits: bool) -> Shelf {
        match offers(window) {
            Some(true) => Shelf::Waiting,
            Some(false) if waits => waited_for_the_errand_it_woke(window),
            Some(false) => in_its_queue(window),
            None => Shelf::Silent,
        }
    }

    fn offers(window: isize) -> Option<bool> {
        match apart(PATIENCE, move || store::offers(window)) {
            Some(Ok(offers)) => Some(offers),
            Some(Err(why)) => {
                crate::note::note(&format!("the Store refused to say what it has: {why}"));
                None
            }
            None => {
                crate::note::note("the Store never said what it has");
                None
            }
        }
    }

    fn waited_for_the_errand_it_woke(window: isize) -> Shelf {
        let since = std::time::Instant::now();
        while since.elapsed() < UNTIL {
            std::thread::sleep(AGAIN);
            if let Some(version) = store::waiting_beside_us() {
                return Shelf::Landed(version);
            }
            if let Some(Ok(Some(queue))) = apart(PATIENCE, move || store::queued(window)) {
                return shelf_of(queue);
            }
        }
        match offers(window) {
            Some(true) => Shelf::Waiting,
            Some(false) => in_its_queue(window),
            None => Shelf::Silent,
        }
    }

    fn in_its_queue(window: isize) -> Shelf {
        match apart(PATIENCE, move || store::queued(window)) {
            Some(Ok(Some(queue))) => shelf_of(queue),
            Some(Ok(None)) => Shelf::Current,
            Some(Err(why)) => {
                crate::note::note(&format!("the Store refused to show its queue: {why}"));
                Shelf::Silent
            }
            None => Shelf::Silent,
        }
    }

    const fn shelf_of(queue: Queue) -> Shelf {
        match queue {
            Queue::Moving => Shelf::Queued,
            Queue::Paused => Shelf::Paused,
        }
    }

    pub fn take(
        window: isize,
        mut told: impl FnMut(&'static str, u64) + Send + 'static,
    ) -> Result<(), Trouble> {
        if let Err(why) = store::reopen_when_replaced() {
            crate::note::note(&format!(
                "CoPanda will not reopen after the Store's update: {why}"
            ));
        }
        let taking = move || {
            store::take(window, move |far| {
                let (stage, how) = super::step(far);
                told(stage, how);
            })
        };
        match apart(AT_LENGTH, taking) {
            Some(Ok(Taken::Done)) => Ok(()),
            Some(Ok(Taken::Nothing)) => Err(Trouble::Gone),
            Some(Ok(Taken::Canceled)) => Err(Trouble::Stopped),
            Some(Ok(Taken::Other(state))) => Err(Trouble::Failed(state)),
            Some(Err(why)) => Err(Trouble::Failed(why.message())),
            None => Err(Trouble::Failed(format!(
                "the Store was left to it for {} minutes and never came back",
                AT_LENGTH.as_secs() / 60
            ))),
        }
    }
}

#[cfg(not(windows))]
mod there {
    use super::{Shelf, Trouble};

    pub const fn sold_here() -> bool {
        false
    }

    pub const fn settled() -> Option<Shelf> {
        Some(Shelf::NotSold)
    }

    pub const fn asked(_window: isize, _waits: bool) -> Shelf {
        Shelf::Silent
    }

    pub fn take(
        _window: isize,
        _told: impl FnMut(&'static str, u64) + Send + 'static,
    ) -> Result<(), Trouble> {
        Err(Trouble::Gone)
    }
}

#[cfg(test)]
#[path = "shop_test.rs"]
mod tests;
