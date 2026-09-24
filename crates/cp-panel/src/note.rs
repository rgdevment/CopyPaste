use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

const UP_TO: u64 = 1024 * 1024;

pub fn where_to() -> std::path::PathBuf {
    folder().join("cp-panel.log")
}

fn folder() -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    if let Some(dir) = cp_win_sys::paths::data_dir() {
        return dir.join("logs");
    }
    std::env::temp_dir()
}

pub fn note(what: &str) {
    let clock = clock_now();
    let path = where_to();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if std::fs::metadata(&path).map_or(0, |it| it.len()) > UP_TO {
        let _ = std::fs::write(&path, b"");
    }
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = writeln!(file, "{clock} {what}");
    }
}

pub fn catch_panics() {
    let before = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        note(&format!("pánico: {info}"));
        before(info);
    }));
}

fn clock_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_secs())
        .unwrap_or(0);
    format!(
        "{:02}:{:02}:{:02}",
        (secs / 3_600) % 24,
        (secs / 60) % 60,
        secs % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_reads_as_hours_minutes_and_seconds() {
        let stamp = clock_now();
        assert_eq!(stamp.len(), 8);
        assert_eq!(stamp.matches(':').count(), 2);
        assert!(
            stamp.chars().all(|one| one.is_ascii_digit() || one == ':'),
            "{stamp}"
        );
    }

    #[test]
    fn the_log_sits_where_the_privacy_note_says_it_does() {
        let where_it_is = where_to();
        assert!(where_it_is.starts_with(folder()));
        assert_eq!(
            where_it_is.file_name().and_then(|name| name.to_str()),
            Some("cp-panel.log")
        );
    }
}
