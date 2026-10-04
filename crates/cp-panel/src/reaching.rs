use crate::here;
use crate::note::note;
use crate::opening::{Landing, Reached, answered, landing_of};
use cp_store::Store;

pub fn reach_for(store: &Store, id: i64) -> Reached {
    let Ok(Some(item)) = store.item(id) else {
        note(&format!("{id} could not be looked up to open it"));
        return Reached::Refused;
    };
    let content = here::content_of(&item, None);
    if let Some(said) = content.text.as_deref()
        && let Some(url) = crate::opening::a_link_worth_opening(said.lines().next().unwrap_or(""))
    {
        let url = url.to_owned();
        let asking = cp_core::reading::begin(move || here::open_link(&url));
        return match answered(asking.waited(crate::opening::PATIENCE)) {
            Reached::Refused => Reached::NoLink,
            other => other,
        };
    }
    let said_path = crate::opening::first_of(&content.paths)
        .map(str::to_owned)
        .or_else(|| {
            content
                .text
                .as_deref()
                .map(|said| said.lines().next().unwrap_or("").trim().to_owned())
                .filter(|said| crate::opening::looks_like_a_path(said))
        });
    let on_disk = said_path.is_some();
    let path = match said_path {
        Some(said) => std::path::PathBuf::from(said),
        None => {
            let Some(bytes) = content.image else {
                note(&format!("{id} has nothing a viewer could be given"));
                return Reached::Refused;
            };
            let seen = crate::opening::where_previews_go();
            crate::opening::sweep_seen(&seen, std::time::SystemTime::now());
            let Some(spilled) = crate::opening::spilled(&seen, bytes) else {
                note(&format!("{id} could not be written out to be seen"));
                return Reached::Refused;
            };
            spilled
        }
    };
    let asking = cp_core::reading::begin(move || {
        let opened = here::open_path(&path);
        landing_of(opened, on_disk, path.exists())
    });
    match asking.waited(crate::opening::PATIENCE) {
        cp_core::reading::Waited::Answered(Landing::Opened) => Reached::Opened,
        cp_core::reading::Waited::Answered(Landing::Gone) => Reached::Missing,
        cp_core::reading::Waited::Answered(Landing::Refused) => Reached::Refused,
        cp_core::reading::Waited::StillRunning => Reached::Working,
        cp_core::reading::Waited::Gone => Reached::Refused,
    }
}
