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
fn a_scheme_the_shell_names_a_package_for_is_found() {
    let packaged = super::associated(
        "ms-windows-store",
        windows::Win32::UI::Shell::ASSOCSTR_APPID,
    );
    let executable = super::associated(
        "ms-windows-store",
        windows::Win32::UI::Shell::ASSOCSTR_EXECUTABLE,
    );
    if packaged.is_some() {
        assert!(
            super::scheme_here("ms-windows-store"),
            "a scheme the shell names a package for, as LinkUnbound from the Store is, has to be \
             found through its AppID whatever the executable says; packaged {packaged:?}, \
             executable {executable:?}"
        );
    }
}

#[test]
fn nothing_answers_for_a_scheme_that_is_not_there() {
    for asked in [
        windows::Win32::UI::Shell::ASSOCSTR_EXECUTABLE,
        windows::Win32::UI::Shell::ASSOCSTR_APPID,
    ] {
        assert!(super::associated("cp-no-hay-nada-asi-9f3a", asked).is_none());
    }
}

#[test]
fn the_open_with_picker_is_not_a_handler() {
    assert!(super::answered_by(
        Some(r"C:\Program Files\LinkUnbound\linkunbound-shell.exe"),
        None
    ));
    assert!(super::answered_by(
        None,
        Some("rgdevment.LinkUnbound-BrowserPicker_kdjgfdc2rb3gc!LinkUnbound")
    ));
    assert!(
        !super::answered_by(Some(r"C:\WINDOWS\system32\OpenWith.exe"), None),
        "the picker Windows offers when nothing answers was taken for an app"
    );
    assert!(!super::answered_by(
        Some(r"c:\windows\SYSTEM32\openwith.EXE"),
        None
    ));
    assert!(!super::answered_by(Some(""), Some("")));
    assert!(!super::answered_by(None, None));
}

#[test]
fn a_scheme_left_undecided_is_not_taken_for_one_that_has_an_app() {
    assert!(
        !super::answered_by(Some(r"C:\WINDOWS\system32\OpenWith.exe"), Some("Undecided")),
        "mailto and tel on a machine with no default answer exactly this, and wrapping a link for \
         them shows the «look for an app» dialog the probe exists to avoid"
    );
}

#[test]
fn only_a_package_counts_where_no_executable_answers() {
    assert!(super::answered_by(
        None,
        Some("AppX1h1kv4gmb0dpfenf5p98f1a1d3btwnwj")
    ));
    assert!(super::answered_by(
        None,
        Some("appxfvdy2xs18pcp2dv99rrcxe3kqmx56dq6")
    ));
    assert!(
        !super::answered_by(None, Some("linkunbound")),
        "a protocol key left with no command answers with its own name, and that is no app"
    );
    assert!(!super::answered_by(None, Some("Undecided")));
    assert!(!super::answered_by(None, Some("App")));
}

#[test]
fn the_machine_never_takes_the_picker_for_a_handler() {
    let executable = super::associated("mailto", windows::Win32::UI::Shell::ASSOCSTR_EXECUTABLE);
    let picked = executable
        .as_deref()
        .and_then(|one| std::path::Path::new(one).file_name())
        .and_then(|leaf| leaf.to_str())
        .is_some_and(|leaf| leaf.eq_ignore_ascii_case("OpenWith.exe"));
    if picked {
        assert!(
            !super::scheme_here("mailto"),
            "the shell sends mailto to the picker here, so nothing answers it"
        );
    }
}
