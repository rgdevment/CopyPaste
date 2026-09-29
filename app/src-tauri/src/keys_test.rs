use super::*;

#[test]
fn what_the_settings_file_ships_with_is_a_combination_the_system_knows() {
    assert!(cp_config::SHORTCUT.parse::<Shortcut>().is_ok());
}

#[test]
fn a_combination_nobody_could_press_is_refused_before_the_system_sees_it() {
    assert!("".parse::<Shortcut>().is_err());
    assert!("Ctrl+".parse::<Shortcut>().is_err());
}

#[test]
fn nothing_is_bound_until_the_system_says_yes() {
    let bound = Bound::default();
    assert!(bound.0.lock().expect("unpoisoned").is_none());
}
