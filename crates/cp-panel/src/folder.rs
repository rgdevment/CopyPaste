use std::time::Duration;

pub const ENTRIES: &str = "entries";
pub const KEYS: [&str; 1] = [ENTRIES];

pub const PATIENCE: Duration = Duration::from_secs(3);
pub const UP_TO: usize = 5_000;

const _: () = assert!(PATIENCE.as_secs() >= 1);
const _: () = assert!(UP_TO >= 100);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parts {
    pub name: String,
    pub parent: String,
}

pub fn parts_of(preview: &str) -> Option<Parts> {
    let first = preview.lines().next()?.trim_end();
    let trimmed = first.trim_end_matches(['\\', '/']);
    if trimmed.is_empty() {
        return None;
    }
    let cut = trimmed.rfind(['\\', '/']);
    let (parent, name) = match cut {
        Some(at) => (&trimmed[..=at], &trimmed[at + 1..]),
        None => ("", trimmed),
    };
    (!name.is_empty()).then(|| Parts {
        name: name.to_owned(),
        parent: parent.to_owned(),
    })
}

pub fn counted_in(path: &std::path::Path) -> Option<usize> {
    let reading = std::fs::read_dir(path).ok()?;
    let mut seen = 0;
    for one in reading {
        if one.is_err() {
            continue;
        }
        seen += 1;
        if seen >= UP_TO {
            break;
        }
    }
    Some(seen)
}

pub fn said_in(meta: Option<&std::collections::HashMap<String, String>>, english: bool) -> String {
    let say = |es: &'static str, en: &'static str| crate::say::pick_in(english, es, en);
    let Some(seen) = meta
        .and_then(|one| one.get(ENTRIES))
        .and_then(|one| one.parse::<usize>().ok())
    else {
        return String::new();
    };
    let what = if seen == 1 {
        say("elemento", "entry")
    } else {
        say("elementos", "entries")
    };
    if seen >= UP_TO {
        return if english {
            format!("more than {seen} {what}")
        } else {
            format!("más de {seen} {what}")
        };
    }
    format!("{seen} {what}")
}

#[cfg(test)]
#[path = "folder_test.rs"]
mod tests;
