use super::there::{approves, ours};
use std::path::Path;

#[test]
fn only_the_two_bytes_that_mean_the_person_turned_it_off_count_as_off() {
    assert!(!approves(&[3, 0, 0, 0]));
    assert!(!approves(&[6, 0, 0, 0]));
    assert!(approves(&[2, 0, 0, 0]));
    assert!(approves(&[0, 0, 0, 0]));
    assert!(approves(&[1, 0, 0, 0]));
    assert!(approves(&[]));
}

#[test]
fn an_entry_that_names_another_program_is_not_ours() {
    let exe = Path::new(r"C:\Programas\CopyPaste\cp-gui.exe");
    assert!(ours(r"C:\Programas\CopyPaste\cp-gui.exe", exe));
    assert!(ours(r#""C:\Programas\CopyPaste\cp-gui.exe""#, exe));
    assert!(ours(r#"  "c:\programas\copypaste\CP-GUI.EXE"  "#, exe));
    assert!(!ours(r"C:\Otro\cp-gui.exe", exe));
    assert!(!ours(r#""""#, exe));
    assert!(!ours("", exe));
}

#[test]
fn an_entry_with_arguments_is_not_taken_for_the_bare_path() {
    let exe = Path::new(r"C:\Programas\CopyPaste\cp-gui.exe");
    assert!(!ours(r"C:\Programas\CopyPaste\cp-gui.exe --hushed", exe));
    assert!(!ours(
        r#""C:\Programas\CopyPaste\cp-gui.exe" --hushed"#,
        exe
    ));
    assert!(!ours(r#""C:\Programas\CopyPaste\cp-gui.exe"#, exe));
}

#[test]
fn only_a_copy_running_from_windowsapps_counts_as_installed_from_the_store() {
    use super::there::packaged;
    assert!(packaged(Path::new(
        r"C:\Program Files\WindowsApps\CopyPaste_3.0.0.0_x64__abc\CopyPaste.exe"
    )));
    assert!(packaged(Path::new(
        r"C:\Program Files\windowsapps\CopyPaste\CopyPaste.exe"
    )));
    assert!(!packaged(Path::new(r"C:\Programas\CopyPaste\cp-gui.exe")));
    assert!(!packaged(Path::new(r"C:\Users\a\WindowsAppsX\cp-gui.exe")));
}
