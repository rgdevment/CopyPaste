const MINUTE: i64 = 60_000;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;

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
