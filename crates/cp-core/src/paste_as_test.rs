use super::*;

fn text_of(kind: Kind, text: &str) -> Content<'_> {
    Content {
        kind: Some(kind),
        text: Some(text.into()),
        ..Default::default()
    }
}

fn rendered(form: Form, content: &Content) -> String {
    match render(form, content).expect("renders") {
        Rendered::Text(text) => text,
        other => panic!("wasn't text: {other:?}"),
    }
}

#[test]
fn every_form_has_a_stable_name_that_comes_back() {
    for form in Form::ALL {
        assert_eq!(Form::from_name(form.as_str()), Some(form));
    }
    let mut names: Vec<&str> = Form::ALL.iter().map(|form| form.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), Form::ALL.len());
    assert_eq!(Form::from_name("Markdown"), None, "the name is exact");
}

#[test]
fn what_was_rendered_can_be_written_back_as_an_item() {
    let text = Rendered::Text("hola".into()).into_item();
    assert_eq!(text.formats[0].id, crate::item::SYNTHETIC_TEXT);
    assert_eq!(
        text.fingerprint(),
        crate::item::Item::plain("hola").fingerprint()
    );
    let jpeg = Rendered::Jpeg(vec![0xFF, 0xD8, 0xFF]).into_item();
    assert_eq!(jpeg.kind, Some(Kind::Image));
    assert_eq!(jpeg.formats[0].id, crate::item::SYNTHETIC_JPEG);
    assert!(matches!(
        jpeg.formats[0].payload,
        crate::item::Payload::Inline(ref bytes) if bytes == &[0xFF, 0xD8, 0xFF]
    ));
    let content = Content {
        text: Some("x".into()),
        ..Default::default()
    };
    let owned = Content {
        text: Some(String::from("propio").into()),
        ..content.clone()
    };
    assert_eq!(
        owned.text.as_deref(),
        Some("propio"),
        "the text can be borrowed or owned"
    );
}

#[test]
fn plain_text_is_offered_as_quote_and_case() {
    assert_eq!(
        forms_for(&text_of(Kind::Text, "hola")),
        vec![Form::TextQuote, Form::TextUpper, Form::TextLower]
    );
    assert!(
        forms_for(&Content::default()).is_empty(),
        "with no text there's nothing to offer"
    );
    let two_lines = text_of(
        Kind::Text,
        "una
otra",
    );
    assert_eq!(
        forms_for(&two_lines),
        vec![
            Form::CodeOneLine,
            Form::TextQuote,
            Form::TextUpper,
            Form::TextLower
        ],
        "joining lines is only offered when there is more than one"
    );
}

#[test]
fn a_quote_marks_every_line_and_keeps_the_empty_ones() {
    let content = text_of(
        Kind::Text,
        "una

otra",
    );
    assert_eq!(
        rendered(Form::TextQuote, &content),
        "> una
>
> otra"
    );
}

#[test]
fn case_forms_change_the_letters_and_nothing_else() {
    let content = text_of(Kind::Text, "Hola Ñandú");
    assert_eq!(rendered(Form::TextUpper, &content), "HOLA ÑANDÚ");
    assert_eq!(rendered(Form::TextLower, &content), "hola ñandú");
}

#[test]
fn a_file_is_also_offered_by_its_name_alone() {
    let content = Content {
        kind: Some(Kind::File),
        paths: vec![
            "C:\\Users\\Mario\\Documentos\\informe.pdf".into(),
            "/tmp/otro.txt".into(),
        ],
        ..Default::default()
    };
    assert_eq!(
        rendered(Form::FileName, &content),
        "informe.pdf
otro.txt"
    );
    let nameless = Content {
        kind: Some(Kind::Folder),
        paths: vec!["/".into()],
        ..Default::default()
    };
    assert!(
        !forms_for(&nameless).contains(&Form::FileName),
        "a path with no name is not offered"
    );
    let padded = Content {
        kind: Some(Kind::File),
        paths: vec!["/tmp/  con espacios.txt  ".into()],
        ..Default::default()
    };
    assert_eq!(
        rendered(Form::FileName, &padded),
        "con espacios.txt",
        "the name arrives clear of spaces"
    );
}

