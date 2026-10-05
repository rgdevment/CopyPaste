use super::*;

#[test]
fn the_offset_is_a_whole_number_of_minutes_inside_the_zones_that_exist() {
    let offset = utc_offset_seconds();
    assert_eq!(offset % 60, 0);
    assert!((-12 * 3_600..=14 * 3_600).contains(&offset), "{offset}");
}
