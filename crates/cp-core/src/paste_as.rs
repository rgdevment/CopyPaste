use crate::kind::Kind;
use serde_json::Value;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Content<'a> {
    pub kind: Option<Kind>,
    pub text: Option<&'a str>,
    pub html: Option<&'a str>,
    pub rich: bool,
    pub png: Option<&'a [u8]>,
    pub paths: Vec<String>,
    pub title: Option<&'a str>,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rendered {
    Text(String),
    Png(Vec<u8>),
    Jpeg(Vec<u8>),
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
    let text = content.text.map(str::trim).unwrap_or_default();
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
            if parse_color(text).is_some() {
                forms.extend([Form::ColorHex, Form::ColorRgb, Form::ColorHsl]);
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
    if content.png.is_some() {
        forms.push(Form::ImageJpeg);
        if content.ocr.is_some_and(|ocr| !ocr.trim().is_empty()) {
            forms.push(Form::ImageOcr);
        }
    }
    if !content.paths.is_empty() {
        forms.push(Form::Path);
    }
    forms
}

pub fn render(form: Form, content: &Content) -> Option<Rendered> {
    let text = content.text.map(str::trim).unwrap_or_default();
    let rendered = match form {
        Form::PlainText => content.text?.to_owned(),
        Form::Markdown => markdown_of_html(content.html?),
        Form::JsonPretty => serde_json::to_string_pretty(&json_of(text)?).ok()?,
        Form::JsonMinified => serde_json::to_string(&json_of(text)?).ok()?,
        Form::JsonKeys => json_keys(&json_of(text)?)?,
        Form::JsonTable => json_table(&json_of(text)?)?,
        Form::ColorHex => hex_of(parse_color(text)?),
        Form::ColorRgb => rgb_of(parse_color(text)?),
        Form::ColorHsl => hsl_of(parse_color(text)?),
        Form::LinkMarkdown => markdown_link(title_of(content).unwrap_or(text), text),
        Form::LinkDomain => domain_of(text)?.to_owned(),
        Form::LinkTitled => format!("{} — {text}", title_of(content)?),
        Form::CodeOneLine => one_line(text),
        Form::CodeBlock => fenced(text),
        Form::CodeDedented => dedent(content.text?),
        Form::TokenHeader => format!("Authorization: Bearer {text}"),
        Form::TokenClaims => {
            let claims = crate::token::claims_of(text)?;
            serde_json::to_string_pretty(&Value::Object(claims.payload)).ok()?
        }
        Form::TokenCurl => format!(
            "curl -H 'Authorization: Bearer {}' \"$URL\"",
            text.replace('\'', "'\\''")
        ),
        Form::ImageJpeg => return jpeg_of(content.png?).map(Rendered::Jpeg),
        Form::ImageOcr => content.ocr?.trim().to_owned(),
        Form::Path => content.paths.join("\n"),
    };
    Some(Rendered::Text(rendered))
}

fn title_of<'a>(content: &Content<'a>) -> Option<&'a str> {
    content
        .title
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
    quoting: Option<usize>,
    opening: String,
    fresh: bool,
}

