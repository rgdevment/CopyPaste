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
