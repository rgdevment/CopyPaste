use super::*;

#[test]
fn a_link_is_grouped_by_its_domain() {
    assert_eq!(
        key_of(Some(Kind::Link), "https://github.com/a/b"),
        "github.com"
    );
    assert_eq!(
        key_of(Some(Kind::Link), "https://www.GitHub.com/c"),
        "github.com",
        "two spellings of one site land in one group"
    );
}

#[test]
fn a_folder_is_grouped_by_its_unit() {
    assert_eq!(key_of(Some(Kind::Folder), r"D:\Mario\Orca"), "D:");
}

#[test]
fn a_kind_that_does_not_group_gets_an_empty_key() {
    assert_eq!(key_of(Some(Kind::Text), "hola"), "");
    assert_eq!(key_of(Some(Kind::Json), "{}"), "");
    assert_eq!(key_of(None, "lo que sea"), "");
}

#[test]
fn a_candidate_nobody_could_read_is_marked_so_it_is_not_tried_forever() {
    assert_eq!(key_of(Some(Kind::Link), "no-es-una-url"), UNKNOWN);
    assert_eq!(key_of(Some(Kind::Folder), "sin_unidad"), UNKNOWN);
    assert_ne!(
        key_of(Some(Kind::Link), "no-es-una-url"),
        "",
        "an empty key would be read as «not computed yet» and queued again"
    );
}

#[test]
fn what_is_not_a_real_group_draws_no_heading() {
    assert!(shown("github.com"));
    assert!(shown("D:"));
    assert!(!shown(""), "no group at all");
    assert!(
        !shown(UNKNOWN),
        "a group we could not name is not a heading"
    );
}
