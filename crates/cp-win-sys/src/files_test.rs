use super::*;

#[test]
fn what_would_run_is_revealed_instead_of_opened() {
    for name in [
        "setup.exe",
        "script.BAT",
        "run.cmd",
        "tool.ps1",
        "install.msi",
        "atajo.lnk",
        "viejo.com",
        "fondo.scr",
    ] {
        assert!(runs_when_opened(Path::new(name)), "{name} se ejecuta");
    }
    for name in [
        "informe.pdf",
        "foto.png",
        "nota.txt",
        "sin-extension",
        "carpeta.d",
    ] {
        assert!(!runs_when_opened(Path::new(name)), "{name} se abre");
    }
}

#[test]
fn a_path_that_is_not_there_is_neither_opened_nor_revealed() {
    let missing = Path::new(r"C:\no-existe-nada-de-nada\ni-esto.txt");
    assert!(!open(missing));
    assert!(!reveal(missing));
    assert_eq!(wide_of(missing), None);
}

#[test]
fn the_wide_path_is_absolute_and_ends_in_a_terminator() {
    let wide = wide_of(Path::new(".")).expect("ruta");
    assert_eq!(wide.last(), Some(&0));
    let text = String::from_utf16_lossy(&wide[..wide.len() - 1]);
    assert!(text.contains(':'), "{text} no es absoluta");
    assert!(
        !text.starts_with(r"\\?\"),
        "{text} lleva el prefijo que el shell no entiende"
    );
}

#[test]
fn a_scheme_nobody_registered_is_known_to_be_missing_without_asking_the_user() {
    assert!(
        !super::scheme_here("cp-no-hay-nada-asi-9f3a"),
        "an unregistered scheme must be answered here, or Windows shows the «look for an app» dialog"
    );
}

#[test]
fn a_scheme_the_machine_does_have_is_found() {
    assert!(
        super::scheme_here("https"),
        "every Windows opens the web, so a check that never says yes would be a check that does nothing"
    );
}

#[test]
fn each_way_of_asking_agrees_about_a_scheme_that_is_not_there() {
    assert!(!super::asked_of_the_shell("cp-no-hay-nada-asi-9f3a"));
    assert!(!super::written_as_a_protocol("cp-no-hay-nada-asi-9f3a"));
}

#[test]
fn the_registry_alone_can_answer_for_a_scheme_the_shell_resolves() {
    assert!(
        super::written_as_a_protocol("https"),
        "an app from the Store registers no plain executable, so the key that makes a scheme a \
         scheme is what has to be looked at when the shell does not answer"
    );
}
