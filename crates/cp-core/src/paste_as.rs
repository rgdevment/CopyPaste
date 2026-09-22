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
mod tests {
    use super::*;

    fn text_of(kind: Kind, text: &str) -> Content<'_> {
        Content {
            kind: Some(kind),
            text: Some(text.into()),
            ..Default::default()
        }
    }

    fn rendered(form: Form, content: &Content) -> String {
        match render(form, content).expect("se puede") {
            Rendered::Text(text) => text,
            other => panic!("no era texto: {other:?}"),
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
        assert_eq!(Form::from_name("Markdown"), None, "el nombre es exacto");
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
            "el texto puede ser prestado o propio"
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
            "sin texto no hay nada que ofrecer"
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
            "juntar las líneas solo se ofrece cuando hay más de una"
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
            "una ruta sin nombre no se ofrece"
        );
        let padded = Content {
            kind: Some(Kind::File),
            paths: vec!["/tmp/  con espacios.txt  ".into()],
            ..Default::default()
        };
        assert_eq!(
            rendered(Form::FileName, &padded),
            "con espacios.txt",
            "el nombre llega limpio de espacios"
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
                assert!(
                    render(form, content).is_some(),
                    "{form:?} sobre {content:?}"
                );
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
            "el texto plano se entrega como se copió, sin recortar"
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
            "en el orden en que estaban las claves"
        );
        assert_eq!(
            rendered(Form::JsonMinified, &content),
            r#"{"b":1,"a":{"x":[1,2]},"c":"hola\tque tal"}"#
        );
        assert_eq!(rendered(Form::JsonKeys, &content), "b\na\nc");
        assert_eq!(
            rendered(Form::JsonTable, &content),
            "b\t1\na\t{\"x\":[1,2]}\nc\thola que tal",
            "una tabulación entre clave y valor; los anidados van minificados"
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
            "las columnas son la unión de claves; lo que falta queda vacío"
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
            "una diferencia de rojo pesa más donde hay mucho rojo"
        );
        assert!(
            redmean_squared([250, 0, 0], [250, 0, 15]) < redmean_squared([10, 0, 0], [10, 0, 15]),
            "y una de azul pesa más donde hay poco rojo"
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
            "fuera de rango se recorta, no se rompe"
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
            "un dominio no distingue mayúsculas"
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
            "sin punto no es dominio"
        );
        assert_eq!(domain_of("https://"), None);
        assert_eq!(domain_of(""), None);
        assert_eq!(
            domain_of("192.168.0.1:8080"),
            Some("192.168.0.1"),
            "sin esquema, lo de antes del puerto es el host"
        );
        assert_eq!(
            domain_of("https://ejemplo.test:abc/"),
            Some("ejemplo.test:abc"),
            "un puerto que no es número no se recorta"
        );
        assert_eq!(domain_of("https://ejemplo.test:/"), Some("ejemplo.test:"));
        assert_eq!(
            domain_of(":ejemplo.test"),
            Some(":ejemplo.test"),
            "un esquema vacío no es esquema"
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
            "se quita lo que todas las líneas comparten y nada más"
        );
        assert_eq!(
            dedent("\n  a\n\n    b\n"),
            "\na\n\n  b\n",
            "las líneas vacías no cuentan"
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
        assert_eq!(&bytes[..2], &[0xFF, 0xD8], "cabecera JPEG");
        let back = image::load_from_memory(bytes).expect("se lee").to_rgb8();
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
                other => panic!("no salió un JPEG: {other:?}"),
            }
        };
        assert!(
            near(jpeg_of_solid([200, 30, 30, 255]), [200, 30, 30]),
            "opaco intacto"
        );
        assert!(
            near(jpeg_of_solid([0, 0, 0, 0]), [255, 255, 255]),
            "lo transparente se vuelve blanco, no negro"
        );
        assert!(
            near(jpeg_of_solid([0, 0, 255, 128]), [127, 127, 255]),
            "azul al 50 % sobre blanco"
        );
        assert!(
            near(jpeg_of_solid([0, 0, 255, 64]), [191, 191, 255]),
            "azul al 25 % sobre blanco"
        );
        assert!(
            near(jpeg_of_solid([200, 30, 30, 128]), [227, 142, 142]),
            "un rojo apagado al 50 % se aclara canal a canal"
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
            panic!("un TIFF también se vuelve JPEG");
        };
        let middle = middle_of_jpeg(&bytes);
        assert!(
            middle[0].abs_diff(10) <= 6
                && middle[1].abs_diff(200) <= 6
                && middle[2].abs_diff(30) <= 6,
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
            "fuera de Unicode"
        );
        assert_eq!(decode_entities("&aacute;&Ntilde;&euro;&hellip;"), "áÑ€…");
        assert_eq!(decode_entities("&yacute;&uuml;"), "ýü");
        assert_eq!(
            decode_entities("&rarr;"),
            "&rarr;",
            "las nombradas que no se conocen se dejan"
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
            "un salto al principio no deja hueco"
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
            "dentro de un preformateado los espacios no se recortan, un cierre suelto se ignora y el ancla vacía no rompe"
        );
        assert_eq!(
            markdown_of_html("<pre><b>x </b>y</pre>"),
            "```\nx y\n```",
            "dentro de un cerco de código los asteriscos serían literales"
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
            "una celda fuera de una fila no se traga el texto"
        );
        assert_eq!(
            markdown_of_html("<table><tr><td>ab<blockquote>x</td></blockquote></tr></table>"),
            "| ab  x |\n| --- |",
            "la celda recorta la salida por debajo del inicio de la cita, y la cita no se cae"
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
            "con texto en medio no se funden"
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
            "nada en común"
        );
        assert_eq!(dedent("\u{a0}\u{a0}x\n\u{a0}\u{a0}\u{a0}y"), "x\n\u{a0}y");
        assert_eq!(dedent("\tx\n\t\ty"), "x\n\ty");
        assert_eq!(
            dedent("\tx\n  y"),
            "\tx\n  y",
            "tabulador y espacios no son lo mismo"
        );
        assert_eq!(
            dedent("  x\n \n  y"),
            "x\n\ny",
            "una línea en blanco corta no estorba"
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
}
