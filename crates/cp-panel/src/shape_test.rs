use super::*;

#[test]
fn what_is_not_json_has_no_shape() {
    assert!(shape_of("").is_none());
    assert!(shape_of("   ").is_none());
    assert!(shape_of("hola").is_none());
    assert!(shape_of("\"solo una cadena\"").is_none());
    assert!(shape_of("42").is_none());
}

#[test]
fn an_object_gives_its_first_level_keys_in_order() {
    let shape =
        shape_of(r#"{"id":4417,"cliente":{"nombre":"Acme"},"total":148900}"#).expect("json");
    assert_eq!(shape.root, Root::Object);
    assert_eq!(shape.named, vec!["id", "cliente", "total"]);
    assert_eq!(shape.counted, 3);
    assert!(shape.whole);
}

#[test]
fn keys_deeper_down_are_not_counted_as_first_level() {
    let shape = shape_of(r#"{"a":{"b":{"c":1}},"d":[{"e":2}]}"#).expect("json");
    assert_eq!(shape.named, vec!["a", "d"]);
    assert_eq!(shape.counted, 2);
    assert_eq!(shape.deep, 3, "tres llaves abiertas a la vez, no cuatro");
}

#[test]
fn an_indented_object_reads_the_same_as_a_flat_one() {
    let flat = shape_of(r#"{"annotations":[1,2],"id":7}"#).expect("json");
    let pretty =
        shape_of("{\n  \"annotations\": [\n    1,\n    2\n  ],\n  \"id\": 7\n}").expect("json");
    assert_eq!(flat.named, pretty.named);
    assert_eq!(flat.counted, pretty.counted);
    assert_eq!(flat.deep, pretty.deep);
    assert_eq!(flat.whole, pretty.whole);
}

#[test]
fn an_array_counts_its_elements() {
    assert_eq!(shape_of("[]").expect("json").counted, 0);
    assert_eq!(shape_of("[1]").expect("json").counted, 1);
    assert_eq!(shape_of("[1,2,3]").expect("json").counted, 3);
    let objects = shape_of(r#"[{"at":1},{"at":2}]"#).expect("json");
    assert_eq!(objects.root, Root::Array);
    assert_eq!(objects.counted, 2);
    assert!(objects.named.is_empty(), "an array has no keys of its own");
}

#[test]
fn a_text_cut_in_the_middle_says_so_and_keeps_what_it_read() {
    let shape = shape_of(r#"{"id":4417,"cliente":{"nombre":"Ac"#).expect("json");
    assert!(!shape.whole, "it never closed");
    assert_eq!(shape.named, vec!["id", "cliente"]);
    assert_eq!(shape.counted, 2, "only what the text allowed");
}

#[test]
fn a_cut_array_still_counts_the_element_it_was_reading() {
    let shape = shape_of(r#"[{"at":1},{"at":2},{"at"#).expect("json");
    assert!(!shape.whole);
    assert_eq!(shape.counted, 3);
}

#[test]
fn a_brace_inside_a_string_is_not_a_level() {
    let shape = shape_of(r#"{"plantilla":"{{nombre}}","otra":"]["}"#).expect("json");
    assert_eq!(shape.named, vec!["plantilla", "otra"]);
    assert_eq!(
        shape.deep, 1,
        "nothing nested, only text that looks like it"
    );
    assert!(shape.whole);
}

#[test]
fn an_escaped_quote_does_not_end_the_string() {
    let shape = shape_of(r#"{"dijo":"ella dijo \"hola\"","luego":2}"#).expect("json");
    assert_eq!(shape.named, vec!["dijo", "luego"]);
    assert_eq!(shape.counted, 2);
    assert!(shape.whole);
}

#[test]
fn a_colon_inside_a_string_does_not_make_a_key() {
    let shape = shape_of(r#"{"url":"https://ejemplo.test:8080/x"}"#).expect("json");
    assert_eq!(shape.named, vec!["url"]);
    assert_eq!(shape.counted, 1);
}

#[test]
fn a_key_with_an_accent_or_an_emoji_survives_whole() {
    let shape = shape_of(r#"{"año":2026,"día":"martes","🔑":1}"#).expect("json");
    assert_eq!(shape.named, vec!["año", "día", "🔑"]);
}

#[test]
fn only_the_first_handful_of_keys_is_kept_but_all_are_counted() {
    let many: String = (0..30)
        .map(|at| format!("\"k{at}\":{at}"))
        .collect::<Vec<_>>()
        .join(",");
    let shape = shape_of(&format!("{{{many}}}")).expect("json");
    assert_eq!(shape.counted, 30);
    assert_eq!(shape.named.len(), NAMES_UP_TO);
    assert_eq!(shape.named[0], "k0");
}

#[test]
fn trailing_space_after_the_close_is_still_whole() {
    assert!(shape_of("{\"a\":1}  \n").expect("json").whole);
}

#[test]
fn what_the_row_says_about_an_object_it_read_whole() {
    let said = said_of(r#"{"id":1,"cliente":{"plan":"pro"}}"#, false).expect("json");
    assert_eq!(said.root, "{ }");
    assert_eq!(said.counted, "2 claves · 2 niveles");
    assert_eq!(said.keys, "id · cliente");
}

#[test]
fn what_the_row_says_when_the_text_was_cut() {
    let said = said_of(r#"{"id":1,"cliente":{"plan":"pr"#, false).expect("json");
    assert_eq!(said.counted, "más de 2 claves · 2 niveles");
    assert_eq!(
        said_of(r#"{"id":1,"c"#, true).expect("json").counted,
        "more than 1 key"
    );
}

#[test]
fn one_of_something_is_said_in_the_singular() {
    assert_eq!(
        said_of(r#"{"solo":1}"#, false).expect("json").counted,
        "1 clave"
    );
    assert_eq!(said_of("[7]", false).expect("json").counted, "1 elemento");
    assert_eq!(
        said_of(r#"{"solo":1}"#, true).expect("json").counted,
        "1 key"
    );
    assert_eq!(said_of("[7]", true).expect("json").counted, "1 element");
}

#[test]
fn a_flat_object_does_not_mention_levels() {
    let said = said_of(r#"{"a":1,"b":2}"#, false).expect("json");
    assert_eq!(said.counted, "2 claves", "one level is not worth saying");
}

#[test]
fn an_array_is_counted_in_elements_and_shows_no_keys() {
    let said = said_of(r#"[{"at":1},{"at":2},{"at":3}]"#, false).expect("json");
    assert_eq!(said.root, "[ ]");
    assert_eq!(said.counted, "3 elementos · 2 niveles");
    assert_eq!(said.keys, "");
}

#[test]
fn what_is_not_json_says_nothing_at_all() {
    assert!(said_of("hola", false).is_none());
}
