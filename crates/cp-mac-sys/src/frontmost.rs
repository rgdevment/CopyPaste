use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication, NSWorkspace};

pub fn frontmost() -> Option<(i32, Option<String>)> {
    let app = NSWorkspace::sharedWorkspace().frontmostApplication()?;
    let pid = app.processIdentifier();
    if pid <= 0 {
        return None;
    }
    let bundle = app.bundleIdentifier().map(|id| id.to_string());
    Some((pid, bundle))
}

pub fn app_name(pid: i32) -> Option<String> {
    let app = NSRunningApplication::runningApplicationWithProcessIdentifier(pid)?;
    app.localizedName().map(|name| name.to_string())
}

pub fn bundle_of(pid: i32) -> Option<String> {
    let app = NSRunningApplication::runningApplicationWithProcessIdentifier(pid)?;
    app.bundleIdentifier().map(|id| id.to_string())
}

pub fn missing_paths(file_urls: &str) -> Vec<String> {
    file_urls
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter(|url| !std::path::Path::new(&crate::files::path_of(url)).exists())
        .map(str::to_owned)
        .collect()
}

pub fn our_pid() -> i32 {
    std::process::id() as i32
}

pub fn ahead() -> i32 {
    ahead_besides(&[our_pid()])
}

pub fn ahead_besides(ours: &[i32]) -> i32 {
    other_than(frontmost().map(|(pid, _)| pid), ours)
}

fn other_than(front: Option<i32>, ours: &[i32]) -> i32 {
    match front {
        Some(pid) if !ours.contains(&pid) => pid,
        _ => 0,
    }
}

pub fn is_ours_in_front() -> bool {
    frontmost().is_some_and(|(pid, _)| pid == our_pid())
}

pub fn in_front() -> Option<String> {
    let (pid, _) = frontmost()?;
    if pid == our_pid() {
        return None;
    }
    app_name(pid).filter(|named| !named.is_empty())
}

pub fn bring_to_front(pid: i32) -> bool {
    let Some(app) = NSRunningApplication::runningApplicationWithProcessIdentifier(pid) else {
        return false;
    };
    app.activateWithOptions(NSApplicationActivationOptions::ActivateAllWindows)
}

pub fn is_alive(pid: i32) -> bool {
    NSRunningApplication::runningApplicationWithProcessIdentifier(pid).is_some()
}

#[cfg(test)]
#[path = "frontmost_test.rs"]
mod tests;
