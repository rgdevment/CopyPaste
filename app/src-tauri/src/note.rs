use std::io::Write;
use std::path::PathBuf;

const UP_TO: u64 = 1024 * 1024;

pub fn where_to() -> PathBuf {
    crate::settings::folder()
        .map_or_else(std::env::temp_dir, |dir| dir.join("logs"))
        .join("cp-gui.log")
}

pub fn note(what: &str) {
    let path = where_to();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if std::fs::metadata(&path).map_or(0, |it| it.len()) > UP_TO {
        let _ = std::fs::write(&path, b"");
    }
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    else {
        return;
    };
    let _ = writeln!(file, "{} {what}", clock());
}

fn clock() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|it| it.as_secs())
        .unwrap_or(0);
    let day = now % 86_400;
    format!("{:02}:{:02}:{:02}", day / 3600, (day % 3600) / 60, day % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_log_sits_where_the_privacy_note_says_it_does() {
        let path = where_to();
        assert_eq!(
            path.file_name().and_then(|it| it.to_str()),
            Some("cp-gui.log")
        );
        if let Some(dir) = crate::settings::folder() {
            assert!(path.starts_with(dir));
        }
    }

    #[test]
    fn the_clock_reads_as_hours_minutes_and_seconds() {
        let said = clock();
        assert_eq!(said.len(), 8);
        assert!(said.chars().filter(|one| *one == ':').count() == 2);
    }
}
