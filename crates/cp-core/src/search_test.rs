use super::{EXCERPT_CHARS, Segment, excerpt, fold, fold_mapped, terms_of};

fn plain(text: &str) -> Segment {
    Segment {
        text: text.into(),
        matched: false,
    }
}

fn hit(text: &str) -> Segment {
    Segment {
        text: text.into(),
        matched: true,
    }
}

fn terms(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| (*word).to_owned()).collect()
}

#[test]
fn each_folded_char_remembers_where_it_came_from() {
    let folded = fold_mapped("Straße");
    assert_eq!(folded.chars.iter().collect::<String>(), "strasse");
    assert_eq!(
        folded.origin,
        vec![0, 1, 2, 3, 4, 4, 6],
        "the ß expands into two and takes two bytes: the origin is the byte"
    );
    let folded = fold_mapped("café");
    assert_eq!(folded.origin, vec![0, 1, 2, 3]);
    let folded = fold_mapped("ñu");
    assert_eq!(folded.origin, vec![0, 2]);
}

#[test]
fn the_excerpt_shows_the_text_as_copied_with_the_match_marked() {
    let found =
        excerpt("el café de la esquina", &terms(&["cafe"]), EXCERPT_CHARS).expect("matches");
    assert_eq!(
        found.segments,
        vec![plain("el "), hit("café"), plain(" de la esquina")],
        "searching drops the accent, but showing it keeps it"
    );
}

#[test]
fn a_prefix_marks_only_what_was_typed() {
    let found = excerpt("la esquina", &terms(&["esq"]), 100).expect("matches");
    assert_eq!(
        found.segments,
        vec![plain("la "), hit("esq"), plain("uina")]
    );
}

#[test]
fn a_word_that_only_appears_inside_another_is_not_a_match() {
    assert!(
        excerpt("encafetado", &terms(&["cafe"]), 100).is_none(),
        "the index would not find it either: terms match by word prefix"
    );
}

#[test]
fn a_term_with_punctuation_matches_the_same_phrase_the_index_does() {
    let found =
        excerpt("Pedido AB-4417 entrega 12 marzo", &terms(&["ab-4417"]), 100).expect("matches");
    assert_eq!(
        found.segments,
        vec![plain("Pedido "), hit("AB-4417"), plain(" entrega 12 marzo")]
    );
}

#[test]
fn an_expanded_letter_is_marked_whole() {
    let found = excerpt("Straße Hauptbahnhof", &terms(&["strasse"]), 100).expect("matches");
    assert_eq!(found.segments, vec![hit("Straße"), plain(" Hauptbahnhof")]);
}

#[test]
fn overlapping_terms_become_one_mark() {
    let found = excerpt("un café", &terms(&["caf", "cafe"]), 100).expect("matches");
    assert_eq!(found.segments, vec![plain("un "), hit("café")]);
}

#[test]
fn every_term_that_matches_is_marked() {
    let found = excerpt("rojo verde azul", &terms(&["rojo", "azul"]), 100).expect("matches");
    assert_eq!(
        found.segments,
        vec![hit("rojo"), plain(" verde "), hit("azul")]
    );
}

#[test]
fn a_long_text_is_cut_around_the_first_match() {
    let text = format!("{}aguja{}", "paja ".repeat(100), " heno".repeat(100));
    let found = excerpt(&text, &terms(&["aguja"]), 60).expect("matches");
    let shown = found.plain();
    assert!(shown.starts_with('…') && shown.ends_with('…'), "{shown}");
    assert_eq!(
        shown.chars().count(),
        62,
        "sixty plus one ellipsis on each side"
    );
    assert!(
        found
            .segments
            .iter()
            .any(|one| one.matched && one.text == "aguja")
    );
    let before = shown.find("aguja").expect("is there");
    assert!(
        before < shown.len() / 2,
        "the match falls in the first third, not at the end of the window"
    );
}

#[test]
fn exactly_a_third_of_the_window_comes_before_the_match() {
    let found = excerpt("abcdefghij aguja", &terms(&["aguja"]), 9).expect("matches");
    assert_eq!(
        found.plain(),
        "…ij aguja",
        "three characters ahead, not one more"
    );
    let found = excerpt("ñññññ aguja", &terms(&["aguja"]), 9).expect("matches");
    assert_eq!(
        found.plain(),
        "…ññ aguja",
        "and the cut falls on a character boundary even though the one before it takes two bytes"
    );
}

#[test]
fn a_combining_mark_travels_with_the_word_it_marks() {
    let found = excerpt("cafe\u{301} rico", &terms(&["cafe"]), 100).expect("matches");
    assert_eq!(found.segments, vec![hit("cafe\u{301}"), plain(" rico")]);
}

#[test]
fn the_window_never_cuts_a_mark_or_a_skin_tone_from_its_base() {
    let text = format!("{} aguja x", "e\u{301}".repeat(30));
    let found = excerpt(&text, &terms(&["aguja"]), 9).expect("matches");
    let shown = found.plain();
    assert!(!shown.starts_with("…\u{301}"), "{shown:?}");
    let text = format!("{}aguja", "👍🏽".repeat(30));
    let found = excerpt(&text, &terms(&["aguja"]), 9).expect("matches");
    let shown = found.plain();
    assert!(!shown.starts_with("…🏽"), "{shown:?}");
}

#[test]
fn a_match_at_the_very_end_only_needs_the_leading_ellipsis() {
    let text = format!("{}fin", "x ".repeat(200));
    let found = excerpt(&text, &terms(&["fin"]), 40).expect("matches");
    let shown = found.plain();
    assert!(shown.starts_with('…'));
    assert!(shown.ends_with("fin"), "{shown}");
}

