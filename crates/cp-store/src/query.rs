use crate::store::{Broken, Filter, Order};
use cp_core::kind::Kind;

pub const SECOND: i64 = 1_000;
pub const MINUTE: i64 = 60 * SECOND;
pub const HOUR: i64 = 60 * MINUTE;
pub const DAY: i64 = 24 * HOUR;
pub const WEEK: i64 = 7 * DAY;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clock {
    pub now: i64,
    pub day_start: i64,
}

pub const COLORS: [(&str, i64); 7] = [
    ("none", 0),
    ("red", 1),
    ("green", 2),
    ("purple", 3),
    ("yellow", 4),
    ("blue", 5),
    ("orange", 6),
];

pub fn parse(input: &str, clock: &Clock) -> Filter {
    let mut filter = Filter::default();
    let mut words: Vec<String> = Vec::new();
    let mut label: Vec<String> = Vec::new();
    for token in tokens(input) {
        match operator(&token, clock) {
            Some(Op::Kinds(kinds, false)) => filter.kinds.extend(kinds),
            Some(Op::Kinds(kinds, true)) => filter.exclude_kinds.extend(kinds),
            Some(Op::Apps(apps, false)) => filter.apps.extend(apps),
            Some(Op::Apps(apps, true)) => filter.exclude_apps.extend(apps),
            Some(Op::Colors(colors)) => filter.colors.extend(colors),
            Some(Op::Since(at)) => filter.since = Some(filter.since.map_or(at, |had| had.max(at))),
            Some(Op::Pinned) => filter.pinned_only = true,
            Some(Op::OnlyBroken) => filter.broken = Broken::Only,
            Some(Op::Label(text)) => label.push(text),
            Some(Op::Order(order)) => filter.order = order,
            None if token.chars().any(char::is_alphanumeric) => words.push(token),
            None => {}
        }
    }
    filter.query = (!words.is_empty()).then(|| words.join(" "));
    filter.label_query = (!label.is_empty()).then(|| label.join(" "));
    filter
}

enum Op {
    Kinds(Vec<Kind>, bool),
    Apps(Vec<String>, bool),
    Colors(Vec<i64>),
    Since(i64),
    Pinned,
    OnlyBroken,
    Label(String),
    Order(Order),
}

fn operator(token: &str, clock: &Clock) -> Option<Op> {
    let (negated, body) = match token.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, token),
    };
    let (key, value, symbol) = split(body)?;
    if value.is_empty() {
        return None;
    }
    let key = key.to_ascii_lowercase();
    let key = key.as_str();
    let values: Vec<&str> = value.split(',').filter(|one| !one.is_empty()).collect();
    if values.is_empty() {
        return None;
    }
    match key {
        "k" | "kind" | "type" | "t" => values
            .iter()
            .map(|one| Kind::from_name(&one.to_ascii_lowercase()))
            .collect::<Option<Vec<Kind>>>()
            .map(|kinds| Op::Kinds(kinds, negated)),
        "a" | "app" => Some(Op::Apps(
            values.iter().map(|one| (*one).to_owned()).collect(),
            negated,
        )),
        "c" | "color" if !negated => values
            .iter()
            .map(|one| color(one, !symbol))
            .collect::<Option<Vec<i64>>>()
            .map(Op::Colors),
        "d" | "date" | "since" if !negated => since(value, clock).map(Op::Since),
        "is" if !negated => match value.to_ascii_lowercase().as_str() {
            "pinned" => Some(Op::Pinned),
            "broken" => Some(Op::OnlyBroken),
            _ => None,
        },
        "l" | "label" if !negated => Some(Op::Label(value.to_owned())),
        "sort" | "order" if !negated => match value.to_ascii_lowercase().as_str() {
            "recent" => Some(Op::Order(Order::Recent)),
            "pasted" => Some(Op::Order(Order::MostPasted)),
            "used" => Some(Op::Order(Order::LastUsed)),
            _ => None,
        },
        _ => None,
    }
}

