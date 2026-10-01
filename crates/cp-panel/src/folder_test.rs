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
fn a_path_with_accents_and_emoji_splits_the_same_way_as_any_other() {
    let parts = parts_of(r"D:\Descargas\Informe final 📎 revisión.pdf").expect("one");
    assert_eq!(parts.name, "Informe final 📎 revisión.pdf");
    assert_eq!(parts.parent, r"D:\Descargas\");
}

#[test]
fn counting_a_path_longer_than_the_windows_ceiling_answers_nothing_not_a_panic() {
    let long = std::env::temp_dir().join(format!(
        "cp-folder-long-{}-{}",
        std::process::id(),
        "x".repeat(300)
    ));
    assert_eq!(counted_in(&long), None);
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
fn counting_a_real_empty_folder_answers_zero_not_nothing() {
    let dir = std::env::temp_dir().join(format!("cp-folder-empty-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("made");
    assert_eq!(counted_in(&dir), Some(0));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_real_folder_past_the_ceiling_stops_counting_at_it() {
    let dir = std::env::temp_dir().join(format!("cp-folder-many-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("made");
    for at in 0..UP_TO + 200 {
        std::fs::write(dir.join(format!("{at}.txt")), b"x").expect("wrote");
    }
    assert_eq!(
        counted_in(&dir),
        Some(UP_TO),
        "counting a real folder with thousands of entries stops at the same ceiling as the synthetic test"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn counting_what_is_not_there_answers_nothing_instead_of_zero() {
    let nowhere = std::env::temp_dir().join("cp-folder-que-no-existe-jamas");
    assert_eq!(counted_in(&nowhere), None);
}

#[test]
fn counting_a_file_instead_of_a_folder_answers_nothing() {
    let dir = std::env::temp_dir().join(format!("cp-folder-is-a-file-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("made");
    let file = dir.join("not-a-folder.txt");
    std::fs::write(&file, b"x").expect("wrote");
    assert_eq!(counted_in(&file), None);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_preview_of_only_whitespace_or_only_emoji_has_no_unit() {
    assert_eq!(unit_of("   "), "");
    assert_eq!(unit_of("🎉📎"), "");
}

#[test]
fn a_drive_letter_is_the_unit_a_folder_belongs_to() {
    assert_eq!(unit_of(r"D:\Mario\Orca"), "D:");
    assert_eq!(unit_of(r"c:\usuarios"), "C:", "the letter is folded up");
}

#[test]
fn a_share_is_grouped_by_its_server_not_by_its_folders() {
    assert_eq!(unit_of(r"\\servidor\legal\Contratos"), r"\\servidor");
    assert_eq!(unit_of(r"\\servidor"), r"\\servidor");
}

#[test]
fn a_unix_root_is_one_single_unit() {
    assert_eq!(unit_of("/home/mario/Documentos"), "/");
}

#[test]
fn what_has_no_unit_gets_no_group_instead_of_a_wrong_one() {
    assert_eq!(unit_of(""), "");
    assert_eq!(unit_of("Descargas"), "");
    assert_eq!(unit_of(r"\\"), "");
    assert_eq!(
        unit_of("https://ejemplo.test/x"),
        "",
        "a scheme is not a drive"
    );
}
