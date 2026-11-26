use super::*;

#[test]
fn a_slow_double_click_lingers_no_more_than_half_a_second() {
    assert_eq!(
        lingering_for(Duration::from_millis(900)),
        Duration::from_millis(500)
    );
}

#[test]
fn a_quick_double_click_lingers_as_long_as_it_takes() {
    assert_eq!(
        lingering_for(Duration::from_millis(300)),
        Duration::from_millis(300)
    );
    assert_eq!(lingering_for(Duration::ZERO), Duration::ZERO);
}
