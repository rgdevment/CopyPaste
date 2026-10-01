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
