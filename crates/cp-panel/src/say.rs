use std::sync::atomic::{AtomicBool, Ordering};

static ENGLISH: AtomicBool = AtomicBool::new(false);

pub fn english_for(locale: Option<&str>) -> bool {
    let asked = locale
        .map(str::to_owned)
        .or_else(sys_locale::get_locale)
        .unwrap_or_default();
    asked.to_lowercase().starts_with("en")
}

pub fn adopt(locale: Option<&str>) {
    ENGLISH.store(english_for(locale), Ordering::Relaxed);
}

pub fn adopt_what_was_kept() {
    let kept = crate::here::data_dir()
        .and_then(|dir| cp_config::read(&cp_config::at(&dir)).ok())
        .and_then(|kept| kept.locale);
    adopt(kept.as_deref());
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
mod tests {
    use super::*;

    #[test]
    fn only_a_locale_that_starts_with_en_gets_english() {
        assert!(english_for(Some("en")));
        assert!(english_for(Some("en-GB")));
        assert!(english_for(Some("EN-us")));
        assert!(!english_for(Some("es")));
        assert!(!english_for(Some("es-CL")));
        assert!(!english_for(Some("pt-BR")));
    }

    #[test]
    fn what_was_chosen_is_what_is_said() {
        assert_eq!(pick_in(false, "hola", "hello"), "hola");
        assert_eq!(pick_in(true, "hola", "hello"), "hello");
    }

    #[test]
    fn the_panel_speaks_spanish_until_somebody_says_otherwise() {
        assert_eq!(
            pick("hola", "hello"),
            pick_in(in_english(), "hola", "hello")
        );
    }
}