impl Default for MarkdownWriter {
    fn default() -> Self {
        Self {
            out: String::new(),
            lists: Vec::new(),
            link: None,
            skipping: None,
            preformatted: false,
            quoting: None,
            opening: String::new(),
            fresh: true,
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
            ("p" | "div" | "tr" | "table" | "section" | "article", _) => self.blank_line(),
            ("h1" | "h2" | "h3" | "h4" | "h5" | "h6", false) => {
                self.blank_line();
                let level = name[1..].parse::<usize>().unwrap_or(1);
                self.out.push_str(&"#".repeat(level));
                self.out.push(' ');
            }
            ("h1" | "h2" | "h3" | "h4" | "h5" | "h6", true) => self.blank_line(),
            ("b" | "strong", false) => self.open("**"),
            ("i" | "em", false) => self.open("*"),
            ("code", false) if !self.preformatted => self.open("`"),
            ("b" | "strong", true) => self.close("**"),
            ("i" | "em", true) => self.close("*"),
            ("code", true) if !self.preformatted => self.close("`"),
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
                self.quoting = Some(self.out.len());
            }
            ("blockquote", true) => {
                if let Some(from) = self.quoting.take() {
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
                    self.out.truncate(from);
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
                    let from = from.min(self.out.len());
                    let label = self.out[from..].trim().to_owned();
                    self.out.truncate(from);
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
            ("td" | "th", true) => self.out.push('\t'),
            ("hr", _) => {
                self.blank_line();
                self.out.push_str("---");
                self.fresh = false;
                self.blank_line();
            }
            _ => {}
        }
    }

    fn open(&mut self, marker: &str) {
        self.opening.push_str(marker);
    }

    fn close(&mut self, marker: &str) {
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
        self.out.truncate(kept);
        self.out.push_str(marker);
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

    fn newline(&mut self) {
        let trimmed = self.out.trim_end_matches(' ').len();
        self.out.truncate(trimmed);
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

fn attribute(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let at = lower
        .find(&format!("{name}="))
        .map(|at| at + name.len() + 1)?;
    let rest = &tag[at..];
    let value = match rest.chars().next()? {
        quote @ ('"' | '\'') => rest[1..].split(quote).next()?,
        _ => rest.split(char::is_whitespace).next()?,
    };
    Some(decode_entities(value))
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
            text: Some(text),
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
    fn plain_text_has_no_forms_of_its_own() {
        assert!(forms_for(&text_of(Kind::Text, "hola")).is_empty());
        assert!(forms_for(&Content::default()).is_empty());
    }

    #[test]
    fn every_form_that_is_offered_can_be_rendered() {
        let png = tiny_png();
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJjcC0zIn0.firma";
        let contents = [
            Content {
                kind: Some(Kind::Text),
                text: Some("hola"),
                html: Some("<b>hola</b>"),
                rich: true,
                ..Default::default()
            },
            text_of(Kind::Json, r#"[{"a": 1}, {"b": 2}]"#),
            text_of(Kind::Json, "[1, 2]"),
            text_of(Kind::Color, "hsla(10, 20%, 30%, 40%)"),
            Content {
                title: Some("Ejemplo"),
                ..text_of(Kind::Link, "https://ejemplo.test/a b")
            },
            text_of(Kind::Link, "https://localhost/"),
            text_of(Kind::Code, "  x\n  y"),
            text_of(Kind::Token, jwt),
            text_of(Kind::Token, "ghp_not_a_real_token_for_tests_0000000000"),
            Content {
                kind: Some(Kind::Image),
                png: Some(&png),
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
            text: Some("hola"),
            html: Some("<b>hola</b>"),
            rich: true,
            ..Default::default()
        };
        assert_eq!(forms_for(&with_html), vec![Form::PlainText, Form::Markdown]);
        assert_eq!(rendered(Form::Markdown, &with_html), "**hola**");
        let rtf_only = Content {
            rich: true,
            ..text_of(Kind::Text, "hola")
        };
        assert_eq!(forms_for(&rtf_only), vec![Form::PlainText]);
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
            vec![Form::ColorHex, Form::ColorRgb, Form::ColorHsl]
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
        let titled = Content {
            title: Some(" Ejemplo, la página "),
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
            png: Some(&png),
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
                png: Some(&png),
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
            png: Some(b"no es png"),
            ..Default::default()
        };
        assert_eq!(render(Form::ImageJpeg, &broken), None);
    }

    #[test]
    fn files_are_offered_by_their_paths() {
        let content = Content {
            kind: Some(Kind::File),
            paths: vec!["/tmp/uno.txt".into(), "/tmp/dos.txt".into()],
            ..Default::default()
        };
        assert_eq!(forms_for(&content), vec![Form::Path]);
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
             Un p&aacute;rrafo con **negrita**, *cursiva* y un [enlace](https://ejemplo.test).\n\n\
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
        assert_eq!(
            decode_entities("&aacute;"),
            "&aacute;",
            "las nombradas raras se dejan"
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
            "a\tb"
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
            "```\na    [**](x)\n```",
            "dentro de un preformateado los espacios no se recortan y el ancla vacía no rompe"
        );
        assert_eq!(
            markdown_of_html("<pre><b>x </b>y</pre>"),
            "```\n**x **y\n```"
        );
        assert_eq!(markdown_of_html("<a href=\"x\"> </a>"), "[](x)");
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
            text: Some("abc'; rm -rf / #"),
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
            title: Some(" [Sección] "),
            ..text_of(Kind::Link, "https://ejemplo.test/a b(c)")
        };
        assert_eq!(
            rendered(Form::LinkMarkdown, &spaced),
            "[Sección](<https://ejemplo.test/a b(c)>)"
        );
        let blank_title = Content {
            title: Some("   "),
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
