use std::io::Write;
use std::path::PathBuf;

pub fn where_to() -> PathBuf {
    std::env::temp_dir().join("cp-gui.log")
}

pub fn note(what: &str) {
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(where_to())
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
    fn the_log_sits_among_the_other_temporary_files() {
        assert_eq!(where_to().parent(), Some(std::env::temp_dir().as_path()));
    }

    #[test]
    fn the_clock_reads_as_hours_minutes_and_seconds() {
        let said = clock();
        assert_eq!(said.len(), 8);
        assert!(said.chars().filter(|one| *one == ':').count() == 2);
    }
}
