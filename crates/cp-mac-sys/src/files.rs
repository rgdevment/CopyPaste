use objc2::rc::Retained;
use objc2_app_kit::NSWorkspace;
use objc2_foundation::{NSArray, NSString, NSURL};
use std::path::Path;

pub fn path_of(file_url: &str) -> String {
    file_url
        .strip_prefix("file://")
        .map(percent_decoded)
        .unwrap_or_else(|| file_url.to_owned())
}

fn percent_decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == b'%' && at + 2 < bytes.len() {
            let pair = std::str::from_utf8(&bytes[at + 1..at + 3]).ok();
            if let Some(byte) = pair.and_then(|hex| u8::from_str_radix(hex, 16).ok()) {
                out.push(byte);
                at += 3;
                continue;
            }
        }
        out.push(bytes[at]);
        at += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn url_of(path: &Path) -> Retained<NSURL> {
    NSURL::fileURLWithPath(&NSString::from_str(&path.display().to_string()))
}

const RUNS_WHEN_OPENED: [&str; 8] = [
    "app", "command", "terminal", "pkg", "scpt", "sh", "tool", "workflow",
];

pub fn runs_when_opened(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| RUNS_WHEN_OPENED.contains(&ext.to_ascii_lowercase().as_str()))
}

pub fn open(path: &Path) -> bool {
    if runs_when_opened(path) {
        return reveal(path);
    }
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
    fn a_file_url_becomes_the_path_it_names() {
        assert_eq!(
            path_of("file:///tmp/cp%20a9/canci%C3%B3n.png"),
            "/tmp/cp a9/canción.png"
        );
        assert_eq!(path_of("file:///tmp/plain.txt"), "/tmp/plain.txt");
        assert_eq!(
            path_of("/ya/es/ruta"),
            "/ya/es/ruta",
            "sin esquema se deja como llegó"
        );
    }

    #[test]
    fn a_percent_that_is_not_an_escape_is_kept() {
        assert_eq!(percent_decoded("100%"), "100%");
        assert_eq!(percent_decoded("%4"), "%4");
        assert_eq!(percent_decoded("%zz"), "%zz");
        assert_eq!(percent_decoded("a%41"), "aA");
        assert_eq!(percent_decoded("%41"), "A");
        assert_eq!(percent_decoded(""), "");
    }

    #[test]
    fn bytes_that_do_not_form_utf8_do_not_panic() {
        assert_eq!(percent_decoded("%ff%fe"), "\u{fffd}\u{fffd}");
    }

    #[test]
    fn what_is_not_there_is_neither_opened_nor_revealed() {
        let ghost = Path::new("/tmp/cp-no-existe-nunca/archivo.txt");
        assert!(!open(ghost), "abrir lo que no existe no lanza nada");
        assert!(!reveal(ghost), "ni activa el Finder");
    }

    #[test]
    fn what_would_run_when_opened_is_only_revealed() {
        for name in ["setup.pkg", "run.SH", "Thing.app", "x.command", "a.scpt"] {
            assert!(runs_when_opened(Path::new(name)), "{name}");
        }
        for name in ["notas.txt", "foto.png", "sin-extension", "archivo.shtml"] {
            assert!(!runs_when_opened(Path::new(name)), "{name}");
        }
        assert!(
            !open(Path::new("/tmp/cp-no-existe-nunca/peligro.sh")),
            "revelar lo que no existe también dice que no"
        );
    }

    #[test]
    fn a_path_with_spaces_and_accents_becomes_a_file_url() {
        let url = url_of(Path::new("/tmp/cp a9/canción.png"));
        let text = url
            .absoluteString()
            .map(|s| s.to_string())
            .unwrap_or_default();
        assert_eq!(
            text, "file:///tmp/cp%20a9/cancio%CC%81n.png",
            "NSURL descompone el acento como lo hace el sistema de archivos"
        );
    }
}
