use super::*;

#[test]
fn every_class_goes_to_its_name_and_back() {
    for kind in Kind::ALL {
        assert_eq!(Kind::from_name(kind.as_str()), Some(kind));
    }
    let mut names: Vec<&str> = Kind::ALL.iter().map(|kind| kind.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), 15, "fifteen classes, fifteen distinct names");
}

#[test]
fn a_name_that_is_not_a_class_is_nobody() {
    assert_eq!(Kind::from_name("Text"), None, "the name is exact");
    assert_eq!(Kind::from_name("imagen"), None);
    assert_eq!(Kind::from_name(""), None);
}

#[test]
fn the_classes_2x_already_recognised() {
    assert_eq!(classify_text("alguien@ejemplo.test"), Kind::Email);
    assert_eq!(classify_text("#FF8800"), Kind::Color);
    assert_eq!(classify_text("rgb(12, 34, 56)"), Kind::Color);
    assert_eq!(classify_text("hsla(120, 50%, 50%, 0.5)"), Kind::Color);
    assert_eq!(classify_text("192.168.1.1"), Kind::Ip);
    assert_eq!(
        classify_text("7ab3f6de-1c4b-4f5e-8a2d-9f0e1b2c3d4e"),
        Kind::Uuid
    );
    assert_eq!(classify_text("+34 600 123 456"), Kind::Phone);
    assert_eq!(classify_text(r#"{"a": 1}"#), Kind::Json);
}

#[test]
fn a_token_is_its_own_class_and_wins_over_the_rest() {
    assert_eq!(
        classify_text("ghp_not_a_real_token_for_tests_0000000000"),
        Kind::Token
    );
    assert_eq!(
        classify_text("  sk-not-a-real-key-for-tests-0123456789\n"),
        Kind::Token,
        "trimmed, like everything else"
    );
    assert_eq!(classify_text("sk-corto"), Kind::Text);
}

#[test]
fn the_classes_2x_had_a_type_for_but_never_assigned() {
    assert_eq!(classify_text("https://ejemplo.test/ruta?q=1"), Kind::Link);
    assert_eq!(classify_file("cancion.mp3", false), Kind::Audio);
    assert_eq!(classify_file("pelicula.MKV", false), Kind::Video);
    assert_eq!(classify_file("foto.heic", false), Kind::Image);
    assert_eq!(classify_file("Documentos", true), Kind::Folder);
    assert_eq!(classify_file("informe.pdf", false), Kind::File);
}

#[test]
fn code_is_recognised_which_2x_never_did() {
    assert_eq!(
        classify_text("fn main() {\n    println!(\"hola\");\n}"),
        Kind::Code
    );
    assert_eq!(
        classify_text("def suma(a, b):\n    return a + b"),
        Kind::Code
    );
    assert_eq!(
        classify_text("SELECT * FROM tabla WHERE id = 1;\nINSERT INTO otra VALUES (1);"),
        Kind::Code
    );
}

#[test]
fn ordinary_prose_is_never_code() {
    for prose in [
        "Esto es una frase normal y corriente.",
        "Nos vemos mañana si puedes",
        "La reunión es a las cinco, en la sala grande",
        "return",
    ] {
        assert_eq!(classify_text(prose), Kind::Text, "«{prose}»");
    }
}

#[test]
fn something_that_only_looks_like_json_is_not_json() {
    for fake in [
        "{esto no es json",
        "[1, 2, 3",
        "{\"sin cerrar\": \"comilla}",
    ] {
        assert_ne!(classify_text(fake), Kind::Json, "«{fake}»");
    }
    assert_eq!(classify_text("[1, 2, 3]"), Kind::Json);
    assert_eq!(classify_text(r#"{"anidado": {"a": [1, 2]}}"#), Kind::Json);
}

#[test]
fn the_edges_of_each_class() {
    assert_eq!(classify_text("#FFF"), Kind::Color);
    assert_eq!(classify_text("#FF8800AA"), Kind::Color);
    assert_ne!(classify_text("#GGGGGG"), Kind::Color);
    assert_ne!(classify_text("256.1.1.1"), Kind::Ip);
    assert_ne!(classify_text("1.2.3"), Kind::Ip);
    assert_ne!(classify_text("no-es-un-uuid-de-verdad"), Kind::Uuid);
    assert_ne!(classify_text("arroba sin@"), Kind::Email);
    assert_ne!(classify_text("12345"), Kind::Phone);
}

#[test]
fn the_four_digit_hex_shorthand_with_alpha_is_not_recognised_as_a_color() {
    assert_ne!(
        classify_text("#f0a5"),
        Kind::Color,
        "is_color only accepts hex lengths of 3, 6 or 8, so the CSS #RGBA shorthand \
         with its own alpha channel falls through to plain text"
    );
}

#[test]
fn rgb_accepts_channel_values_far_outside_the_valid_range() {
    assert_eq!(
        classify_text("rgb(300, -5, 999)"),
        Kind::Color,
        "is_color only checks that each part parses as a float, with no bound on 0..=255"
    );
    assert_eq!(
        classify_text("hsl(1e9, 1e9%, 1e9%)"),
        Kind::Color,
        "scientific notation parses fine too, however meaningless the hue is"
    );
}

#[test]
fn several_lines_never_become_a_single_line_class() {
    assert_eq!(
        classify_text("alguien@ejemplo.test\notra línea"),
        Kind::Text
    );
    assert_eq!(classify_text("#FF8800\n#00FF88"), Kind::Text);
}

#[test]
fn empty_and_blank_are_plain_text() {
    assert_eq!(classify_text(""), Kind::Text);
    assert_eq!(classify_text("   \n  "), Kind::Text);
}

#[test]
fn every_kind_has_a_stable_name_for_the_database() {
    for kind in [
        Kind::Text,
        Kind::Code,
        Kind::Json,
        Kind::Link,
        Kind::Email,
        Kind::Phone,
        Kind::Color,
        Kind::Ip,
        Kind::Uuid,
        Kind::Image,
        Kind::File,
        Kind::Folder,
        Kind::Audio,
        Kind::Video,
    ] {
        assert!(!kind.as_str().is_empty());
        assert_eq!(kind.as_str(), kind.as_str().to_lowercase());
    }
}

#[test]
fn an_uppercase_uuid_is_still_a_uuid() {
    assert_eq!(
        classify_text("6BA7B810-9DAD-11D1-80B4-00C04FD430C8"),
        Kind::Uuid
    );
}

#[test]
fn a_byte_order_mark_hides_the_json_from_its_own_classifier() {
    assert_ne!(
        classify_text("\u{FEFF}{\"a\":1}"),
        Kind::Json,
        "a JSON file saved with a BOM, as Windows editors often do, is no longer \
         recognised as JSON because the first byte is no longer the opening brace"
    );
    assert_eq!(
        classify_text("{\"a\":1}"),
        Kind::Json,
        "without the BOM the very same object is read fine, so the BOM is the whole difference"
    );
}

#[test]
fn a_domain_written_in_its_own_alphabet_is_not_read_as_an_address() {
    assert_ne!(
        classify_text("alguien@café.test"),
        Kind::Email,
        "the domain check only accepts ASCII, so an accented domain is not an email here"
    );
}

#[test]
fn an_extension_after_the_digits_is_not_read_as_a_phone_number() {
    assert_ne!(
        classify_text("+1 555 123 4567 x123"),
        Kind::Phone,
        "the trailing «x123» extension is not a digit, a space, or any of «()+-.», so it \
         breaks the shape check"
    );
}