#[test]
fn every_form_that_is_offered_can_be_rendered() {
    let png = tiny_png();
    let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJjcC0zIn0.firma";
    let contents = [
        Content {
            kind: Some(Kind::Text),
            text: Some("hola".into()),
            html: Some("<b>hola</b>".into()),
            rich: true,
            ..Default::default()
        },
        text_of(Kind::Json, r#"[{"a": 1}, {"b": 2}]"#),
        text_of(Kind::Json, "[1, 2]"),
        text_of(Kind::Color, "hsla(10, 20%, 30%, 40%)"),
        text_of(Kind::Color, "#FFA500"),
        Content {
            title: Some("Ejemplo".into()),
            ..text_of(Kind::Link, "https://ejemplo.test/a b")
        },
        text_of(Kind::Link, "https://localhost/"),
        text_of(Kind::Code, "  x\n  y"),
        text_of(Kind::Token, jwt),
        text_of(Kind::Token, "ghp_not_a_real_token_for_tests_0000000000"),
        Content {
            kind: Some(Kind::Image),
            image: Some(&png),
            ocr: Some("leído"),
            paths: vec!["/tmp/a.png".into()],
            ..Default::default()
        },
    ];
    for content in &contents {
        let forms = forms_for(content);
        assert!(!forms.is_empty(), "{content:?}");
        for form in forms {
            assert!(render(form, content).is_some(), "{form:?} on {content:?}");
        }
    }
}

#[test]
fn a_rich_text_offers_plain_and_markdown_only_with_html() {
    let with_html = Content {
        kind: Some(Kind::Text),
        text: Some("hola".into()),
        html: Some("<b>hola</b>".into()),
        rich: true,
        ..Default::default()
    };
    assert_eq!(
        forms_for(&with_html),
        vec![
            Form::PlainText,
            Form::Markdown,
            Form::TextQuote,
            Form::TextUpper,
            Form::TextLower
        ]
    );
    assert_eq!(rendered(Form::Markdown, &with_html), "**hola**");
    let spaced = Content {
        text: Some("  hola \n".into()),
        ..with_html.clone()
    };
    assert_eq!(
        rendered(Form::PlainText, &spaced),
        "  hola \n",
        "plain text is delivered exactly as copied, untrimmed"
    );
    let rtf_only = Content {
        rich: true,
        ..text_of(Kind::Text, "hola")
    };
    assert_eq!(
        forms_for(&rtf_only),
        vec![
            Form::PlainText,
            Form::TextQuote,
            Form::TextUpper,
            Form::TextLower
        ]
    );
}

#[test]
fn json_is_offered_formatted_minified_by_keys_and_as_a_table() {
    let content = text_of(
        Kind::Json,
        r#"{"b": 1, "a": {"x": [1, 2]}, "c": "hola\tque tal"}"#,
    );
    assert_eq!(
        forms_for(&content),
        vec![
            Form::JsonPretty,
            Form::JsonMinified,
            Form::JsonKeys,
            Form::JsonTable
        ]
    );
    assert_eq!(
        rendered(Form::JsonPretty, &content),
        "{\n  \"b\": 1,\n  \"a\": {\n    \"x\": [\n      1,\n      2\n    ]\n  },\n  \"c\": \"hola\\tque tal\"\n}",
        "in the order the keys were in"
    );
    assert_eq!(
        rendered(Form::JsonMinified, &content),
        r#"{"b":1,"a":{"x":[1,2]},"c":"hola\tque tal"}"#
    );
    assert_eq!(rendered(Form::JsonKeys, &content), "b\na\nc");
    assert_eq!(
        rendered(Form::JsonTable, &content),
        "b\t1\na\t{\"x\":[1,2]}\nc\thola que tal",
        "a tab between key and value; nested ones go minified"
    );
}

#[test]
fn a_list_of_objects_becomes_a_table_with_a_header() {
    let content = text_of(
        Kind::Json,
        r#"[{"name": "ana", "age": 3}, {"name": "bo", "city": "Lima"}]"#,
    );
    assert_eq!(rendered(Form::JsonKeys, &content), "name\nage\ncity");
    assert_eq!(
        rendered(Form::JsonTable, &content),
        "name\tage\tcity\nana\t3\t\nbo\t\tLima",
        "the columns are the union of the keys; what's missing stays empty"
    );
}

#[test]
fn json_that_is_not_an_object_has_no_keys_and_no_table() {
    let content = text_of(Kind::Json, "[1, 2, 3]");
    assert_eq!(
        forms_for(&content),
        vec![Form::JsonPretty, Form::JsonMinified]
    );
    assert_eq!(render(Form::JsonKeys, &content), None);
    let broken = text_of(Kind::Json, "{no es json}");
    assert!(forms_for(&broken).is_empty());
}

#[test]
fn json_pretty_printing_keeps_only_the_last_value_for_a_repeated_key() {
    let content = text_of(Kind::Json, r#"{"a": 1, "a": 2}"#);
    assert_eq!(
        rendered(Form::JsonPretty, &content),
        "{\n  \"a\": 2\n}",
        "serde_json's map keeps the last occurrence of a repeated key and discards \
         the earlier one silently, with no warning that anything was dropped"
    );
}

#[test]
fn a_json_integer_with_more_digits_than_any_native_type_holds_loses_its_exact_value() {
    let huge = "123456789012345678901234567890";
    let raw = format!("{{\"n\": {huge}}}");
    let content = text_of(Kind::Json, &raw);
    assert!(
        forms_for(&content).contains(&Form::JsonPretty),
        "it is still read and offered, not refused"
    );
    assert_ne!(
        rendered(Form::JsonPretty, &content),
        format!("{{\n  \"n\": {huge}\n}}"),
        "without the arbitrary_precision feature, a number wider than u64 or i64 falls \
         back to a lossy f64, so the digits that come back out are not the ones that went in"
    );
}

#[test]
fn nesting_right_up_to_serde_jsons_depth_limit_still_renders() {
    let depth = 127;
    let nested = format!("{}1{}", "[".repeat(depth), "]".repeat(depth));
    assert_eq!(crate::kind::classify_text(&nested), Kind::Json);
    let content = text_of(Kind::Json, &nested);
    let pretty = rendered(Form::JsonPretty, &content);
    assert!(pretty.starts_with("[\n"));
    assert!(pretty.trim_end().ends_with(']'));
}

#[test]
fn one_level_past_serde_jsons_depth_limit_is_still_called_json_but_offers_no_json_form() {
    let depth = 128;
    let nested = format!("{}1{}", "[".repeat(depth), "]".repeat(depth));
    assert_eq!(
        crate::kind::classify_text(&nested),
        Kind::Json,
        "classify_text only checks that the brackets balance, so it keeps calling \
         this JSON regardless of depth"
    );
    let content = text_of(Kind::Json, &nested);
    assert!(
        forms_for(&content).is_empty(),
        "serde_json::from_str refuses anything past 127 levels of nesting, so the item \
         is tagged as JSON but paste-as offers none of the JSON forms for it, with \
         nothing telling the person why it has no conversions"
    );
}

#[test]
fn a_colour_goes_round_the_three_notations() {
    let content = text_of(Kind::Color, "#FF8800");
    assert_eq!(
        forms_for(&content),
        vec![
            Form::ColorHex,
            Form::ColorRgb,
            Form::ColorHsl,
            Form::ColorName
        ]
    );
    assert_eq!(rendered(Form::ColorHex, &content), "#FF8800");
    assert_eq!(rendered(Form::ColorRgb, &content), "rgb(255, 136, 0)");
    assert_eq!(rendered(Form::ColorHsl, &content), "hsl(32, 100%, 50%)");
    assert_eq!(
        rendered(Form::ColorHex, &text_of(Kind::Color, "rgb(255, 136, 0)")),
        "#FF8800"
    );
    assert_eq!(
        rendered(Form::ColorRgb, &text_of(Kind::Color, "hsl(32, 100%, 50%)")),
        "rgb(255, 136, 0)"
    );
    assert_eq!(
        rendered(Form::ColorHex, &text_of(Kind::Color, "#f80")),
        "#FF8800"
    );
}

#[test]
fn a_colour_that_has_a_css_name_is_called_by_it() {
    for (text, name) in [
        ("#FFA500", "orange"),
        ("#ffa500", "orange"),
        ("rgb(255, 0, 0)", "red"),
        ("hsl(0, 0%, 50%)", "gray"),
        ("#000", "black"),
        ("#fff", "white"),
        ("#663399", "rebeccapurple"),
        ("rgba(255, 165, 0, 1)", "orange"),
    ] {
        assert_eq!(rendered(Form::ColorName, &text_of(Kind::Color, text)), name);
    }
}

#[test]
fn a_colour_close_to_a_name_borrows_it() {
    for (text, name) in [
        ("#FF8800", "darkorange"),
        ("#FF9500", "darkorange"),
        ("#FFCC00", "gold"),
        ("#050505", "black"),
        ("#FEFEFE", "white"),
        ("#F0F0F0", "whitesmoke"),
        ("#0000CC", "mediumblue"),
    ] {
        assert_eq!(rendered(Form::ColorName, &text_of(Kind::Color, text)), name);
    }
}

#[test]
fn a_colour_far_from_every_name_is_not_offered_one() {
    for text in [
        "#123456", "#3B82F6", "#1DB954", "#222222", "#00CC00", "#CC0000",
    ] {
        let content = text_of(Kind::Color, text);
        assert!(!forms_for(&content).contains(&Form::ColorName), "{text}");
        assert_eq!(render(Form::ColorName, &content), None, "{text}");
    }
}

#[test]
fn a_translucent_colour_has_no_name() {
    for text in [
        "#FF880080",
        "rgba(255, 165, 0, 0.5)",
        "hsla(39, 100%, 50%, 99%)",
    ] {
        let content = text_of(Kind::Color, text);
        assert!(!forms_for(&content).contains(&Form::ColorName), "{text}");
        assert_eq!(render(Form::ColorName, &content), None, "{text}");
    }
}

#[test]
fn the_spelling_that_wins_is_the_common_one() {
    for (text, name) in [
        ("#00FFFF", "cyan"),
        ("#FF00FF", "magenta"),
        ("#808080", "gray"),
        ("#2F4F4F", "darkslategray"),
        ("#A9A9A9", "darkgray"),
    ] {
        assert_eq!(rendered(Form::ColorName, &text_of(Kind::Color, text)), name);
    }
}

#[test]
fn the_named_table_has_no_two_names_for_one_colour() {
    let mut values: Vec<[u8; 3]> = NAMED.iter().map(|(_, rgb)| *rgb).collect();
    values.sort_unstable();
    values.dedup();
    assert_eq!(values.len(), NAMED.len());
    for (name, _) in NAMED {
        assert!(name.bytes().all(|b| b.is_ascii_lowercase()), "{name}");
    }
}

#[test]
fn the_distance_is_a_metric_that_weighs_green_most() {
    assert_eq!(redmean_squared([0, 0, 0], [0, 0, 0]), 0);
    assert_eq!(
        redmean_squared([10, 20, 30], [40, 50, 60]),
        redmean_squared([40, 50, 60], [10, 20, 30])
    );
    let red = redmean_squared([128, 0, 0], [128 + 15, 0, 0]);
    let green = redmean_squared([0, 128, 0], [0, 128 + 15, 0]);
    let blue = redmean_squared([0, 0, 128], [0, 0, 128 + 15]);
    assert!(green > red && green > blue);
    assert!(
        redmean_squared([250, 0, 0], [235, 0, 0]) > redmean_squared([10, 0, 0], [25, 0, 0]),
        "a difference in red weighs more where there is a lot of red"
    );
    assert!(
        redmean_squared([250, 0, 0], [250, 0, 15]) < redmean_squared([10, 0, 0], [10, 0, 15]),
        "and one in blue weighs more where there is little red"
    );
    assert_eq!(
        redmean_squared([255, 0, 0], [240, 0, 0]),
        (1024 + 495) * 225
    );
    assert_eq!(
        redmean_squared([0, 0, 255], [0, 0, 240]),
        (1024 + 510) * 225
    );
    assert!(named_color(parse_color("#FF9500").unwrap()).is_some());
    assert_eq!(NAME_WITHIN, 30);
}

#[test]
fn transparency_survives_every_notation() {
    let content = text_of(Kind::Color, "rgba(255, 136, 0, 0.5)");
    assert_eq!(rendered(Form::ColorHex, &content), "#FF880080");
    assert_eq!(rendered(Form::ColorRgb, &content), "rgba(255, 136, 0, 0.5)");
    assert_eq!(
        rendered(Form::ColorHsl, &content),
        "hsla(32, 100%, 50%, 0.5)"
    );
    assert_eq!(
        rendered(Form::ColorRgb, &text_of(Kind::Color, "#FF880080")),
        "rgba(255, 136, 0, 0.5)"
    );
}

#[test]
fn grey_has_no_hue_and_the_extremes_do_not_divide_by_zero() {
    assert_eq!(
        rendered(Form::ColorHsl, &text_of(Kind::Color, "#808080")),
        "hsl(0, 0%, 50%)"
    );
    assert_eq!(
        rendered(Form::ColorHsl, &text_of(Kind::Color, "#000000")),
        "hsl(0, 0%, 0%)"
    );
    assert_eq!(
        rendered(Form::ColorHsl, &text_of(Kind::Color, "#FFFFFF")),
        "hsl(0, 0%, 100%)"
    );
    assert_eq!(
        rendered(
            Form::ColorRgb,
            &text_of(Kind::Color, "hsl(400, 150%, -10%)")
        ),
        "rgb(0, 0, 0)",
        "out of range it gets clamped, not broken"
    );
}

#[test]
fn every_sextant_of_the_hue_wheel_round_trips_against_a_reference_table() {
    for (hex, hsl) in [
        ("#FF0000", "hsl(0, 100%, 50%)"),
        ("#FFFF00", "hsl(60, 100%, 50%)"),
        ("#00FF00", "hsl(120, 100%, 50%)"),
        ("#00FFFF", "hsl(180, 100%, 50%)"),
        ("#0000FF", "hsl(240, 100%, 50%)"),
        ("#FF00FF", "hsl(300, 100%, 50%)"),
        ("#FF0080", "hsl(330, 100%, 50%)"),
        ("#80FF00", "hsl(90, 100%, 50%)"),
        ("#00FF80", "hsl(150, 100%, 50%)"),
        ("#0080FF", "hsl(210, 100%, 50%)"),
        ("#8000FF", "hsl(270, 100%, 50%)"),
        ("#3498DB", "hsl(204, 70%, 53%)"),
        ("#2ECC71", "hsl(145, 63%, 49%)"),
        ("#9B59B6", "hsl(283, 39%, 53%)"),
        ("#E67E22", "hsl(28, 80%, 52%)"),
        ("#1A0B2E", "hsl(266, 61%, 11%)"),
        ("#F0E68C", "hsl(54, 77%, 75%)"),
    ] {
        assert_eq!(
            rendered(Form::ColorHsl, &text_of(Kind::Color, hex)),
            hsl,
            "{hex}"
        );
    }
    for (hsl, hex) in [
        ("hsl(0, 100%, 50%)", "#FF0000"),
        ("hsl(60, 100%, 50%)", "#FFFF00"),
        ("hsl(120, 100%, 50%)", "#00FF00"),
        ("hsl(180, 100%, 50%)", "#00FFFF"),
        ("hsl(240, 100%, 50%)", "#0000FF"),
        ("hsl(300, 100%, 50%)", "#FF00FF"),
        ("hsl(330, 100%, 50%)", "#FF0080"),
        ("hsl(32, 50%, 50%)", "#BF8440"),
        ("hsl(210, 40%, 70%)", "#94B2D1"),
        ("hsl(90, 60%, 30%)", "#4D7A1F"),
        ("hsl(270, 25%, 80%)", "#CCBFD9"),
        ("hsl(15, 80%, 20%)", "#5C1F0A"),
        ("hsl(400, 50%, 50%)", "#BF9540"),
        ("hsl(-60, 100%, 50%)", "#FF00FF"),
    ] {
        assert_eq!(
            rendered(Form::ColorHex, &text_of(Kind::Color, hsl)),
            hex,
            "{hsl}"
        );
    }
}

#[test]
fn short_and_eight_digit_hex_expand_digit_by_digit() {
    assert_eq!(
        rendered(Form::ColorRgb, &text_of(Kind::Color, "#0af")),
        "rgb(0, 170, 255)"
    );
    assert_eq!(
        rendered(Form::ColorRgb, &text_of(Kind::Color, "#FF880099")),
        "rgba(255, 136, 0, 0.6)"
    );
    assert_eq!(
        rendered(Form::ColorRgb, &text_of(Kind::Color, "#12345678")),
        "rgba(18, 52, 86, 0.47)"
    );
    assert_eq!(
        rendered(
            Form::ColorHex,
            &text_of(Kind::Color, "hsla(200, 50%, 40%, 0.25)")
        ),
        "#33779940"
    );
}

#[test]
fn a_colour_that_does_not_parse_offers_nothing_extra() {
    assert!(forms_for(&text_of(Kind::Color, "#GGGGGG")).is_empty());
    assert_eq!(parse_color("rgb(1, 2)"), None);
    assert_eq!(parse_color("hsl(1, 2, 3, 4, 5)"), None);
    assert_eq!(parse_color("#12345"), None);
}

#[test]
fn a_link_is_offered_as_markdown_domain_and_with_its_title() {
    let bare = text_of(Kind::Link, "https://www.ejemplo.test/ruta?x=1#f");
    assert_eq!(forms_for(&bare), vec![Form::LinkMarkdown, Form::LinkDomain]);
    assert_eq!(
        rendered(Form::LinkMarkdown, &bare),
        "[https://www.ejemplo.test/ruta?x=1#f](https://www.ejemplo.test/ruta?x=1#f)"
    );
    assert_eq!(rendered(Form::LinkDomain, &bare), "ejemplo.test");
    assert_eq!(
        rendered(
            Form::LinkDomain,
            &text_of(Kind::Link, "https://EJEMPLO.TEST/X")
        ),
        "ejemplo.test",
        "a domain does not distinguish case"
    );
    let titled = Content {
        title: Some(" Ejemplo, la página ".into()),
        ..bare.clone()
    };
    assert_eq!(forms_for(&titled).last(), Some(&Form::LinkTitled));
    assert_eq!(
        rendered(Form::LinkMarkdown, &titled),
        "[Ejemplo, la página](https://www.ejemplo.test/ruta?x=1#f)"
    );
    assert_eq!(
        rendered(Form::LinkTitled, &titled),
        "Ejemplo, la página — https://www.ejemplo.test/ruta?x=1#f"
    );
}

#[test]
fn the_domain_drops_credentials_port_and_path_but_not_a_subdomain() {
    assert_eq!(
        domain_of("https://user:pw@api.ejemplo.test:8443/v1"),
        Some("api.ejemplo.test")
    );
    assert_eq!(
        domain_of("ftp://files.ejemplo.test"),
        Some("files.ejemplo.test")
    );
    assert_eq!(
        domain_of("mailto:alguien@ejemplo.test"),
        Some("ejemplo.test")
    );
    assert_eq!(
        domain_of("https://localhost:3000/"),
        None,
        "no dot means no domain"
    );
    assert_eq!(domain_of("https://"), None);
    assert_eq!(domain_of(""), None);
    assert_eq!(
        domain_of("192.168.0.1:8080"),
        Some("192.168.0.1"),
        "with no scheme, what's before the port is the host"
    );
    assert_eq!(
        domain_of("https://ejemplo.test:abc/"),
        Some("ejemplo.test:abc"),
        "a port that is not a number is not trimmed off"
    );
    assert_eq!(domain_of("https://ejemplo.test:/"), Some("ejemplo.test:"));
    assert_eq!(
        domain_of(":ejemplo.test"),
        Some(":ejemplo.test"),
        "an empty scheme is no scheme"
    );
}

#[test]
fn code_can_be_one_line_a_block_or_dedented() {
    let source = "    fn main() {\n        println!(\"hola\");\n    }\n";
    let content = text_of(Kind::Code, source);
    assert_eq!(
        forms_for(&content),
        vec![Form::CodeOneLine, Form::CodeBlock, Form::CodeDedented]
    );
    assert_eq!(
        rendered(Form::CodeOneLine, &content),
        "fn main() { println!(\"hola\"); }"
    );
    assert_eq!(
        rendered(Form::CodeBlock, &content),
        "```\nfn main() {\n        println!(\"hola\");\n    }\n```"
    );
    assert_eq!(
        rendered(Form::CodeDedented, &content),
        "fn main() {\n    println!(\"hola\");\n}\n",
        "what's stripped is only what every line shares, nothing more"
    );
    assert_eq!(
        dedent("\n  a\n\n    b\n"),
        "\na\n\n  b\n",
        "empty lines don't count"
    );
}

#[test]
fn a_token_becomes_a_header_or_a_curl_and_a_jwt_shows_its_claims() {
    let opaque = text_of(Kind::Token, "ghp_not_a_real_token_for_tests_0000000000");
    assert_eq!(forms_for(&opaque), vec![Form::TokenHeader, Form::TokenCurl]);
    assert_eq!(
        rendered(Form::TokenHeader, &opaque),
        "Authorization: Bearer ghp_not_a_real_token_for_tests_0000000000"
    );
    assert_eq!(
        rendered(Form::TokenCurl, &opaque),
        "curl -H 'Authorization: Bearer ghp_not_a_real_token_for_tests_0000000000' \"$URL\""
    );
    let jwt = text_of(
        Kind::Token,
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJjcC0zIiwiZW52Ijoic3RhZ2luZyJ9.firma",
    );
    assert_eq!(forms_for(&jwt).last(), Some(&Form::TokenClaims));
    assert_eq!(
        rendered(Form::TokenClaims, &jwt),
        "{\n  \"sub\": \"cp-3\",\n  \"env\": \"staging\"\n}"
    );
}

fn tiny_png() -> Vec<u8> {
    let mut out = Vec::new();
    let mut image = image::RgbaImage::new(2, 2);
    image.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));
    image.put_pixel(1, 0, image::Rgba([0, 0, 0, 0]));
    image.put_pixel(0, 1, image::Rgba([0, 255, 0, 255]));
    image.put_pixel(1, 1, image::Rgba([0, 0, 255, 128]));
    image
        .write_with_encoder(image::codecs::png::PngEncoder::new(&mut out))
        .expect("png");
    out
}

