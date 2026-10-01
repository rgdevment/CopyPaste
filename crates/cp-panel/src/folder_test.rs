use super::*;

#[test]
fn a_windows_path_splits_into_name_and_the_folder_above() {
    let parts = parts_of(r"D:\Mario\Orca\CopyPaste\rc1").expect("one");
    assert_eq!(parts.name, "rc1");
    assert_eq!(parts.parent, r"D:\Mario\Orca\CopyPaste\");
}

#[test]
fn a_trailing_separator_does_not_eat_the_name() {
    let parts = parts_of(r"D:\Mario\Pictures\Screenshots\").expect("one");
    assert_eq!(parts.name, "Screenshots");
    assert_eq!(parts.parent, r"D:\Mario\Pictures\");
}

#[test]
fn a_network_share_keeps_its_two_leading_slashes() {
    let parts = parts_of(r"\\servidor\legal\Contratos 2026").expect("one");
    assert_eq!(parts.name, "Contratos 2026");
    assert_eq!(parts.parent, r"\\servidor\legal\");
}

#[test]
fn a_unix_path_reads_the_same_way() {
    let parts = parts_of("/home/mario/Documentos").expect("one");
    assert_eq!(parts.name, "Documentos");
    assert_eq!(parts.parent, "/home/mario/");
}

#[test]
fn a_bare_name_has_no_folder_above_it() {
    let parts = parts_of("Descargas").expect("one");
    assert_eq!(parts.name, "Descargas");
    assert_eq!(parts.parent, "");
}

#[test]
fn nothing_at_all_has_no_parts() {
    assert!(parts_of("").is_none());
    assert!(parts_of("   ").is_none(), "only trailing space is trimmed");
    assert!(parts_of("\\").is_none());
    assert!(parts_of("///").is_none());
}

#[test]
fn only_the_first_line_is_read_because_that_is_the_first_folder() {
    assert_eq!(parts_of("/uno/dos\n/tres/cuatro").expect("one").name, "dos");
}

#[test]
fn a_folder_nobody_counted_yet_says_nothing() {
    assert_eq!(said_in(None, false), "");
    assert_eq!(said_in(Some(&std::collections::HashMap::new()), false), "");
}

fn counted(seen: usize) -> std::collections::HashMap<String, String> {
    std::collections::HashMap::from([(ENTRIES.to_owned(), seen.to_string())])
}

#[test]
fn what_was_counted_is_said_in_both_tongues() {
    assert_eq!(said_in(Some(&counted(412)), false), "412 elementos");
    assert_eq!(said_in(Some(&counted(412)), true), "412 entries");
    assert_eq!(said_in(Some(&counted(1)), false), "1 elemento");
    assert_eq!(said_in(Some(&counted(1)), true), "1 entry");
    assert_eq!(said_in(Some(&counted(0)), false), "0 elementos");
}

#[test]
fn a_count_that_hit_the_ceiling_admits_it_is_a_floor() {
    let said = said_in(Some(&counted(UP_TO)), false);
    assert_eq!(said, format!("más de {UP_TO} elementos"));
    assert_eq!(
        said_in(Some(&counted(UP_TO)), true),
        format!("more than {UP_TO} entries")
    );
}

#[test]
fn a_value_that_is_not_a_number_is_ignored_rather_than_shown_raw() {
    let bad = std::collections::HashMap::from([(ENTRIES.to_owned(), "muchos".to_owned())]);
    assert_eq!(said_in(Some(&bad), false), "");
}

#[test]
fn counting_a_real_folder_sees_what_is_in_it() {
    let dir = std::env::temp_dir().join(format!("cp-folder-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("made");
    for at in 0..3 {
        std::fs::write(dir.join(format!("{at}.txt")), b"x").expect("wrote");
    }
    assert_eq!(counted_in(&dir), Some(3));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn counting_what_is_not_there_answers_nothing_instead_of_zero() {
    let nowhere = std::env::temp_dir().join("cp-folder-que-no-existe-jamas");
    assert_eq!(counted_in(&nowhere), None);
}
