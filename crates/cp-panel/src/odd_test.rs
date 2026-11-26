pub const LONG_WORD: usize = 5_000;

pub fn texts() -> Vec<(&'static str, String)> {
    let zalgo = format!(
        "Z{}a{}l{}go",
        "\u{346}\u{359}\u{30d}".repeat(40),
        "\u{36f}".repeat(60),
        "\u{489}".repeat(30)
    );
    vec![
        ("zwj emoji", "👩‍💻 code 🏳️‍🌈 pride 👨‍👩‍👧‍👦".into()),
        ("flags", "🇨🇱🇯🇵🇺🇸🇩🇪 flags".into()),
        (
            "combining marks",
            "cafe\u{301} re\u{301}sume\u{301} nin\u{303}o".into(),
        ),
        ("zalgo", zalgo),
        ("rtl mixed", "مرحبا hello שלום world 123 العالم".into()),
        ("rtl only", "שלום עולם\nمرحبا بالعالم".into()),
        ("cjk", "日本語のテキスト 中文文本 한국어 텍스트".into()),
        ("long word", "x".repeat(LONG_WORD)),
        ("long cjk word", "界".repeat(LONG_WORD)),
        ("only spaces", "   \u{a0}\u{3000}   ".into()),
        ("only newlines", "\n\n\n\r\n\n\r".into()),
        ("tabs", "\t\t\ta\t\tb\t\t".into()),
        (
            "zero width joiners",
            "a\u{200d}b\u{200b}c\u{200d}\u{200b}d".into(),
        ),
        ("only zero width", "\u{200b}\u{200d}\u{feff}\u{2060}".into()),
        (
            "nul and controls",
            "a\0b\u{1}\u{7}\u{1b}[31mred\u{8}\u{7f}\u{0085}z".into(),
        ),
        (
            "line separators",
            "one\u{2028}two\u{2029}three\u{b}four\u{c}five".into(),
        ),
        ("replacement", "\u{FFFD}\u{FFFD} \u{FFFD}".into()),
        (
            "verbatim path",
            r"\?\C:\Users\Mario\Documents\archivo con espacios y ñandú.txt".into(),
        ),
        (
            "verbatim unc",
            r"\?\UNC\servidor\recurso\carpeta\日本語.txt".into(),
        ),
        (
            "idn url",
            "https://bücher.例え.example:8080/pfad%20mit%20leer?q=%E6%97%A5%E6%9C%AC&x=%ZZ#frag"
                .into(),
        ),
        (
            "punycode url",
            "https://xn--bcher-kva.example/ü/%F0%9F%91%A9".into(),
        ),
        (
            "json unicode keys",
            r#"{"clé": "é", "日本語": ["👩‍💻", 1, null], "ключ": {"a": true}, "": 0}"#.into(),
        ),
        ("bad colour", "#ZZZZZZ".into()),
        ("good colour", "#12ab34".into()),
        ("mixed lines", "primera línea\nلسطين\n日本\n\n\tfin".into()),
    ]
}
