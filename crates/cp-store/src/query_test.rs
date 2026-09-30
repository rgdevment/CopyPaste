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
    let filter = parsed("the coffee shop on the corner");
    assert_eq!(
        filter.query.as_deref(),
        Some("the coffee shop on the corner")
    );
    assert_eq!(
        filter,
        Filter {
            query: Some("the coffee shop on the corner".into()),
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
    assert_eq!(parsed("k:photo").query.as_deref(), Some("k:photo"));
    assert_eq!(
        parsed("k:image,photo").query.as_deref(),
        Some("k:image,photo"),
        "a list with one unknown value is not half a list"
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
    let filter = parsed("write to john@example.test");
    assert!(filter.apps.is_empty());
    assert_eq!(filter.query.as_deref(), Some("write to john@example.test"));
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
        "the app name keeps its own case"
    );
}

#[test]
fn a_repeated_order_keeps_the_last_and_a_repeated_class_repeats() {
    assert_eq!(parsed("sort:pasted sort:used").order, Order::LastUsed);
    assert_eq!(parsed("k:json k:json").kinds, vec![Kind::Json, Kind::Json]);
    assert_eq!(
        parsed("k:ïmage").query.as_deref(),
        Some("k:ïmage"),
        "with a diacritic mark it is not a class"
    );
}

#[test]
fn a_hash_followed_by_a_number_is_text_not_a_colour() {
    let filter = parsed("PR #3");
    assert_eq!(filter.query.as_deref(), Some("PR #3"));
    assert!(
        filter.colors.is_empty(),
        "«#3» is a PR number, not the colour purple"
    );
    assert_eq!(parsed("#3,red").query.as_deref(), Some("#3,red"));
    assert_eq!(
        parsed("c:3").colors,
        vec![3],
        "by key, the index does count"
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
    assert_eq!(parsed("~yesterday").query.as_deref(), Some("~yesterday"));
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
    let filter = parsed("label:invoice l:may");
    assert_eq!(filter.label_query.as_deref(), Some("invoice may"));
    assert_eq!(filter.query, None);
}

#[test]
fn the_order_is_a_word_too() {
    assert_eq!(parsed("sort:pasted").order, Order::MostPasted);
    assert_eq!(parsed("order:used").order, Order::LastUsed);
    let explicit = parsed("sort:recent");
    assert_eq!(explicit.order, Order::Recent);
    assert_eq!(explicit.query, None, "it is an operator, not text");
}

#[test]
fn a_negated_state_or_date_is_not_an_operator() {
    assert_eq!(parsed("-is:pinned").query.as_deref(), Some("-is:pinned"));
    assert_eq!(parsed("-~1h").query.as_deref(), Some("-~1h"));
    assert_eq!(parsed("-c:red").query.as_deref(), Some("-c:red"));
    assert_eq!(parsed("-l:invoice").query.as_deref(), Some("-l:invoice"));
    assert_eq!(parsed("-sort:used").query.as_deref(), Some("-sort:used"));
    assert_eq!(parsed("-l:invoice").label_query, None);
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
            "«{typed}» is the whole history"
        );
    }
    assert_eq!(parsed("hello ...").query.as_deref(), Some("hello"));
}

#[test]
fn a_colon_inside_ordinary_text_is_not_a_key() {
    assert_eq!(
        parsed("https://example.test").query.as_deref(),
        Some("https://example.test"),
        "https is not a key"
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
    let filter = parsed("  request  k:json,text -@safari #red ~7d is:pinned l:may sort:used ");
    assert_eq!(filter.query.as_deref(), Some("request"));
    assert_eq!(filter.kinds, vec![Kind::Json, Kind::Text]);
    assert_eq!(filter.exclude_apps, vec!["safari"]);
    assert_eq!(filter.colors, vec![1]);
    assert_eq!(filter.since, Some(CLOCK.now - WEEK));
    assert!(filter.pinned_only);
    assert_eq!(filter.label_query.as_deref(), Some("may"));
    assert_eq!(filter.order, Order::LastUsed);
}

#[test]
fn quotes_group_words_and_then_disappear() {
    assert_eq!(parsed("\"two words\"").query.as_deref(), Some("two words"));
    assert_eq!(
        parsed("not closing \"the quote").query.as_deref(),
        Some("not closing the quote")
    );
}
