use unicode_normalization::UnicodeNormalization;

pub fn fold(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    fold_each(input, |c, _| out.push(c));
    out
}

pub(crate) struct Folded {
    pub chars: Vec<char>,
    pub origin: Vec<usize>,
}

pub(crate) fn fold_mapped(input: &str) -> Folded {
    let mut chars = Vec::with_capacity(input.len());
    let mut origin = Vec::with_capacity(input.len());
    fold_each(input, |c, at| {
        chars.push(c);
        origin.push(at);
    });
    Folded { chars, origin }
}

fn fold_each(input: &str, mut push: impl FnMut(char, usize)) {
    for (at, c) in input.char_indices() {
        if c.is_ascii() {
            push(c.to_ascii_lowercase(), at);
            continue;
        }
        for decomposed in std::iter::once(c).nfkd() {
            if is_combining_mark(decomposed) {
                continue;
            }
            match expand_ligature(decomposed) {
                Some(expanded) => expanded.chars().for_each(|c| push(c, at)),
                None => decomposed.to_lowercase().for_each(|c| push(c, at)),
            }
        }
    }
}

pub fn terms_of(query: &str) -> Vec<String> {
    fold(query)
        .split_whitespace()
        .filter(|word| word.chars().any(char::is_alphanumeric))
        .map(str::to_owned)
        .collect()
}

pub const EXCERPT_CHARS: usize = 160;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub text: String,
    pub matched: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Excerpt {
    pub segments: Vec<Segment>,
}

impl Excerpt {
    pub fn plain(&self) -> String {
        self.segments.iter().map(|one| one.text.as_str()).collect()
    }
}

pub fn excerpt(text: &str, terms: &[String], width: usize) -> Option<Excerpt> {
    let folded = fold_mapped(text);
    let tokens = tokens_of(&folded.chars);
    let mut hits: Vec<(usize, usize)> = terms
        .iter()
        .flat_map(|term| matches_of(&folded.chars, &tokens, term))
        .map(|(from, to)| {
            let last = folded.origin[to - 1];
            (
                folded.origin[from],
                after_marks(text, last + char_at(text, last).len_utf8()),
            )
        })
        .collect();
    hits.sort_unstable();
    let hits = merged(hits);
    let first = *hits.first()?;

    let start = text[..first.0]
        .char_indices()
        .rev()
        .nth(width / 3)
        .map_or(0, |(at, c)| after_marks(text, at + c.len_utf8()));
    let end = text[start..]
        .char_indices()
        .nth(width.max(1))
        .map_or(text.len(), |(at, _)| after_marks(text, start + at));

    let mut segments = Vec::new();
    if start > 0 {
        segments.push(ellipsis());
    }
    let mut cursor = start;
    for (from, to) in hits {
        if from >= end {
            break;
        }
        let (from, to) = (from.max(start), to.min(end));
        segments.extend(segment(text, cursor, from, false));
        segments.extend(segment(text, from, to, true));
        cursor = to;
    }
    segments.extend(segment(text, cursor, end, false));
    if end < text.len() {
        segments.push(ellipsis());
    }
    Some(Excerpt { segments })
}

fn after_marks(text: &str, mut at: usize) -> usize {
    for c in text[at..].chars() {
        if is_combining_mark(c) || clings_to_previous(c) {
            at += c.len_utf8();
        } else {
            break;
        }
    }
    at
}

fn clings_to_previous(c: char) -> bool {
    matches!(c as u32, 0xFE0E | 0xFE0F | 0x200D | 0x1F3FB..=0x1F3FF)
}

fn char_at(text: &str, at: usize) -> char {
    text[at..]
        .chars()
        .next()
        .expect("an origin points to a character")
}

fn segment(text: &str, from: usize, to: usize, matched: bool) -> Option<Segment> {
    (from < to).then(|| Segment {
        text: text[from..to].into(),
        matched,
    })
}

fn ellipsis() -> Segment {
    Segment {
        text: "…".into(),
        matched: false,
    }
}

fn tokens_of(chars: &[char]) -> Vec<(usize, usize)> {
    let mut tokens = Vec::new();
    let mut open: Option<usize> = None;
    for (at, c) in chars.iter().enumerate() {
        match (c.is_alphanumeric(), open) {
            (true, None) => open = Some(at),
            (false, Some(from)) => {
                tokens.push((from, at));
                open = None;
            }
            _ => {}
        }
    }
    if let Some(from) = open {
        tokens.push((from, chars.len()));
    }
    tokens
}

fn matches_of(chars: &[char], tokens: &[(usize, usize)], term: &str) -> Vec<(usize, usize)> {
    let wanted: Vec<Vec<char>> = term
        .split(|c: char| !c.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| part.chars().collect())
        .collect();
    let Some((last, head)) = wanted.split_last() else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for window in tokens.windows(wanted.len()) {
        let whole = head
            .iter()
            .zip(window)
            .all(|(part, (from, to))| chars[*from..*to] == part[..]);
        let (from, to) = window[wanted.len() - 1];
        if whole && chars[from..to].starts_with(last) {
            found.push((window[0].0, from + last.len()));
        }
    }
    found
}

fn merged(sorted: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    let mut out: Vec<(usize, usize)> = Vec::new();
    for (from, to) in sorted {
        match out.last_mut() {
            Some(last) if from <= last.1 => last.1 = last.1.max(to),
            _ => out.push((from, to)),
        }
    }
    out
}

fn is_combining_mark(c: char) -> bool {
    matches!(c as u32, 0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x20D0..=0x20FF)
}

fn expand_ligature(c: char) -> Option<&'static str> {
    match c {
        'ß' => Some("ss"),
        'æ' | 'Æ' => Some("ae"),
        'œ' | 'Œ' => Some("oe"),
        'ø' | 'Ø' => Some("o"),
        'ł' | 'Ł' => Some("l"),
        'đ' | 'Đ' => Some("d"),
        'þ' | 'Þ' => Some("th"),
        _ => None,
    }
}

#[cfg(test)]
#[path = "search_test.rs"]
mod tests;
