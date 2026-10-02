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

pub fn linkunbound_here() -> bool {
    scheme_here(cp_core::linkunbound::SCHEME)
}

fn scheme_here(scheme: &str) -> bool {
    let asked = NSString::from_str(&format!("{scheme}:"));
    let Some(url) = NSURL::URLWithString(&asked) else {
        return false;
    };
    NSWorkspace::sharedWorkspace()
        .URLForApplicationToOpenURL(&url)
        .is_some()
}

pub fn open_link(url: &str) -> bool {
    cp_core::linkunbound::opened_by(url, linkunbound_here(), shown)
}

fn shown(url: &str) -> bool {
    let said = NSString::from_str(url);
    match NSURL::URLWithString(&said) {
        Some(url) => NSWorkspace::sharedWorkspace().openURL(&url),
        None => false,
    }
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
#[path = "files_test.rs"]
mod tests;
