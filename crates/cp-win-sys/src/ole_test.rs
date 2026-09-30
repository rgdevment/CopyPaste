use super::*;

#[test]
fn entering_twice_is_not_a_problem() {
    let first = Ole::enter();
    let second = Ole::enter();
    drop(second);
    drop(first);
}

#[test]
fn asking_for_nothing_gets_nothing_and_does_not_touch_the_clipboard() {
    assert!(indexed_contents(0xC000, 0, CHUNK).is_empty());
}

#[test]
fn a_format_nobody_offers_comes_back_absent_for_every_index() {
    let seen = indexed_contents(0xFFFE, 3, CHUNK);
    assert_eq!(seen, vec![None, None, None]);
}
