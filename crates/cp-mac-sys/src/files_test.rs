use super::*;

#[test]
fn a_file_url_becomes_the_path_it_names() {
    assert_eq!(
        path_of("file:///tmp/cp%20a9/canci%C3%B3n.png"),
        "/tmp/cp a9/canción.png"
    );
    assert_eq!(path_of("file:///tmp/plain.txt"), "/tmp/plain.txt");
    assert_eq!(
        path_of("/already/a/path"),
        "/already/a/path",
        "with no scheme it is left as it arrived"
    );
}

#[test]
fn a_percent_that_is_not_an_escape_is_kept() {
    assert_eq!(percent_decoded("100%"), "100%");
    assert_eq!(percent_decoded("%4"), "%4");
    assert_eq!(percent_decoded("%zz"), "%zz");
    assert_eq!(percent_decoded("a%41"), "aA");
    assert_eq!(percent_decoded("%41"), "A");
    assert_eq!(percent_decoded(""), "");
}

#[test]
fn bytes_that_do_not_form_utf8_do_not_panic() {
    assert_eq!(percent_decoded("%ff%fe"), "\u{fffd}\u{fffd}");
}

#[test]
fn what_is_not_there_is_neither_opened_nor_revealed() {
    let ghost = Path::new("/tmp/cp-never-exists/file.txt");
    assert!(!open(ghost), "opening what is not there launches nothing");
    assert!(!reveal(ghost), "nor does it activate Finder");
}

#[test]
fn what_would_run_when_opened_is_only_revealed() {
    for name in ["setup.pkg", "run.SH", "Thing.app", "x.command", "a.scpt"] {
        assert!(runs_when_opened(Path::new(name)), "{name}");
    }
    for name in ["notes.txt", "photo.png", "no-extension", "file.shtml"] {
        assert!(!runs_when_opened(Path::new(name)), "{name}");
    }
    assert!(
        !open(Path::new("/tmp/cp-never-exists/danger.sh")),
        "revealing what is not there also says no"
    );
}

#[test]
fn a_path_with_spaces_and_accents_becomes_a_file_url() {
    let url = url_of(Path::new("/tmp/cp a9/canción.png"));
    let text = url
        .absoluteString()
        .map(|s| s.to_string())
        .unwrap_or_default();
    assert_eq!(
        text, "file:///tmp/cp%20a9/cancio%CC%81n.png",
        "NSURL decomposes the accent the way the file system does"
    );
}
