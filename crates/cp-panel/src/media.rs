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

pub fn clock_of(millis: i64) -> String {
    let whole = millis.max(0) / 1_000;
    let seconds = whole % 60;
    let minutes = (whole / 60) % 60;
    let hours = whole / 3_600;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

pub fn said_in(meta: Option<&std::collections::HashMap<String, String>>) -> (String, String) {
    let Some(meta) = meta else {
        return (String::new(), String::new());
    };
    let read = |key: &str| meta.get(key).and_then(|one| one.parse::<i64>().ok());
    let clock = read(DURATION).map(clock_of).unwrap_or_default();
    let measures = match (read(WIDTH), read(HEIGHT)) {
        (Some(wide), Some(high)) if wide > 0 && high > 0 => format!("{wide}×{high}"),
        _ => String::new(),
    };
    (clock, measures)
}

#[cfg(test)]
#[path = "media_test.rs"]
mod tests;
