use cp_core::kind::Kind;
use std::path::PathBuf;

pub fn can_drag(kind: Option<Kind>, paths: &[String]) -> bool {
    match kind {
        Some(Kind::Image) => true,
        Some(Kind::File | Kind::Folder | Kind::Video | Kind::Audio) => {
            crate::opening::first_of(paths).is_some_and(crate::opening::looks_like_a_path)
        }
        _ => false,
    }
}

pub fn on_disk(paths: &[String]) -> Vec<PathBuf> {
    paths
        .iter()
        .map(|one| one.trim())
        .filter(|one| crate::opening::looks_like_a_path(one))
        .map(PathBuf::from)
        .filter(|one| one.exists())
        .collect()
}

pub fn files_for(
    paths: &[String],
    image: Option<&[u8]>,
    now: std::time::SystemTime,
) -> Vec<PathBuf> {
    let there = on_disk(paths);
    if !there.is_empty() {
        return there;
    }
    let Some(bytes) = image else {
        return Vec::new();
    };
    let seen = crate::opening::where_previews_go();
    crate::opening::sweep_seen(&seen, now);
    crate::opening::spilled(&seen, bytes).into_iter().collect()
}

#[cfg(test)]
#[path = "dragging_test.rs"]
mod tests;
