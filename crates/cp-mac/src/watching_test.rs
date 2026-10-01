use super::*;

#[test]
fn the_pasteboard_counts_one_per_copy_so_a_miss_can_be_told() {
    let watching = every(cp_core::watching::EVERY, || {});
    assert_eq!(watching.missed(), Some(0));
    assert!(watching.close());
}

#[test]
fn both_ends_of_a_write_of_ours_are_told_to_the_watcher() {
    let watching = every(Duration::from_secs(3_600), || {});
    assert!(watching.writing());
    assert!(watching.ours());
    assert!(watching.close());
}
