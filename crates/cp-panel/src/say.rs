use std::sync::atomic::{AtomicBool, Ordering};

static ENGLISH: AtomicBool = AtomicBool::new(false);

pub fn english_for(locale: Option<&str>) -> bool {
    let asked = locale
        .map(str::to_owned)
        .or_else(sys_locale::get_locale)
        .unwrap_or_default();
    asked.to_lowercase().starts_with("en")
}

pub fn adopt_english(english: bool) {
    ENGLISH.store(english, Ordering::Relaxed);
}

pub fn in_english() -> bool {
    ENGLISH.load(Ordering::Relaxed)
}

pub fn pick(es: &'static str, en: &'static str) -> &'static str {
    pick_in(in_english(), es, en)
}

pub fn pick_in(english: bool, es: &'static str, en: &'static str) -> &'static str {
    if english { en } else { es }
}

#[cfg(test)]
#[path = "say_test.rs"]
mod tests;
