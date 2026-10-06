use cp_core::capture::Captured;
use cp_core::item::Item;
use cp_core::paste::Failure;
use cp_core::paste_as::Content;
use raw_window_handle::RawWindowHandle;
use std::path::{Path, PathBuf};

pub use platform::{
    Dragged, THUMBNAILS_FILES, Watching, ahead_now, capture_insisting, content_of, data_dir,
    drag_out, dress, forward, in_front, media_of, ocr_available, open_link, open_path,
    ours_up_front, paste_into, read_stuck, stay_out_of_the_dock, system_is_light, text_in,
    thumb_of_file, thumbs_dir, to_clipboard, towards, utc_offset_at, watch_start,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Landed {
    Nothing,
    Short { placed: usize, wanted: usize },
    Whole,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sent {
    Nobody,
    Done,
    Degraded(Failure),
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use cp_win_sys::clipboard::Clipboard;

    pub use cp_core::thumbnail::THUMBNAILS_FILES;
    pub use cp_core::watching::Watching;
    pub fn watch_start(on_fresh: impl FnMut() + Send + 'static) -> Watching {
        cp_win::watching::every(cp_core::watching::EVERY, on_fresh)
    }

    pub fn data_dir() -> Option<PathBuf> {
        cp_win_sys::paths::data_dir()
    }

    pub fn thumbs_dir() -> Option<PathBuf> {
        cp_win_sys::paths::thumbs_dir()
    }

    pub fn capture_insisting() -> Captured {
        cp_win::capture::capture_insisting(cp_win::capture::PATIENCE, cp_core::watch::RETRY)
    }

    pub fn in_front() -> Option<String> {
        cp_win_sys::source::in_front()
    }

    pub fn open_path(path: &Path) -> bool {
        cp_win_sys::files::open(path)
    }

    pub fn open_link(url: &str) -> bool {
        cp_win_sys::files::open_link(url)
    }

    pub fn content_of<'a>(item: &'a Item, ocr: Option<&'a str>) -> Content<'a> {
        cp_win::content::content_of(item, ocr)
    }

    pub fn ocr_available() -> bool {
        cp_win_sys::ocr::is_available()
    }

    pub fn text_in(image: &[u8]) -> Option<String> {
        cp_win_sys::ocr::text_in(image)
    }

    pub fn thumb_of_file(path: &Path, side: i32) -> Option<Vec<u8>> {
        let dib = cp_win_sys::thumbnail::dib_of_file(path, side)?;
        cp_core::dib::to_png(&dib)
    }

    pub fn utc_offset_at(millis: i64) -> i64 {
        cp_win_sys::clock::utc_offset_at(millis)
    }

    pub fn towards(ahead: isize) -> crate::landing::Towards {
        cp_win_sys::source::described(ahead).map_or(
            crate::landing::Towards::Elsewhere,
            |(process, class)| {
                let hosts = cp_win_sys::source::hosts_a_pseudoconsole(ahead);
                crate::landing::towards_of(&process, &class, hosts)
            },
        )
    }

    pub fn to_clipboard(
        item: &Item,
        landing: &crate::landing::Landing,
        starting: impl FnOnce(),
        ours: impl FnOnce(),
    ) -> Landed {
        let mut ready = cp_win::restore::ready_for(item);
        match landing.offer() {
            crate::landing::Offer::Files(paths) => ready.offer_files(&paths),
            crate::landing::Offer::Text(text) => ready.offer_text(&text),
            crate::landing::Offer::Nothing => {}
        }
        starting();
        let Some(clipboard) = Clipboard::to_write() else {
            ours();
            return Landed::Nothing;
        };
        let landed = match cp_win::restore::place(&clipboard, &ready) {
            cp_win::restore::Restored::Written {
                formats,
                incomplete: true,
            } => {
                let (placed, wanted) = ready.fitted(formats);
                Landed::Short { placed, wanted }
            }
            cp_win::restore::Restored::Written { .. } => Landed::Whole,
            _ => Landed::Nothing,
        };
        drop(clipboard);
        ours();
        landed
    }

    pub fn read_stuck() -> bool {
        cp_win_sys::clipboard::read_stuck_for().is_some()
    }

    pub fn media_of(path: &Path) -> Vec<(&'static str, String)> {
        crate::media::said_of(
            cp_win_sys::media::info_for(path).map(|one| crate::media::Facts {
                duration: one.duration,
                width: one.width,
                height: one.height,
            }),
        )
    }

    pub fn system_is_light() -> bool {
        cp_win_sys::theme::wants_light().unwrap_or(false)
    }

    pub fn ahead_now() -> isize {
        cp_win_sys::frontmost::ahead()
    }

    pub fn ours_up_front() -> Option<bool> {
        Some(ahead_now() == 0)
    }

    pub fn stay_out_of_the_dock() {}

    pub fn dress(handle: RawWindowHandle, wanted: &str, light: bool) {
        let RawWindowHandle::Win32(win32) = handle else {
            return;
        };
        let backdrop = cp_win_sys::backdrop::Backdrop::from_name(wanted)
            .unwrap_or(cp_win_sys::backdrop::Backdrop::Mica);
        cp_win_sys::backdrop::apply(win32.hwnd.get(), backdrop, !light);
    }

    pub fn forward(handle: RawWindowHandle) {
        let RawWindowHandle::Win32(win32) = handle else {
            return;
        };
        if let Some(target) = cp_win_sys::frontmost::target_at(win32.hwnd.get()) {
            cp_win_sys::frontmost::bring_forward(target.window);
        }
    }

    pub use cp_win_sys::dragging::Dragged;

    pub fn drag_out(handle: RawWindowHandle, paths: &[&Path]) -> Dragged {
        let RawWindowHandle::Win32(win32) = handle else {
            return Dragged::Elsewhere;
        };
        cp_win_sys::dragging::from_window(win32.hwnd.get(), paths)
    }

    pub fn paste_into(ahead: isize, hide: impl FnOnce()) -> Sent {
        let Some(target) = cp_win_sys::frontmost::target_at(ahead) else {
            return Sent::Nobody;
        };
        match cp_win::paste::paste_into(&target, hide) {
            cp_win::paste::Outcome::Sent { .. } => Sent::Done,
            cp_win::paste::Outcome::Degraded(why) => Sent::Degraded(why),
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use cp_core::destination::Destination;
    use cp_mac_sys::pasteboard::Pasteboard;

    pub use cp_core::thumbnail::THUMBNAILS_FILES;
    pub use cp_core::watching::Watching;
    pub fn watch_start(on_fresh: impl FnMut() + Send + 'static) -> Watching {
        cp_mac::watching::every(cp_core::watching::EVERY, on_fresh)
    }

    pub fn data_dir() -> Option<PathBuf> {
        cp_mac_sys::paths::data_dir()
    }

    pub fn thumbs_dir() -> Option<PathBuf> {
        cp_mac_sys::paths::thumbs_dir()
    }

    pub fn capture_insisting() -> Captured {
        cp_mac::capture::capture_insisting(cp_mac::capture::PATIENCE, cp_core::watch::RETRY)
    }

    pub fn in_front() -> Option<String> {
        cp_mac_sys::frontmost::in_front()
    }

    pub fn open_path(path: &Path) -> bool {
        cp_mac_sys::files::open(path)
    }

    pub fn open_link(url: &str) -> bool {
        cp_mac_sys::files::open_link(url)
    }

    pub fn content_of<'a>(item: &'a Item, ocr: Option<&'a str>) -> Content<'a> {
        cp_mac::content::content_of(item, ocr)
    }

    pub fn ocr_available() -> bool {
        true
    }

    pub fn text_in(image: &[u8]) -> Option<String> {
        cp_mac_sys::ocr::searchable_text(image)
    }

    pub fn thumb_of_file(_path: &Path, _side: i32) -> Option<Vec<u8>> {
        None
    }

    pub fn utc_offset_at(millis: i64) -> i64 {
        cp_mac_sys::clock::utc_offset_at(millis)
    }

    pub fn towards(ahead: isize) -> crate::landing::Towards {
        let Ok(pid) = i32::try_from(ahead) else {
            return crate::landing::Towards::Elsewhere;
        };
        if pid <= 0 {
            return crate::landing::Towards::Elsewhere;
        }
        let bundle = cp_mac_sys::frontmost::bundle_of(pid);
        let hosts = cp_mac_sys::processes::hosts_a_terminal(pid);
        crate::landing::towards_by_bundle(bundle.as_deref(), hosts)
    }

    pub fn to_clipboard(
        item: &Item,
        landing: &crate::landing::Landing,
        starting: impl FnOnce(),
        ours: impl FnOnce(),
    ) -> Landed {
        let offered = match landing.offer() {
            crate::landing::Offer::Text(text) => Some(text),
            _ => None,
        };
        let pb = Pasteboard::general_from_any_thread();
        starting();
        let landed = match cp_mac::restore::to_pasteboard_offering(&pb, item, offered.as_deref()) {
            cp_mac::restore::Restored::Written {
                formats,
                incomplete: true,
            } => Landed::Short {
                placed: formats,
                wanted: item.formats.len(),
            },
            cp_mac::restore::Restored::Written { .. } => Landed::Whole,
            _ => Landed::Nothing,
        };
        ours();
        landed
    }

    pub fn read_stuck() -> bool {
        false
    }

    pub fn media_of(path: &Path) -> Vec<(&'static str, String)> {
        crate::media::said_of(
            cp_mac_sys::media::info_for(path).map(|one| crate::media::Facts {
                duration: one.duration,
                width: one.width,
                height: one.height,
            }),
        )
    }

    pub fn system_is_light() -> bool {
        cp_mac_sys::theme::wants_light().unwrap_or(false)
    }

    pub fn ahead_now() -> isize {
        cp_mac_sys::frontmost::ahead() as isize
    }

    pub fn ours_up_front() -> Option<bool> {
        cp_mac_sys::activation::is_ours_up_front()
    }

    pub fn stay_out_of_the_dock() {
        cp_mac_sys::activation::as_accessory();
    }

    pub fn dress(_handle: RawWindowHandle, _wanted: &str, _light: bool) {}

    pub fn forward(handle: RawWindowHandle) {
        let RawWindowHandle::AppKit(appkit) = handle else {
            return;
        };
        if !cp_mac_sys::floating::keys_without_activating(appkit.ns_view) {
            cp_mac_sys::frontmost::bring_to_front(cp_mac_sys::frontmost::our_pid());
        }
    }

    pub use cp_mac_sys::dragging::Dragged;

    pub fn drag_out(handle: RawWindowHandle, paths: &[&Path]) -> Dragged {
        let RawWindowHandle::AppKit(appkit) = handle else {
            return Dragged::Elsewhere;
        };
        cp_mac_sys::dragging::from_view(appkit.ns_view, paths)
    }

    pub fn paste_into(ahead: isize, hide: impl FnOnce()) -> Sent {
        let Ok(pid) = i32::try_from(ahead) else {
            return Sent::Nobody;
        };
        if pid == 0 || !cp_mac_sys::frontmost::is_alive(pid) {
            return Sent::Nobody;
        }
        let target = Destination {
            pid,
            bundle_id: None,
        };
        let Some(paster) = cp_mac::paste::Paster::new() else {
            hide();
            return Sent::Degraded(Failure::SendDenied);
        };
        match paster.paste_into(&target, hide) {
            cp_mac::paste::Outcome::Sent { .. } => Sent::Done,
            cp_mac::paste::Outcome::Degraded(why) => Sent::Degraded(why),
        }
    }
}

#[cfg(test)]
#[path = "here_test.rs"]
mod tests;
