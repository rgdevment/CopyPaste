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

#[test]
fn a_span_of_one_counts_in_the_singular() {
    assert_eq!(super::span_in(false, 86_400_000), "1 día");
    assert_eq!(super::span_in(false, 30 * 86_400_000), "1 mes");
    assert_eq!(super::span_in(false, 365 * 86_400_000), "1 año");
    assert_eq!(super::span_in(true, 86_400_000), "1 day");
    assert_eq!(super::span_in(true, 30 * 86_400_000), "1 month");
}

#[test]
fn a_span_of_more_than_one_counts_in_the_plural() {
    assert_eq!(super::span_in(false, 3 * 86_400_000), "3 días");
    assert_eq!(super::span_in(false, 60 * 86_400_000), "2 meses");
    assert_eq!(super::span_in(true, 730 * 86_400_000), "2 years");
}

#[test]
fn a_span_shorter_than_a_minute_says_so_without_a_number_that_reads_as_zero() {
    assert_eq!(super::span_in(false, 0), "< 1 min");
    assert_eq!(super::span_in(false, -5_000), "< 1 min");
}

#[test]
fn the_short_spans_read_the_same_in_both_tongues() {
    assert_eq!(super::span_in(false, 5 * 60_000), "5 min");
    assert_eq!(super::span_in(true, 5 * 60_000), "5 min");
    assert_eq!(super::span_in(false, 3 * 3_600_000), "3 h");
}

const NOON_UTC: i64 = 1_791_201_600_000;
const SANTIAGO: i64 = -3 * 3_600;

#[test]
fn the_calendar_day_is_counted_where_the_person_lives() {
    assert_eq!(civil_of(0), (1970, 0, 1));
    assert_eq!(civil_of(20_731), (2026, 9, 5));
    assert_eq!(civil_of(11_016), (2000, 1, 29));
    assert_eq!(day_of(1_791_167_400_000, 0), 20_731);
    assert_eq!(
        day_of(1_791_167_400_000, SANTIAGO),
        20_730,
        "02:30 in UTC is still the evening before in Santiago"
    );
}

#[test]
fn what_was_copied_falls_into_now_today_yesterday_or_before() {
    assert_eq!(when_of(NOON_UTC, 1_791_201_300_000, 0), When::Now);
    assert_eq!(when_of(NOON_UTC, 1_791_167_400_000, 0), When::Today);
    assert_eq!(when_of(NOON_UTC, 1_791_155_400_000, 0), When::Yesterday);
    assert_eq!(when_of(NOON_UTC, 1_788_426_000_000, 0), When::Before);
    assert_eq!(
        when_of(NOON_UTC, 1_791_167_400_000, SANTIAGO),
        When::Yesterday,
        "the same instant is yesterday for someone three hours behind"
    );
    assert_eq!(when_of(NOON_UTC, NOON_UTC + 60_000, 0), When::Today);
}

#[test]
fn inside_a_group_the_age_is_a_clock_or_a_date() {
    assert_eq!(age_in_group(NOON_UTC, 1_791_201_300_000, 0, false), "5 min");
    assert_eq!(age_in_group(NOON_UTC, 1_791_167_400_000, 0, false), "02:30");
    assert_eq!(age_in_group(NOON_UTC, 1_791_155_400_000, 0, true), "23:10");
    assert_eq!(age_in_group(NOON_UTC, 1_788_426_000_000, 0, false), "3 sep");
    assert_eq!(age_in_group(NOON_UTC, 1_788_426_000_000, 0, true), "3 Sep");
    assert_eq!(
        age_in_group(NOON_UTC, 1_767_222_000_000, 0, false),
        "31 dic 2025"
    );
    assert_eq!(clock_of(1_791_167_400_000, SANTIAGO), "23:30");
}

#[test]
fn every_group_has_a_name_in_both_languages() {
    for when in [When::Now, When::Today, When::Yesterday, When::Before] {
        assert!(!when_said(when, false).is_empty());
        assert_ne!(when_said(when, false), when_said(when, true));
    }
}
