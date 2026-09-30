use super::*;

#[test]
fn the_verbatim_prefix_is_stripped_for_the_shell() {
    let here = shell_path(std::path::Path::new(".")).expect("ruta");
    let text = here.to_string_lossy();
    assert!(!text.starts_with(VERBATIM), "el shell no entiende {text}");
    assert!(here.is_absolute());
}

#[test]
fn a_path_that_is_not_there_has_no_shell_path() {
    let missing = std::path::Path::new(r"C:\no-existe-nada-de-nada");
    assert_eq!(shell_path(missing), None);
}

#[test]
fn entering_twice_is_not_a_problem() {
    let first = Apartment::enter();
    let second = Apartment::enter();
    drop(second);
    drop(first);
}
