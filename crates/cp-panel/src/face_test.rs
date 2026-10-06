use super::*;

fn seen<'a>(kind: Option<Kind>, body: &'a str) -> Seen<'a> {
    Seen {
        kind,
        found: false,
        thumb: false,
        paints: false,
        body,
        keys: "",
    }
}

#[test]
fn a_search_hit_a_picture_and_a_colour_win_over_the_kind() {
    let mut one = seen(Some(Kind::Code), "fn main() {}");
    one.found = true;
    assert_eq!(Face::of(&one), Face::Found);
    one.thumb = true;
    assert_eq!(
        Face::of(&one),
        Face::FoundThumb,
        "a picture found by its words keeps its thumbnail"
    );
    one.found = false;
    one.found = false;
    one.thumb = true;
    assert_eq!(Face::of(&one), Face::Thumb);
    one.thumb = false;
    one.paints = true;
    assert_eq!(Face::of(&one), Face::Paint);
}

#[test]
fn a_short_text_takes_one_line_and_a_long_one_two() {
    assert_eq!(
        Face::of(&seen(Some(Kind::Text), "git push")),
        Face::Words(1)
    );
    let long = "Hola equipo: el despliegue del jueves queda a las 10:00, revisen la lista.";
    assert_eq!(Face::of(&seen(Some(Kind::Text), long)), Face::Words(2));
    assert_eq!(Face::of(&seen(None, "4")), Face::Words(1));
}

#[test]
fn code_and_tokens_open_a_block_as_tall_as_their_first_two_lines() {
    assert_eq!(
        Face::of(&seen(Some(Kind::Code), "let a = 1;")),
        Face::Block(1)
    );
    assert_eq!(
        Face::of(&seen(Some(Kind::Code), "let a = 1;\n\n  let b = 2;")),
        Face::Block(2)
    );
    assert_eq!(
        Face::of(&seen(Some(Kind::Token), "eyJhbGciOi")),
        Face::Block(1)
    );
}

#[test]
fn the_other_kinds_have_a_face_of_their_own() {
    let mut json = seen(Some(Kind::Json), "{}");
    json.keys = "severity · time";
    assert_eq!(Face::of(&json), Face::Keys(1));
    json.keys = "severity · time · serviceContext · req · traceID · spanID · traceSource";
    assert_eq!(Face::of(&json), Face::Keys(2));
    assert_eq!(Face::of(&seen(Some(Kind::Link), "https://a.b")), Face::Link);
    assert_eq!(Face::of(&seen(Some(Kind::Folder), "C:\\x")), Face::File);
    assert_eq!(Face::of(&seen(Some(Kind::Audio), "C:\\a.mp3")), Face::Media);
}

#[test]
fn every_face_is_as_tall_as_its_parts() {
    assert_eq!(
        Face::Words(1).shut_px(),
        9.0 + 22.0 + 6.0 + 19.0 + 10.0 + 6.0
    );
    assert_eq!(Face::Words(2).shut_px() - Face::Words(1).shut_px(), LINE);
    assert_eq!(Face::Block(2).body_px(), BLOCK_PAD + 2.0 * MONO_LINE);
    assert_eq!(Face::Thumb.body_px(), THUMB);
    assert_eq!(Face::Thumb.as_str(), "thumb");
    assert_eq!(Face::Keys(2).lines(), 2);
    assert_eq!(Face::Link.lines(), 1);
}

#[test]
fn the_opening_skips_blank_lines_and_drops_the_indent_they_share() {
    let (one, two) = opening_of("\n    if ready {\n        go();\n    }\n");
    assert_eq!(one, "if ready {");
    assert_eq!(two, "    go();");
    assert_eq!(opening_of("solo"), ("solo".to_owned(), String::new()));
    assert_eq!(opening_of(""), (String::new(), String::new()));
}

#[test]
fn the_lines_are_counted_only_when_more_than_are_shown() {
    assert_eq!(lines_said(1, 1, false), "");
    assert_eq!(lines_said(2, 2, false), "");
    assert_eq!(lines_said(12, 2, false), "12 líneas");
    assert_eq!(lines_said(12, 2, true), "12 lines");
}

#[test]
fn the_aside_joins_what_is_there_and_skips_what_is_not() {
    assert_eq!(aside_said(&["", " firefox ", "×3", ""]), "firefox · ×3");
    assert_eq!(aside_said(&[]), "");
}

#[test]
fn an_open_text_shows_up_to_twelve_lines_and_says_when_there_is_more() {
    let short = opened_of(Face::Words(1), false, "hola", false);
    assert_eq!((short.lines, short.more.as_str()), (1, ""));
    let long = "una línea de texto\n".repeat(20);
    let opened = opened_of(Face::Words(2), false, &long, false);
    assert_eq!(opened.lines, 12);
    assert_eq!(opened.more, "20 líneas · mostrando el comienzo");
}

#[test]
fn an_open_json_is_formatted_and_stops_at_six_lines() {
    let opened = opened_of(
        Face::Keys(1),
        true,
        r#"{"a":1,"b":{"c":2},"d":[1,2],"e":"x"}"#,
        false,
    );
    assert_eq!(opened.lines, 6);
    assert!(opened.text.starts_with("{\n  \"a\": 1,"), "{}", opened.text);
    assert_eq!(opened.text.lines().count(), 6);
    assert!(opened.more.starts_with("… "), "{}", opened.more);
    let broken = opened_of(Face::Keys(1), true, "{no es json", false);
    assert_eq!(broken.text, "{no es json");
}

#[test]
fn open_code_keeps_its_lines_and_counts_the_rest() {
    let code = (1..=15)
        .map(|n| format!("line {n}"))
        .collect::<Vec<_>>()
        .join("\n");
    let opened = opened_of(Face::Block(2), false, &code, true);
    assert_eq!(opened.lines, 12);
    assert_eq!(opened.more, "… 3 more lines");
    let one = opened_of(Face::Block(1), false, "only", false);
    assert_eq!((one.lines, one.more.as_str()), (1, ""));
}

#[test]
fn an_open_card_is_as_tall_as_its_body_its_note_and_its_keys() {
    let quiet = Opened {
        text: String::new(),
        lines: 3,
        more: String::new(),
    };
    assert_eq!(open_body_px(Face::Words(2), &quiet, 0, 0.0), 3.0 * LINE);
    let noted = Opened {
        text: String::new(),
        lines: 3,
        more: "…".into(),
    };
    assert_eq!(
        open_body_px(Face::Block(2), &noted, 0, 0.0),
        BLOCK_PAD + 3.0 * MONO_LINE + GAP + NOTE
    );
    assert_eq!(
        open_body_px(Face::Block(1), &quiet, 7, 0.0),
        BLOCK_PAD + 7.0 * MONO_LINE
    );
    assert_eq!(open_body_px(Face::File, &quiet, 0, 40.0), 40.0);
    assert_eq!(
        open_px(10.0) - 10.0,
        TOP + META + GAP + GAP + KEYS_ROW + BOTTOM + BETWEEN
    );
}

#[test]
fn a_colour_sits_on_a_single_row_while_closed() {
    assert_eq!(Face::Paint.shut_px(), TOP + META + BOTTOM + BETWEEN);
    assert_eq!(Face::FoundThumb.body_px(), THUMB);
    assert_eq!(Face::FoundThumb.as_str(), "found-thumb");
}
