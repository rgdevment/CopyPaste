use cp_core::destination::Destination;
use cp_core::paste::{Attempt, Failure, Focus, Next, ORDER, Phase, Route, SETTLE, route_for};
use cp_mac_sys::frontmost;
use cp_mac_sys::keyboard::{self, QWERTY_V};
use cp_mac_sys::keystroke::{self, Keystroke};
use cp_mac_sys::permissions::Readiness;
use cp_mac_sys::{menu, runloop};

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

    pub fn paste_into(&self, target: &Destination, hide_panel: impl FnOnce()) -> Outcome {
        self.paste_via(route_of(&Readiness::probe()), target, hide_panel)
    }

    pub fn paste_via(
        &self,
        route: Option<Route>,
        target: &Destination,
        hide_panel: impl FnOnce(),
    ) -> Outcome {
        let started = std::time::Instant::now();
        let mut attempt = Attempt::default();

        let Some(route) = route else {
            return Outcome::Degraded(Failure::SendDenied);
        };

        hide_panel();

        if !frontmost::is_alive(target.pid) {
            return Outcome::Degraded(Failure::TargetGone);
        }

        if self.focus_of(target) != Focus::OnTarget {
            frontmost::bring_to_front(target.pid);
        }

        while self.focus_of(target) != Focus::OnTarget {
            if !frontmost::is_alive(target.pid) {
                return Outcome::Degraded(Failure::TargetGone);
            }
            if attempt.on_failure(Failure::NotForeground) != Next::Retry {
                return Outcome::Degraded(Failure::NotForeground);
            }
            frontmost::bring_to_front(target.pid);
            self.wait(SETTLE.as_secs_f64());
        }

        let waiting = std::time::Instant::now();
        while keystroke::modifiers_still_held() && waiting.elapsed().as_millis() < 120 {
            self.wait(0.004);
        }

        attempt.sending();
        let sent = match route {
            Route::Keystroke => self.keys.command(self.keycode()),
            Route::Menu => menu::press_paste(target.pid).is_ok(),
        };
        if sent {
            Outcome::Sent {
                took: started.elapsed(),
                via: route,
            }
        } else {
            Outcome::Degraded(Failure::SendDenied)
        }
    }

    fn wait(&self, seconds: f64) {
        runloop::pump(seconds);
    }

    fn focus_of(&self, target: &Destination) -> Focus {
        match frontmost::frontmost() {
            Some((pid, _)) if pid == target.pid => Focus::OnTarget,
            Some(_) => Focus::Elsewhere,
            None => Focus::Unknown,
        }
    }
}

pub fn phases() -> &'static [Phase] {
    ORDER
}
