use cp_core::paste::{Attempt, Failure, Focus, Next, ORDER, Phase};
use cp_win_sys::frontmost::{self, Attached, Target};
use cp_win_sys::keystroke;
use std::time::{Duration, Instant};

const SETTLE: Duration = Duration::from_millis(60);
const MODIFIERS_GO: Duration = Duration::from_millis(120);
const TARGET_ANSWERS_MS: u32 = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Sent { took: Duration },
    Degraded(Failure),
}

pub fn phases() -> &'static [Phase] {
    ORDER
}

pub fn paste_into(target: &Target, hide_panel: impl FnOnce()) -> Outcome {
    let started = Instant::now();
    let mut attempt = Attempt::default();

    hide_panel();

    if !frontmost::is_alive(target.window) {
        return Outcome::Degraded(Failure::TargetGone);
    }
    if frontmost::out_of_reach(target.window) {
        return Outcome::Degraded(Failure::TargetElevated);
    }

    if focus_of(target) != Focus::OnTarget {
        frontmost::bring_forward(target.window);
    }
    while focus_of(target) != Focus::OnTarget {
        if !frontmost::is_alive(target.window) {
            return Outcome::Degraded(Failure::TargetGone);
        }
        if attempt.on_failure(Failure::NotForeground) != Next::Retry {
            return Outcome::Degraded(Failure::NotForeground);
        }
        frontmost::bring_forward(target.window);
        std::thread::sleep(SETTLE);
    }

    let attached = Attached::to(target.thread);
    if let (Some(attached), Some(inner)) = (attached.as_ref(), target.focus)
        && frontmost::is_alive(inner)
        && frontmost::answers(inner, TARGET_ANSWERS_MS)
    {
        attached.focus_on(inner);
    }

    let waiting = Instant::now();
    while keystroke::modifiers_still_held() && waiting.elapsed() < MODIFIERS_GO {
        std::thread::sleep(Duration::from_millis(4));
    }

    attempt.sending();
    let sent = keystroke::send(&keystroke::paste_batch());
    drop(attached);

    if sent {
        Outcome::Sent {
            took: started.elapsed(),
        }
    } else {
        Outcome::Degraded(Failure::SendDenied)
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
mod tests {
    use super::*;

    #[test]
    fn the_clipboard_is_written_before_anything_touches_the_focus() {
        assert_eq!(phases().first(), Some(&Phase::WriteClipboard));
        assert_eq!(phases().last(), Some(&Phase::Send));
    }

    #[test]
    fn the_order_is_the_one_the_core_fixes() {
        assert_eq!(phases(), ORDER);
    }

    #[test]
    fn an_elevated_target_degrades_without_retrying() {
        let mut attempt = Attempt::default();
        assert_eq!(attempt.on_failure(Failure::TargetElevated), Next::Degrade);
    }

    #[test]
    fn the_waits_are_shorter_than_the_paste_they_guard() {
        assert!(SETTLE < MODIFIERS_GO);
        assert!(MODIFIERS_GO < Duration::from_millis(500));
        assert!(u128::from(TARGET_ANSWERS_MS) < MODIFIERS_GO.as_millis() * 2);
    }

    #[test]
    fn a_target_that_is_us_is_on_target() {
        let Some(window) = frontmost::foreground() else {
            return;
        };
        let target = Target {
            window,
            focus: None,
            thread: 0,
        };
        assert_eq!(focus_of(&target), Focus::OnTarget);
    }

    #[test]
    fn a_window_that_is_not_in_front_is_elsewhere() {
        let Some(front) = frontmost::foreground() else {
            return;
        };
        let other = Target {
            window: windows::Win32::Foundation::HWND(std::ptr::dangling_mut()),
            focus: None,
            thread: 0,
        };
        assert_ne!(other.window, front);
        assert_eq!(focus_of(&other), Focus::Elsewhere);
    }
}
