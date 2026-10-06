use super::*;

#[test]
fn only_a_successful_answer_of_one_means_translated() {
    assert!(is_translated(0, 1));
    assert!(!is_translated(0, 0));
    assert!(
        !is_translated(-1, 1),
        "an Intel Mac has no such key and answers with an error"
    );
    assert!(!is_translated(-1, 0));
}

#[test]
fn a_test_run_natively_is_not_translated_and_the_call_answers() {
    if cfg!(target_arch = "aarch64") {
        assert!(!translated());
    } else {
        let _ = translated();
    }
}
