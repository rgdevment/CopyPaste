use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

const TAIL: u64 = 2 * 1024 * 1024;
const LINES: usize = 2_000;
const LOGS: [&str; 4] = [
    "cp-gui.log.1",
    "cp-gui.log",
    "cp-panel.log.1",
    "cp-panel.log",
];

fn logs() -> PathBuf {
    let log = crate::note::where_to();
    log.parent()
        .map_or_else(std::env::temp_dir, Path::to_path_buf)
}

#[tauri::command]
pub fn open_log(app: tauri::AppHandle) -> Result<(), String> {
    let dir = logs();
    std::fs::create_dir_all(&dir).map_err(|why| why.to_string())?;
    let dir = seen_outside(dir);
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(|why| why.to_string())
}

#[cfg(windows)]
fn seen_outside(dir: PathBuf) -> PathBuf {
    if crate::update::route() != crate::update::Route::Store {
        return dir;
    }
    cp_win_sys::paths::in_package(&dir)
        .filter(|it| it.exists())
        .unwrap_or(dir)
}

#[cfg(not(windows))]
fn seen_outside(dir: PathBuf) -> PathBuf {
    dir
}

#[tauri::command(async)]
pub fn save_report(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
) -> Result<Option<String>, String> {
    let Some(chosen) = app
        .dialog()
        .file()
        .set_parent(&window)
        .set_file_name(named(&cp_core::stamp::now()))
        .add_filter("Text", &["txt"])
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let path = chosen.into_path().map_err(|why| why.to_string())?;
    let version = app.package_info().version.to_string();
    std::fs::write(&path, report(&version)).map_err(|why| why.to_string())?;
    crate::note::note("a report was saved");
    Ok(Some(path.to_string_lossy().into_owned()))
}

pub fn named(stamp: &str) -> String {
    let day = stamp.get(..10).unwrap_or("today");
    format!("copypaste-report-{day}.txt")
}

fn report(version: &str) -> String {
    let route = match crate::update::route() {
        crate::update::Route::Store => "Microsoft Store",
        crate::update::Route::Download => "download",
    };
    let mut out = format!(
        "CopyPaste {version}\n{}\n{} {}\ninstalled from: {route}\nwritten: {}\n",
        system(),
        std::env::consts::OS,
        std::env::consts::ARCH,
        cp_core::stamp::now()
    );
    let dir = logs();
    for name in LOGS {
        out.push_str(&format!("\n===== {name} =====\n"));
        match tail_of(&dir.join(name), TAIL) {
            Some(text) => out.push_str(&fold(&text, LINES)),
            None => out.push_str("(not there)\n"),
        }
    }
    let home = std::env::home_dir().map(|at| at.to_string_lossy().into_owned());
    let user = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .ok();
    redact(&out, home.as_deref(), user.as_deref())
}

pub fn tail_of(path: &Path, up_to: u64) -> Option<String> {
    let mut file = std::fs::File::open(path).ok()?;
    let size = file.metadata().ok()?.len();
    let from = size.saturating_sub(up_to);
    file.seek(SeekFrom::Start(from)).ok()?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).ok()?;
    let text = String::from_utf8_lossy(&bytes);
    let whole = if from > 0 {
        text.split_once('\n').map_or("", |(_, rest)| rest)
    } else {
        &text
    };
    Some(whole.to_owned())
}

pub fn fold(text: &str, keep: usize) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut said: Option<&str> = None;
    let mut again = 0;
    for line in text.lines() {
        let what = line.splitn(4, ' ').nth(3).unwrap_or(line);
        if said == Some(what) {
            again += 1;
            continue;
        }
        if again > 0 {
            lines.push(repeated(again));
        }
        again = 0;
        said = Some(what);
        lines.push(line.to_owned());
    }
    if again > 0 {
        lines.push(repeated(again));
    }
    let from = lines.len().saturating_sub(keep);
    let mut out = lines[from..].join("\n");
    out.push('\n');
    out
}

fn repeated(again: usize) -> String {
    if again == 1 {
        return "  (the line above, once more)".into();
    }
    format!("  (the line above, {again} more times)")
}

pub fn redact(text: &str, home: Option<&str>, user: Option<&str>) -> String {
    let mut out = text.to_owned();
    if let Some(home) = home.filter(|it| it.len() > 3) {
        out = swap(&out, home, "~", false);
        out = swap(&out, &home.replace('\\', "/"), "~", false);
    }
    if let Some(user) = user.filter(|it| it.len() > 1) {
        out = swap(&out, user, "<user>", true);
    }
    out
}

fn swap(text: &str, needle: &str, with: &str, whole_word: bool) -> String {
    let mut out = String::with_capacity(text.len());
    let mut kept = 0;
    for (start, _) in text.char_indices() {
        if start < kept {
            continue;
        }
        let Some(end) = found_at(text, start, needle) else {
            continue;
        };
        if whole_word
            && !(bounded(text[..start].chars().next_back()) && bounded(text[end..].chars().next()))
        {
            continue;
        }
        out.push_str(&text[kept..start]);
        out.push_str(with);
        kept = end;
    }
    out.push_str(&text[kept..]);
    out
}

fn found_at(text: &str, start: usize, needle: &str) -> Option<usize> {
    let mut there = text[start..].char_indices();
    let mut end = start;
    for wanted in needle.chars() {
        let (at, got) = there.next()?;
        if got != wanted && !got.to_lowercase().eq(wanted.to_lowercase()) {
            return None;
        }
        end = start + at + got.len_utf8();
    }
    Some(end)
}

fn bounded(next: Option<char>) -> bool {
    next.is_none_or(|it| !it.is_alphanumeric())
}

#[cfg(windows)]
fn system() -> String {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    let Ok(key) = winreg::RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
    else {
        return "Windows".into();
    };
    let read = |name: &str| key.get_value::<String, _>(name).unwrap_or_default();
    let build = read("CurrentBuildNumber");
    let name = read("ProductName");
    let name = if build.parse::<u32>().is_ok_and(|it| it >= 22_000) {
        name.replace("Windows 10", "Windows 11")
    } else {
        name
    };
    let patch = key
        .get_value::<u32, _>("UBR")
        .map(|it| format!(".{it}"))
        .unwrap_or_default();
    format!("{name} {} (build {build}{patch})", read("DisplayVersion"))
}

#[cfg(target_os = "macos")]
fn system() -> String {
    let said = std::process::Command::new("/usr/bin/sw_vers")
        .arg("-productVersion")
        .output()
        .ok()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .unwrap_or_default();
    format!("macOS {said}")
}

#[cfg(not(any(windows, target_os = "macos")))]
fn system() -> String {
    std::env::consts::OS.into()
}

#[cfg(test)]
#[path = "report_test.rs"]
mod tests;
