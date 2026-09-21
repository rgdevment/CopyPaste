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
        .expect("un origen apunta a un carácter")
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
mod tests {
    use super::{EXCERPT_CHARS, Segment, excerpt, fold, fold_mapped, terms_of};

    fn plain(text: &str) -> Segment {
        Segment {
            text: text.into(),
            matched: false,
        }
    }

    fn hit(text: &str) -> Segment {
        Segment {
            text: text.into(),
            matched: true,
        }
    }

    fn terms(words: &[&str]) -> Vec<String> {
        words.iter().map(|word| (*word).to_owned()).collect()
    }

    #[test]
    fn each_folded_char_remembers_where_it_came_from() {
        let folded = fold_mapped("Straße");
        assert_eq!(folded.chars.iter().collect::<String>(), "strasse");
        assert_eq!(
            folded.origin,
            vec![0, 1, 2, 3, 4, 4, 6],
            "la ß se abre en dos y ocupa dos bytes: el origen es el byte"
        );
        let folded = fold_mapped("café");
        assert_eq!(folded.origin, vec![0, 1, 2, 3]);
        let folded = fold_mapped("ñu");
        assert_eq!(folded.origin, vec![0, 2]);
    }

    #[test]
    fn the_excerpt_shows_the_text_as_copied_with_the_match_marked() {
        let found = excerpt("el café de la esquina", &terms(&["cafe"]), EXCERPT_CHARS)
            .expect("hay coincidencia");
        assert_eq!(
            found.segments,
            vec![plain("el "), hit("café"), plain(" de la esquina")],
            "se busca sin tilde y se enseña con ella"
        );
    }

    #[test]
    fn a_prefix_marks_only_what_was_typed() {
        let found = excerpt("la esquina", &terms(&["esq"]), 100).expect("hay");
        assert_eq!(
            found.segments,
            vec![plain("la "), hit("esq"), plain("uina")]
        );
    }

    #[test]
    fn a_word_that_only_appears_inside_another_is_not_a_match() {
        assert!(
            excerpt("encafetado", &terms(&["cafe"]), 100).is_none(),
            "el índice tampoco lo encontraría: los términos van por prefijo de palabra"
        );
    }

    #[test]
    fn a_term_with_punctuation_matches_the_same_phrase_the_index_does() {
        let found =
            excerpt("Pedido AB-4417 entrega 12 marzo", &terms(&["ab-4417"]), 100).expect("hay");
        assert_eq!(
            found.segments,
            vec![plain("Pedido "), hit("AB-4417"), plain(" entrega 12 marzo")]
        );
    }

    #[test]
    fn an_expanded_letter_is_marked_whole() {
        let found = excerpt("Straße Hauptbahnhof", &terms(&["strasse"]), 100).expect("hay");
        assert_eq!(found.segments, vec![hit("Straße"), plain(" Hauptbahnhof")]);
    }

    #[test]
    fn overlapping_terms_become_one_mark() {
        let found = excerpt("un café", &terms(&["caf", "cafe"]), 100).expect("hay");
        assert_eq!(found.segments, vec![plain("un "), hit("café")]);
    }

    #[test]
    fn every_term_that_matches_is_marked() {
        let found = excerpt("rojo verde azul", &terms(&["rojo", "azul"]), 100).expect("hay");
        assert_eq!(
            found.segments,
            vec![hit("rojo"), plain(" verde "), hit("azul")]
        );
    }

    #[test]
    fn a_long_text_is_cut_around_the_first_match() {
        let text = format!("{}aguja{}", "paja ".repeat(100), " heno".repeat(100));
        let found = excerpt(&text, &terms(&["aguja"]), 60).expect("hay");
        let shown = found.plain();
        assert!(shown.starts_with('…') && shown.ends_with('…'), "{shown}");
        assert_eq!(
            shown.chars().count(),
            62,
            "sesenta más los dos puntos suspensivos"
        );
        assert!(
            found
                .segments
                .iter()
                .any(|one| one.matched && one.text == "aguja")
        );
        let before = shown.find("aguja").expect("está");
        assert!(
            before < shown.len() / 2,
            "la coincidencia queda en el primer tercio, no al final de la ventana"
        );
    }

    #[test]
    fn exactly_a_third_of_the_window_comes_before_the_match() {
        let found = excerpt("abcdefghij aguja", &terms(&["aguja"]), 9).expect("hay");
        assert_eq!(
            found.plain(),
            "…ij aguja",
            "tres caracteres delante, ni uno más"
        );
        let found = excerpt("ñññññ aguja", &terms(&["aguja"]), 9).expect("hay");
        assert_eq!(
            found.plain(),
            "…ññ aguja",
            "y el corte cae en un límite de carácter aunque el anterior ocupe dos bytes"
        );
    }

