use super::*;

const JANUARY: i64 = 1_768_478_400_000;
const JULY: i64 = 1_784_116_800_000;

#[test]
fn the_offset_at_any_instant_is_whole_minutes_inside_the_zones_that_exist() {
    for at in [0, JANUARY, JULY] {
        let offset = utc_offset_at(at);
        assert_eq!(offset % 60, 0, "{at}");
        assert!((-12 * 3_600..=14 * 3_600).contains(&offset), "{offset}");
    }
}

#[test]
fn summer_and_winter_differ_by_at_most_the_hour_daylight_saving_moves() {
    assert!((utc_offset_at(JANUARY) - utc_offset_at(JULY)).abs() <= 3_600);
}
