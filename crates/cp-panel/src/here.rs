use cp_core::capture::Captured;
use cp_core::item::Item;
use cp_core::paste::Failure;
use cp_core::paste_as::Content;
use raw_window_handle::RawWindowHandle;
use std::path::{Path, PathBuf};

pub use platform::{
    THUMBNAILS_FILES, Watching, ahead_now, capture_insisting, content_of, data_dir, dress, forward,
    in_front, ocr_available, paste_into, stay_out_of_the_dock, system_is_light, text_in,
    thumb_of_file, thumbs_dir, to_clipboard,
};

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
    pub use cp_win::watching::Watching;

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

    pub fn to_clipboard(item: &Item, ours: impl FnOnce()) -> bool {
        let Some(clipboard) = Clipboard::open() else {
            return false;
        };
        let written = matches!(
            cp_win::restore::to_clipboard(&clipboard, item),
            cp_win::restore::Restored::Written { .. }
        );
        if written {
            ours();
            drop(clipboard);
        }
        written
    }

    pub fn system_is_light() -> bool {
        cp_win_sys::theme::wants_light().unwrap_or(false)
    }

    pub fn ahead_now() -> isize {
        cp_win_sys::frontmost::ahead()
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
    pub use cp_mac::watching::Watching;

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

    pub fn to_clipboard(item: &Item, ours: impl FnOnce()) -> bool {
        let pb = Pasteboard::general_from_any_thread();
        let written = matches!(
            cp_mac::restore::to_pasteboard(&pb, item),
            cp_mac::restore::Restored::Written { .. }
        );
        if written {
            ours();
        }
        written
    }

    pub fn system_is_light() -> bool {
        cp_mac_sys::theme::wants_light().unwrap_or(false)
    }

    pub fn ahead_now() -> isize {
        cp_mac_sys::frontmost::ahead() as isize
    }

    pub fn stay_out_of_the_dock() {
        cp_mac_sys::activation::as_accessory();
    }

    pub fn dress(_handle: RawWindowHandle, _wanted: &str, _light: bool) {}

    pub fn forward(_handle: RawWindowHandle) {
        cp_mac_sys::frontmost::bring_to_front(cp_mac_sys::frontmost::our_pid());
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
mod tests {
    use super::*;

    #[test]
    fn what_the_platform_promises_about_file_thumbnails_is_what_it_does() {
        if !THUMBNAILS_FILES {
            assert!(
                thumb_of_file(Path::new("/"), 128).is_none(),
                "si no dibuja miniaturas de archivos, no puede devolver una"
            );
        }
    }

    #[test]
    fn the_folders_the_panel_writes_into_hang_from_the_same_root() {
        let Some(root) = data_dir() else {
            return;
        };
        assert!(thumbs_dir().expect("miniaturas").starts_with(&root));
    }

    #[test]
    fn asking_the_system_for_its_theme_answers_the_same_twice() {
        assert_eq!(system_is_light(), system_is_light());
    }

    #[test]
    fn nobody_in_front_reads_as_zero_and_pasting_there_reaches_nobody() {
        assert_eq!(paste_into(0, || {}), Sent::Nobody);
    }
}
