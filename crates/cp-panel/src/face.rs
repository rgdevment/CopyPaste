use cp_core::kind::Kind;

pub const TOP: f32 = 9.0;
pub const META: f32 = 22.0;
pub const GAP: f32 = 6.0;
pub const BOTTOM: f32 = 10.0;
pub const BETWEEN: f32 = 6.0;
pub const LINE: f32 = 19.0;
pub const MONO_LINE: f32 = 17.0;
pub const KEYS_LINE: f32 = 16.0;
pub const BLOCK_PAD: f32 = 12.0;
pub const THUMB: f32 = 50.0;
pub const PAINT: f32 = 24.0;
pub const TILE: f32 = 34.0;

pub const OPEN_THUMB: f32 = 146.0;
pub const NOTE: f32 = 15.0;
pub const KEYS_ROW: f32 = 22.0;
pub const OPEN_PAINT: f32 = 48.0;

const OPEN_WORDS: i32 = 12;
const OPEN_CODE: i32 = 12;
const OPEN_JSON: i32 = 6;
const OPEN_LINK: i32 = 3;
const OPEN_PER_LINE: usize = 58;
const MONO_PER_LINE: usize = 54;
const WORDS_PER_LINE: usize = 52;
const KEYS_PER_LINE: usize = 56;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Found,
    FoundThumb,
    Thumb,
    Paint,
    Block(i32),
    Keys(i32),
    Link,
    File,
    Media,
    Words(i32),
}

pub struct Seen<'a> {
    pub kind: Option<Kind>,
    pub found: bool,
    pub thumb: bool,
    pub paints: bool,
    pub body: &'a str,
    pub keys: &'a str,
}

