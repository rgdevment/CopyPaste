use super::*;

fn find(name: &str) -> Family {
    families()
        .into_iter()
        .find(|one| one.name == name)
        .unwrap_or_else(|| panic!("{name} is not installed"))
}

#[test]
fn a_plain_windows_has_segoe_ui_among_its_families() {
    let found = families();
    assert!(found.iter().any(|one| one.name == "Segoe UI"), "{found:?}");
}

#[test]
fn consolas_is_monospaced_and_segoe_ui_is_not() {
    assert!(find("Consolas").mono);
    assert!(!find("Segoe UI").mono);
}

#[test]
fn marlett_is_a_symbol_family_and_segoe_ui_is_not() {
    assert!(find("Marlett").symbol);
    assert!(!find("Segoe UI").symbol);
}

#[test]
fn every_family_has_a_name_and_a_local_name() {
    assert!(
        families()
            .iter()
            .all(|one| !one.name.is_empty() && !one.local.is_empty())
    );
}

#[test]
fn the_message_font_has_a_face() {
    let face = message_face();
    assert!(
        face.as_deref().is_some_and(|one| !one.is_empty()),
        "{face:?}"
    );
}

#[test]
fn a_gdi_face_maps_to_its_directwrite_family() {
    assert_eq!(family_of_gdi("Segoe UI").as_deref(), Some("Segoe UI"));
}

#[test]
fn a_face_that_is_too_long_maps_to_nothing() {
    assert_eq!(family_of_gdi(&"x".repeat(64)), None);
}

#[test]
fn the_substitutes_list_the_shell_dialog_font() {
    let found = substitutes();
    assert!(
        found
            .iter()
            .any(|(key, _)| key.eq_ignore_ascii_case("MS Shell Dlg")),
        "{found:?}"
    );
}

#[test]
fn a_face_nobody_installed_maps_to_nothing() {
    assert_eq!(family_of_gdi("Zzz Nobody Installed This"), None);
}

#[test]
fn a_gdi_weight_name_maps_to_its_family() {
    assert_eq!(
        family_of_gdi("Segoe UI Semibold").as_deref(),
        Some("Segoe UI")
    );
}
