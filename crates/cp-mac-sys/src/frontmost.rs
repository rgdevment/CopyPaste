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
    let Some((pid, _)) = frontmost() else {
        return 0;
    };
    if pid == our_pid() { 0 } else { pid }
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
mod tests {
    use super::*;

    #[test]
    fn what_is_in_front_is_never_ourselves() {
        if let Some(named) = in_front() {
            assert!(!named.is_empty());
            assert_ne!(Some(named), app_name(our_pid()));
        }
        assert_ne!(ahead(), our_pid());
    }

    #[test]
    fn nobody_in_front_reads_as_zero_and_never_as_a_pid() {
        let pid = ahead();
        assert!(pid >= 0);
        if pid != 0 {
            assert!(is_alive(pid));
        }
    }

    #[test]
    fn missing_paths_reports_what_is_gone_and_only_that() {
        let dir = std::env::temp_dir().join(format!("cp-missing-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("a folder");
        let present = dir.join("está aquí.txt");
        std::fs::write(&present, b"x").expect("a file");
        let urls = format!(
            "file://{}\nfile://{}/does-not-exist.txt\n\n",
            present.display().to_string().replace(' ', "%20"),
            dir.display()
        );
        let missing = missing_paths(&urls);
        std::fs::remove_dir_all(&dir).ok();
        assert_eq!(missing.len(), 1);
        assert!(missing[0].ends_with("/does-not-exist.txt"));
        assert!(missing_paths("").is_empty());
    }
}