fn solid_png(pixel: [u8; 4]) -> Vec<u8> {
    let mut out = Vec::new();
    let image = image::RgbaImage::from_pixel(16, 16, image::Rgba(pixel));
    image
        .write_with_encoder(image::codecs::png::PngEncoder::new(&mut out))
        .expect("png");
    out
}

fn middle_of_jpeg(bytes: &[u8]) -> [u8; 3] {
    assert_eq!(&bytes[..2], &[0xFF, 0xD8], "JPEG header");
    let back = image::load_from_memory(bytes).expect("reads").to_rgb8();
    back.get_pixel(8, 8).0
}

#[test]
fn an_image_is_offered_as_jpeg_and_as_what_was_read_in_it() {
    let png = tiny_png();
    let silent = Content {
        kind: Some(Kind::Image),
        image: Some(&png),
        ..Default::default()
    };
    assert_eq!(forms_for(&silent), vec![Form::ImageJpeg]);
    let read = Content {
        ocr: Some(" Pedido 4417 "),
        ..silent.clone()
    };
    assert_eq!(forms_for(&read), vec![Form::ImageJpeg, Form::ImageOcr]);
    assert_eq!(rendered(Form::ImageOcr, &read), "Pedido 4417");
    let blank = Content {
        ocr: Some("   "),
        ..silent.clone()
    };
    assert_eq!(forms_for(&blank), vec![Form::ImageJpeg]);
}