#[test]
fn a_match_at_the_start_only_needs_the_trailing_ellipsis() {
    let text = format!("inicio{}", " x".repeat(200));
    let found = excerpt(&text, &terms(&["inicio"]), 40).expect("matches");
    let shown = found.plain();
    assert!(shown.starts_with("inicio"), "{shown}");
    assert!(shown.ends_with('…'));
}

#[test]
fn a_short_text_comes_back_whole() {
    let found = excerpt("nota corta", &terms(&["nota"]), EXCERPT_CHARS).expect("matches");
    assert_eq!(found.plain(), "nota corta");
}

#[test]
fn a_second_match_past_the_window_does_not_stretch_it() {
    let text = format!("aguja {}aguja", "paja ".repeat(100));
    let found = excerpt(&text, &terms(&["aguja"]), 30).expect("matches");
    assert_eq!(found.segments.iter().filter(|one| one.matched).count(), 1);
    assert!(found.plain().chars().count() <= 31);
}

#[test]
fn nothing_to_look_for_is_nothing_found() {
    assert!(excerpt("algo", &[], 100).is_none());
    assert!(excerpt("", &terms(&["algo"]), 100).is_none());
    assert!(excerpt("otra cosa", &terms(&["algo"]), 100).is_none());
    assert!(excerpt("otra cosa", &terms(&["!!!"]), 100).is_none());
}

#[test]
fn the_terms_are_the_words_the_index_would_look_for() {
    assert_eq!(terms_of("Café  AB-4417 ... !!"), vec!["cafe", "ab-4417"]);
    assert!(terms_of("   ").is_empty());
    assert!(terms_of("!!! ...").is_empty());
}

#[test]
fn every_ligature_in_the_table_is_expanded() {
    for (given, expected) in [
        ("ß", "ss"),
        ("æ", "ae"),
        ("Æ", "ae"),
        ("œ", "oe"),
        ("Œ", "oe"),
        ("ø", "o"),
        ("Ø", "o"),
        ("ł", "l"),
        ("Ł", "l"),
        ("đ", "d"),
        ("Đ", "d"),
        ("þ", "th"),
        ("Þ", "th"),
    ] {
        assert_eq!(fold(given), expected, "«{given}» did not expand correctly");
    }
}

#[test]
fn combining_marks_disappear_whatever_the_script() {
    assert_eq!(fold("ñ"), "n");
    assert_eq!(fold("ç"), "c");
    assert_eq!(fold("ü"), "u");
    assert_eq!(fold("å"), "a");
    assert_eq!(fold("ĉ"), "c");
    assert_eq!(fold("ṩ"), "s");
}

#[test]
fn what_has_nothing_to_fold_comes_out_as_it_went_in() {
    assert_eq!(fold(""), "");
    assert_eq!(fold("plain ascii"), "plain ascii");
    assert_eq!(fold("日本語"), "日本語");
    assert_eq!(fold("123 !?"), "123 !?");
}

#[test]
fn folding_twice_changes_nothing() {
    for text in ["Straße", "encyclopædia", "Łódź", "café", "ÞÓRR"] {
        let once = fold(text);
        assert_eq!(fold(&once), once, "«{text}» was not stable");
    }
}

#[test]
fn folds_both_sides_of_the_index() {
    assert_eq!(fold("Straße"), "strasse");
    assert_eq!(fold("encyclopædia"), "encyclopaedia");
    assert_eq!(fold("Łódź"), "lodz");
    assert_eq!(fold("el café"), "el cafe");
}

#[test]
fn the_capital_eszett_does_not_fold_like_its_lowercase_twin() {
    assert_eq!(fold("ß"), "ss");
    assert_ne!(
        fold("ẞ"),
        "ss",
        "U+1E9E, the capital ß used in all-caps German text, only lowercases to ß \
         instead of expanding to ss, so STRASSE written with it would not be found \
         by searching straße"
    );
    assert_eq!(fold("ẞ"), "ß");
}

#[test]
fn the_turkish_dotted_capital_i_folds_like_a_plain_i_but_the_dotless_one_does_not() {
    assert_eq!(
        fold("İ"),
        "i",
        "the dot above decomposes away as a combining mark, leaving a plain i"
    );
    assert_eq!(fold("İ"), fold("I"));
    assert_ne!(
        fold("ı"),
        "i",
        "the dotless lowercase i has no decomposition and is a different letter, so it is \
         never folded onto an ascii i"
    );
}

#[test]
fn nfc_and_nfd_spellings_of_the_same_letter_fold_the_same() {
    let nfc = "caf\u{00e9}";
    let nfd = "cafe\u{0301}";
    assert_ne!(nfc.as_bytes(), nfd.as_bytes(), "different bytes going in");
    assert_eq!(fold(nfc), fold(nfd), "the same folded result coming out");
}

#[test]
fn terms_of_an_empty_or_symbol_only_query_is_empty() {
    assert_eq!(terms_of(""), Vec::<String>::new());
    assert_eq!(terms_of("   "), Vec::<String>::new());
    assert_eq!(terms_of("!?#"), Vec::<String>::new());
}

#[test]
fn a_leading_dash_stays_glued_to_its_word() {
    assert_eq!(terms_of("gato -perro"), vec!["gato", "-perro"]);
}

#[test]
fn quotes_are_not_a_token_boundary() {
    assert_eq!(terms_of("\"hola mundo\""), vec!["\"hola", "mundo\""]);
}

#[test]
fn a_term_longer_than_the_text_finds_nothing_and_does_not_panic() {
    assert_eq!(excerpt("ab", &["abcdef".to_owned()], 100), None);
}

#[test]
fn a_width_of_zero_still_shows_something() {
    let found = excerpt("a match in here", &["match".to_owned()], 0).expect("a hit");
    assert!(found.segments.iter().any(|one| one.matched));
}
