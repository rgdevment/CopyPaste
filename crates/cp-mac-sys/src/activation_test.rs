use super::*;

#[test]
fn off_the_main_thread_the_answer_is_that_there_is_none() {
    if MainThreadMarker::new().is_some() {
        return;
    }
    assert_eq!(
        is_ours_up_front(),
        None,
        "asking AppKit from another thread has no answer, and none is not a no"
    );
}

#[test]
fn on_the_main_thread_it_answers_one_way_or_the_other() {
    if MainThreadMarker::new().is_none() {
        return;
    }
    assert!(is_ours_up_front().is_some());
}