#[test]
fn the_jpeg_is_a_real_jpeg_with_transparency_laid_on_white() {
    let near = |got: [u8; 3], wanted: [u8; 3]| {
        got.iter()
            .zip(wanted)
            .all(|(got, wanted)| got.abs_diff(wanted) <= 6)
    };
    let jpeg_of_solid = |pixel: [u8; 4]| {
        let png = solid_png(pixel);
        let content = Content {
            image: Some(&png),
            ..Default::default()
        };
        match render(Form::ImageJpeg, &content) {
            Some(Rendered::Jpeg(bytes)) => middle_of_jpeg(&bytes),
            other => panic!("no JPEG came out: {other:?}"),
        }
    };
    assert!(
        near(jpeg_of_solid([200, 30, 30, 255]), [200, 30, 30]),
        "opaque untouched"
    );
    assert!(
        near(jpeg_of_solid([0, 0, 0, 0]), [255, 255, 255]),
        "transparent turns white, not black"
    );
    assert!(
        near(jpeg_of_solid([0, 0, 255, 128]), [127, 127, 255]),
        "blue at 50% over white"
    );
    assert!(
        near(jpeg_of_solid([0, 0, 255, 64]), [191, 191, 255]),
        "blue at 25% over white"
    );
    assert!(
        near(jpeg_of_solid([200, 30, 30, 128]), [227, 142, 142]),
        "a muted red at 50% lightens channel by channel"
    );
    assert_eq!(
        render(Form::ImageJpeg, &text_of(Kind::Image, "no bytes")),
        None
    );
    let broken = Content {
        image: Some(b"no es png"),
        ..Default::default()
    };
    assert_eq!(render(Form::ImageJpeg, &broken), None);
}

