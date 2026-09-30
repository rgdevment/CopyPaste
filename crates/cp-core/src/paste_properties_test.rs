use super::*;
use proptest::prelude::*;

fn any_failure() -> impl Strategy<Value = Failure> {
    prop_oneof![
        Just(Failure::ForegroundTimeout),
        Just(Failure::NotForeground),
        Just(Failure::NoKeyboardFocus),
        Just(Failure::TargetGone),
        Just(Failure::SendDenied),
    ]
}

proptest! {
    #[test]
    fn the_retries_are_bounded(failures in prop::collection::vec(any_failure(), 1..40)) {
        let mut attempt = Attempt::default();
        let retried = failures
            .iter()
            .filter(|failure| attempt.on_failure(**failure) == Next::Retry)
            .count();
        prop_assert!(retried <= RACE_RETRIES as usize, "retried {retried} times");
    }

    #[test]
    fn nothing_is_retried_after_a_send(failures in prop::collection::vec(any_failure(), 1..10)) {
        let mut attempt = Attempt::default();
        attempt.sending();
        for failure in failures {
            prop_assert_eq!(attempt.on_failure(failure), Next::Degrade);
        }
    }

    #[test]
    fn a_denial_never_becomes_a_retry(before in prop::collection::vec(any_failure(), 0..3)) {
        let mut attempt = Attempt::default();
        for failure in before {
            attempt.on_failure(failure);
        }
        prop_assert_eq!(attempt.on_failure(Failure::SendDenied), Next::Degrade);
    }

    #[test]
    fn the_order_never_puts_the_send_before_a_check(index in 0usize..ORDER.len()) {
        if ORDER[index] == Phase::Send {
            prop_assert_eq!(index, ORDER.len() - 1);
        }
        if index > 0 {
            prop_assert_ne!(ORDER[index], Phase::WriteClipboard);
        }
    }
}
