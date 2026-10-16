use super::{decode, encode, fts_expression};
use cp_core::search::{Term, terms_of};

#[test]
fn the_terms_travel_to_the_database_and_back_unchanged() {
    let mut terms = terms_of("jira cormanillo abr");
    terms[1].alike = vec!["cormanilo".into(), "cormanito".into()];
    assert_eq!(decode(&encode(&terms)), terms);
    assert!(decode("").is_empty());
}

#[test]
fn a_word_being_typed_weighs_its_exact_spelling_above_its_prefix() {
    assert_eq!(
        fts_expression(&terms_of("jira abr")).as_deref(),
        Some(r#""jira" AND ("abr" OR "abr"*)"#)
    );
}

#[test]
fn a_guess_joins_the_word_it_stands_for() {
    let terms = [Term {
        text: "cormanillo".into(),
        whole: true,
        alike: vec!["cormanilo".into()],
    }];
    assert_eq!(
        fts_expression(&terms).as_deref(),
        Some(r#"("cormanillo" OR "cormanilo")"#)
    );
}

#[test]
fn a_quote_inside_a_word_is_escaped() {
    assert_eq!(
        fts_expression(&terms_of("it's\"x ")).as_deref(),
        Some(r#""it's""x""#)
    );
}
