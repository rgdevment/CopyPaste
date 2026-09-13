#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    WriteClipboard,
    HidePanel,
    BringTargetForward,
    VerifyForeground,
    VerifyKeyboardFocus,
    Send,
}

/// El portapapeles se escribe antes que nada que toque el foco: así el peor
/// resultado posible sigue siendo «está en tu portapapeles, pégalo tú».
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
    ForegroundTimeout,
    NoKeyboardFocus,
    TargetGone,
    SendDenied,
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

const RACE_RETRIES: u8 = 2;

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
            Failure::ForegroundTimeout | Failure::NoKeyboardFocus
        );
        if !retriable || self.tries >= RACE_RETRIES {
            return Next::Degrade;
        }
        self.tries += 1;
        Next::Retry
    }
}

/// Regla ganada en campo: lo desconocido pega. Solo una respuesta positiva de
/// que el foco está en otro sitio justifica abortar.
pub fn aborts(focus: Focus) -> bool {
    focus == Focus::Elsewhere
}

/// Secure Input no se traga los eventos sintéticos: lo que rompe es la
/// activación. Abortar dejaría sin pegar justo a quien tiene el flag pegado
/// por una aplicación ajena, que es a quien la comprobación pretendía ayudar.
pub fn aborts_on(_warning: Warning) -> bool {
    false
}

/// Medido el 12/09/2026 en macOS 26.6.2: ni `CGEventPostToPid` ni `AXPress`
/// entregan a una aplicación que no está al frente.
pub const REQUIRES_FOREGROUND: bool = true;

#[cfg(test)]
mod tests {
    use super::*;

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
    fn races_are_retried_twice_and_then_given_up() {
        let mut attempt = Attempt::default();
        assert_eq!(attempt.on_failure(Failure::ForegroundTimeout), Next::Retry);
        assert_eq!(attempt.on_failure(Failure::NoKeyboardFocus), Next::Retry);
        assert_eq!(
            attempt.on_failure(Failure::ForegroundTimeout),
            Next::Degrade
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
            Just(Failure::NoKeyboardFocus),
            Just(Failure::TargetGone),
            Just(Failure::SendDenied),
        ]
    }

    proptest! {
        /// Por muchos fallos que lleguen, el número de reintentos tiene techo.
        #[test]
        fn the_retries_are_bounded(failures in prop::collection::vec(any_failure(), 1..40)) {
            let mut attempt = Attempt::default();
            let retried = failures
                .iter()
                .filter(|failure| attempt.on_failure(**failure) == Next::Retry)
                .count();
            prop_assert!(retried <= RACE_RETRIES as usize, "reintentó {retried} veces");
        }

        /// Después de un envío con éxito no hay nada que reintentar, venga el
        /// fallo que venga: un pegado doble es peor que ninguno.
        #[test]
        fn nothing_is_retried_after_a_send(failures in prop::collection::vec(any_failure(), 1..10)) {
            let mut attempt = Attempt::default();
            attempt.sending();
            for failure in failures {
                prop_assert_eq!(attempt.on_failure(failure), Next::Degrade);
            }
        }

        /// Un envío denegado es UIPI o TCC: reintentar no cambia nada.
        #[test]
        fn a_denial_never_becomes_a_retry(before in prop::collection::vec(any_failure(), 0..3)) {
            let mut attempt = Attempt::default();
            for failure in before {
                attempt.on_failure(failure);
            }
            prop_assert_eq!(attempt.on_failure(Failure::SendDenied), Next::Degrade);
        }

        /// El portapapeles se escribe antes que cualquier fase que toque el
        /// foco, y enviar es siempre lo último.
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