fn split(body: &str) -> Option<(&str, &str, bool)> {
    let symbol = match body.chars().next()? {
        '/' => Some("k"),
        '@' => Some("a"),
        '#' => Some("c"),
        '~' => Some("d"),
        _ => None,
    };
    if let Some(key) = symbol {
        return Some((key, &body[1..], true));
    }
    let (key, value) = body.split_once(':')?;
    let known = key.chars().all(|c| c.is_ascii_alphabetic());
    known.then_some((key, value, false))
}

fn color(name: &str, by_index: bool) -> Option<i64> {
    let lower = name.to_ascii_lowercase();
    if let Ok(index) = lower.parse::<i64>() {
        return (by_index && COLORS.iter().any(|(_, value)| *value == index)).then_some(index);
    }
    COLORS
        .iter()
        .find(|(known, _)| *known == lower)
        .map(|(_, value)| *value)
}

fn since(value: &str, clock: &Clock) -> Option<i64> {
    let lower = value.to_ascii_lowercase();
    if lower == "today" {
        return Some(clock.day_start);
    }
    let unit = match lower.chars().last()? {
        'm' => MINUTE,
        'h' => HOUR,
        'd' => DAY,
        'w' => WEEK,
        _ => return None,
    };
    let amount: i64 = lower[..lower.len() - 1].parse().ok()?;
    (amount > 0).then(|| clock.now.saturating_sub(amount.saturating_mul(unit)))
}

