use super::*;

#[test]
fn the_percentile_is_the_nearest_rank_and_survives_an_empty_sample() {
    let sample = [5.0, 1.0, 3.0, 2.0, 4.0];
    assert_eq!(percentile(&sample, 50.0), 3.0);
    assert_eq!(percentile(&sample, 95.0), 5.0);
    assert_eq!(percentile(&sample, 0.0), 1.0);
    assert_eq!(percentile(&[], 95.0), 0.0);
}
