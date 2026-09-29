use super::*;

#[test]
fn windows_always_has_an_opinion_about_its_own_theme() {
    assert!(wants_light().is_some());
}

#[test]
fn asking_twice_gives_the_same_answer() {
    assert_eq!(wants_light(), wants_light());
}
