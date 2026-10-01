use super::*;

#[test]
fn the_general_view_has_no_second_control_to_offer() {
    assert!(ways_of(Layout::Everything).is_empty());
}

#[test]
fn every_other_view_offers_exactly_two_ways_of_looking() {
    for layout in Layout::ALL {
        if layout == Layout::Everything {
            continue;
        }
        assert_eq!(
            ways_of(layout).len(),
            2,
            "{layout:?} needs two ways, not one and not three"
        );
    }
}

#[test]
fn the_three_grouped_views_share_the_same_pair() {
    let grouped = ways_of(Layout::Link);
    assert_eq!(ways_of(Layout::Folder), grouped);
    assert_eq!(ways_of(Layout::Papers), grouped);
    assert_eq!(grouped[0].key, "by-group");
    assert_eq!(grouped[1].key, "newest");
}

#[test]
fn only_the_newest_way_asks_to_stop_grouping() {
    for layout in Layout::ALL {
        for way in ways_of(layout) {
            if way.key == "newest" {
                assert!(way.recent, "{layout:?} newest has to drop the grouping");
            } else {
                assert!(!way.recent, "{layout:?} {} is not an ordering", way.key);
            }
        }
    }
}

#[test]
fn the_first_way_is_what_a_view_opens_with() {
    assert_eq!(chosen(Layout::Json, "nada de eso").key, "keys");
    assert_eq!(chosen(Layout::Image, "").key, "grid");
    assert_eq!(chosen(Layout::Link, "inventada").key, "by-group");
}

#[test]
fn a_way_that_was_chosen_comes_back_as_itself() {
    assert_eq!(chosen(Layout::Json, "raw").key, "raw");
    assert_eq!(chosen(Layout::Image, "rows").key, "rows");
    assert_eq!(chosen(Layout::Audio, "tight").key, "tight");
    assert_eq!(chosen(Layout::Folder, "newest").key, "newest");
}

#[test]
fn a_way_from_another_view_is_not_honoured_in_this_one() {
    assert_eq!(
        chosen(Layout::Json, "grid").key,
        "keys",
        "the grid belongs to images, so json falls back to its own first"
    );
}

#[test]
fn only_the_two_ways_that_ask_for_the_general_body_are_plain() {
    assert!(
        chosen(Layout::Json, "raw").plain,
        "raw json is read as text"
    );
    assert!(chosen(Layout::Audio, "tight").plain);
    assert!(chosen(Layout::Video, "tight").plain);
    for layout in Layout::ALL {
        for way in ways_of(layout) {
            let expected = way.key == "raw" || way.key == "tight";
            assert_eq!(
                way.plain, expected,
                "{layout:?} {} disagrees about yielding the body",
                way.key
            );
        }
    }
}

#[test]
fn a_plain_way_is_never_an_ordering() {
    for layout in Layout::ALL {
        for way in ways_of(layout) {
            assert!(
                !(way.plain && way.recent),
                "{} cannot both yield the body and change the order",
                way.key
            );
        }
    }
}

#[test]
fn nothing_is_left_untranslated() {
    for layout in Layout::ALL {
        for way in ways_of(layout) {
            assert!(!way.es.is_empty(), "{} has no spanish", way.key);
            assert!(!way.en.is_empty(), "{} has no english", way.key);
            assert_ne!(way.es, way.en, "{} was left in one tongue", way.key);
            assert_eq!(label_of(*way, false), way.es);
            assert_eq!(label_of(*way, true), way.en);
        }
    }
}

#[test]
fn no_view_offers_the_same_way_twice() {
    for layout in Layout::ALL {
        let ways = ways_of(layout);
        for (at, one) in ways.iter().enumerate() {
            for other in &ways[at + 1..] {
                assert_ne!(one.key, other.key, "{layout:?} repeats {}", one.key);
            }
        }
    }
}
