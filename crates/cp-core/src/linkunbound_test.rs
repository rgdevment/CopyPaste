use super::{SCHEME, asked_for, encoded, opened_by, opens_the_web};

fn decoded(said: &str) -> String {
    let mut bytes = Vec::with_capacity(said.len());
    let mut chars = said.bytes();
    while let Some(one) = chars.next() {
        if one != b'%' {
            bytes.push(one);
            continue;
        }
        let high = chars.next().expect("half a pair");
        let low = chars.next().expect("half a pair");
        let pair = [high, low];
        let pair = std::str::from_utf8(&pair).expect("ascii");
        bytes.push(u8::from_str_radix(pair, 16).expect("two hex digits"));
    }
    String::from_utf8(bytes).expect("what went in was text")
}

fn carried(said: &str) -> String {
    let form = asked_for(said, true).expect("a web link with linkunbound here");
    let url = form
        .strip_prefix(&format!("{SCHEME}://open?url="))
        .expect("the shape linkunbound answers to");
    decoded(url)
}

#[test]
fn a_link_with_every_character_that_cuts_it_survives_the_round_trip() {
    let said = "https://example.com/a?b=1&c=2#top+more/deep";
    assert_eq!(carried(said), said);
}

#[test]
fn a_link_with_letters_that_are_not_ascii_survives_the_round_trip() {
    let said = "https://ejemplo.cl/camión?qué=sí&año=2026#sección";
    assert_eq!(carried(said), said);
}

#[test]
fn nothing_that_would_cut_the_link_is_left_unencoded() {
    let said = encoded("https://a.b/c?d=1&e=2#f+g");
    for cutter in [':', '/', '?', '&', '=', '#', '+'] {
        assert!(
            !said.contains(cutter),
            "a bare {cutter} ends the url for whoever reads it: {said}"
        );
    }
}

#[test]
fn a_plus_never_arrives_as_a_space() {
    assert_eq!(encoded("a+b"), "a%2Bb");
}

#[test]
fn only_the_letters_and_digits_travel_as_themselves() {
    assert_eq!(encoded("aZ09"), "aZ09");
    assert_eq!(encoded("-_.~"), "%2D%5F%2E%7E");
}

#[test]
fn the_web_is_http_and_https_and_nothing_else() {
    assert!(opens_the_web("http://example.com"));
    assert!(opens_the_web("https://example.com"));
    assert!(opens_the_web("HTTPS://EXAMPLE.COM"));
    assert!(!opens_the_web("mailto:yo@ejemplo.cl"));
    assert!(!opens_the_web("file:///C:/temp/a.txt"));
    assert!(!opens_the_web("linkunbound://open?url=x"));
    assert!(!opens_the_web("tisty://task/1"));
    assert!(!opens_the_web(""));
    assert!(
        !opens_the_web("https://"),
        "a scheme with nothing behind it is not a link anyone can open"
    );
}

#[test]
fn a_link_that_is_not_the_web_is_opened_by_whoever_already_opened_it() {
    assert_eq!(asked_for("mailto:yo@ejemplo.cl", true), None);
    assert_eq!(asked_for("file:///C:/temp/a.txt", true), None);
}

#[test]
fn without_linkunbound_here_the_real_link_is_what_travels() {
    assert_eq!(asked_for("https://example.com", false), None);
}

#[test]
fn with_linkunbound_here_a_web_link_takes_its_shape() {
    assert_eq!(
        asked_for("https://example.com/a", true).as_deref(),
        Some("linkunbound://open?url=https%3A%2F%2Fexample%2Ecom%2Fa")
    );
}

#[test]
fn a_link_that_arrives_with_spaces_in_front_is_not_encoded_with_them() {
    let form = asked_for("  https://example.com/a", true).expect("a web link");
    assert!(
        !form.contains("%20"),
        "the spaces were never part of the link: {form}"
    );
    assert_eq!(carried("  https://example.com/a"), "https://example.com/a");
}

fn tried(url: &str, here: bool, works: bool) -> (bool, Vec<String>) {
    let mut asked = Vec::new();
    let done = opened_by(url, here, |one| {
        asked.push(one.to_owned());
        works || !one.starts_with(SCHEME)
    });
    (done, asked)
}

#[test]
fn with_linkunbound_here_the_real_link_is_never_opened_twice() {
    let (done, asked) = tried("https://example.com/a", true, true);
    assert!(done);
    assert_eq!(asked.len(), 1);
    assert!(asked[0].starts_with(SCHEME), "{asked:?}");
}

#[test]
fn if_linkunbound_is_there_but_will_not_open_the_real_link_still_does() {
    let (done, asked) = tried("https://example.com/a", true, false);
    assert!(done, "the person asked for a link and must get the link");
    assert_eq!(asked.len(), 2);
    assert!(asked[0].starts_with(SCHEME));
    assert_eq!(asked[1], "https://example.com/a");
}

#[test]
fn without_linkunbound_nothing_but_the_real_link_is_ever_tried() {
    let (done, asked) = tried("https://example.com/a", false, false);
    assert!(done);
    assert_eq!(asked, vec!["https://example.com/a".to_owned()]);
}

#[test]
fn a_link_that_is_not_the_web_goes_straight_out_even_with_linkunbound_here() {
    let (_, asked) = tried("mailto:yo@ejemplo.cl", true, true);
    assert_eq!(asked, vec!["mailto:yo@ejemplo.cl".to_owned()]);
}