impl Face {
    pub fn of(seen: &Seen<'_>) -> Self {
        if seen.found {
            return if seen.thumb {
                Self::FoundThumb
            } else {
                Self::Found
            };
        }
        if seen.thumb {
            return Self::Thumb;
        }
        if seen.paints {
            return Self::Paint;
        }
        match seen.kind {
            Some(Kind::Code | Kind::Token) => {
                let (_, second) = opening_of(seen.body);
                Self::Block(if second.is_empty() { 1 } else { 2 })
            }
            Some(Kind::Json) => Self::Keys(lines_for(seen.keys, KEYS_PER_LINE)),
            Some(Kind::Link) => Self::Link,
            Some(Kind::File | Kind::Folder) => Self::File,
            Some(Kind::Audio | Kind::Video) => Self::Media,
            _ => Self::Words(lines_for(
                &crate::view::squeezed_of(seen.body),
                WORDS_PER_LINE,
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Found => "found",
            Self::FoundThumb => "found-thumb",
            Self::Thumb => "thumb",
            Self::Paint => "paint",
            Self::Block(_) => "block",
            Self::Keys(_) => "keys",
            Self::Link => "link",
            Self::File => "file",
            Self::Media => "media",
            Self::Words(_) => "words",
        }
    }

    pub fn lines(self) -> i32 {
        match self {
            Self::Block(lines) | Self::Keys(lines) | Self::Words(lines) => lines,
            _ => 1,
        }
    }

    pub fn body_px(self) -> f32 {
        match self {
            Self::Found | Self::Link => LINE,
            Self::Thumb | Self::FoundThumb => THUMB,
            Self::Paint => PAINT,
            Self::Block(lines) => BLOCK_PAD + MONO_LINE * lines as f32,
            Self::Keys(lines) => KEYS_LINE * lines as f32,
            Self::File | Self::Media => TILE,
            Self::Words(lines) => LINE * lines as f32,
        }
    }

    pub fn shut_px(self) -> f32 {
        match self {
            Self::Paint => TOP + META + BOTTOM + BETWEEN,
            _ => TOP + META + GAP + self.body_px() + BOTTOM + BETWEEN,
        }
    }
}

pub struct Opened {
    pub text: String,
    pub lines: i32,
    pub more: String,
}

pub fn opened_of(face: Face, json: bool, body: &str, english: bool) -> Opened {
    match face {
        Face::Words(_) | Face::Found => {
            let wrapped = wrapped_lines(body, OPEN_PER_LINE);
            Opened {
                text: body.to_owned(),
                lines: wrapped.clamp(1, OPEN_WORDS),
                more: shown_of(body.lines().count(), wrapped, OPEN_WORDS, english),
            }
        }
        Face::Keys(_) if json => mono_opened(&pretty(body), OPEN_JSON, english),
        Face::Block(_) | Face::Keys(_) => mono_opened(body, OPEN_CODE, english),
        Face::Link => Opened {
            text: body.trim().to_owned(),
            lines: wrapped_lines(body.trim(), MONO_PER_LINE).clamp(1, OPEN_LINK),
            more: String::new(),
        },
        _ => Opened {
            text: body.to_owned(),
            lines: 0,
            more: String::new(),
        },
    }
}

fn mono_opened(text: &str, cap: i32, english: bool) -> Opened {
    let total = text.lines().count().max(1);
    let lines = i32::try_from(total).unwrap_or(i32::MAX).clamp(1, cap);
    let rest = total - usize::try_from(lines).unwrap_or(0);
    let more = match (rest, english) {
        (0, _) => String::new(),
        (1, true) => "… 1 more line".to_owned(),
        (1, false) => "… 1 línea más".to_owned(),
        (rest, true) => format!("… {rest} more lines"),
        (rest, false) => format!("… {rest} líneas más"),
    };
    Opened {
        text: text
            .lines()
            .take(usize::try_from(lines).unwrap_or(0))
            .collect::<Vec<_>>()
            .join("\n"),
        lines,
        more,
    }
}

fn shown_of(total: usize, wrapped: i32, cap: i32, english: bool) -> String {
    if wrapped <= cap {
        return String::new();
    }
    if english {
        format!("{total} lines · showing the first")
    } else {
        format!("{total} líneas · mostrando el comienzo")
    }
}

fn pretty(body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|value| serde_json::to_string_pretty(&value).ok())
        .unwrap_or_else(|| body.to_owned())
}

fn wrapped_lines(text: &str, per_line: usize) -> i32 {
    let total: usize = text
        .lines()
        .map(|line| line.chars().count().div_ceil(per_line).max(1))
        .sum();
    i32::try_from(total.max(1)).unwrap_or(i32::MAX)
}

pub fn open_body_px(face: Face, opened: &Opened, claims: i32, room: f32) -> f32 {
    let note = if opened.more.is_empty() {
        0.0
    } else {
        GAP + NOTE
    };
    match face {
        Face::Thumb | Face::FoundThumb => OPEN_THUMB + GAP + NOTE,
        Face::Words(_) | Face::Found => LINE * opened.lines as f32 + note,
        Face::Block(_) | Face::Keys(_) | Face::Link if claims > 0 => {
            BLOCK_PAD + MONO_LINE * claims as f32
        }
        Face::Block(_) | Face::Keys(_) | Face::Link => {
            BLOCK_PAD + MONO_LINE * opened.lines as f32 + note
        }
        Face::Paint => OPEN_PAINT,
        Face::File | Face::Media => room,
    }
}

pub fn open_px(body: f32) -> f32 {
    TOP + META + GAP + body + GAP + KEYS_ROW + BOTTOM + BETWEEN
}

pub fn open_lost_px(body: f32) -> f32 {
    open_px(body) - GAP - KEYS_ROW
}

fn lines_for(text: &str, per_line: usize) -> i32 {
    if text.chars().count() <= per_line {
        1
    } else {
        2
    }
}

pub fn opening_of(body: &str) -> (String, String) {
    let shown: Vec<&str> = body
        .lines()
        .filter(|line| !line.trim().is_empty())
        .take(2)
        .collect();
    let indent = shown
        .iter()
        .map(|line| line.chars().take_while(|one| one.is_whitespace()).count())
        .min()
        .unwrap_or(0);
    let mut lines = shown.iter().map(|line| {
        let cut = line
            .char_indices()
            .nth(indent)
            .map_or(line.len(), |(at, _)| at);
        line[cut.min(line.len())..].trim_end().to_owned()
    });
    let first = lines.next().unwrap_or_default();
    let second = lines.next().unwrap_or_default();
    (first, second)
}

pub fn lines_said(total: usize, shown: i32, english: bool) -> String {
    let shown = usize::try_from(shown).unwrap_or(0);
    if total <= shown.max(1) {
        return String::new();
    }
    if english {
        format!("{total} lines")
    } else {
        format!("{total} líneas")
    }
}

pub fn aside_said(parts: &[&str]) -> String {
    parts
        .iter()
        .map(|one| one.trim())
        .filter(|one| !one.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

#[cfg(test)]
#[path = "face_test.rs"]
mod tests;
