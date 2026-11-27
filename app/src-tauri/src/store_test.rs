use super::*;

#[test]
fn only_what_closing_copanda_will_not_clear_counts_as_paused() {
    use StoreQueueItemExtendedState as Reason;
    for reason in [
        Reason::Paused,
        Reason::PausedLowBattery,
        Reason::PausedWiFiRequired,
    ] {
        assert_eq!(queue_of(StoreQueueItemState::Paused, reason), Queue::Paused);
    }
    for reason in [Reason::PausedPackagesInUse, Reason::PausedReadyToInstall] {
        assert_eq!(queue_of(StoreQueueItemState::Paused, reason), Queue::Moving);
    }
    assert_eq!(
        queue_of(StoreQueueItemState::Active, Reason::ActiveDownloading),
        Queue::Moving
    );
}

#[test]
fn a_package_version_reads_as_the_release_it_carries() {
    assert_eq!(said((3, 5, 1)), "3.5.1");
}

#[test]
fn a_copy_run_outside_any_package_is_not_one_the_store_sold() {
    assert!(!sold_here());
    assert_eq!(waiting_beside_us(), None);
}
