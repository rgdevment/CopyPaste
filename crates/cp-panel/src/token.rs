use cp_core::token::Claims;

const REGISTERED: [&str; 7] = ["iss", "sub", "aud", "exp", "iat", "nbf", "jti"];
const ROWS_AT_MOST: usize = 12;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Said {
    pub who: String,
    pub life: String,
    pub names: String,
    pub values: String,
}

pub fn said_of(text: &str, now: i64, english: bool) -> Option<Said> {
    let claims = cp_core::token::claims_of(text)?;
    let (names, values) = table_of(&claims, now, english);
    Some(Said {
        who: joined(&[
            claims.subject().unwrap_or(""),
            claims.issuer().unwrap_or(""),
        ]),
        life: life_of(&claims, now, english),
        names: names.join("\n"),
        values: values.join("\n"),
    })
}

pub fn rows_in(text: &str) -> usize {
    cp_core::token::claims_of(text)
        .map(|claims| table_of(&claims, 0, false).0.len())
        .unwrap_or(0)
}

fn table_of(claims: &Claims, now: i64, english: bool) -> (Vec<String>, Vec<String>) {
    let mut names = Vec::new();
    let mut values = Vec::new();
    let ordered = REGISTERED
        .iter()
        .filter(|name| claims.payload.contains_key(**name))
        .copied()
        .chain(
            claims
                .payload
                .keys()
                .filter(|name| !REGISTERED.contains(&name.as_str()))
                .map(String::as_str),
        );
    let mut over = 0usize;
    for name in ordered {
        if names.len() == ROWS_AT_MOST {
            over += 1;
            continue;
        }
        let Some(value) = claims.payload.get(name) else {
            continue;
        };
        let said = said_as(name, value, now, english);
        if said.is_empty() {
            continue;
        }
        names.push(name.to_owned());
        values.push(said);
    }
    if over > 0 {
        names.push(crate::say::pick_in(english, "y más", "and more").to_owned());
        values.push(format!("+{over}"));
    }
    (names, values)
}

fn said_as(name: &str, value: &serde_json::Value, now: i64, english: bool) -> String {
    if matches!(name, "exp" | "iat" | "nbf")
        && let Some(seconds) = value.as_i64()
    {
        return moment_of(seconds, now, english);
    }
    one_line(&match value {
        serde_json::Value::String(said) => said.clone(),
        serde_json::Value::Null => "null".to_owned(),
        serde_json::Value::Object(_) => "{ … }".to_owned(),
        serde_json::Value::Array(_) => "[ … ]".to_owned(),
        other => other.to_string(),
    })
}

fn gap_between(at: i64, now: i64) -> i64 {
    i64::try_from(at.saturating_sub(now).unsigned_abs()).unwrap_or(i64::MAX)
}

fn moment_of(seconds: i64, now: i64, english: bool) -> String {
    let at = seconds.saturating_mul(1_000);
    let span = crate::age::span_in(english, gap_between(at, now));
    match (at <= now, english) {
        (true, true) => format!("{span} ago"),
        (true, false) => format!("hace {span}"),
        (false, true) => format!("in {span}"),
        (false, false) => format!("en {span}"),
    }
}

fn life_of(claims: &Claims, now: i64, english: bool) -> String {
    let Some(seconds) = claims.expires_at() else {
        return String::new();
    };
    let at = seconds.saturating_mul(1_000);
    let span = crate::age::span_in(english, gap_between(at, now));
    match (at <= now, english) {
        (true, true) => format!("{span} ago"),
        (true, false) => format!("hace {span}"),
        (false, true) => format!("expires in {span}"),
        (false, false) => format!("caduca en {span}"),
    }
}

fn one_line(said: &str) -> String {
    said.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn joined(parts: &[&str]) -> String {
    parts
        .iter()
        .map(|one| one.trim())
        .filter(|one| !one.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

#[cfg(test)]
#[path = "token_test.rs"]
mod tests;
