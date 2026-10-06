use super::*;

#[test]
fn macos_always_has_an_opinion_about_its_own_theme() {
    assert!(wants_light().is_some());
}

#[test]
fn asking_twice_gives_the_same_answer() {
    assert_eq!(wants_light(), wants_light());
}

#[test]
fn off_the_main_thread_the_system_preference_answers() {
    if MainThreadMarker::new().is_some() {
        return;
    }
    assert_eq!(seen_by_the_app(), None);
    assert_eq!(wants_light(), Some(said_by_the_defaults()));
}
