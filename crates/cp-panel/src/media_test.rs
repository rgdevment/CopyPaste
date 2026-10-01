use super::*;

#[test]
fn a_shell_that_knows_nothing_leaves_no_meta() {
    assert!(said_of(None).is_empty());
    assert!(said_of(Some(Facts::default())).is_empty());
}

#[test]
fn seconds_become_whole_milliseconds() {
    let said = said_of(Some(Facts {
        duration: Some(138.4),
        ..Default::default()
    }));
    assert_eq!(said, vec![(DURATION, "138400".to_owned())]);
}

#[test]
fn a_duration_that_makes_no_sense_is_not_written_down() {
    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let said = said_of(Some(Facts {
            duration: Some(bad),
            ..Default::default()
        }));
        assert!(said.is_empty(), "{bad} should not reach the store");
    }
}

#[test]
fn a_zero_side_is_not_a_measurement() {
    let said = said_of(Some(Facts {
        duration: None,
        width: Some(0),
        height: Some(1080),
    }));
    assert_eq!(said, vec![(HEIGHT, "1080".to_owned())]);
}

#[test]
fn the_three_keys_travel_together_when_the_shell_knows_them() {
    let said = said_of(Some(Facts {
        duration: Some(2.5),
        width: Some(1920),
        height: Some(1080),
    }));
    assert_eq!(
        said,
        vec![
            (DURATION, "2500".to_owned()),
            (WIDTH, "1920".to_owned()),
            (HEIGHT, "1080".to_owned())
        ]
    );
    assert_eq!(said.len(), KEYS.len());
}

#[test]
fn every_key_written_is_a_key_the_views_ask_for() {
    let said = said_of(Some(Facts {
        duration: Some(1.0),
        width: Some(2),
        height: Some(3),
    }));
    for (key, _) in &said {
        assert!(KEYS.contains(key), "«{key}» is not in KEYS");
    }
}

#[test]
fn a_clock_reads_the_way_a_player_shows_it() {
    assert_eq!(clock_of(0), "0:00");
    assert_eq!(clock_of(3_000), "0:03");
    assert_eq!(clock_of(138_000), "2:18");
    assert_eq!(clock_of(2_320_000), "38:40");
    assert_eq!(clock_of(3_600_000), "1:00:00");
    assert_eq!(clock_of(7_384_000), "2:03:04");
}

#[test]
fn a_clock_never_goes_backwards() {
    assert_eq!(clock_of(-5_000), "0:00");
}

fn meta(pairs: &[(&str, &str)]) -> std::collections::HashMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}

#[test]
fn a_row_with_no_meta_yet_says_nothing_instead_of_zero() {
    assert_eq!(said_in(None), (String::new(), String::new()));
    assert_eq!(
        said_in(Some(&meta(&[]))),
        (String::new(), String::new()),
        "measured but empty is still nothing to show"
    );
}

#[test]
fn what_the_shell_measured_becomes_a_clock_and_a_pair_of_sides() {
    let said = said_in(Some(&meta(&[
        (DURATION, "138000"),
        (WIDTH, "1920"),
        (HEIGHT, "1080"),
    ])));
    assert_eq!(said, ("2:18".to_owned(), "1920×1080".to_owned()));
}

#[test]
fn an_audio_has_a_clock_and_no_sides() {
    let said = said_in(Some(&meta(&[(DURATION, "2320000")])));
    assert_eq!(said, ("38:40".to_owned(), String::new()));
}

#[test]
fn half_a_measurement_is_not_shown_as_a_size() {
    let said = said_in(Some(&meta(&[(WIDTH, "1920")])));
    assert_eq!(said.1, "", "one side alone says nothing");
    let zeroed = said_in(Some(&meta(&[(WIDTH, "0"), (HEIGHT, "1080")])));
    assert_eq!(zeroed.1, "");
}

#[test]
fn a_meta_value_that_is_not_a_number_is_ignored_rather_than_shown_raw() {
    let said = said_in(Some(&meta(&[
        (DURATION, "un rato"),
        (WIDTH, "ancho"),
        (HEIGHT, "1080"),
    ])));
    assert_eq!(said, (String::new(), String::new()));
}
