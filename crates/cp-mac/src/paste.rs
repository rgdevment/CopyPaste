use cp_core::destination::Destination;
use cp_core::paste::{Attempt, Failure, Focus, Next, ORDER, Phase, Route, SETTLE, route_for};
use cp_mac_sys::frontmost;
use cp_mac_sys::keyboard::{self, QWERTY_V};
use cp_mac_sys::keystroke::{self, Keystroke};
use cp_mac_sys::menu;
use cp_mac_sys::permissions::Readiness;
use std::time::{Duration, Instant};

pub const KEYS_LET_GO: Duration = Duration::from_millis(120);
pub const KEYS_LOOKED_AT: Duration = Duration::from_millis(4);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Sent {
        took: std::time::Duration,
        via: Route,
    },
    Degraded(Failure),
}

pub fn route_of(ready: &Readiness) -> Option<Route> {
    route_for(ready.can_post, ready.accessibility)
}

pub fn around_protected_input(route: Route, accessibility: bool) -> Option<Route> {
    match route {
        Route::Menu => Some(Route::Menu),
        Route::Keystroke => accessibility.then_some(Route::Menu),
    }
}

pub struct Paster {
    keys: Keystroke,
}

impl Paster {
    pub fn new() -> Option<Self> {
        Some(Self {
            keys: Keystroke::new()?,
        })
    }

    pub fn keycode(&self) -> u16 {
        keyboard::keycode_with_command('v').unwrap_or(QWERTY_V)
    }

    pub fn start_via(
        &self,
        route: Option<Route>,
        target: Destination,
        hide_panel: impl FnOnce(),
    ) -> Result<Pasting, Outcome> {
        let Some(route) = route else {
            return Err(Outcome::Degraded(Failure::SendDenied));
        };
        hide_panel();
        Ok(Pasting::new(route, target, Instant::now()))
    }

    pub fn advance(&self, pasting: &mut Pasting) -> Advance {
        let pid = pasting.target.pid;
        let front = frontmost::frontmost().map(|(front, _)| front);
        let step = pasting.next(
            frontmost::is_alive(pid),
            focus_from(front, pid),
            keystroke::modifiers_still_held(),
            Instant::now(),
        );
        match step {
            Step::Raise => {
                frontmost::bring_to_front(pid);
                Advance::After(SETTLE)
            }
            Step::Wait(pause) => Advance::After(pause),
            Step::Stop(why) => Advance::Done(Outcome::Degraded(why)),
            Step::Send => Advance::Done(self.send(pasting)),
        }
    }

    pub fn paste_via(
        &self,
        route: Option<Route>,
        target: &Destination,
        hide_panel: impl FnOnce(),
    ) -> Outcome {
        let mut pasting = match self.start_via(route, target.clone(), hide_panel) {
            Ok(pasting) => pasting,
            Err(outcome) => return outcome,
        };
        loop {
            match self.advance(&mut pasting) {
                Advance::Done(outcome) => return outcome,
                Advance::After(pause) => std::thread::sleep(pause),
            }
        }
    }

    fn send(&self, pasting: &mut Pasting) -> Outcome {
        let Some(route) = pasting.route_now(&Readiness::probe()) else {
            return Outcome::Degraded(Failure::InputProtected);
        };
        pasting.attempt.sending();
        let sent = match route {
            Route::Keystroke => self.keys.command(self.keycode()),
            Route::Menu => menu::press_paste(pasting.target.pid).is_ok(),
        };
        if sent {
            Outcome::Sent {
                took: pasting.started.elapsed(),
                via: route,
            }
        } else {
            Outcome::Degraded(Failure::SendDenied)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Raise,
    Wait(Duration),
    Send,
    Stop(Failure),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Advance {
    After(Duration),
    Done(Outcome),
}

#[derive(Debug)]
pub struct Pasting {
    route: Route,
    target: Destination,
    started: Instant,
    attempt: Attempt,
    raised: bool,
    keys_since: Option<Instant>,
}

impl Pasting {
    pub fn new(route: Route, target: Destination, now: Instant) -> Self {
        Self {
            route,
            target,
            started: now,
            attempt: Attempt::default(),
            raised: false,
            keys_since: None,
        }
    }

    pub fn next(&mut self, alive: bool, focus: Focus, keys_held: bool, now: Instant) -> Step {
        if !alive {
            return Step::Stop(Failure::TargetGone);
        }
        if focus != Focus::OnTarget {
            if self.raised && self.attempt.on_failure(Failure::NotForeground) != Next::Retry {
                return Step::Stop(Failure::ForegroundTimeout);
            }
            self.raised = true;
            return Step::Raise;
        }
        let since = *self.keys_since.get_or_insert(now);
        if keys_held && now.duration_since(since) < KEYS_LET_GO {
            return Step::Wait(KEYS_LOOKED_AT);
        }
        Step::Send
    }

    pub fn route_now(&self, ready: &Readiness) -> Option<Route> {
        if ready.secure_input {
            around_protected_input(self.route, ready.accessibility)
        } else {
            Some(self.route)
        }
    }
}

pub fn focus_from(front: Option<i32>, target: i32) -> Focus {
    match front {
        Some(pid) if pid == target => Focus::OnTarget,
        Some(_) => Focus::Elsewhere,
        None => Focus::Unknown,
    }
}

pub fn phases() -> &'static [Phase] {
    ORDER
}

#[cfg(test)]
#[path = "paste_test.rs"]
mod tests;
