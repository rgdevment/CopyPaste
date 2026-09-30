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
#[path = "query_test.rs"]
mod tests;
