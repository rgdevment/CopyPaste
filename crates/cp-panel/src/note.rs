use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn where_to() -> std::path::PathBuf {
    std::env::temp_dir().join("cp-panel.log")
}

pub fn note(what: &str) {
    let clock = clock_now();
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(where_to())
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
    fn the_log_sits_next_to_the_other_temporary_files() {
        let where_it_is = where_to();
        assert!(where_it_is.starts_with(std::env::temp_dir()));
        assert_eq!(
            where_it_is.file_name().and_then(|name| name.to_str()),
            Some("cp-panel.log")
        );
    }
}
