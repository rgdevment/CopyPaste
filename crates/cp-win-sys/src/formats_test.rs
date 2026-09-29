use super::*;

#[test]
fn the_standard_ids_are_the_ones_windows_documents() {
    for (id, name) in [
        (1u32, "CF_TEXT"),
        (8, "CF_DIB"),
        (13, "CF_UNICODETEXT"),
        (15, "CF_HDROP"),
        (17, "CF_DIBV5"),
    ] {
        assert_eq!(standard_name(id), Some(name));
    }
}

#[test]
fn a_registered_id_is_not_a_standard_one() {
    assert_eq!(standard_name(49_161), None);
    assert_eq!(standard_name(0), None);
}

#[test]
fn every_standard_id_maps_to_exactly_one_name() {
    for (id, _) in STANDARD {
        let matches = STANDARD.iter().filter(|(other, _)| other == id).count();
        assert_eq!(matches, 1, "the {id} identifier is there twice");
    }
}

#[test]
fn a_name_goes_back_to_the_id_it_came_from() {
    for id in [CF_TEXT, CF_DIB, CF_UNICODETEXT, CF_HDROP, CF_DIBV5] {
        assert_eq!(standard_id(&name_of(id)), Some(id));
    }
}

#[test]
fn a_name_that_is_not_standard_has_no_standard_id() {
    assert_eq!(standard_id("Rich Text Format"), None);
    assert_eq!(standard_id(""), None);
}

#[test]
fn a_registered_name_still_resolves_to_an_id() {
    let id = id_of("Rich Text Format").expect("se registra");
    assert!(id >= 0xC000, "los registrados viven por encima de 0xC000");
    assert_eq!(id_of("Rich Text Format"), Some(id), "y siempre el mismo");
    assert_eq!(name_of(id), "Rich Text Format");
}

#[test]
fn an_unknown_id_still_gets_a_name() {
    assert_eq!(name_of(0), "#0");
}
