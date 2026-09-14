#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    WriteClipboard,
    HidePanel,
    BringTargetForward,
    VerifyForeground,
    VerifyKeyboardFocus,
    Send,
}

pub const ORDER: &[Phase] = &[
    Phase::WriteClipboard,
    Phase::HidePanel,
    Phase::BringTargetForward,
    Phase::VerifyForeground,
    Phase::VerifyKeyboardFocus,
    Phase::Send,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    OnTarget,
    Elsewhere,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Failure {
    NotForeground,
    ForegroundTimeout,
    NoKeyboardFocus,
    TargetGone,
    SendDenied,
    TargetElevated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Next {
    Retry,
    Degrade,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Warning {
    SecureInputActive,
}

pub const RACE_RETRIES: u8 = 11;
pub const SETTLE: std::time::Duration = std::time::Duration::from_millis(60);
const _: () = assert!(
    (RACE_RETRIES as u128 + 1) * SETTLE.as_millis() < 1_000,
    "a paste that takes a second is a paste the user already gave up on"
);

#[derive(Debug, Default)]
pub struct Attempt {
    tries: u8,
    sent: bool,
}

impl Attempt {
    pub fn sending(&mut self) {
        self.sent = true;
    }

    pub fn on_failure(&mut self, failure: Failure) -> Next {
        if self.sent {
            return Next::Degrade;
        }
        let retriable = matches!(
            failure,
            Failure::ForegroundTimeout | Failure::NoKeyboardFocus | Failure::NotForeground
        );
        if !retriable || self.tries >= RACE_RETRIES {
            return Next::Degrade;
        }
        self.tries += 1;
        Next::Retry
    }
}

pub fn aborts(focus: Focus) -> bool {
    focus == Focus::Elsewhere
}

pub fn aborts_on(_warning: Warning) -> bool {
    false
}

pub const REQUIRES_FOREGROUND: bool = true;

impl Failure {
    pub fn is_permanent(self) -> bool {
        matches!(self, Failure::TargetElevated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_slow_app_gets_the_dozen_tries_the_2x_needed() {
        let mut attempt = Attempt::default();
        let mut tries = 1;
        while attempt.on_failure(Failure::NotForeground) == Next::Retry {
            tries += 1;
        }
        assert_eq!(tries, 12, "Office y Electron tardan cientos de ms en venir");
    }

    #[test]
    fn an_elevated_target_is_never_retried() {
        let mut attempt = Attempt::default();
        assert_eq!(attempt.on_failure(Failure::TargetElevated), Next::Degrade);
        assert_eq!(attempt.on_failure(Failure::TargetElevated), Next::Degrade);
        assert!(Failure::TargetElevated.is_permanent());
    }

    #[test]
    fn what_a_second_try_could_fix_is_not_permanent() {
        for failure in [
            Failure::NotForeground,
            Failure::ForegroundTimeout,
            Failure::NoKeyboardFocus,
        ] {
            assert!(!failure.is_permanent(), "{failure:?}");
            assert_eq!(Attempt::default().on_failure(failure), Next::Retry);
        }
    }

    #[test]
    fn a_target_that_is_gone_is_not_worth_retrying_either() {
        assert_eq!(
            Attempt::default().on_failure(Failure::TargetGone),
            Next::Degrade
        );
    }

    #[test]
    fn the_clipboard_is_written_before_anything_touches_focus() {
        let write = ORDER.iter().position(|p| *p == Phase::WriteClipboard);
        let hide = ORDER.iter().position(|p| *p == Phase::HidePanel);
        let forward = ORDER.iter().position(|p| *p == Phase::BringTargetForward);
        assert_eq!(write, Some(0));
        assert!(write < hide && write < forward);
    }

    #[test]
    fn focus_is_verified_before_sending_and_never_after() {
        let verify = ORDER
            .iter()
            .position(|p| *p == Phase::VerifyKeyboardFocus)
            .unwrap();
        let send = ORDER.iter().position(|p| *p == Phase::Send).unwrap();
        assert!(verify < send);
        assert_eq!(send, ORDER.len() - 1);
    }

    #[test]
    fn what_is_unknown_gets_pasted() {
        assert!(!aborts(Focus::Unknown));
        assert!(!aborts(Focus::OnTarget));
        assert!(aborts(Focus::Elsewhere));
    }

    #[test]
    fn a_successful_send_is_never_retried() {
        let mut attempt = Attempt::default();
        attempt.sending();
        assert_eq!(
            attempt.on_failure(Failure::ForegroundTimeout),
            Next::Degrade,
            "un pegado doble es peor que ninguno"
        );
    }

    #[test]
    fn races_are_retried_and_then_given_up() {
        let mut attempt = Attempt::default();
        for _ in 0..RACE_RETRIES {
            assert_eq!(attempt.on_failure(Failure::ForegroundTimeout), Next::Retry);
        }
        assert_eq!(
            attempt.on_failure(Failure::NoKeyboardFocus),
            Next::Degrade,
            "el presupuesto se comparte entre las dos carreras"
        );
    }

    #[test]
    fn a_denied_send_is_reported_not_retried() {
        let mut attempt = Attempt::default();
        assert_eq!(
            attempt.on_failure(Failure::SendDenied),
            Next::Degrade,
            "es UIPI o TCC: reintentar no cambia nada"
        );
    }

    #[test]
    fn secure_input_warns_but_never_aborts() {
        assert!(!aborts_on(Warning::SecureInputActive));
    }
}

#[cfg(test)]
mod properties {
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
            prop_assert!(retried <= RACE_RETRIES as usize, "reintentó {retried} veces");
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
}
