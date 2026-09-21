use objc2_app_kit::NSWorkspace;

pub fn frontmost() -> Option<(i32, Option<String>)> {
    let app = NSWorkspace::sharedWorkspace().frontmostApplication()?;
    let pid = app.processIdentifier();
    let bundle = app.bundleIdentifier().map(|id| id.to_string());
    Some((pid, bundle))
}

pub fn app_name(pid: i32) -> Option<String> {
    let app = NSRunningApplication::runningApplicationWithProcessIdentifier(pid)?;
    app.localizedName().map(|name| name.to_string())
}

pub fn missing_paths(file_urls: &str) -> Vec<String> {
    file_urls
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter(|url| !std::path::Path::new(&path_of(url)).exists())
        .map(str::to_owned)
        .collect()
}

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

pub fn our_pid() -> i32 {
    std::process::id() as i32
}
use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication};

pub fn bring_to_front(pid: i32) -> bool {
    let Some(app) = NSRunningApplication::runningApplicationWithProcessIdentifier(pid) else {
        return false;
    };
    app.activateWithOptions(NSApplicationActivationOptions::ActivateAllWindows)
}

pub fn is_alive(pid: i32) -> bool {
    NSRunningApplication::runningApplicationWithProcessIdentifier(pid).is_some()
}