fn tokens(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for c in input.chars() {
        match c {
            '"' => quoted = !quoted,
            c if c.is_whitespace() && !quoted => {
                if !current.is_empty() {
                    out.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLOCK: Clock = Clock {
        now: 1_000 * DAY + 5 * HOUR,
        day_start: 1_000 * DAY,
    };

    fn parsed(input: &str) -> Filter {
        parse(input, &CLOCK)
    }

    #[test]
    fn plain_words_are_the_search() {
        let filter = parsed("el café de la esquina");
        assert_eq!(filter.query.as_deref(), Some("el café de la esquina"));
        assert_eq!(
            filter,
            Filter {
                query: Some("el café de la esquina".into()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn nothing_typed_is_the_whole_history() {
        assert_eq!(parsed(""), Filter::default());
        assert_eq!(parsed("   "), Filter::default());
    }

    #[test]
    fn a_class_by_key_or_by_symbol() {
        assert_eq!(parsed("k:json").kinds, vec![Kind::Json]);
        assert_eq!(parsed("kind:Link").kinds, vec![Kind::Link]);
        assert_eq!(parsed("/image").kinds, vec![Kind::Image]);
        assert_eq!(parsed("t:code,json").kinds, vec![Kind::Code, Kind::Json]);
    }

    #[test]
    fn a_class_can_be_left_out() {
        let filter = parsed("-k:image -/video");
        assert!(filter.kinds.is_empty());
        assert_eq!(filter.exclude_kinds, vec![Kind::Image, Kind::Video]);
        assert_eq!(filter.query, None);
    }

    #[test]
    fn a_prefix_is_only_an_operator_when_its_value_is_known() {
        assert_eq!(parsed("/nada").query.as_deref(), Some("/nada"));
        assert_eq!(parsed("k:foto").query.as_deref(), Some("k:foto"));
        assert_eq!(
            parsed("k:image,foto").query.as_deref(),
            Some("k:image,foto"),
            "una lista con un desconocido no es media lista"
        );
        assert_eq!(parsed("#FF8800").query.as_deref(), Some("#FF8800"));
        assert_eq!(parsed("c:9").query.as_deref(), Some("c:9"));
        assert_eq!(parsed("d:soon").query.as_deref(), Some("d:soon"));
        assert_eq!(parsed("is:new").query.as_deref(), Some("is:new"));
        assert_eq!(parsed("sort:size").query.as_deref(), Some("sort:size"));
    }

    #[test]
    fn an_application_by_key_by_symbol_and_with_spaces() {
        assert_eq!(parsed("@slack").apps, vec!["slack"]);
        assert_eq!(parsed("a:Code").apps, vec!["Code"]);
        assert_eq!(parsed("app:\"Google Chrome\"").apps, vec!["Google Chrome"]);
        assert_eq!(parsed("-@code").exclude_apps, vec!["code"]);
        assert_eq!(parsed("a:slack,code").apps, vec!["slack", "code"]);
    }

    #[test]
    fn an_email_is_not_an_application() {
        let filter = parsed("escribe a juan@ejemplo.test");
        assert!(filter.apps.is_empty());
        assert_eq!(filter.query.as_deref(), Some("escribe a juan@ejemplo.test"));
    }

    #[test]
    fn a_colour_by_name_or_by_number() {
        assert_eq!(parsed("c:red").colors, vec![1]);
        assert_eq!(parsed("#Blue").colors, vec![5]);
        assert_eq!(parsed("color:3").colors, vec![3]);
        assert_eq!(parsed("c:red,orange").colors, vec![1, 6]);
        assert_eq!(parsed("c:none").colors, vec![0]);
    }

    #[test]
    fn keys_states_classes_colours_and_dates_ignore_case() {
        assert_eq!(parsed("K:image").kinds, vec![Kind::Image]);
        assert_eq!(parsed("Kind:LINK").kinds, vec![Kind::Link]);
        assert!(parsed("IS:pinned").pinned_only);
        assert!(parsed("is:Pinned").pinned_only);
        assert_eq!(parsed("SORT:Used").order, Order::LastUsed);
        assert_eq!(parsed("D:TODAY").since, Some(CLOCK.day_start));
        assert_eq!(parsed("#RED").colors, vec![1]);
        assert_eq!(
            parsed("A:Slack").apps,
            vec!["Slack"],
            "el nombre de la app se conserva"
        );
    }

    #[test]
    fn a_repeated_order_keeps_the_last_and_a_repeated_class_repeats() {
        assert_eq!(parsed("sort:pasted sort:used").order, Order::LastUsed);
        assert_eq!(parsed("k:json k:json").kinds, vec![Kind::Json, Kind::Json]);
        assert_eq!(
            parsed("k:ímage").query.as_deref(),
            Some("k:ímage"),
            "con acento no es una clase"
        );
    }

    #[test]
    fn a_hash_followed_by_a_number_is_text_not_a_colour() {
        let filter = parsed("PR #3");
        assert_eq!(filter.query.as_deref(), Some("PR #3"));
        assert!(
            filter.colors.is_empty(),
            "«#3» es un número de PR, no el morado"
        );
        assert_eq!(parsed("#3,red").query.as_deref(), Some("#3,red"));
        assert_eq!(
            parsed("c:3").colors,
            vec![3],
            "por clave, el índice sí vale"
        );
    }

    #[test]
    fn a_relative_date_counts_back_from_now() {
        assert_eq!(parsed("d:today").since, Some(CLOCK.day_start));
        assert_eq!(parsed("~1h").since, Some(CLOCK.now - HOUR));
        assert_eq!(parsed("since:7d").since, Some(CLOCK.now - WEEK));
        assert_eq!(parsed("date:30m").since, Some(CLOCK.now - 30 * MINUTE));
        assert_eq!(parsed("~2w").since, Some(CLOCK.now - 2 * WEEK));
    }

    #[test]
    fn two_dates_keep_the_narrower_one() {
        assert_eq!(parsed("~7d ~1h").since, Some(CLOCK.now - HOUR));
        assert_eq!(parsed("~1h ~7d").since, Some(CLOCK.now - HOUR));
    }

    #[test]
    fn a_date_that_is_not_a_date_stays_text() {
        assert_eq!(parsed("~0h").query.as_deref(), Some("~0h"));
        assert_eq!(parsed("~h").query.as_deref(), Some("~h"));
        assert_eq!(parsed("~1y").query.as_deref(), Some("~1y"));
        assert_eq!(parsed("~ayer").query.as_deref(), Some("~ayer"));
    }

    #[test]
    fn an_absurd_amount_does_not_overflow() {
        assert!(parsed("~99999999999999999d").since.is_some());
        let dawn = Clock {
            now: i64::MIN + 1,
            day_start: i64::MIN,
        };
        assert_eq!(parse("~1h", &dawn).since, Some(i64::MIN));
    }

    #[test]
    fn pinned_and_broken_are_states_not_words() {
        assert!(parsed("is:pinned").pinned_only);
        assert_eq!(parsed("is:broken").broken, Broken::Only);
        assert_eq!(parsed("is:pinned").query, None);
    }

    #[test]
    fn the_label_is_searched_on_its_own_column() {
        let filter = parsed("label:factura l:mayo");
        assert_eq!(filter.label_query.as_deref(), Some("factura mayo"));
        assert_eq!(filter.query, None);
    }

    #[test]
    fn the_order_is_a_word_too() {
        assert_eq!(parsed("sort:pasted").order, Order::MostPasted);
        assert_eq!(parsed("order:used").order, Order::LastUsed);
        let explicit = parsed("sort:recent");
        assert_eq!(explicit.order, Order::Recent);
        assert_eq!(explicit.query, None, "es un operador, no texto");
    }

    #[test]
    fn a_negated_state_or_date_is_not_an_operator() {
        assert_eq!(parsed("-is:pinned").query.as_deref(), Some("-is:pinned"));
        assert_eq!(parsed("-~1h").query.as_deref(), Some("-~1h"));
        assert_eq!(parsed("-c:red").query.as_deref(), Some("-c:red"));
        assert_eq!(parsed("-l:factura").query.as_deref(), Some("-l:factura"));
        assert_eq!(parsed("-sort:used").query.as_deref(), Some("-sort:used"));
        assert_eq!(parsed("-l:factura").label_query, None);
        assert_eq!(parsed("-sort:used").order, Order::Recent);
    }

    #[test]
    fn the_units_are_milliseconds() {
        assert_eq!(SECOND, 1_000);
        assert_eq!(MINUTE, 60_000);
        assert_eq!(HOUR, 3_600_000);
        assert_eq!(DAY, 86_400_000);
        assert_eq!(WEEK, 604_800_000);
    }

    #[test]
    fn an_empty_value_is_text() {
        assert_eq!(parsed("k:").query.as_deref(), Some("k:"));
        assert_eq!(parsed("a:,").query.as_deref(), Some("a:,"));
    }

    #[test]
    fn a_word_with_nothing_to_search_for_does_not_blank_the_list() {
        for typed in ["@", "#", "~", "/", "!!!", "...", "- -"] {
            assert_eq!(
                parsed(typed),
                Filter::default(),
                "«{typed}» es el historial entero"
            );
        }
        assert_eq!(parsed("hola ...").query.as_deref(), Some("hola"));
    }

    #[test]
    fn a_colon_inside_ordinary_text_is_not_a_key() {
        assert_eq!(
            parsed("https://ejemplo.test").query.as_deref(),
            Some("https://ejemplo.test"),
            "https no es una clave"
        );
        assert_eq!(parsed("12:30").query.as_deref(), Some("12:30"));
        assert_eq!(parsed("a-b:c").query.as_deref(), Some("a-b:c"));
    }

    #[test]
    fn the_example_from_the_design_note() {
        let filter = parsed("token @code ~1h");
        assert_eq!(
            filter,
            Filter {
                query: Some("token".into()),
                apps: vec!["code".into()],
                since: Some(CLOCK.now - HOUR),
                ..Default::default()
            }
        );
    }

    #[test]
    fn everything_at_once() {
        let filter = parsed("  pedido  k:json,text -@safari #red ~7d is:pinned l:mayo sort:used ");
        assert_eq!(filter.query.as_deref(), Some("pedido"));
        assert_eq!(filter.kinds, vec![Kind::Json, Kind::Text]);
        assert_eq!(filter.exclude_apps, vec!["safari"]);
        assert_eq!(filter.colors, vec![1]);
        assert_eq!(filter.since, Some(CLOCK.now - WEEK));
        assert!(filter.pinned_only);
        assert_eq!(filter.label_query.as_deref(), Some("mayo"));
        assert_eq!(filter.order, Order::LastUsed);
    }

    #[test]
    fn quotes_group_words_and_then_disappear() {
        assert_eq!(
            parsed("\"dos palabras\"").query.as_deref(),
            Some("dos palabras")
        );
        assert_eq!(
            parsed("sin cerrar \"la cita").query.as_deref(),
            Some("sin cerrar la cita")
        );
    }
}
