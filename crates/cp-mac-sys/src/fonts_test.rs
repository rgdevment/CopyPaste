use super::*;

#[test]
fn a_mac_has_menlo_and_knows_it_is_monospaced() {
    let found = families();
    let menlo = found.iter().find(|one| one.name == "Menlo");
    assert!(menlo.is_some_and(|one| one.mono), "{found:?}");
}
