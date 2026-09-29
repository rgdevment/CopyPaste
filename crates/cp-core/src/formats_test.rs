use super::*;

pub(super) const PROBE: Catalog = Catalog {
    hangs: &["hangs/forever"],
    wasteful: &["huge/icon"],
    wanted: &[
        "plain/text",
        "rich/text",
        "cheap/image",
        "costly/image",
        "one/file",
    ],
    aliases: &[("old/text", "plain/text")],
    concealed: &["secret/marker"],
    denied_when_zero: &["may/record"],
    opaque_prefixes: &["dyn."],
    text: &["plain/text", "rich/text"],
    files: &["one/file"],
    images_by_preference: &["cheap/image", "costly/image"],
    equivalents: &[&["cheap/markup", "costly/markup"]],
    embeddable: &["embedded/document"],
    page_archives: &["whole/page"],
};

const NOTHING: Catalog = Catalog {
    hangs: &[],
    wasteful: &[],
    wanted: &[],
    aliases: &[],
    concealed: &[],
    denied_when_zero: &[],
    opaque_prefixes: &[],
    text: &[],
    files: &[],
    images_by_preference: &[],
    equivalents: &[],
    embeddable: &[],
    page_archives: &[],
};

#[test]
fn an_empty_catalogue_notes_everything_and_promises_nothing() {
    assert_eq!(NOTHING.decide("any/thing"), Take::Presence);
    assert_eq!(NOTHING.decide(""), Take::Presence);
    assert_eq!(NOTHING.classify(&["any/thing"]), None);
    assert_eq!(NOTHING.preferred_image(&["any/thing"]), None);
    assert_eq!(NOTHING.refusal(&["any/thing"]), None);
    assert_eq!(NOTHING.declines("any/thing", &[0, 0, 0, 0]), None);
    assert!(!NOTHING.costlier_twin("any/thing", &["other/thing"]));
    assert_eq!(NOTHING.canonical("x"), "x");
}

#[test]
fn nothing_offered_at_all_is_not_an_item() {
    assert_eq!(PROBE.classify(&[]), None);
    assert_eq!(PROBE.preferred_image(&[]), None);
    assert_eq!(PROBE.refusal(&[]), None);
}

#[test]
fn an_empty_type_name_is_just_unknown() {
    assert_eq!(PROBE.decide(""), Take::Presence);
    assert_eq!(PROBE.canonical(""), "");
}

#[test]
fn what_hangs_is_never_asked_for() {
    assert_eq!(PROBE.decide("hangs/forever"), Take::Never);
}

#[test]
fn the_expensive_and_the_unknown_are_only_noted() {
    assert_eq!(PROBE.decide("huge/icon"), Take::Presence);
    assert_eq!(PROBE.decide("dyn.abc123"), Take::Presence);
    assert_eq!(PROBE.decide("who/knows"), Take::Presence);
}

#[test]
fn wasteful_beats_wanted_when_a_type_is_in_both() {
    const GREEDY: Catalog = Catalog {
        hangs: &[],
        wasteful: &["costly/image"],
        wanted: &["costly/image", "plain/text"],
        aliases: &[],
        concealed: &[],
        denied_when_zero: &[],
        opaque_prefixes: &[],
        text: &["plain/text"],
        files: &[],
        images_by_preference: &["costly/image"],
        equivalents: &[],
        embeddable: &[],
        page_archives: &[],
    };
    assert_eq!(
        GREEDY.decide("costly/image"),
        Take::Presence,
        "being on the wanted list doesn't save a type that wastes"
    );
    assert_eq!(GREEDY.decide("plain/text"), Take::Payload);
}

#[test]
fn a_legacy_name_is_the_modern_one() {
    assert_eq!(PROBE.canonical("old/text"), "plain/text");
    assert_eq!(PROBE.decide("old/text"), Take::Payload);
}

#[test]
fn the_cheapest_representation_wins() {
    assert_eq!(
        PROBE.preferred_image(&["costly/image", "cheap/image"]),
        Some("cheap/image")
    );
    assert_eq!(
        PROBE.preferred_image(&["costly/image"]),
        Some("costly/image")
    );
    assert_eq!(PROBE.preferred_image(&["plain/text"]), None);
}

#[test]
fn a_secret_is_recognised_by_the_type_alone() {
    assert_eq!(
        PROBE.refusal(&["plain/text", "secret/marker"]),
        Some(Refusal::Marked("secret/marker"))
    );
    assert_eq!(PROBE.refusal(&["plain/text"]), None);
}

#[test]
fn classifying_never_discards_the_rest() {
    let mixed = ["plain/text", "cheap/image", "one/file"];
    assert_eq!(PROBE.classify(&mixed), Some(Family::Files));
    let kept: Vec<&str> = mixed
        .iter()
        .filter(|id| PROBE.decide(id) == Take::Payload)
        .copied()
        .collect();
    assert_eq!(
        kept.len(),
        3,
        "classifying isn't picking one and discarding the rest"
    );
}

#[test]
fn an_image_offered_with_its_text_is_still_an_image() {
    assert_eq!(
        PROBE.classify(&["plain/text", "cheap/image"]),
        Some(Family::Image)
    );
}

#[test]
fn a_copied_file_keeps_its_path_and_notes_its_icon() {
    let files = Some(Family::Files);
    assert_eq!(PROBE.decide_in(files, "cheap/image"), Take::Presence);
    assert_eq!(PROBE.decide_in(files, "costly/image"), Take::Presence);
    assert_eq!(PROBE.decide_in(files, "one/file"), Take::Payload);
    assert_eq!(PROBE.decide_in(files, "old/text"), Take::Payload);
    assert_eq!(PROBE.decide_in(files, "hangs/forever"), Take::Never);
}

#[test]
fn outside_a_file_copy_the_family_changes_nothing() {
    for family in [None, Some(Family::Text), Some(Family::Image)] {
        for id in [
            "cheap/image",
            "costly/image",
            "old/text",
            "hangs/forever",
            "huge/icon",
            "who/knows",
        ] {
            assert_eq!(
                PROBE.decide_in(family, id),
                PROBE.decide(id),
                "{family:?} {id}"
            );
        }
    }
}

#[test]
fn a_copied_image_notes_the_page_archive_and_a_text_keeps_it() {
    const ARCHIVING: Catalog = Catalog {
        wanted: &["plain/text", "cheap/image", "whole/page"],
        ..PROBE
    };
    assert_eq!(
        ARCHIVING.decide_in(Some(Family::Image), "whole/page"),
        Take::Presence,
        "a 43 KB logo shouldn't cost the whole page"
    );
    assert_eq!(
        ARCHIVING.decide_in(Some(Family::Text), "whole/page"),
        Take::Payload,
        "for styled text, the archive is the content"
    );
    assert_eq!(ARCHIVING.decide_in(None, "whole/page"), Take::Payload);
    assert_eq!(
        ARCHIVING.decide_in(Some(Family::Image), "cheap/image"),
        Take::Payload
    );
}

#[test]
fn a_file_copy_notes_the_icon_under_its_legacy_name_too() {
    const ICONIC: Catalog = Catalog {
        aliases: &[("legacy/picture", "cheap/image")],
        ..PROBE
    };
    assert_eq!(
        ICONIC.decide_in(Some(Family::Files), "legacy/picture"),
        Take::Presence
    );
    assert_eq!(
        ICONIC.decide_in(Some(Family::Text), "legacy/picture"),
        Take::Payload
    );
}