#[test]
fn a_tiff_from_safari_becomes_a_jpeg_too() {
    let mut tiff = Vec::new();
    let image = image::RgbaImage::from_pixel(16, 16, image::Rgba([10, 200, 30, 255]));
    image
        .write_with_encoder(image::codecs::tiff::TiffEncoder::new(std::io::Cursor::new(
            &mut tiff,
        )))
        .expect("tiff");
    let content = Content {
        kind: Some(Kind::Image),
        image: Some(&tiff),
        ..Default::default()
    };
    let Some(Rendered::Jpeg(bytes)) = render(Form::ImageJpeg, &content) else {
        panic!("a TIFF becomes a JPEG too");
    };
    let middle = middle_of_jpeg(&bytes);
    assert!(
        middle[0].abs_diff(10) <= 6 && middle[1].abs_diff(200) <= 6 && middle[2].abs_diff(30) <= 6,
        "{middle:?}"
    );
}

#[test]
fn a_null_cell_is_empty_in_the_table() {
    let content = text_of(Kind::Json, r#"[{"a": null, "b": true}]"#);
    assert_eq!(rendered(Form::JsonTable, &content), "a\tb\n\ttrue");
    assert_eq!(
        rendered(Form::JsonTable, &text_of(Kind::Json, r#"{"k": null}"#)),
        "k\t"
    );
}

#[test]
fn files_are_offered_by_their_paths() {
    let content = Content {
        kind: Some(Kind::File),
        paths: vec!["/tmp/uno.txt".into(), "/tmp/dos.txt".into()],
        ..Default::default()
    };
    assert_eq!(forms_for(&content), vec![Form::Path, Form::FileName]);
    assert_eq!(rendered(Form::Path, &content), "/tmp/uno.txt\n/tmp/dos.txt");
}

#[test]
fn a_form_that_does_not_apply_renders_nothing() {
    let plain = text_of(Kind::Text, "hola");
    for form in [
        Form::Markdown,
        Form::JsonPretty,
        Form::ColorHex,
        Form::LinkDomain,
        Form::TokenClaims,
        Form::ImageJpeg,
        Form::ImageOcr,
    ] {
        assert_eq!(render(form, &plain), None, "{form:?}");
    }
}

#[test]
fn html_from_a_word_processor_becomes_readable_markdown() {
    let html = "<html><head><style>p{color:red}</style><title>x</title></head><body>\
            <h2>Informe</h2>\
            <p>Un p&aacute;rrafo con <b>negrita</b>, <i>cursiva</i> y un \
            <a href=\"https://ejemplo.test\">enlace</a>.</p>\
            <ul><li>uno</li><li>dos <strong>fuerte</strong></li></ul>\
            <ol><li>primero</li><li>segundo</li></ol>\
            <p>Fin &amp; c&#243;digo <code>x &lt; y</code></p>\
            </body></html>";
    assert_eq!(
        markdown_of_html(html),
        "## Informe\n\n\
             Un párrafo con **negrita**, *cursiva* y un [enlace](https://ejemplo.test).\n\n\
             - uno\n- dos **fuerte**\n\n\
             1. primero\n2. segundo\n\n\
             Fin & código `x < y`"
    );
}

#[test]
fn the_windows_clipboard_header_is_left_out_and_only_the_fragment_counts() {
    let html = "Version:0.9\r\nStartHTML:0000000105\r\nEndHTML:0000000250\r\n\
            <html><body><!--StartFragment--><p>solo <b>esto</b></p><!--EndFragment--></body></html>";
    assert_eq!(markdown_of_html(html), "solo **esto**");
}

#[test]
fn preformatted_blocks_keep_their_whitespace_and_nested_lists_indent() {
    let html = "<pre>fn main() {\n    hola\n}</pre><ul><li>a<ul><li>b</li></ul></li></ul>";
    assert_eq!(
        markdown_of_html(html),
        "```\nfn main() {\n    hola\n}\n```\n\n- a\n  - b"
    );
}

#[test]
fn quotes_images_rules_and_line_breaks() {
    let html = "<blockquote>dicho</blockquote><img alt=\"foto\" src=\"a.png\"><hr>uno<br>dos";
    assert_eq!(
        markdown_of_html(html),
        "> dicho\n\n![foto](a.png)\n\n---\n\nuno\ndos"
    );
}

#[test]
fn whitespace_between_tags_collapses_like_a_browser_would() {
    let html = "<p>\n   varias\n   palabras   <b> juntas </b>\n</p>\n<p>otro</p>";
    assert_eq!(markdown_of_html(html), "varias palabras **juntas**\n\notro");
}

#[test]
fn what_is_not_html_at_all_comes_out_as_text() {
    assert_eq!(markdown_of_html("2 < 3 y 4 > 1"), "2 < 3 y 4 > 1");
    assert_eq!(markdown_of_html(""), "");
    assert_eq!(markdown_of_html("<"), "<");
    assert_eq!(markdown_of_html("<p></p><div></div>"), "");
}

#[test]
fn entities_of_every_shape_are_decoded_and_the_rest_left_alone() {
    assert_eq!(
        decode_entities("&amp;&lt;&gt;&quot;&apos;&nbsp;"),
        "&<>\"' "
    );
    assert_eq!(decode_entities("&#65;&#x42;&#X43;"), "ABC");
    assert_eq!(
        decode_entities("&desconocida; & suelto &;"),
        "&desconocida; & suelto &;"
    );
    assert_eq!(
        decode_entities("&#99999999;"),
        "&#99999999;",
        "outside Unicode"
    );
    assert_eq!(decode_entities("&aacute;&Ntilde;&euro;&hellip;"), "áÑ€…");
    assert_eq!(decode_entities("&yacute;&uuml;"), "ýü");
    assert_eq!(
        decode_entities("&rarr;"),
        "&rarr;",
        "unknown named entities are left alone"
    );
}

#[test]
fn a_script_or_style_is_skipped_until_its_own_closing_tag() {
    assert_eq!(markdown_of_html("<script>a</b>b</script>c"), "c");
    assert_eq!(markdown_of_html("<style>x</script>y</style>z"), "z");
    assert_eq!(
        markdown_of_html("<head><title>t</title></head>cuerpo"),
        "cuerpo"
    );
}

#[test]
fn headings_tables_and_lists_keep_their_distance_from_what_follows() {
    assert_eq!(markdown_of_html("<h1>t</h1>x"), "# t\n\nx");
    assert_eq!(
        markdown_of_html("<table><tr><td>a</td><th>b</th></tr></table>"),
        "| a | b |\n| --- | --- |"
    );
    assert_eq!(markdown_of_html("a<ul><li>b</li></ul>"), "a\n\n- b");
    assert_eq!(markdown_of_html("<pre><code>x</code></pre>"), "```\nx\n```");
    assert_eq!(markdown_of_html("<b>a </b>b"), "**a** b");
    assert_eq!(markdown_of_html("<b></b>vacío"), "vacío");
    assert_eq!(markdown_of_html("<p> x</p>"), "x");
    assert_eq!(markdown_of_html("<blockquote> q</blockquote>"), "> q");
    assert_eq!(
        markdown_of_html("<br>x"),
        "x",
        "a line break at the very start leaves no gap"
    );
    assert_eq!(markdown_of_html("<ul><li> a</li></ul>"), "- a");
    assert_eq!(markdown_of_html("<ol><li> a</li></ol>"), "1. a");
    assert_eq!(markdown_of_html("uno<br><br>dos"), "uno\ndos");
    assert_eq!(markdown_of_html("<img src=\"a.png\"> x"), "![](a.png) x");
    assert_eq!(markdown_of_html("<pre>a\n</pre> b"), "```\na\n```\n\nb");
}

#[test]
fn a_multibyte_character_right_after_a_tag_does_not_panic() {
    assert_eq!(markdown_of_html("<p>ñandú</p>"), "ñandú");
    assert_eq!(markdown_of_html("<b>—</b>€"), "**—**€");
    assert_eq!(markdown_of_html("ñ<ñ"), "ñ<ñ");
}

#[test]
fn an_angle_bracket_that_never_closes_is_text_and_costs_one_pass() {
    let hostile = "<a".repeat(200_000);
    let started = std::time::Instant::now();
    let out = markdown_of_html(&hostile);
    assert!(started.elapsed().as_secs() < 2, "{:?}", started.elapsed());
    assert_eq!(out.len(), hostile.len());
}

#[test]
fn comments_and_quoted_attributes_may_contain_a_closing_bracket() {
    assert_eq!(markdown_of_html("<!-- a > b -->x"), "x");
    assert_eq!(
        markdown_of_html("<a href=\"x\" title=\"a>b\">t</a>"),
        "[t](x)"
    );
    assert_eq!(markdown_of_html("<!-- sin cierre"), "<!-- sin cierre");
}

#[test]
fn a_stray_close_inside_a_preformatted_link_does_not_panic() {
    assert_eq!(
        markdown_of_html("<pre>a    <a href=\"x\"></b></a></pre>"),
        "```\na    [](x)\n```",
        "inside a preformatted block spaces are not trimmed, a stray closing tag is ignored, and an empty anchor does not break"
    );
    assert_eq!(
        markdown_of_html("<pre><b>x </b>y</pre>"),
        "```\nx y\n```",
        "inside a code fence the asterisks would be literal"
    );
    assert_eq!(markdown_of_html("<a href=\"x\"> </a>"), "[](x)");
}

#[test]
fn a_sheets_range_becomes_a_markdown_table() {
    let html = "<meta charset='utf-8'><google-sheets-html-origin><style>td{}</style>\
            <table><colgroup><col/></colgroup><tbody>\
            <tr><td>a</td><td>b|c</td></tr><tr><td>1</td><td><br>2</td></tr>\
            </tbody></table></google-sheets-html-origin>";
    assert_eq!(
        markdown_of_html(html),
        "| a | b\\|c |\n| --- | --- |\n| 1 | 2 |"
    );
    assert_eq!(
        markdown_of_html("x<table><tr><td>a</td></tr></table>y"),
        "x\n\n| a |\n| --- |\n\ny"
    );
    assert_eq!(markdown_of_html("<table><tr></tr></table>"), "");
}

#[test]
fn google_wraps_everything_in_a_bold_tag_that_is_not_bold() {
    let docs = "<b style=\"font-weight:normal;\" id=\"docs-internal-guid-1\">\
            <span style=\"font-size:11pt;font-weight:400;\">dasas</span></b>";
    assert_eq!(markdown_of_html(docs), "dasas");
    assert_eq!(
        markdown_of_html(
            "<b style=\"font-weight:normal\"><span style=\"font-weight:700\">n</span> \
                <span style=\"font-style:italic\">c</span> \
                <span style=\"font-weight:bold;font-style:italic\">nc</span></b>"
        ),
        "**n** *c* ***nc***"
    );
    assert_eq!(markdown_of_html("<b>de verdad</b>"), "**de verdad**");
    assert_eq!(
        markdown_of_html("<strong style=\"color:red\">x</strong>"),
        "**x**"
    );
}

#[test]
fn a_quote_closed_after_a_cell_does_not_panic_and_quotes_nest() {
    assert_eq!(
        markdown_of_html("a<td><blockquote>x</td></blockquote>"),
        "a\n\n> x",
        "a cell outside a row does not swallow the text"
    );
    assert_eq!(
        markdown_of_html("<table><tr><td>ab<blockquote>x</td></blockquote></tr></table>"),
        "| ab  x |\n| --- |",
        "the cell cuts off the output before the start of the quote, and the quote does not collapse"
    );
    assert_eq!(
        markdown_of_html("<blockquote><blockquote>x</blockquote>y</blockquote>"),
        "> > x\n>\n> y"
    );
}

#[test]
fn the_browser_writes_styles_with_a_space_after_the_colon() {
    assert_eq!(
        markdown_of_html(
            "<span style=\"font-weight: 700;\">n</span> <span style=\"font-style: italic;\">c</span>"
        ),
        "**n** *c*"
    );
    assert_eq!(
        markdown_of_html("<i style=\"font-style: normal\">x</i>"),
        "x"
    );
    assert_eq!(
        markdown_of_html("<span style=\"font-style: oblique\">x</span>"),
        "*x*"
    );
}

#[test]
fn an_attribute_is_only_the_one_that_starts_after_whitespace() {
    assert_eq!(
        markdown_of_html("<img data-src=\"lazy.jpg\" src=\"real.jpg\">"),
        "![](real.jpg)"
    );
    assert_eq!(
        markdown_of_html("<a data-href=\"x\" href=\"y\">l</a>"),
        "[l](y)"
    );
    assert_eq!(
        markdown_of_html("<a title=\"href=no\" href=\"y\">l</a>"),
        "[l](y)"
    );
    assert_eq!(markdown_of_html("<a HREF=\"y\">l</a>"), "[l](y)");
}

#[test]
fn adjacent_runs_of_the_same_style_merge_into_one() {
    assert_eq!(markdown_of_html("<b>a</b><b>b</b>"), "**ab**");
    assert_eq!(markdown_of_html("<b>a </b><b>b</b>"), "**a** **b**");
    assert_eq!(markdown_of_html("<code>a</code><code>b</code>"), "`ab`");
    assert_eq!(
        markdown_of_html("<b>a</b> <b>b</b>"),
        "**a** **b**",
        "with text in between they do not merge"
    );
    assert_eq!(markdown_of_html("<b>a</b><i>b</i>"), "**a***b*");
}

#[test]
fn a_start_saved_before_a_truncation_never_lands_inside_a_character() {
    for hostile in [
        "<a>ab <td></a>é</td>",
        "<table><tr><td>ab<a href=\"x\"></td>aé</a></tr></table>",
        "<td>ab <blockquote></td></blockquote>",
        "<a>ab <blockquote></a></blockquote>",
        "<b>ab </b><a>é</a><td>ñ</td>",
        "<blockquote>ab <b>x </b>ñ</blockquote>",
    ] {
        let out = markdown_of_html(hostile);
        assert!(!out.is_empty() || hostile.is_empty(), "{hostile}");
    }
}

#[test]
fn a_quote_keeps_every_paragraph_inside_it() {
    assert_eq!(
        markdown_of_html("<blockquote><p>a</p><p>b</p></blockquote>c"),
        "> a\n>\n> b\n\nc"
    );
    assert_eq!(markdown_of_html("<blockquote></blockquote>x"), "x");
}

#[test]
fn percentages_in_rgb_channels_and_in_alpha_are_scaled() {
    assert_eq!(
        rendered(Form::ColorHex, &text_of(Kind::Color, "rgb(100%, 0%, 50%)")),
        "#FF0080"
    );
    assert_eq!(
        rendered(Form::ColorRgb, &text_of(Kind::Color, "rgba(0, 0, 0, 50%)")),
        "rgba(0, 0, 0, 0.5)"
    );
    assert_eq!(
        rendered(
            Form::ColorHsl,
            &text_of(Kind::Color, "hsla(0, 100%, 50%, 25%)")
        ),
        "hsla(0, 100%, 50%, 0.25)"
    );
}

#[test]
fn dedent_counts_characters_not_bytes_and_only_strips_what_every_line_shares() {
    assert_eq!(
        dedent("\u{a0}\u{a0}x\n   y"),
        "\u{a0}\u{a0}x\n   y",
        "nothing in common"
    );
    assert_eq!(dedent("\u{a0}\u{a0}x\n\u{a0}\u{a0}\u{a0}y"), "x\n\u{a0}y");
    assert_eq!(dedent("\tx\n\t\ty"), "x\n\ty");
    assert_eq!(
        dedent("\tx\n  y"),
        "\tx\n  y",
        "a tab and spaces are not the same"
    );
    assert_eq!(
        dedent("  x\n \n  y"),
        "x\n\ny",
        "a short blank line does not get in the way"
    );
}

#[test]
fn a_curl_form_cannot_break_out_of_its_quotes() {
    let odd = Content {
        kind: Some(Kind::Token),
        text: Some("abc'; rm -rf / #".into()),
        ..Default::default()
    };
    assert_eq!(
        rendered(Form::TokenCurl, &odd),
        "curl -H 'Authorization: Bearer abc'\\''; rm -rf / #' \"$URL\""
    );
}

#[test]
fn a_markdown_link_survives_odd_urls_and_titles() {
    let spaced = Content {
        title: Some(" [Sección] ".into()),
        ..text_of(Kind::Link, "https://ejemplo.test/a b(c)")
    };
    assert_eq!(
        rendered(Form::LinkMarkdown, &spaced),
        "[Sección](<https://ejemplo.test/a b(c)>)"
    );
    let blank_title = Content {
        title: Some("   ".into()),
        ..text_of(Kind::Link, "https://ejemplo.test")
    };
    assert_eq!(
        rendered(Form::LinkMarkdown, &blank_title),
        "[https://ejemplo.test](https://ejemplo.test)"
    );
    assert!(!forms_for(&blank_title).contains(&Form::LinkTitled));
    assert!(!forms_for(&text_of(Kind::Link, "https://localhost/")).contains(&Form::LinkDomain));
}

#[test]
fn a_code_fence_is_always_longer_than_any_backticks_inside() {
    assert_eq!(
        rendered(Form::CodeBlock, &text_of(Kind::Code, "x")),
        "```\nx\n```"
    );
    assert_eq!(
        rendered(Form::CodeBlock, &text_of(Kind::Code, "a ``` b")),
        "````\na ``` b\n````"
    );
    assert_eq!(
        rendered(Form::CodeBlock, &text_of(Kind::Code, "`````")),
        "``````\n`````\n``````"
    );
}

#[test]
fn a_link_with_no_target_is_just_its_text() {
    assert_eq!(markdown_of_html("<a>sin destino</a>"), "sin destino");
    assert_eq!(
        markdown_of_html("<a href='x.html'>comillas simples</a>"),
        "[comillas simples](x.html)"
    );
    assert_eq!(
        markdown_of_html("<a href=x.html target=_blank>sin comillas</a>"),
        "[sin comillas](x.html)"
    );
}
