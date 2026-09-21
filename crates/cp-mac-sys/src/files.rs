use objc2::rc::Retained;
use objc2_app_kit::NSWorkspace;
use objc2_foundation::{NSArray, NSString, NSURL};
use std::path::Path;

fn url_of(path: &Path) -> Retained<NSURL> {
    NSURL::fileURLWithPath(&NSString::from_str(&path.display().to_string()))
}

pub fn open(path: &Path) -> bool {
    path.exists() && NSWorkspace::sharedWorkspace().openURL(&url_of(path))
}

pub fn reveal(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    let url = url_of(path);
    let urls = NSArray::from_slice(&[&*url]);
    NSWorkspace::sharedWorkspace().activateFileViewerSelectingURLs(&urls);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_is_not_there_is_neither_opened_nor_revealed() {
        let ghost = Path::new("/tmp/cp-no-existe-nunca/archivo.txt");
        assert!(!open(ghost), "abrir lo que no existe no lanza nada");
        assert!(!reveal(ghost), "ni activa el Finder");
    }

    #[test]
    fn a_path_with_spaces_and_accents_becomes_a_file_url() {
        let url = url_of(Path::new("/tmp/cp a9/canción.png"));
        let text = url
            .absoluteString()
            .map(|s| s.to_string())
            .unwrap_or_default();
        assert!(text.starts_with("file:///tmp/cp%20a9/"), "{text}");
        assert!(text.ends_with(".png"));
    }
}
