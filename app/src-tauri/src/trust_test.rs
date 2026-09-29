use super::*;

#[test]
fn where_nothing_is_asked_nothing_is_offered() {
    let said = trust();
    if !said.offered {
        assert!(!said.pastes && !said.secure_input);
    }
}

#[cfg(target_os = "macos")]
#[test]
fn on_a_mac_the_question_always_has_an_answer() {
    assert!(trust().offered);
}

#[cfg(target_os = "macos")]
#[test]
fn pasting_is_never_claimed_without_one_of_the_two_permissions() {
    let said = trust();
    let ready = cp_mac_sys::permissions::Readiness::probe();
    assert_eq!(said.pastes, ready.can_post || ready.accessibility);
}
