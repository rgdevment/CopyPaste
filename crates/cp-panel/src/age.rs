const MINUTE: i64 = 60_000;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;

pub fn span_in(english: bool, span: i64) -> String {
    let say = |es: &'static str, en: &'static str| crate::say::pick_in(english, es, en);
    let span = span.max(0);
    if span < MINUTE {
        return "< 1 min".to_owned();
    }
    if span < HOUR {
        return format!("{} min", span / MINUTE);
    }
    if span < DAY {
        return format!("{} h", span / HOUR);
    }
    if span < 30 * DAY {
        return counted(span / DAY, say("día", "day"), say("días", "days"));
    }
    if span < 365 * DAY {
        return counted(
            span / (30 * DAY),
            say("mes", "month"),
            say("meses", "months"),
        );
    }
    counted(span / (365 * DAY), say("año", "year"), say("años", "years"))
}

fn counted(how_many: i64, one: &str, many: &str) -> String {
    format!("{how_many} {}", if how_many == 1 { one } else { many })
}

pub fn age_text(now: i64, at: i64) -> String {
    age_in(crate::say::in_english(), now, at)
}

fn age_in(english: bool, now: i64, at: i64) -> String {
    let say = |es: &'static str, en: &'static str| crate::say::pick_in(english, es, en);
    let gone = now - at;
    if gone < MINUTE {
        return say("ahora", "now").into();
    }
    if gone < HOUR {
        return format!("{} min", gone / MINUTE);
    }
    if gone < DAY {
        return format!("{} h", gone / HOUR);
    }
    if gone < 2 * DAY {
        return say("ayer", "yesterday").into();
    }
    if gone < 7 * DAY {
        return format!("{} {}", gone / DAY, say("d", "d"));
    }
    if gone < 30 * DAY {
        return format!("{} {}", gone / (7 * DAY), say("sem", "w"));
    }
    if gone < 365 * DAY {
        return format!("{} {}", gone / (30 * DAY), say("mes", "mo"));
    }
    format!("{} {}", gone / (365 * DAY), say("a", "y"))
}

#[cfg(test)]
#[path = "age_test.rs"]
mod tests;