    #[test]
    fn a_combining_mark_travels_with_the_word_it_marks() {
        let found = excerpt("cafe\u{301} rico", &terms(&["cafe"]), 100).expect("hay");
        assert_eq!(found.segments, vec![hit("cafe\u{301}"), plain(" rico")]);
    }

    #[test]
    fn the_window_never_cuts_a_mark_or_a_skin_tone_from_its_base() {
        let text = format!("{} aguja x", "e\u{301}".repeat(30));
        let found = excerpt(&text, &terms(&["aguja"]), 9).expect("hay");
        let shown = found.plain();
        assert!(!shown.starts_with("…\u{301}"), "{shown:?}");
        let text = format!("{}aguja", "👍🏽".repeat(30));
        let found = excerpt(&text, &terms(&["aguja"]), 9).expect("hay");
        let shown = found.plain();
        assert!(!shown.starts_with("…🏽"), "{shown:?}");
    }

    #[test]
    fn a_match_at_the_very_end_only_needs_the_leading_ellipsis() {
        let text = format!("{}fin", "x ".repeat(200));
        let found = excerpt(&text, &terms(&["fin"]), 40).expect("hay");
        let shown = found.plain();
        assert!(shown.starts_with('…'));
        assert!(shown.ends_with("fin"), "{shown}");
    }

    #[test]
    fn a_match_at_the_start_only_needs_the_trailing_ellipsis() {
        let text = format!("inicio{}", " x".repeat(200));
        let found = excerpt(&text, &terms(&["inicio"]), 40).expect("hay");
        let shown = found.plain();
        assert!(shown.starts_with("inicio"), "{shown}");
        assert!(shown.ends_with('…'));
    }

    #[test]
    fn a_short_text_comes_back_whole() {
        let found = excerpt("nota corta", &terms(&["nota"]), EXCERPT_CHARS).expect("hay");
        assert_eq!(found.plain(), "nota corta");
    }

    #[test]
    fn a_second_match_past_the_window_does_not_stretch_it() {
        let text = format!("aguja {}aguja", "paja ".repeat(100));
        let found = excerpt(&text, &terms(&["aguja"]), 30).expect("hay");
        assert_eq!(found.segments.iter().filter(|one| one.matched).count(), 1);
        assert!(found.plain().chars().count() <= 31);
    }

    #[test]
    fn nothing_to_look_for_is_nothing_found() {
        assert!(excerpt("algo", &[], 100).is_none());
        assert!(excerpt("", &terms(&["algo"]), 100).is_none());
        assert!(excerpt("otra cosa", &terms(&["algo"]), 100).is_none());
        assert!(excerpt("otra cosa", &terms(&["!!!"]), 100).is_none());
    }

    #[test]
    fn the_terms_are_the_words_the_index_would_look_for() {
        assert_eq!(terms_of("Café  AB-4417 ... !!"), vec!["cafe", "ab-4417"]);
        assert!(terms_of("   ").is_empty());
        assert!(terms_of("!!! ...").is_empty());
    }

    #[test]
    fn every_ligature_in_the_table_is_expanded() {
        for (given, expected) in [
            ("ß", "ss"),
            ("æ", "ae"),
            ("Æ", "ae"),
            ("œ", "oe"),
            ("Œ", "oe"),
            ("ø", "o"),
            ("Ø", "o"),
            ("ł", "l"),
            ("Ł", "l"),
            ("đ", "d"),
            ("Đ", "d"),
            ("þ", "th"),
            ("Þ", "th"),
        ] {
            assert_eq!(fold(given), expected, "«{given}» no se expandió bien");
        }
    }

    #[test]
    fn combining_marks_disappear_whatever_the_script() {
        assert_eq!(fold("ñ"), "n");
        assert_eq!(fold("ç"), "c");
        assert_eq!(fold("ü"), "u");
        assert_eq!(fold("å"), "a");
        assert_eq!(fold("ĉ"), "c");
        assert_eq!(fold("ṩ"), "s");
    }

    #[test]
    fn what_has_nothing_to_fold_comes_out_as_it_went_in() {
        assert_eq!(fold(""), "");
        assert_eq!(fold("plain ascii"), "plain ascii");
        assert_eq!(fold("日本語"), "日本語");
        assert_eq!(fold("123 !?"), "123 !?");
    }

    #[test]
    fn folding_twice_changes_nothing() {
        for text in ["Straße", "encyclopædia", "Łódź", "café", "ÞÓRR"] {
            let once = fold(text);
            assert_eq!(fold(&once), once, "«{text}» no era estable");
        }
    }

    #[test]
    fn folds_both_sides_of_the_index() {
        assert_eq!(fold("Straße"), "strasse");
        assert_eq!(fold("encyclopædia"), "encyclopaedia");
        assert_eq!(fold("Łódź"), "lodz");
        assert_eq!(fold("el café"), "el cafe");
    }
}
