use cp_core::paste::{Attempt, Failure, Focus, Next, ORDER, Phase, SETTLE};
use cp_win_sys::frontmost::{self, Attached, Target};
use cp_win_sys::keystroke;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::HWND;

const MODIFIERS_GO: Duration = Duration::from_millis(120);
const TARGET_ANSWERS_MS: u32 = 200;
const KEYS_LOOKED_AT: Duration = Duration::from_millis(4);
const FOCUS_LANDS: Duration = Duration::from_millis(150);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Sent { took: Duration },
    Degraded(Failure),
}

pub fn phases() -> &'static [Phase] {
    ORDER
}

pub fn paste_into(target: &Target, hide_panel: impl FnOnce()) -> Outcome {
    let mut pasting = match Pasting::start(*target, hide_panel) {
        Ok(pasting) => pasting,
        Err(outcome) => return outcome,
    };
    loop {
        match pasting.advance() {
            Advance::After(pause) => std::thread::sleep(pause),
            Advance::Done(outcome) => return outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Advance {
    After(Duration),
    Done(Outcome),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Forward,
    Focus,
    Keys,
}

pub struct Pasting {
    target: Target,
    started: Instant,
    attempt: Attempt,
    stage: Stage,
    inner: Option<HWND>,
    fronted: Option<Instant>,
    keys_since: Option<Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hold {
    Go,
    Wait,
    Stop,
}

impl Pasting {
    pub fn start(target: Target, hide_panel: impl FnOnce()) -> Result<Self, Outcome> {
        let started = Instant::now();
        hide_panel();
        if !frontmost::is_alive(target.window) {
            return Err(Outcome::Degraded(Failure::TargetGone));
        }
        if frontmost::out_of_reach(target.window) {
            return Err(Outcome::Degraded(Failure::TargetElevated));
        }
        if focus_of(&target) != Focus::OnTarget {
            frontmost::bring_forward(target.window);
        }
        Ok(Self {
            target,
            started,
            attempt: Attempt::default(),
            stage: Stage::Forward,
            inner: None,
            fronted: None,
            keys_since: None,
        })
    }

    pub fn advance(&mut self) -> Advance {
        match self.stage {
            Stage::Forward => self.forward(),
            Stage::Focus => self.focus(),
            Stage::Keys => self.keys(),
        }
    }

    fn forward(&mut self) -> Advance {
        if focus_of(&self.target) != Focus::OnTarget {
            if !frontmost::is_alive(self.target.window) {
                return Advance::Done(Outcome::Degraded(Failure::TargetGone));
            }
            if self.attempt.on_failure(Failure::NotForeground) != Next::Retry {
                return Advance::Done(Outcome::Degraded(Failure::ForegroundTimeout));
            }
            frontmost::bring_forward(self.target.window);
            return Advance::After(SETTLE);
        }
        self.fronted.get_or_insert_with(Instant::now);
        self.inner = self.target.focus.filter(|inner| {
            frontmost::is_alive(*inner) && frontmost::answers(*inner, TARGET_ANSWERS_MS)
        });
        self.stage = Stage::Focus;
        self.focus()
    }

    fn focus(&mut self) -> Advance {
        if let Some(waiting) = self.still_in_front() {
            return waiting;
        }
        if let Some(inner) = self.inner
            && let Some(attached) = Attached::to(self.target.thread)
            && !attached.focus_on(inner)
        {
            drop(attached);
            if self.attempt.on_failure(Failure::NoKeyboardFocus) != Next::Retry {
                return Advance::Done(Outcome::Degraded(Failure::NoKeyboardFocus));
            }
            return Advance::After(SETTLE);
        }
        self.stage = Stage::Keys;
        self.keys()
    }

    fn keys(&mut self) -> Advance {
        if let Some(waiting) = self.still_in_front() {
            return waiting;
        }
        let since = *self.keys_since.get_or_insert_with(Instant::now);
        let fronted = self.fronted.unwrap_or(self.started);
        if let Some(pause) = focus_still_landing(fronted.elapsed()) {
            return Advance::After(pause);
        }
        if keystroke::modifiers_still_held() && since.elapsed() < MODIFIERS_GO {
            return Advance::After(KEYS_LOOKED_AT);
        }
        self.attempt.sending();
        let attached = Attached::to(self.target.thread);
        let sent = keystroke::send(&keystroke::paste_batch());
        drop(attached);
        Advance::Done(if sent {
            Outcome::Sent {
                took: self.started.elapsed(),
            }
        } else {
            Outcome::Degraded(Failure::SendDenied)
        })
    }

    fn still_in_front(&mut self) -> Option<Advance> {
        match hold_of(focus_of(&self.target)) {
            Hold::Go => None,
            Hold::Stop => Some(Advance::Done(Outcome::Degraded(Failure::NotForeground))),
            Hold::Wait if self.attempt.on_failure(Failure::NotForeground) == Next::Retry => {
                Some(Advance::After(SETTLE))
            }
            Hold::Wait => Some(Advance::Done(Outcome::Degraded(Failure::ForegroundTimeout))),
        }
    }
}

fn focus_still_landing(since_fronted: Duration) -> Option<Duration> {
    (since_fronted < FOCUS_LANDS).then(|| FOCUS_LANDS - since_fronted)
}

fn hold_of(focus: Focus) -> Hold {
    match focus {
        Focus::OnTarget => Hold::Go,
        Focus::Unknown => Hold::Wait,
        Focus::Elsewhere => Hold::Stop,
    }
}

fn focus_of(target: &Target) -> Focus {
    match frontmost::foreground() {
        Some(window) if window == target.window => Focus::OnTarget,
        Some(_) => Focus::Elsewhere,
        None => Focus::Unknown,
    }
}

#[cfg(test)]
#[path = "paste_test.rs"]
mod tests;
