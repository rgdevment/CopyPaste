use cp_core::item::Item;

pub const DURATION: &str = "duration-ms";
pub const WIDTH: &str = "pixels-wide";
pub const HEIGHT: &str = "pixels-high";

pub const KEYS: [&str; 3] = [DURATION, WIDTH, HEIGHT];

pub fn first_path_of(item: &Item) -> Option<String> {
    crate::here::content_of(item, None).paths.into_iter().next()
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Facts {
    pub duration: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

pub fn said_of(facts: Option<Facts>) -> Vec<(&'static str, String)> {
    let Some(facts) = facts else {
        return Vec::new();
    };
    let mut said = Vec::with_capacity(KEYS.len());
    if let Some(seconds) = facts.duration.filter(|one| one.is_finite() && *one > 0.0) {
        said.push((DURATION, format!("{:.0}", seconds * 1_000.0)));
    }
    if let Some(wide) = facts.width.filter(|one| *one > 0) {
        said.push((WIDTH, wide.to_string()));
    }
    if let Some(high) = facts.height.filter(|one| *one > 0) {
        said.push((HEIGHT, high.to_string()));
    }
    said
}

#[cfg(test)]
#[path = "media_test.rs"]
mod tests;
