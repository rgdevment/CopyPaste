use super::*;

const EVERY_SPARE: [&[&str]; 3] = [SPARE, PLAIN_SPARE, FORMS_SPARE];

fn every_spare() -> impl Iterator<Item = &'static &'static str> {
    EVERY_SPARE.into_iter().flatten()
}

#[test]
fn what_the_settings_file_ships_with_is_a_combination_the_system_knows() {
    for role in Role::ALL {
        assert!(
            role.of(&Config::default()).parse::<Shortcut>().is_ok(),
            "{role:?}"
        );
    }
}

#[test]
fn each_shortcut_reads_and_writes_its_own_line_of_the_settings() {
    let mut config = Config::default();
    for role in Role::ALL {
        assert_eq!(role.of(&config), role.of(&Config::default()));
        role.set(&mut config, format!("Ctrl+Alt+F{}", role as u8 + 1));
    }
    assert_eq!(config.shortcut, "Ctrl+Alt+F1");
    assert_eq!(config.plain_shortcut, "Ctrl+Alt+F2");
    assert_eq!(config.forms_shortcut, "Ctrl+Alt+F3");
}

#[test]
fn the_window_names_each_shortcut_the_way_the_settings_spell_it() {
    let roles: Vec<Role> =
        serde_json::from_str(r#"["panel","plain","forms"]"#).expect("names it knows");
    assert_eq!(roles, Role::ALL);
    assert_eq!(Role::default(), Role::Panel);
}

#[test]
fn every_spare_list_starts_with_the_shortcut_it_ships_with() {
    for role in Role::ALL {
        assert_eq!(
            role.spare().first(),
            Some(&role.of(&Config::default())),
            "{role:?}"
        );
    }
}

#[test]
fn a_combination_nobody_could_press_is_refused_before_the_system_sees_it() {
    assert!("".parse::<Shortcut>().is_err());
    assert!("Ctrl+".parse::<Shortcut>().is_err());
}

#[test]
fn nothing_is_bound_until_the_system_says_yes() {
    let bound = Bound::default();
    assert!(bound.0.lock().expect("unpoisoned").is_empty());
}

#[test]
fn every_spare_combination_is_one_the_system_can_be_asked_for() {
    assert!(EVERY_SPARE.iter().all(|one| !one.is_empty()));
    for said in every_spare() {
        assert!(
            said.parse::<Shortcut>().is_ok(),
            "«{said}» would be dropped in silence"
        );
    }
}

#[test]
fn the_spare_list_spells_its_modifiers_the_way_the_picker_does() {
    for said in every_spare() {
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
    for list in EVERY_SPARE {
        let parsed: Vec<Shortcut> = list
            .iter()
            .filter_map(|said| said.parse::<Shortcut>().ok())
            .collect();
        for (at, one) in parsed.iter().enumerate() {
            for other in &parsed[at + 1..] {
                assert_ne!(one, other, "one combination is offered twice");
            }
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
    for said in every_spare() {
        let one: Shortcut = said.parse().expect("a real combination");
        assert!(!spoken.contains(&one), "«{said}» is already spoken for");
    }
}

#[test]
fn settings_that_cannot_be_read_take_no_combination_beyond_the_panel() {
    let unread = unread();
    assert_eq!(unread.shortcut, cp_config::SHORTCUT);
    assert_eq!(unread.plain_shortcut, "");
    assert_eq!(unread.forms_shortcut, "");
}
