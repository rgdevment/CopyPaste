use super::*;

#[test]
fn every_class_keeps_the_name_the_database_stores() {
    for (kind, name) in [
        (Kind::Text, "text"),
        (Kind::Code, "code"),
        (Kind::Json, "json"),
        (Kind::Link, "link"),
        (Kind::Email, "email"),
        (Kind::Phone, "phone"),
        (Kind::Color, "color"),
        (Kind::Ip, "ip"),
        (Kind::Uuid, "uuid"),
        (Kind::Image, "image"),
        (Kind::File, "file"),
        (Kind::Folder, "folder"),
        (Kind::Audio, "audio"),
        (Kind::Video, "video"),
    ] {
        assert_eq!(
            kind.as_str(),
            name,
            "the name travels straight into the kind column"
        );
    }
}

#[test]
fn an_address_needs_every_piece_at_once() {
    assert!(is_email("a@b.co"));
    assert!(!is_email("@ejemplo.test"), "no user");
    assert!(!is_email("a@b@c.co"), "two at-signs");
    assert!(!is_email("a@.co"), "no domain name");
    assert!(!is_email("a@b.c"), "a one-letter top-level domain");
    assert!(!is_email("a@b.c1"), "a top-level domain with a digit");
    assert!(!is_email("a b@c.co"), "a space in the user part");
    assert!(!is_email("a@b c.co"), "a space in the domain");
    assert!(!is_email("a@bc"), "no dot");
}

#[test]
fn a_scheme_on_its_own_is_not_an_address() {
    assert!(is_url("http://a"));
    assert!(!is_url("http://"), "the whole scheme and nothing else");
    assert!(!is_url("https://"));
}

#[test]
fn a_colour_needs_length_and_digits_at_once() {
    assert!(is_color("#FF8800"));
    assert!(
        !is_color("#GGG"),
        "three characters that are not hexadecimal"
    );
    assert!(!is_color("rgb(1, 2)"), "missing one component");
    assert!(
        !is_color("rgb(a, b, c)"),
        "three components that are not numbers"
    );
}

#[test]
fn a_uuid_needs_shape_and_digits_at_once() {
    assert!(is_uuid("6ba7b810-9dad-11d1-80b4-00c04fd430c8"));
    assert!(!is_uuid("1-2-3-4-5"), "five groups of the wrong length");
    assert!(
        !is_uuid("zzzzzzzz-9dad-11d1-80b4-00c04fd430c8"),
        "the right length with characters that are not hexadecimal"
    );
}

#[test]
fn a_number_needs_shape_and_a_reason_to_be_a_phone() {
    assert!(is_phone("+1234567"), "the international prefix is enough");
    assert!(is_phone("(123) 4567"), "the area parenthesis is enough");
    assert!(is_phone("123456789"), "nine digits are enough on their own");
    assert!(
        !is_phone("1234567"),
        "seven loose digits are not a phone number"
    );
    assert!(!is_phone("(123) 456-7890 ñ"), "one letter breaks the shape");
}

#[test]
fn nothing_at_all_is_not_an_object() {
    assert!(
        !is_json(""),
        "without a first character there is no delimiter"
    );
    assert!(is_json("{}"));
}

#[test]
fn a_quote_inside_a_string_does_not_close_it() {
    assert!(balanced(r#"{"\""}"#), "the escaped quote stays inside");
    assert!(!balanced(r#"{"a": {}"#), "a brace is left unclosed");
    assert!(!balanced(r#"{"a"#), "the string was left open");
}

#[test]
fn one_marker_alone_is_not_code() {
    assert!(looks_like_code("const a = 1;\n  return a;"), "two markers");
    assert!(!looks_like_code("const"), "not even one complete marker");
    assert!(
        !looks_like_code("const x"),
        "one marker, one line, no indent"
    );
    assert!(
        !looks_like_code("  const x"),
        "one indented marker, but a single line"
    );
    assert!(
        !looks_like_code("const x\nmás texto"),
        "one marker across two lines, neither indented"
    );
}
