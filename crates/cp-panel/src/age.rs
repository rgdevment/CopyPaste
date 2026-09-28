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
mod tests {
    use super::*;

    #[test]
    fn the_age_reads_the_way_the_card_shows_it() {
        let now = 10 * DAY;
        assert_eq!(age_text(now, now - 5_000), "ahora");
        assert_eq!(age_text(now, now - 9 * MINUTE), "9 min");
        assert_eq!(age_text(now, now - 3 * HOUR), "3 h");
        assert_eq!(age_text(now, now - 30 * HOUR), "ayer");
        assert_eq!(age_text(now, now - 3 * DAY), "3 d");
        assert_eq!(age_text(now, now - 9 * DAY), "1 sem");
        assert_eq!(age_text(now, now - 100 * DAY), "3 mes");
        assert_eq!(age_text(now, now - 800 * DAY), "2 a");
    }

    #[test]
    fn each_unit_starts_exactly_at_its_threshold() {
        let now = 1_000 * DAY;
        assert_eq!(age_text(now, now - MINUTE + 1), "ahora");
        assert_eq!(age_text(now, now - MINUTE), "1 min");
        assert_eq!(age_text(now, now - HOUR + 1), "59 min");
        assert_eq!(age_text(now, now - HOUR), "1 h");
        assert_eq!(age_text(now, now - DAY + 1), "23 h");
        assert_eq!(age_text(now, now - DAY), "ayer");
        assert_eq!(age_text(now, now - 2 * DAY + 1), "ayer");
        assert_eq!(age_text(now, now - 2 * DAY), "2 d");
        assert_eq!(age_text(now, now - 7 * DAY + 1), "6 d");
        assert_eq!(age_text(now, now - 7 * DAY), "1 sem");
        assert_eq!(age_text(now, now - 30 * DAY + 1), "4 sem");
        assert_eq!(age_text(now, now - 30 * DAY), "1 mes");
        assert_eq!(age_text(now, now - 365 * DAY + 1), "12 mes");
        assert_eq!(age_text(now, now - 365 * DAY), "1 a");
    }

    #[test]
    fn a_clock_that_runs_behind_the_item_is_still_now() {
        assert_eq!(age_text(100, 5_000), "ahora");
    }

    #[test]
    fn the_same_ages_read_in_english_when_english_is_the_tongue() {
        let now = 10 * DAY;
        assert_eq!(age_in(true, now, now - 5_000), "now");
        assert_eq!(age_in(true, now, now - 9 * MINUTE), "9 min");
        assert_eq!(age_in(true, now, now - 30 * HOUR), "yesterday");
        assert_eq!(age_in(true, now, now - 9 * DAY), "1 w");
        assert_eq!(age_in(true, now, now - 100 * DAY), "3 mo");
        assert_eq!(age_in(true, now, now - 800 * DAY), "2 y");
    }
}
