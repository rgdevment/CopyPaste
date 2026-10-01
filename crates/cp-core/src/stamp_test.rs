use super::*;

#[test]
fn the_epoch_itself_reads_as_the_first_day_of_1970() {
    assert_eq!(said_of(0), "1970-01-01 00:00:00Z");
}

#[test]
fn a_leap_day_is_not_skipped_in_either_kind_of_leap_year() {
    assert_eq!(said_of(1_709_164_800), "2024-02-29 00:00:00Z");
    assert_eq!(said_of(951_782_400), "2000-02-29 00:00:00Z");
    assert_eq!(said_of(1_709_251_199), "2024-02-29 23:59:59Z");
}

#[test]
fn a_date_before_the_epoch_does_not_wrap_around() {
    assert_eq!(said_of(-2_203_891_200), "1900-03-01 00:00:00Z");
    assert_eq!(said_of(-1), "1969-12-31 23:59:59Z");
}

#[test]
fn the_shape_is_the_same_whatever_the_moment() {
    for secs in [0, 1, 86_399, 86_400, 951_825_600, 1_759_345_200, -1] {
        let said = said_of(secs);
        assert_eq!(said.len(), 20, "{said}");
        assert!(
            said.ends_with('Z'),
            "a time nobody can place is a time nobody can match"
        );
        assert_eq!(said.matches('-').count(), 2, "{said}");
        assert_eq!(said.matches(':').count(), 2, "{said}");
    }
}

#[test]
fn the_clock_of_the_moment_is_a_date_of_this_century() {
    let said = now();
    assert_eq!(said.len(), 20, "{said}");
    assert!(said.starts_with("20"), "{said}");
    assert!(said.ends_with('Z'), "{said}");
}
