use super::*;

#[test]
fn the_period_keeps_up_with_a_person_without_spinning() {
    assert!(
        EVERY <= Duration::from_millis(250),
        "a copy must not take long to be noticed"
    );
    assert!(
        EVERY >= Duration::from_millis(16),
        "nor should it poll faster than the screen"
    );
}

#[test]
fn stopping_is_what_drop_does_and_it_waits_for_the_thread() {
    let watching = Watching::every(Duration::from_millis(5), || {});
    assert!(watching.close(), "the thread ended in time");
    assert!(
        watching.close(),
        "and closing what is already closed is no error"
    );
    drop(watching);
}

#[test]
fn a_long_period_does_not_make_stopping_take_that_long() {
    let watching = Watching::every(Duration::from_secs(3600), || {});
    let started = std::time::Instant::now();
    drop(watching);
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "closing the application must not wait for the next poll"
    );
}

#[test]
fn the_pasteboard_counts_one_per_copy_so_a_miss_can_be_told() {
    let watching = Watching::every(Duration::from_secs(3600), || {});
    assert_eq!(watching.missed(), Some(0));
}

#[test]
fn both_ends_of_a_write_of_ours_are_told_to_the_watcher() {
    let watching = Watching::every(Duration::from_secs(3600), || {});
    assert!(watching.writing());
    assert!(watching.ours());
}
