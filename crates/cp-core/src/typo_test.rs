use super::{distance, tolerance};

#[test]
fn short_words_are_never_guessed() {
    assert_eq!(tolerance("sop"), 0);
    assert_eq!(distance("sop", "sopa", false), None);
    assert_eq!(distance("sop", "sopa", true), None);
}

#[test]
fn the_longer_the_word_the_more_mistakes_it_forgives() {
    assert_eq!(tolerance("sopa"), 1);
    assert_eq!(tolerance("cormanil"), 2);
    assert_eq!(distance("sopas", "sopa", false), Some(1));
    assert_eq!(distance("sopxyz", "sopa", false), None);
    assert_eq!(distance("cormanillo", "cormanilo", false), Some(1));
    assert_eq!(distance("cromanilo", "cormanilo", false), Some(1));
    assert_eq!(distance("kormanyllo", "cormanilo", false), None);
}

#[test]
fn a_swap_of_two_letters_is_one_mistake() {
    assert_eq!(distance("piza", "pzia", false), Some(1));
    assert_eq!(distance("reunoin", "reunion", false), Some(1));
}

#[test]
fn a_word_being_typed_is_compared_with_the_start_of_the_other() {
    assert_eq!(distance("cormanill", "cormanilo", true), Some(1));
    assert_eq!(distance("cormanil", "cormanilo", true), Some(0));
    assert_eq!(distance("cromani", "cormanilo", true), Some(1));
    assert_eq!(distance("cromani", "cormanilo", false), None);
    assert_eq!(distance("zzzzz", "cormanilo", true), None);
}

#[test]
fn letters_beyond_the_basic_alphabet_count_as_one() {
    assert_eq!(distance("ñandú", "ñandu", false), Some(1));
    assert_eq!(distance("日本語です", "日本語でず", false), Some(1));
}
