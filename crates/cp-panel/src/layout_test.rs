use super::*;

#[test]
fn no_filter_at_all_is_the_general_layout() {
    assert_eq!(layout_for(&[]), Layout::Everything);
}

#[test]
fn one_kind_gets_the_layout_that_kind_asked_for() {
    assert_eq!(layout_for(&[Kind::Json]), Layout::Json);
    assert_eq!(layout_for(&[Kind::Image]), Layout::Image);
    assert_eq!(layout_for(&[Kind::Link]), Layout::Link);
    assert_eq!(layout_for(&[Kind::Video]), Layout::Video);
    assert_eq!(layout_for(&[Kind::Audio]), Layout::Audio);
    assert_eq!(layout_for(&[Kind::Folder]), Layout::Folder);
    assert_eq!(layout_for(&[Kind::File]), Layout::Papers);
}

#[test]
fn more_than_one_kind_goes_back_to_the_general_layout() {
    assert_eq!(layout_for(&[Kind::Json, Kind::Image]), Layout::Everything);
    assert_eq!(
        layout_for(&[Kind::Video, Kind::Audio, Kind::Folder]),
        Layout::Everything
    );
}

#[test]
fn the_same_kind_twice_is_still_one_kind() {
    assert_eq!(layout_for(&[Kind::Json, Kind::Json]), Layout::Json);
    assert_eq!(
        layout_for(&[Kind::Image, Kind::Image, Kind::Image]),
        Layout::Image
    );
}

#[test]
fn a_kind_with_nothing_designed_for_it_stays_general() {
    for kind in [
        Kind::Text,
        Kind::Code,
        Kind::Email,
        Kind::Phone,
        Kind::Color,
        Kind::Ip,
        Kind::Uuid,
        Kind::Token,
    ] {
        assert_eq!(
            layout_for(&[kind]),
            Layout::Everything,
            "{kind:?} has no layout of its own yet"
        );
    }
}

#[test]
fn every_kind_the_store_knows_answers_something() {
    for kind in Kind::ALL {
        let _ = layout_for(&[kind]);
    }
    assert_eq!(Kind::ALL.len(), 15, "a new kind needs a decision here");
}

#[test]
fn the_seven_kinds_with_a_layout_are_the_seven_that_were_drawn() {
    let with_one: Vec<Kind> = Kind::ALL
        .into_iter()
        .filter(|kind| layout_for(&[*kind]) != Layout::Everything)
        .collect();
    assert_eq!(
        with_one,
        vec![
            Kind::Json,
            Kind::Link,
            Kind::Image,
            Kind::File,
            Kind::Folder,
            Kind::Audio,
            Kind::Video
        ],
        "the maquette drew these seven and no others"
    );
}

#[test]
fn no_two_layouts_answer_to_the_same_name() {
    for (at, one) in Layout::ALL.iter().enumerate() {
        for other in &Layout::ALL[at + 1..] {
            assert_ne!(one.as_str(), other.as_str(), "{one:?} and {other:?}");
        }
        assert!(!one.as_str().is_empty());
    }
}

#[test]
fn only_the_views_the_maquette_grouped_ask_for_grouping() {
    assert!(Layout::Link.groups(), "by domain");
    assert!(Layout::Folder.groups(), "by drive");
    assert!(Layout::Papers.groups(), "by format");
    for one in Layout::ALL {
        if matches!(one, Layout::Link | Layout::Folder | Layout::Papers) {
            continue;
        }
        assert!(!one.groups(), "{one:?} was drawn as one flat list");
    }
}

#[test]
fn every_name_slint_compares_against_is_a_name_rust_still_writes() {
    let ui = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");
    let known: Vec<String> = Layout::ALL
        .iter()
        .map(|one| one.as_str().to_owned())
        .chain(
            cp_core::kind::Kind::ALL
                .iter()
                .map(|one| one.as_str().to_owned()),
        )
        .chain(crate::papers::FAMILIES.iter().map(|one| (*one).to_owned()))
        .chain(
            Layout::ALL
                .iter()
                .flat_map(|one| crate::ways::ways_of(*one))
                .map(|one| one.key.to_owned()),
        )
        .collect();

    let named = ["layout", "shown-as", "kind", "way", "family"];
    let mut looked = 0;
    for entry in std::fs::read_dir(&ui).expect("the ui folder") {
        let path = entry.expect("an entry").path();
        if path.extension().and_then(|one| one.to_str()) != Some("slint") {
            continue;
        }
        let said = std::fs::read_to_string(&path).expect("read");
        for (at, _) in said.match_indices("== \"") {
            let before = said[..at].trim_end();
            if !named.iter().any(|one| before.ends_with(one)) {
                continue;
            }
            let rest = &said[at + 4..];
            let Some(end) = rest.find('"') else { continue };
            let name = &rest[..end];
            looked += 1;
            assert!(
                known.iter().any(|one| one == name),
                "{}: «{name}» is compared against but Rust writes no such name, so that branch is \
                 dead and nothing says so",
                path.display()
            );
        }
    }
    assert!(
        looked > 5,
        "the guard found nothing to guard, so it guards nothing: {looked}"
    );
}

#[test]
fn the_general_view_json_links_and_pictures_wear_the_same_card() {
    for layout in Layout::ALL {
        let expected = matches!(
            layout,
            Layout::Everything | Layout::Json | Layout::Link | Layout::Image
        );
        assert_eq!(layout.shows_cards(), expected, "{}", layout.as_str());
    }
}
