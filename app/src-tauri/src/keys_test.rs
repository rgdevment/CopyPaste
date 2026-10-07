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

#[test]
fn every_spare_combination_is_one_the_system_can_be_asked_for() {
    assert!(!SPARE.is_empty());
    for said in SPARE {
        assert!(
            said.parse::<Shortcut>().is_ok(),
            "«{said}» would be dropped in silence"
        );
    }
}

#[test]
fn the_spare_list_spells_its_modifiers_the_way_the_picker_does() {
    for said in SPARE {
        let parts: Vec<&str> = said.split('+').collect();
        let order = ["Ctrl", "Alt", "Shift", "Cmd"];
        let mut seen = 0;
        for part in &parts[..parts.len() - 1] {
            let at = order
                .iter()
                .position(|one| one == part)
                .unwrap_or_else(|| panic!("«{part}» is not a modifier the picker emits"));
            assert!(at >= seen, "«{said}» is spelled out of the picker's order");
            seen = at;
        }
    }
}

#[test]
fn no_two_spare_entries_are_the_same_combination_spelled_differently() {
    let parsed: Vec<Shortcut> = SPARE
        .iter()
        .filter_map(|said| said.parse::<Shortcut>().ok())
        .collect();
    for (at, one) in parsed.iter().enumerate() {
        for other in &parsed[at + 1..] {
            assert_ne!(one, other, "one combination is offered twice");
        }
    }
}

#[cfg(target_os = "macos")]
const SPOKEN_FOR: &[&str] = &[
    "Shift+Cmd+V",
    "Alt+Shift+Cmd+V",
    "Alt+Cmd+V",
    "Cmd+Space",
    "Alt+Cmd+Space",
    "Ctrl+Space",
    "Ctrl+Alt+Space",
    "Ctrl+Cmd+Space",
];
#[cfg(not(target_os = "macos"))]
const SPOKEN_FOR: &[&str] = &[
    "Ctrl+Shift+V",
    "Ctrl+Alt+C",
    "Ctrl+Shift+C",
    "Ctrl+Shift+Space",
    "Ctrl+Alt+Space",
    "Alt+Space",
];

#[test]
fn no_spare_takes_a_combination_the_system_or_everyday_apps_already_use() {
    let spoken: Vec<Shortcut> = SPOKEN_FOR
        .iter()
        .map(|said| said.parse::<Shortcut>().expect("a real combination"))
        .collect();
    for said in SPARE {
        let one: Shortcut = said.parse().expect("a real combination");
        assert!(!spoken.contains(&one), "«{said}» is already spoken for");
    }
}
