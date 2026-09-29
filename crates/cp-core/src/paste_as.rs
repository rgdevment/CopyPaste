use crate::kind::Kind;
use serde_json::Value;
use std::borrow::Cow;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Content<'a> {
    pub kind: Option<Kind>,
    pub text: Option<Cow<'a, str>>,
    pub html: Option<Cow<'a, str>>,
    pub rich: bool,
    pub image: Option<&'a [u8]>,
    pub paths: Vec<String>,
    pub title: Option<Cow<'a, str>>,
    pub ocr: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    PlainText,
    Markdown,
    JsonPretty,
    JsonMinified,
    JsonKeys,
    JsonTable,
    ColorHex,
    ColorRgb,
    ColorHsl,
    ColorName,
    LinkMarkdown,
    LinkDomain,
    LinkTitled,
    CodeOneLine,
    CodeBlock,
    CodeDedented,
    TokenHeader,
    TokenClaims,
    TokenCurl,
    ImageJpeg,
    ImageOcr,
    Path,
    FileName,
    TextQuote,
    TextUpper,
    TextLower,
}

impl Form {
    pub const ALL: [Form; 26] = [
        Form::PlainText,
        Form::Markdown,
        Form::JsonPretty,
        Form::JsonMinified,
        Form::JsonKeys,
        Form::JsonTable,
        Form::ColorHex,
        Form::ColorRgb,
        Form::ColorHsl,
        Form::ColorName,
        Form::LinkMarkdown,
        Form::LinkDomain,
        Form::LinkTitled,
        Form::CodeOneLine,
        Form::CodeBlock,
        Form::CodeDedented,
        Form::TokenHeader,
        Form::TokenClaims,
        Form::TokenCurl,
        Form::ImageJpeg,
        Form::ImageOcr,
        Form::Path,
        Form::FileName,
        Form::TextQuote,
        Form::TextUpper,
        Form::TextLower,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Form::PlainText => "plain-text",
            Form::Markdown => "markdown",
            Form::JsonPretty => "json-pretty",
            Form::JsonMinified => "json-minified",
            Form::JsonKeys => "json-keys",
            Form::JsonTable => "json-table",
            Form::ColorHex => "color-hex",
            Form::ColorRgb => "color-rgb",
            Form::ColorHsl => "color-hsl",
            Form::ColorName => "color-name",
            Form::LinkMarkdown => "link-markdown",
            Form::LinkDomain => "link-domain",
            Form::LinkTitled => "link-titled",
            Form::CodeOneLine => "code-one-line",
            Form::CodeBlock => "code-block",
            Form::CodeDedented => "code-dedented",
            Form::TokenHeader => "token-header",
            Form::TokenClaims => "token-claims",
            Form::TokenCurl => "token-curl",
            Form::ImageJpeg => "image-jpeg",
            Form::ImageOcr => "image-ocr",
            Form::Path => "path",
            Form::FileName => "file-name",
            Form::TextQuote => "text-quote",
            Form::TextUpper => "text-upper",
            Form::TextLower => "text-lower",
        }
    }

    pub fn from_name(name: &str) -> Option<Form> {
        Form::ALL.into_iter().find(|form| form.as_str() == name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rendered {
    Text(String),
    Jpeg(Vec<u8>),
}

impl Rendered {
    pub fn into_item(self) -> crate::item::Item {
        use crate::item::{Format, Item, Payload, SYNTHETIC_JPEG};
        match self {
            Rendered::Text(text) => Item::plain(&text),
            Rendered::Jpeg(bytes) => Item {
                kind: Some(Kind::Image),
                formats: vec![Format {
                    id: SYNTHETIC_JPEG.into(),
                    payload: Payload::stored(bytes),
                }],
            },
        }
    }
}

pub const JPEG_QUALITY: u8 = 85;

pub fn forms_for(content: &Content) -> Vec<Form> {
    let mut forms = Vec::new();
    if content.rich && content.text.is_some() {
        forms.push(Form::PlainText);
    }
    if content.html.is_some() {
        forms.push(Form::Markdown);
    }
    let text = content.text.as_deref().map(str::trim).unwrap_or_default();
    match content.kind {
        Some(Kind::Json) => {
            if let Ok(value) = serde_json::from_str::<Value>(text) {
                forms.extend([Form::JsonPretty, Form::JsonMinified]);
                if json_keys(&value).is_some() {
                    forms.push(Form::JsonKeys);
                }
                if json_table(&value).is_some() {
                    forms.push(Form::JsonTable);
                }
            }
        }
        Some(Kind::Color) => {
            if let Some(color) = parse_color(text) {
                forms.extend([Form::ColorHex, Form::ColorRgb, Form::ColorHsl]);
                if named_color(color).is_some() {
                    forms.push(Form::ColorName);
                }
            }
        }
        Some(Kind::Link) => {
            forms.push(Form::LinkMarkdown);
            if domain_of(text).is_some() {
                forms.push(Form::LinkDomain);
            }
            if title_of(content).is_some() {
                forms.push(Form::LinkTitled);
            }
        }
        Some(Kind::Code) => {
            forms.extend([Form::CodeOneLine, Form::CodeBlock, Form::CodeDedented]);
        }
        Some(Kind::Token) => {
            forms.extend([Form::TokenHeader, Form::TokenCurl]);
            if crate::token::claims_of(text).is_some() {
                forms.push(Form::TokenClaims);
            }
        }
        _ => {}
    }
    if content.image.is_some() {
        forms.push(Form::ImageJpeg);
        if content.ocr.is_some_and(|ocr| !ocr.trim().is_empty()) {
            forms.push(Form::ImageOcr);
        }
    }
    if !content.paths.is_empty() {
        forms.push(Form::Path);
        if names_of(&content.paths).is_some() {
            forms.push(Form::FileName);
        }
    }
    if matches!(content.kind, Some(Kind::Text) | None) && !text.is_empty() {
        if text.lines().count() > 1 {
            forms.push(Form::CodeOneLine);
        }
        forms.extend([Form::TextQuote, Form::TextUpper, Form::TextLower]);
    }
    forms
}

pub fn render(form: Form, content: &Content) -> Option<Rendered> {
    let text = content.text.as_deref().map(str::trim).unwrap_or_default();
    let rendered = match form {
        Form::PlainText => content.text.as_deref()?.to_owned(),
        Form::Markdown => markdown_of_html(content.html.as_deref()?),
        Form::JsonPretty => serde_json::to_string_pretty(&json_of(text)?).ok()?,
        Form::JsonMinified => serde_json::to_string(&json_of(text)?).ok()?,
        Form::JsonKeys => json_keys(&json_of(text)?)?,
        Form::JsonTable => json_table(&json_of(text)?)?,
        Form::ColorHex => hex_of(parse_color(text)?),
        Form::ColorRgb => rgb_of(parse_color(text)?),
        Form::ColorHsl => hsl_of(parse_color(text)?),
        Form::ColorName => named_color(parse_color(text)?)?.to_owned(),
        Form::LinkMarkdown => markdown_link(title_of(content).unwrap_or(text), text),
        Form::LinkDomain => lowercase_domain(text)?,
        Form::LinkTitled => format!("{} — {text}", title_of(content)?),
        Form::CodeOneLine => one_line(text),
        Form::CodeBlock => fenced(text),
        Form::CodeDedented => dedent(content.text.as_deref()?),
        Form::TokenHeader => format!("Authorization: Bearer {text}"),
        Form::TokenClaims => {
            let claims = crate::token::claims_of(text)?;
            serde_json::to_string_pretty(&Value::Object(claims.payload)).ok()?
        }
        Form::TokenCurl => format!(
            "curl -H 'Authorization: Bearer {}' \"$URL\"",
            text.replace('\'', "'\\''")
        ),
        Form::ImageJpeg => return jpeg_of(content.image?).map(Rendered::Jpeg),
        Form::ImageOcr => content.ocr?.trim().to_owned(),
        Form::Path => content.paths.join("\n"),
        Form::FileName => names_of(&content.paths)?,
        Form::TextQuote => quoted(content.text.as_deref()?),
        Form::TextUpper => content.text.as_deref()?.to_uppercase(),
        Form::TextLower => content.text.as_deref()?.to_lowercase(),
    };
    Some(Rendered::Text(rendered))
}

fn names_of(paths: &[String]) -> Option<String> {
    let names: Vec<&str> = paths
        .iter()
        .map(|path| {
            path.rsplit(['/', '\\'])
                .next()
                .unwrap_or(path.as_str())
                .trim()
        })
        .filter(|name| !name.is_empty())
        .collect();
    (!names.is_empty()).then(|| names.join("\n"))
}

fn quoted(text: &str) -> String {
    text.lines()
        .map(|line| {
            if line.trim().is_empty() {
                ">".to_owned()
            } else {
                format!("> {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn title_of<'a>(content: &'a Content<'_>) -> Option<&'a str> {
    content
        .title
        .as_deref()
        .map(str::trim)
        .filter(|title| !title.is_empty())
}

fn markdown_link(label: &str, url: &str) -> String {
    let label = label.replace(['[', ']'], " ");
    if url.contains([' ', '(', ')', '<', '>']) {
        format!("[{}](<{}>)", label.trim(), url.replace(['<', '>'], ""))
    } else {
        format!("[{}]({url})", label.trim())
    }
}

fn fenced(text: &str) -> String {
    let longest = text.split(|c| c != '`').map(str::len).max().unwrap_or(0);
    let fence = "`".repeat(longest.max(2) + 1);
    format!("{fence}\n{text}\n{fence}")
}

fn json_of(text: &str) -> Option<Value> {
    serde_json::from_str(text).ok()
}

fn json_keys(value: &Value) -> Option<String> {
    let keys: Vec<&str> = match value {
        Value::Object(map) => map.keys().map(String::as_str).collect(),
        Value::Array(rows) => {
            let mut seen = Vec::new();
            for row in rows {
                let Value::Object(map) = row else {
                    return None;
                };
                for key in map.keys() {
                    if !seen.contains(&key.as_str()) {
                        seen.push(key.as_str());
                    }
                }
            }
            seen
        }
        _ => return None,
    };
    (!keys.is_empty()).then(|| keys.join("\n"))
}

fn json_table(value: &Value) -> Option<String> {
    match value {
        Value::Object(map) => {
            let rows: Vec<String> = map
                .iter()
                .map(|(key, value)| format!("{key}\t{}", cell(value)))
                .collect();
            (!rows.is_empty()).then(|| rows.join("\n"))
        }
        Value::Array(rows) => {
            let header = json_keys(value)?;
            let columns: Vec<&str> = header.lines().collect();
            let mut lines = vec![columns.join("\t")];
            for row in rows {
                let Value::Object(map) = row else {
                    return None;
                };
                let cells: Vec<String> = columns
                    .iter()
                    .map(|column| map.get(*column).map(cell).unwrap_or_default())
                    .collect();
                lines.push(cells.join("\t"));
            }
            Some(lines.join("\n"))
        }
        _ => None,
    }
}

fn cell(value: &Value) -> String {
    match value {
        Value::String(text) => text.replace(['\t', '\n'], " "),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f64,
}

pub fn parse_color(text: &str) -> Option<Rgba> {
    if let Some(hex) = text.strip_prefix('#') {
        let digits: Vec<u8> = hex
            .chars()
            .map(|c| c.to_digit(16).map(|d| d as u8))
            .collect::<Option<_>>()?;
        let (r, g, b, a) = match digits.as_slice() {
            [r, g, b] => (r * 17, g * 17, b * 17, 255),
            [r, r2, g, g2, b, b2] => (r * 16 + r2, g * 16 + g2, b * 16 + b2, 255),
            [r, r2, g, g2, b, b2, a, a2] => (r * 16 + r2, g * 16 + g2, b * 16 + b2, a * 16 + a2),
            _ => return None,
        };
        return Some(Rgba {
            r,
            g,
            b,
            a: f64::from(a) / 255.0,
        });
    }
    let lower = text.to_ascii_lowercase();
    let (kind, inner) = lower
        .split_once('(')
        .and_then(|(kind, rest)| Some((kind, rest.strip_suffix(')')?)))?;
    let parts: Vec<(f64, bool)> = inner
        .split(',')
        .map(|part| {
            let part = part.trim();
            let percent = part.ends_with('%');
            part.trim_end_matches('%')
                .parse::<f64>()
                .ok()
                .map(|value| (value, percent))
        })
        .collect::<Option<_>>()?;
    let alpha = |value: Option<&(f64, bool)>| match value {
        Some((value, true)) => (value / 100.0).clamp(0.0, 1.0),
        Some((value, false)) => value.clamp(0.0, 1.0),
        None => 1.0,
    };
    let byte = |(value, percent): (f64, bool)| {
        channel(if percent {
            value * 255.0 / 100.0
        } else {
            value
        })
    };
    match (kind, parts.as_slice()) {
        ("rgb", [r, g, b]) | ("rgba", [r, g, b, _]) => Some(Rgba {
            r: byte(*r),
            g: byte(*g),
            b: byte(*b),
            a: alpha(parts.get(3)),
        }),
        ("hsl", [(h, _), (s, _), (l, _)]) | ("hsla", [(h, _), (s, _), (l, _), _]) => {
            let (r, g, b) = rgb_of_hsl(*h, s / 100.0, l / 100.0);
            Some(Rgba {
                r,
                g,
                b,
                a: alpha(parts.get(3)),
            })
        }
        _ => None,
    }
}

fn channel(value: f64) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

fn rgb_of_hsl(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let h = h.rem_euclid(360.0) / 360.0;
    let (s, l) = (s.clamp(0.0, 1.0), l.clamp(0.0, 1.0));
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let hue = |mut t: f64| {
        t = t.rem_euclid(1.0);
        let value = if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        };
        channel(value * 255.0)
    };
    (hue(h + 1.0 / 3.0), hue(h), hue(h - 1.0 / 3.0))
}

pub const NAME_WITHIN: u32 = 30;

pub fn named_color(color: Rgba) -> Option<&'static str> {
    if color.a < 1.0 {
        return None;
    }
    let (name, apart) = NAMED
        .iter()
        .map(|(name, rgb)| (*name, redmean_squared([color.r, color.g, color.b], *rgb)))
        .min_by_key(|(_, apart)| *apart)?;
    (apart <= 512 * NAME_WITHIN * NAME_WITHIN).then_some(name)
}

fn redmean_squared(a: [u8; 3], b: [u8; 3]) -> u32 {
    let reds = u32::from(a[0]) + u32::from(b[0]);
    let delta = |i: usize| {
        let d = i32::from(a[i]) - i32::from(b[i]);
        (d * d) as u32
    };
    (1024 + reds) * delta(0) + 2048 * delta(1) + (1024 + 510 - reds) * delta(2)
}

const NAMED: [(&str, [u8; 3]); 139] = [
    ("aliceblue", [0xF0, 0xF8, 0xFF]),
    ("antiquewhite", [0xFA, 0xEB, 0xD7]),
    ("aquamarine", [0x7F, 0xFF, 0xD4]),
    ("azure", [0xF0, 0xFF, 0xFF]),
    ("beige", [0xF5, 0xF5, 0xDC]),
    ("bisque", [0xFF, 0xE4, 0xC4]),
    ("black", [0x00, 0x00, 0x00]),
    ("blanchedalmond", [0xFF, 0xEB, 0xCD]),
    ("blue", [0x00, 0x00, 0xFF]),
    ("blueviolet", [0x8A, 0x2B, 0xE2]),
    ("brown", [0xA5, 0x2A, 0x2A]),
    ("burlywood", [0xDE, 0xB8, 0x87]),
    ("cadetblue", [0x5F, 0x9E, 0xA0]),
    ("chartreuse", [0x7F, 0xFF, 0x00]),
    ("chocolate", [0xD2, 0x69, 0x1E]),
    ("coral", [0xFF, 0x7F, 0x50]),
    ("cornflowerblue", [0x64, 0x95, 0xED]),
    ("cornsilk", [0xFF, 0xF8, 0xDC]),
    ("crimson", [0xDC, 0x14, 0x3C]),
    ("cyan", [0x00, 0xFF, 0xFF]),
    ("darkblue", [0x00, 0x00, 0x8B]),
    ("darkcyan", [0x00, 0x8B, 0x8B]),
    ("darkgoldenrod", [0xB8, 0x86, 0x0B]),
    ("darkgray", [0xA9, 0xA9, 0xA9]),
    ("darkgreen", [0x00, 0x64, 0x00]),
    ("darkkhaki", [0xBD, 0xB7, 0x6B]),
    ("darkmagenta", [0x8B, 0x00, 0x8B]),
    ("darkolivegreen", [0x55, 0x6B, 0x2F]),
    ("darkorange", [0xFF, 0x8C, 0x00]),
    ("darkorchid", [0x99, 0x32, 0xCC]),
    ("darkred", [0x8B, 0x00, 0x00]),
    ("darksalmon", [0xE9, 0x96, 0x7A]),
    ("darkseagreen", [0x8F, 0xBC, 0x8F]),
    ("darkslateblue", [0x48, 0x3D, 0x8B]),
    ("darkslategray", [0x2F, 0x4F, 0x4F]),
    ("darkturquoise", [0x00, 0xCE, 0xD1]),
    ("darkviolet", [0x94, 0x00, 0xD3]),
    ("deeppink", [0xFF, 0x14, 0x93]),
    ("deepskyblue", [0x00, 0xBF, 0xFF]),
    ("dimgray", [0x69, 0x69, 0x69]),
    ("dodgerblue", [0x1E, 0x90, 0xFF]),
    ("firebrick", [0xB2, 0x22, 0x22]),
    ("floralwhite", [0xFF, 0xFA, 0xF0]),
    ("forestgreen", [0x22, 0x8B, 0x22]),
    ("gainsboro", [0xDC, 0xDC, 0xDC]),
    ("ghostwhite", [0xF8, 0xF8, 0xFF]),
    ("gold", [0xFF, 0xD7, 0x00]),
    ("goldenrod", [0xDA, 0xA5, 0x20]),
    ("gray", [0x80, 0x80, 0x80]),
    ("green", [0x00, 0x80, 0x00]),
    ("greenyellow", [0xAD, 0xFF, 0x2F]),
    ("honeydew", [0xF0, 0xFF, 0xF0]),
    ("hotpink", [0xFF, 0x69, 0xB4]),
    ("indianred", [0xCD, 0x5C, 0x5C]),
    ("indigo", [0x4B, 0x00, 0x82]),
    ("ivory", [0xFF, 0xFF, 0xF0]),
    ("khaki", [0xF0, 0xE6, 0x8C]),
    ("lavender", [0xE6, 0xE6, 0xFA]),
    ("lavenderblush", [0xFF, 0xF0, 0xF5]),
    ("lawngreen", [0x7C, 0xFC, 0x00]),
    ("lemonchiffon", [0xFF, 0xFA, 0xCD]),
    ("lightblue", [0xAD, 0xD8, 0xE6]),
    ("lightcoral", [0xF0, 0x80, 0x80]),
    ("lightcyan", [0xE0, 0xFF, 0xFF]),
    ("lightgoldenrodyellow", [0xFA, 0xFA, 0xD2]),
    ("lightgray", [0xD3, 0xD3, 0xD3]),
    ("lightgreen", [0x90, 0xEE, 0x90]),
    ("lightpink", [0xFF, 0xB6, 0xC1]),
    ("lightsalmon", [0xFF, 0xA0, 0x7A]),
    ("lightseagreen", [0x20, 0xB2, 0xAA]),
    ("lightskyblue", [0x87, 0xCE, 0xFA]),
    ("lightslategray", [0x77, 0x88, 0x99]),
    ("lightsteelblue", [0xB0, 0xC4, 0xDE]),
    ("lightyellow", [0xFF, 0xFF, 0xE0]),
    ("lime", [0x00, 0xFF, 0x00]),
    ("limegreen", [0x32, 0xCD, 0x32]),
    ("linen", [0xFA, 0xF0, 0xE6]),
    ("magenta", [0xFF, 0x00, 0xFF]),
    ("maroon", [0x80, 0x00, 0x00]),
    ("mediumaquamarine", [0x66, 0xCD, 0xAA]),
    ("mediumblue", [0x00, 0x00, 0xCD]),
    ("mediumorchid", [0xBA, 0x55, 0xD3]),
    ("mediumpurple", [0x93, 0x70, 0xDB]),
    ("mediumseagreen", [0x3C, 0xB3, 0x71]),
    ("mediumslateblue", [0x7B, 0x68, 0xEE]),
    ("mediumspringgreen", [0x00, 0xFA, 0x9A]),
    ("mediumturquoise", [0x48, 0xD1, 0xCC]),
    ("mediumvioletred", [0xC7, 0x15, 0x85]),
    ("midnightblue", [0x19, 0x19, 0x70]),
    ("mintcream", [0xF5, 0xFF, 0xFA]),
    ("mistyrose", [0xFF, 0xE4, 0xE1]),
    ("moccasin", [0xFF, 0xE4, 0xB5]),
    ("navajowhite", [0xFF, 0xDE, 0xAD]),
    ("navy", [0x00, 0x00, 0x80]),
    ("oldlace", [0xFD, 0xF5, 0xE6]),
    ("olive", [0x80, 0x80, 0x00]),
    ("olivedrab", [0x6B, 0x8E, 0x23]),
    ("orange", [0xFF, 0xA5, 0x00]),
    ("orangered", [0xFF, 0x45, 0x00]),
    ("orchid", [0xDA, 0x70, 0xD6]),
    ("palegoldenrod", [0xEE, 0xE8, 0xAA]),
    ("palegreen", [0x98, 0xFB, 0x98]),
    ("paleturquoise", [0xAF, 0xEE, 0xEE]),
    ("palevioletred", [0xDB, 0x70, 0x93]),
    ("papayawhip", [0xFF, 0xEF, 0xD5]),
    ("peachpuff", [0xFF, 0xDA, 0xB9]),
    ("peru", [0xCD, 0x85, 0x3F]),
    ("pink", [0xFF, 0xC0, 0xCB]),
    ("plum", [0xDD, 0xA0, 0xDD]),
    ("powderblue", [0xB0, 0xE0, 0xE6]),
    ("purple", [0x80, 0x00, 0x80]),
    ("rebeccapurple", [0x66, 0x33, 0x99]),
    ("red", [0xFF, 0x00, 0x00]),
    ("rosybrown", [0xBC, 0x8F, 0x8F]),
    ("royalblue", [0x41, 0x69, 0xE1]),
    ("saddlebrown", [0x8B, 0x45, 0x13]),
    ("salmon", [0xFA, 0x80, 0x72]),
    ("sandybrown", [0xF4, 0xA4, 0x60]),
    ("seagreen", [0x2E, 0x8B, 0x57]),
    ("seashell", [0xFF, 0xF5, 0xEE]),
    ("sienna", [0xA0, 0x52, 0x2D]),
    ("silver", [0xC0, 0xC0, 0xC0]),
    ("skyblue", [0x87, 0xCE, 0xEB]),
    ("slateblue", [0x6A, 0x5A, 0xCD]),
    ("slategray", [0x70, 0x80, 0x90]),
    ("snow", [0xFF, 0xFA, 0xFA]),
    ("springgreen", [0x00, 0xFF, 0x7F]),
    ("steelblue", [0x46, 0x82, 0xB4]),
    ("tan", [0xD2, 0xB4, 0x8C]),
    ("teal", [0x00, 0x80, 0x80]),
    ("thistle", [0xD8, 0xBF, 0xD8]),
    ("tomato", [0xFF, 0x63, 0x47]),
    ("turquoise", [0x40, 0xE0, 0xD0]),
    ("violet", [0xEE, 0x82, 0xEE]),
    ("wheat", [0xF5, 0xDE, 0xB3]),
    ("white", [0xFF, 0xFF, 0xFF]),
    ("whitesmoke", [0xF5, 0xF5, 0xF5]),
    ("yellow", [0xFF, 0xFF, 0x00]),
    ("yellowgreen", [0x9A, 0xCD, 0x32]),
];

fn hex_of(color: Rgba) -> String {
    if color.a < 1.0 {
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            color.r,
            color.g,
            color.b,
            channel(color.a * 255.0)
        )
    } else {
        format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
    }
}

fn rgb_of(color: Rgba) -> String {
    if color.a < 1.0 {
        format!(
            "rgba({}, {}, {}, {})",
            color.r,
            color.g,
            color.b,
            trim_float(color.a)
        )
    } else {
        format!("rgb({}, {}, {})", color.r, color.g, color.b)
    }
}

fn hsl_of(color: Rgba) -> String {
    let (r, g, b) = (
        f64::from(color.r) / 255.0,
        f64::from(color.g) / 255.0,
        f64::from(color.b) / 255.0,
    );
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let delta = max - min;
    let (h, s) = if delta == 0.0 {
        (0.0, 0.0)
    } else {
        let s = delta / (1.0 - (2.0 * l - 1.0).abs());
        let h = if max == r {
            60.0 * (((g - b) / delta).rem_euclid(6.0))
        } else if max == g {
            60.0 * ((b - r) / delta + 2.0)
        } else {
            60.0 * ((r - g) / delta + 4.0)
        };
        (h, s)
    };
    let (h, s, l) = (
        h.round() as i64,
        (s * 100.0).round() as i64,
        (l * 100.0).round() as i64,
    );
    if color.a < 1.0 {
        format!("hsla({h}, {s}%, {l}%, {})", trim_float(color.a))
    } else {
        format!("hsl({h}, {s}%, {l}%)")
    }
}

fn trim_float(value: f64) -> String {
    let text = format!("{value:.2}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

pub fn domain_of(url: &str) -> Option<&str> {
    let rest = match url.trim().split_once(':') {
        Some((scheme, rest))
            if scheme.starts_with(|c: char| c.is_ascii_alphabetic())
                && scheme
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c)) =>
        {
            rest.trim_start_matches('/')
        }
        _ => url.trim(),
    };
    if rest.is_empty() {
        return None;
    }
    let authority = rest.split(['/', '?', '#']).next()?;
    let host = authority.rsplit('@').next()?;
    let host = match host.rsplit_once(':') {
        Some((name, port)) if !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) => name,
        _ => host,
    };
    let host = host.strip_prefix("www.").unwrap_or(host);
    (!host.is_empty() && host.contains('.')).then_some(host)
}

fn lowercase_domain(url: &str) -> Option<String> {
    domain_of(url).map(str::to_ascii_lowercase)
}

fn one_line(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn dedent(text: &str) -> String {
    let common = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| &line[..line.len() - line.trim_start().len()])
        .reduce(|shared, next| {
            let kept = shared
                .chars()
                .zip(next.chars())
                .take_while(|(a, b)| a == b)
                .map(|(a, _)| a.len_utf8())
                .sum();
            &shared[..kept]
        })
        .unwrap_or("");
    let mut out = text
        .lines()
        .map(|line| {
            line.strip_prefix(common)
                .unwrap_or_else(|| line.trim_start())
        })
        .collect::<Vec<_>>()
        .join("\n");
    if text.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn jpeg_of(png: &[u8]) -> Option<Vec<u8>> {
    let decoded = image::load_from_memory(png).ok()?.to_rgba8();
    let (width, height) = decoded.dimensions();
    let mut flat = image::RgbImage::new(width, height);
    for (x, y, pixel) in decoded.enumerate_pixels() {
        let alpha = u32::from(pixel[3]);
        let over_white = |c: u8| ((u32::from(c) * alpha + 255 * (255 - alpha)) / 255) as u8;
        flat.put_pixel(
            x,
            y,
            image::Rgb([
                over_white(pixel[0]),
                over_white(pixel[1]),
                over_white(pixel[2]),
            ]),
        );
    }
    let mut out = Vec::new();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, JPEG_QUALITY);
    flat.write_with_encoder(encoder).ok()?;
    Some(out)
}

pub fn markdown_of_html(html: &str) -> String {
    let fragment = fragment_of(html);
    let mut writer = MarkdownWriter::default();
    let mut rest = fragment;
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix('<')
            && after
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || "/!?".contains(c))
        {
            match tag_end(after) {
                Some(end) => {
                    writer.tag(&after[..end]);
                    rest = &after[end + 1..];
                    continue;
                }
                None => {
                    writer.text(&decode_entities(rest));
                    break;
                }
            }
        }
        let first = rest.chars().next().map_or(1, char::len_utf8);
        let next = rest[first..].find('<').map_or(rest.len(), |at| at + first);
        writer.text(&decode_entities(&rest[..next]));
        rest = &rest[next..];
    }
    writer.finish()
}

fn tag_end(after: &str) -> Option<usize> {
    if after.starts_with("!--") {
        return after.find("-->").map(|at| at + 2);
    }
    let mut quote: Option<char> = None;
    for (at, c) in after.char_indices() {
        match (quote, c) {
            (Some(open), c) if c == open => quote = None,
            (None, '"' | '\'') => quote = Some(c),
            (None, '>') => return Some(at),
            _ => {}
        }
    }
    None
}

fn fragment_of(html: &str) -> &str {
    let start = html
        .find("<!--StartFragment-->")
        .map_or(0, |at| at + "<!--StartFragment-->".len());
    let end = html.find("<!--EndFragment-->").unwrap_or(html.len());
    if start <= end {
        &html[start..end]
    } else {
        html
    }
}

const SKIPPED: [&str; 4] = ["script", "style", "head", "title"];

struct MarkdownWriter {
    out: String,
    lists: Vec<Option<usize>>,
    link: Option<(String, usize)>,
    skipping: Option<&'static str>,
    preformatted: bool,
    quoting: Vec<usize>,
    opening: String,
    fresh: bool,
    inline: Vec<&'static str>,
    last_closed: Option<(&'static str, usize)>,
    row: Option<Vec<String>>,
    cell: Option<usize>,
    rows: usize,
}

impl Default for MarkdownWriter {
    fn default() -> Self {
        Self {
            out: String::new(),
            lists: Vec::new(),
            link: None,
            skipping: None,
            preformatted: false,
            quoting: Vec::new(),
            opening: String::new(),
            fresh: true,
            inline: Vec::new(),
            last_closed: None,
            row: None,
            cell: None,
            rows: 0,
        }
    }
}

impl MarkdownWriter {
    fn tag(&mut self, raw: &str) {
        let raw = raw.trim();
        if raw.starts_with('!') {
            return;
        }
        let closing = raw.starts_with('/');
        let body = raw.trim_start_matches('/').trim_end_matches('/').trim();
        let name = body
            .split(|c: char| c.is_whitespace())
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        if let Some(skipped) = self.skipping {
            if closing && name == skipped {
                self.skipping = None;
            }
            return;
        }
        match (name.as_str(), closing) {
            ("script" | "style" | "head" | "title", false) => {
                self.skipping = SKIPPED.iter().copied().find(|tag| *tag == name);
            }
            ("br", _) => {
                if !self.at_line_start() {
                    self.out.push('\n');
                    self.fresh = true;
                }
            }
            ("p" | "div" | "section" | "article", _) => self.blank_line(),
            ("table", false) => {
                self.blank_line();
                self.rows = 0;
            }
            ("table", true) => {
                self.row = None;
                self.blank_line();
            }
            ("tr", false) => self.row = Some(Vec::new()),
            ("tr", true) => self.end_row(),
            ("td" | "th", false) => self.cell = Some(self.out.len()),
            ("td" | "th", true) => self.end_cell(),
            ("h1" | "h2" | "h3" | "h4" | "h5" | "h6", false) => {
                self.blank_line();
                let level = name[1..].parse::<usize>().unwrap_or(1);
                self.out.push_str(&"#".repeat(level));
                self.out.push(' ');
            }
            ("h1" | "h2" | "h3" | "h4" | "h5" | "h6", true) => self.blank_line(),
            ("b" | "strong" | "i" | "em" | "code" | "span", false) => {
                let marker = inline_marker(&name, attribute(body, "style").as_deref());
                let marker = if self.preformatted { "" } else { marker };
                self.inline.push(marker);
                self.open(marker);
            }
            ("b" | "strong" | "i" | "em" | "code" | "span", true) => {
                if let Some(marker) = self.inline.pop() {
                    self.close(marker);
                }
            }
            ("pre", false) => {
                self.blank_line();
                self.out.push_str("```\n");
                self.preformatted = true;
            }
            ("pre", true) => {
                self.preformatted = false;
                self.newline();
                self.out.push_str("```");
                self.blank_line();
            }
            ("ul", false) => {
                self.blank_line_if_top_level();
                self.lists.push(None);
            }
            ("ol", false) => {
                self.blank_line_if_top_level();
                self.lists.push(Some(0));
            }
            ("ul" | "ol", true) => {
                self.lists.pop();
                if self.lists.is_empty() {
                    self.blank_line();
                }
            }
            ("li", false) => {
                self.newline();
                let depth = self.lists.len().saturating_sub(1);
                self.out.push_str(&"  ".repeat(depth));
                match self.lists.last_mut() {
                    Some(Some(count)) => {
                        *count += 1;
                        self.out.push_str(&format!("{count}. "));
                    }
                    _ => self.out.push_str("- "),
                }
            }
            ("blockquote", false) => {
                self.blank_line();
                self.quoting.push(self.out.len());
            }
            ("blockquote", true) => {
                if let Some(from) = self.quoting.pop() {
                    self.newline();
                    let quoted = self.out[from..]
                        .trim_end()
                        .lines()
                        .map(|line| {
                            if line.is_empty() {
                                ">".to_owned()
                            } else {
                                format!("> {line}")
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    self.truncate_to(from);
                    self.out.push_str(&quoted);
                    self.fresh = false;
                }
                self.blank_line();
            }
            ("a", false) => {
                let href = attribute(body, "href").unwrap_or_default();
                self.link = Some((href, self.out.len()));
            }
            ("a", true) => {
                if let Some((href, from)) = self.link.take() {
                    let label = self.out[from..].trim().to_owned();
                    self.truncate_to(from);
                    if href.is_empty() {
                        self.out.push_str(&label);
                    } else {
                        self.out.push_str(&format!("[{label}]({href})"));
                    }
                }
            }
            ("img", false) => {
                let alt = attribute(body, "alt").unwrap_or_default();
                let src = attribute(body, "src").unwrap_or_default();
                self.out.push_str(&format!("![{alt}]({src})"));
                self.fresh = false;
            }
            ("hr", _) => {
                self.blank_line();
                self.out.push_str("---");
                self.fresh = false;
                self.blank_line();
            }
            _ => {}
        }
    }

    fn end_cell(&mut self) {
        let Some(from) = self.cell.take() else {
            return;
        };
        if self.row.is_none() {
            return;
        }
        let text = self.out[from..]
            .trim()
            .replace('\n', " ")
            .replace('|', "\\|");
        self.truncate_to(from);
        if let Some(row) = &mut self.row {
            row.push(text);
        }
    }

    fn end_row(&mut self) {
        self.end_cell();
        let Some(row) = self.row.take() else {
            return;
        };
        if row.is_empty() {
            return;
        }
        self.newline();
        self.out.push_str(&format!("| {} |", row.join(" | ")));
        if self.rows == 0 {
            self.out
                .push_str(&format!("\n|{}", " --- |".repeat(row.len())));
        }
        self.rows += 1;
        self.fresh = false;
        self.newline();
    }

    fn open(&mut self, marker: &'static str) {
        if let Some((closed, at)) = self.last_closed.take()
            && closed == marker
            && at == self.out.len()
        {
            self.truncate_to(at - marker.len());
            return;
        }
        self.opening.push_str(marker);
    }

    fn close(&mut self, marker: &'static str) {
        if marker.is_empty() {
            return;
        }
        if let Some(unopened) = self.opening.strip_suffix(marker) {
            self.opening = unopened.to_owned();
            return;
        }
        if self.preformatted {
            self.out.push_str(marker);
            return;
        }
        let kept = self.out.trim_end_matches(' ').len();
        let had_space = kept < self.out.len();
        self.truncate_to(kept);
        self.out.push_str(marker);
        self.last_closed = Some((marker, self.out.len()));
        if had_space {
            self.out.push(' ');
        }
    }

    fn text(&mut self, text: &str) {
        if self.skipping.is_some() {
            return;
        }
        if self.preformatted {
            let opening = std::mem::take(&mut self.opening);
            self.out.push_str(&opening);
            self.out.push_str(text);
            self.fresh = text.ends_with('\n');
            return;
        }
        let leading = text.starts_with(char::is_whitespace);
        let trailing = text.ends_with(char::is_whitespace);
        let words: Vec<&str> = text.split_whitespace().collect();
        if leading && !self.at_line_start() && !self.out.ends_with(' ') {
            self.out.push(' ');
        }
        if words.is_empty() {
            return;
        }
        let opening = std::mem::take(&mut self.opening);
        self.out.push_str(&opening);
        self.out.push_str(&words.join(" "));
        self.fresh = false;
        if trailing {
            self.out.push(' ');
        }
    }

    fn at_line_start(&self) -> bool {
        self.fresh
    }

    fn truncate_to(&mut self, len: usize) {
        self.out.truncate(len);
        if let Some((_, from)) = &mut self.link {
            *from = (*from).min(len);
        }
        if let Some(from) = &mut self.cell {
            *from = (*from).min(len);
        }
        for from in &mut self.quoting {
            *from = (*from).min(len);
        }
    }

    fn newline(&mut self) {
        let trimmed = self.out.trim_end_matches(' ').len();
        self.truncate_to(trimmed);
        if !self.out.is_empty() && !self.out.ends_with('\n') {
            self.out.push('\n');
        }
        self.fresh = true;
    }

    fn blank_line(&mut self) {
        self.newline();
        if !self.out.is_empty() && !self.out.ends_with("\n\n") {
            self.out.push('\n');
        }
    }

    fn blank_line_if_top_level(&mut self) {
        if self.lists.is_empty() {
            self.blank_line();
        }
    }

    fn finish(mut self) -> String {
        self.newline();
        let mut lines: Vec<&str> = self.out.lines().map(str::trim_end).collect();
        lines.dedup_by(|a, b| a.is_empty() && b.is_empty());
        lines.join("\n").trim_end().to_owned()
    }
}

fn rule_of(style: &str, property: &str) -> Option<String> {
    style
        .split(';')
        .filter_map(|rule| rule.split_once(':'))
        .find(|(key, _)| key.trim() == property)
        .map(|(_, value)| value.trim().to_owned())
}

fn inline_marker(name: &str, style: Option<&str>) -> &'static str {
    let style = style.unwrap_or_default().to_ascii_lowercase();
    let weight = rule_of(&style, "font-weight");
    let bold = match (name, weight.as_deref()) {
        ("b" | "strong", Some("normal" | "400" | "300" | "200" | "100")) => false,
        ("b" | "strong", _) => true,
        (_, Some("bold" | "bolder" | "600" | "700" | "800" | "900")) => true,
        _ => false,
    };
    let slant = rule_of(&style, "font-style");
    let italic = match (name, slant.as_deref()) {
        ("i" | "em", Some("normal")) => false,
        ("i" | "em", _) => true,
        (_, Some("italic" | "oblique")) => true,
        _ => false,
    };
    match (name, bold, italic) {
        ("code", _, _) => "`",
        (_, true, true) => "***",
        (_, true, false) => "**",
        (_, false, true) => "*",
        _ => "",
    }
}

fn attribute(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let wanted = format!("{name}=");
    let at = lower
        .match_indices(&wanted)
        .map(|(at, _)| at)
        .find(|at| lower[..*at].ends_with(char::is_whitespace))
        .map(|at| at + wanted.len())?;
    let rest = &tag[at..];
    let value = match rest.chars().next()? {
        quote @ ('"' | '\'') => rest[1..].split(quote).next()?,
        _ => rest.split(char::is_whitespace).next()?,
    };
    Some(decode_entities(value))
}

const NAMED_ENTITIES: [(&str, char); 84] = [
    ("iexcl", '¡'),
    ("cent", '¢'),
    ("pound", '£'),
    ("euro", '€'),
    ("yen", '¥'),
    ("sect", '§'),
    ("copy", '©'),
    ("laquo", '«'),
    ("reg", '®'),
    ("deg", '°'),
    ("plusmn", '±'),
    ("micro", 'µ'),
    ("para", '¶'),
    ("middot", '·'),
    ("raquo", '»'),
    ("frac12", '½'),
    ("iquest", '¿'),
    ("times", '×'),
    ("divide", '÷'),
    ("ndash", '–'),
    ("mdash", '—'),
    ("lsquo", '‘'),
    ("rsquo", '’'),
    ("ldquo", '“'),
    ("rdquo", '”'),
    ("hellip", '…'),
    ("trade", '™'),
    ("bull", '•'),
    ("Agrave", 'À'),
    ("Aacute", 'Á'),
    ("Acirc", 'Â'),
    ("Atilde", 'Ã'),
    ("Auml", 'Ä'),
    ("Aring", 'Å'),
    ("AElig", 'Æ'),
    ("Ccedil", 'Ç'),
    ("Egrave", 'È'),
    ("Eacute", 'É'),
    ("Ecirc", 'Ê'),
    ("Euml", 'Ë'),
    ("Igrave", 'Ì'),
    ("Iacute", 'Í'),
    ("Icirc", 'Î'),
    ("Iuml", 'Ï'),
    ("Ntilde", 'Ñ'),
    ("Ograve", 'Ò'),
    ("Oacute", 'Ó'),
    ("Ocirc", 'Ô'),
    ("Otilde", 'Õ'),
    ("Ouml", 'Ö'),
    ("Oslash", 'Ø'),
    ("Ugrave", 'Ù'),
    ("Uacute", 'Ú'),
    ("Ucirc", 'Û'),
    ("Uuml", 'Ü'),
    ("szlig", 'ß'),
    ("agrave", 'à'),
    ("aacute", 'á'),
    ("acirc", 'â'),
    ("atilde", 'ã'),
    ("auml", 'ä'),
    ("aring", 'å'),
    ("aelig", 'æ'),
    ("ccedil", 'ç'),
    ("egrave", 'è'),
    ("eacute", 'é'),
    ("ecirc", 'ê'),
    ("euml", 'ë'),
    ("igrave", 'ì'),
    ("iacute", 'í'),
    ("icirc", 'î'),
    ("iuml", 'ï'),
    ("ntilde", 'ñ'),
    ("ograve", 'ò'),
    ("oacute", 'ó'),
    ("ocirc", 'ô'),
    ("otilde", 'õ'),
    ("ouml", 'ö'),
    ("oslash", 'ø'),
    ("ugrave", 'ù'),
    ("uacute", 'ú'),
    ("ucirc", 'û'),
    ("uuml", 'ü'),
    ("yacute", 'ý'),
];

fn named_entity(name: &str) -> Option<char> {
    NAMED_ENTITIES
        .iter()
        .find(|(known, _)| *known == name)
        .map(|(_, c)| *c)
}

fn decode_entities(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        let after = &rest[at + 1..];
        let Some(end) = after.find(';').filter(|end| *end <= 10) else {
            out.push('&');
            rest = after;
            continue;
        };
        let entity = &after[..end];
        let decoded = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            "nbsp" => Some(' '),
            named if named.chars().all(|c| c.is_ascii_alphabetic()) => named_entity(named),
            _ => entity
                .strip_prefix('#')
                .and_then(|number| match number.strip_prefix(['x', 'X']) {
                    Some(hex) => u32::from_str_radix(hex, 16).ok(),
                    None => number.parse().ok(),
                })
                .and_then(char::from_u32),
        };
        match decoded {
            Some(c) => {
                out.push(c);
                rest = &after[end + 1..];
            }
            None => {
                out.push('&');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
#[path = "paste_as_test.rs"]
mod tests;
