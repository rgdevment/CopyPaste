use super::*;

#[test]
fn every_copied_path_is_read_not_just_the_first() {
    let paths = [r"C:\uno.txt", r"C:\dos.txt", r"C:\una carpeta"];
    assert_eq!(paths_in(&drop_of(&paths)), paths);
}

#[test]
fn a_legacy_ansi_drop_is_read_too() {
    let paths = [r"C:\uno.txt", r"C:\dos.txt"];
    assert_eq!(paths_in(&ansi_drop_of(&paths)), paths);
}

#[test]
fn a_single_path_comes_back_alone() {
    assert_eq!(paths_in(&drop_of(&[r"C:\solo.png"])), [r"C:\solo.png"]);
}

#[test]
fn nonsense_is_not_a_drop() {
    assert!(paths_in(&[]).is_empty());
    assert!(paths_in(&[0, 0, 0]).is_empty());
    assert!(paths_in(&u32::MAX.to_le_bytes()).is_empty());
}

#[test]
fn a_path_with_accents_and_spaces_survives() {
    let paths = [r"C:\My Documents\report ñ.pdf"];
    assert_eq!(paths_in(&drop_of(&paths)), paths);
}

#[test]
fn the_drop_we_build_is_the_shape_the_shell_reads() {
    let drop = drop_of(&[r"C:\a.txt"]);
    assert_eq!(
        &drop[..4],
        &20u32.to_le_bytes(),
        "los nombres empiezan tras la cabecera"
    );
    assert_eq!(drop[16], 1, "y van en UTF-16");
    assert_eq!(&drop[drop.len() - 4..], &[0, 0, 0, 0], "doble terminador");
    assert!(paths_in(&drop_of::<&str>(&[])).is_empty());
}
