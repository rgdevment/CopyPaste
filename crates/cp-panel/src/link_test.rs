use super::*;

fn of(text: &str) -> Parts {
    parts_of(text).expect("a link")
}

#[test]
fn a_plain_url_splits_into_domain_and_the_rest() {
    let parts = of("https://github.com/rgdevment/CopyPaste/pull/95");
    assert_eq!(parts.domain, "github.com");
    assert_eq!(parts.path, "/rgdevment/CopyPaste/pull/95");
}

#[test]
fn the_www_is_not_part_of_the_domain_anyone_recognises() {
    assert_eq!(of("https://www.github.com/x").domain, "github.com");
    assert_eq!(of("www.github.com").domain, "github.com");
}

#[test]
fn a_domain_is_folded_so_two_spellings_group_together() {
    assert_eq!(of("https://GitHub.COM/x").domain, "github.com");
}

#[test]
fn a_port_belongs_to_the_address_not_to_the_domain() {
    let parts = of("http://ejemplo.test:8080/panel");
    assert_eq!(parts.domain, "ejemplo.test");
    assert_eq!(parts.path, "/panel");
}

#[test]
fn credentials_in_the_url_do_not_become_the_domain() {
    assert_eq!(
        of("https://mario:clave@ejemplo.test/x").domain,
        "ejemplo.test"
    );
}

#[test]
fn a_domain_with_no_path_has_an_empty_one_rather_than_a_slash() {
    let parts = of("https://docs.rs");
    assert_eq!(parts.domain, "docs.rs");
    assert_eq!(parts.path, "");
}

#[test]
fn a_query_or_a_fragment_counts_as_the_path() {
    assert_eq!(of("https://ejemplo.test?a=1").path, "?a=1");
    assert_eq!(of("https://ejemplo.test#arriba").path, "#arriba");
}

#[test]
fn any_scheme_is_cut_the_same_way() {
    assert_eq!(of("ftp://archivos.test/pub").domain, "archivos.test");
    assert_eq!(of("ejemplo.test/sin/esquema").domain, "ejemplo.test");
}

#[test]
fn what_has_no_dot_is_not_a_domain() {
    assert!(parts_of("localhost:3000").is_none());
    assert!(parts_of("solo-texto").is_none());
    assert!(parts_of("").is_none());
    assert!(parts_of("https://").is_none());
}

#[test]
fn only_the_first_line_is_read_because_that_is_the_first_link() {
    assert_eq!(
        of("https://uno.test/a\nhttps://dos.test/b").domain,
        "uno.test"
    );
}

#[test]
fn surrounding_space_does_not_reach_the_domain() {
    assert_eq!(of("  https://ejemplo.test/x  ").domain, "ejemplo.test");
}
