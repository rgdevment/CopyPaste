use super::*;

#[test]
fn only_a_locale_that_starts_with_es_gets_spanish() {
    assert!(english_for(Some("en")));
    assert!(english_for(Some("en-GB")));
    assert!(!english_for(Some("es")));
    assert!(!english_for(Some("es-CL")));
    assert!(!english_for(Some("ES-mx")));
    assert!(english_for(Some("pt-BR")));
    assert!(english_for(Some("fr")));
    assert!(english_for(Some("de-DE")));
    assert!(english_for(Some("")));
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

#[test]
fn adopting_the_tongue_already_spoken_changes_nothing() {
    let now = in_english();
    adopt_english(now);
    assert_eq!(in_english(), now);
}
