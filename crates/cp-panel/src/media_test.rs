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
