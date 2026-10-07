use super::*;

#[test]
fn each_status_the_system_reports_maps_to_its_own_state() {
    assert_eq!(state_of(0), LoginState::Off);
    assert_eq!(state_of(1), LoginState::On);
    assert_eq!(state_of(2), LoginState::NeedsApproval);
    assert_eq!(state_of(3), LoginState::Missing);
}

#[test]
fn a_status_nobody_knows_reads_as_off_not_as_on() {
    assert_eq!(state_of(-1), LoginState::Off);
    assert_eq!(state_of(99), LoginState::Off);
}

#[test]
fn a_bundle_the_system_does_not_know_is_never_reported_as_on() {
    assert_ne!(state(), LoginState::On);
}
