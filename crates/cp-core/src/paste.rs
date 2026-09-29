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
pub enum Route {
    Keystroke,
    Menu,
}

pub fn route_for(can_post_events: bool, can_drive_menus: bool) -> Option<Route> {
    if can_post_events {
        return Some(Route::Keystroke);
    }
    can_drive_menus.then_some(Route::Menu)
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
#[path = "paste_test.rs"]
mod tests;

#[cfg(test)]
#[path = "paste_properties.rs"]
mod properties;
