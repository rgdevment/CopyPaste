use super::tests::PROBE;
use super::*;

#[test]
fn a_spreadsheet_is_text_even_when_it_offers_a_picture_of_itself() {
    let excel = ["plain/text", "cheap/image", "embedded/document"];
    assert_eq!(PROBE.classify(&excel), Some(Family::Text));
}

#[test]
fn an_image_with_its_address_alongside_is_still_an_image() {
    let browser = ["plain/text", "cheap/image"];
    assert_eq!(PROBE.classify(&browser), Some(Family::Image));
}

#[test]
fn an_embeddable_marker_without_text_does_not_hide_the_image() {
    assert_eq!(
        PROBE.classify(&["cheap/image", "embedded/document"]),
        Some(Family::Image)
    );
}

#[test]
fn files_still_win_over_everything() {
    assert_eq!(
        PROBE.classify(&["one/file", "plain/text", "cheap/image", "embedded/document"]),
        Some(Family::Files)
    );
}

#[test]
fn the_costlier_wrapping_of_the_same_text_is_only_noted() {
    let firefox = ["costly/markup", "cheap/markup", "plain/text"];
    assert!(PROBE.costlier_twin("costly/markup", &firefox));
    assert!(!PROBE.costlier_twin("cheap/markup", &firefox));
}

#[test]
fn the_only_wrapping_on_offer_is_never_the_costlier_one() {
    assert!(!PROBE.costlier_twin("costly/markup", &["costly/markup"]));
    assert!(!PROBE.costlier_twin("costly/image", &["costly/image"]));
}

#[test]
fn a_type_in_no_group_has_no_twin() {
    assert!(!PROBE.costlier_twin("plain/text", &["plain/text", "cheap/markup"]));
}

#[test]
fn a_marker_that_says_zero_refuses_the_copy() {
    assert_eq!(
        PROBE.declines("may/record", &[0, 0, 0, 0]),
        Some(Refusal::Declined("may/record"))
    );
    assert_eq!(PROBE.declines("may/record", &[1, 0, 0, 0]), None);
}

#[test]
fn a_marker_too_short_to_read_is_obeyed_anyway() {
    for unreadable in [&[][..], &[0][..], &[0, 0, 0][..], &[1, 0, 0][..]] {
        assert_eq!(
            PROBE.declines("may/record", unreadable),
            Some(Refusal::Declined("may/record")),
            "{unreadable:?}"
        );
    }
}

#[test]
fn only_a_declared_marker_is_read() {
    assert_eq!(PROBE.declines("plain/text", &[0, 0, 0, 0]), None);
    assert_eq!(PROBE.declines("secret/marker", &[0, 0, 0, 0]), None);
}

#[test]
fn a_longer_marker_is_read_by_its_first_four_bytes() {
    assert_eq!(
        PROBE.declines("may/record", &[0, 0, 0, 0, 9, 9]),
        Some(Refusal::Declined("may/record"))
    );
    assert_eq!(PROBE.declines("may/record", &[1, 0, 0, 0, 0, 0]), None);
}

#[test]
fn the_four_bytes_are_one_little_endian_number() {
    assert_eq!(PROBE.declines("may/record", &[0, 0, 0, 1]), None);
    assert_eq!(PROBE.declines("may/record", &[0, 1, 0, 0]), None);
}

#[test]
fn either_road_to_a_refusal_names_the_marker() {
    assert_eq!(
        PROBE
            .refusal(&["plain/text", "secret/marker"])
            .map(Refusal::marker),
        Some("secret/marker")
    );
    assert_eq!(
        PROBE
            .declines("may/record", &[0, 0, 0, 0])
            .map(Refusal::marker),
        Some("may/record")
    );
}
