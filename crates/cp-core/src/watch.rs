#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seen {
    Nothing,
    Ours,
    Fresh { skipped: Option<u64> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cadence {
    OnePerCopy,
    Opaque,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Retry {
    pub attempts: u8,
    pub pause: std::time::Duration,
}

pub const RETRY: Retry = Retry {
    attempts: 5,
    pause: std::time::Duration::from_millis(150),
};

const _: () = assert!(RETRY.attempts >= 2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Retried<T> {
    Done(T),
    Superseded,
    Exhausted,
}

pub fn insist<T>(
    retry: Retry,
    count: impl Fn() -> i64,
    mut attempt: impl FnMut() -> Option<T>,
    pause: impl Fn(std::time::Duration),
) -> Retried<T> {
    let started_at = count();
    for done in 0..retry.attempts.max(1) {
        if done > 0 {
            pause(retry.pause);
            if count() != started_at {
                return Retried::Superseded;
            }
        }
        if let Some(got) = attempt() {
            return Retried::Done(got);
        }
    }
    Retried::Exhausted
}

#[derive(Debug)]
pub struct Watcher {
    last: Option<i64>,
    ours: Option<i64>,
    missed: u64,
    cadence: Cadence,
}

impl Watcher {
    pub fn new(cadence: Cadence) -> Self {
        Self {
            last: None,
            ours: None,
            missed: 0,
            cadence,
        }
    }

    pub fn wrote(&mut self, count: i64) {
        self.ours = Some(count);
    }

    pub fn tick(&mut self, count: i64) -> Seen {
        let Some(last) = self.last else {
            self.last = Some(count);
            return Seen::Nothing;
        };
        if count == last {
            return Seen::Nothing;
        }
        self.last = Some(count);
        let ours = self.ours.take();
        if ours == Some(count) {
            return Seen::Ours;
        }
        let skipped = match self.cadence {
            Cadence::OnePerCopy => {
                let skipped = count.saturating_sub(last).saturating_sub(1).max(0) as u64;
                self.missed = self.missed.saturating_add(skipped);
                Some(skipped)
            }
            Cadence::Opaque => None,
        };
        Seen::Fresh { skipped }
    }

    pub fn missed(&self) -> Option<u64> {
        match self.cadence {
            Cadence::OnePerCopy => Some(self.missed),
            Cadence::Opaque => None,
        }
    }

    pub fn cadence(&self) -> Cadence {
        self.cadence
    }
}

#[cfg(test)]
mod insisting {
    use super::*;
    use std::cell::Cell;
    use std::time::Duration;

    fn quick() -> Retry {
        Retry {
            attempts: 4,
            pause: Duration::from_millis(7),
        }
    }

    #[test]
    fn a_source_that_answers_the_first_time_is_not_asked_twice() {
        let asked = Cell::new(0);
        let paused = Cell::new(0);
        let got = insist(
            quick(),
            || 1,
            || {
                asked.set(asked.get() + 1);
                Some("hola")
            },
            |_| paused.set(paused.get() + 1),
        );
        assert_eq!(got, Retried::Done("hola"));
        assert_eq!((asked.get(), paused.get()), (1, 0));
    }

    #[test]
    fn a_source_that_takes_a_while_is_asked_again_after_a_pause() {
        let asked = Cell::new(0);
        let pauses = std::cell::RefCell::new(Vec::new());
        let got = insist(
            quick(),
            || 1,
            || {
                asked.set(asked.get() + 1);
                (asked.get() == 3).then_some("al fin")
            },
            |how_long| pauses.borrow_mut().push(how_long),
        );
        assert_eq!(got, Retried::Done("al fin"));
        assert_eq!(asked.get(), 3);
        assert_eq!(
            *pauses.borrow(),
            vec![Duration::from_millis(7); 2],
            "una pausa entre cada dos intentos, ninguna antes del primero"
        );
    }

    #[test]
    fn a_source_that_never_answers_is_given_up_on_after_the_attempts() {
        let asked = Cell::new(0);
        let got: Retried<()> = insist(
            quick(),
            || 1,
            || {
                asked.set(asked.get() + 1);
                None
            },
            |_| {},
        );
        assert_eq!(got, Retried::Exhausted);
        assert_eq!(asked.get(), 4, "exactamente los intentos de la política");
    }

    #[test]
    fn a_newer_copy_while_waiting_wins_and_the_old_one_is_dropped() {
        let asked = Cell::new(0);
        let counter = Cell::new(10);
        let got: Retried<()> = insist(
            quick(),
            || counter.get(),
            || {
                asked.set(asked.get() + 1);
                None
            },
            |_| counter.set(11),
        );
        assert_eq!(got, Retried::Superseded);
        assert_eq!(
            asked.get(),
            1,
            "lo copiado después lo verá el vigilante; no se lee dos veces"
        );
    }

    #[test]
    fn one_attempt_is_one_ask_and_no_pause() {
        let asked = Cell::new(0);
        let paused = Cell::new(0);
        let policy = Retry {
            attempts: 1,
            pause: Duration::from_millis(7),
        };
        let got: Retried<()> = insist(
            policy,
            || 1,
            || {
                asked.set(asked.get() + 1);
                None
            },
            |_| paused.set(paused.get() + 1),
        );
        assert_eq!(got, Retried::Exhausted);
        assert_eq!((asked.get(), paused.get()), (1, 0));
    }

    #[test]
    fn zero_attempts_still_asks_once() {
        let asked = Cell::new(0);
        let policy = Retry {
            attempts: 0,
            pause: Duration::ZERO,
        };
        let got = insist(
            policy,
            || 1,
            || {
                asked.set(asked.get() + 1);
                Some(())
            },
            |_| {},
        );
        assert_eq!(got, Retried::Done(()));
        assert_eq!(asked.get(), 1);
    }

    #[test]
    fn the_shipped_policy_waits_less_than_a_person_notices() {
        let worst = RETRY.pause.as_millis() * u128::from(RETRY.attempts - 1);
        assert!(
            worst <= 1_000,
            "{worst} ms de pausas acumuladas es demasiado"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mac() -> Watcher {
        Watcher::new(Cadence::OnePerCopy)
    }

    #[test]
    fn the_counter_at_its_limits_does_not_overflow() {
        let mut watcher = mac();
        watcher.tick(i64::MIN);
        let seen = watcher.tick(i64::MAX);
        assert!(matches!(seen, Seen::Fresh { .. }));

        let mut other = mac();
        other.tick(i64::MAX);
        assert_eq!(other.tick(i64::MAX), Seen::Nothing);
        assert!(matches!(
            other.tick(i64::MIN),
            Seen::Fresh { skipped: Some(0) }
        ));
    }

    #[test]
    fn repeated_giant_jumps_do_not_wrap_the_counter_of_losses() {
        let mut watcher = mac();
        watcher.tick(0);
        for step in 1..=4 {
            watcher.tick(i64::MAX / 4 * step);
        }
        assert!(watcher.missed() > Some(0), "algo se perdió, y se sabe");
    }

    #[test]
    fn a_negative_counter_is_handled_like_any_other() {
        let mut watcher = mac();
        watcher.tick(-100);
        assert_eq!(watcher.tick(-99), Seen::Fresh { skipped: Some(0) });
        assert_eq!(watcher.tick(-95), Seen::Fresh { skipped: Some(3) });
    }

    #[test]
    fn the_first_look_only_sets_the_mark() {
        let mut watcher = mac();
        assert_eq!(watcher.tick(42), Seen::Nothing);
        assert_eq!(watcher.missed(), Some(0));
    }

    #[test]
    fn a_still_counter_is_not_an_event() {
        let mut watcher = mac();
        watcher.tick(7);
        assert_eq!(watcher.tick(7), Seen::Nothing);
    }

    #[test]
    fn one_step_is_one_copy_with_nothing_lost() {
        let mut watcher = mac();
        watcher.tick(7);
        assert_eq!(watcher.tick(8), Seen::Fresh { skipped: Some(0) });
        assert_eq!(watcher.missed(), Some(0));
    }

    #[test]
    fn a_burst_is_counted_not_ignored() {
        let mut watcher = mac();
        watcher.tick(10);
        assert_eq!(watcher.tick(14), Seen::Fresh { skipped: Some(3) });
        assert_eq!(
            watcher.missed(),
            Some(3),
            "tres copias ocurrieron y se perdieron"
        );
    }

    #[test]
    fn our_own_write_is_discarded_exactly() {
        let mut watcher = mac();
        watcher.tick(20);
        watcher.wrote(21);
        assert_eq!(watcher.tick(21), Seen::Ours);
        assert_eq!(watcher.missed(), Some(0));
    }

    #[test]
    fn a_real_copy_right_after_ours_is_not_swallowed() {
        let mut watcher = mac();
        watcher.tick(20);
        watcher.wrote(21);
        assert_eq!(watcher.tick(21), Seen::Ours);
        assert_eq!(
            watcher.tick(22),
            Seen::Fresh { skipped: Some(0) },
            "la ventana ciega de la 2.x se comía esta"
        );
    }

    #[test]
    fn a_counter_that_goes_backwards_is_a_new_session_not_a_burst() {
        let mut watcher = mac();
        watcher.tick(5000);
        assert_eq!(watcher.tick(3), Seen::Fresh { skipped: Some(0) });
        assert_eq!(watcher.missed(), Some(0));
    }
}

#[cfg(test)]
mod windows_counter {
    use super::*;

    fn windows() -> Watcher {
        Watcher::new(Cadence::Opaque)
    }

    #[test]
    fn one_ordinary_copy_is_not_a_burst_of_four() {
        let mut watcher = windows();
        watcher.tick(51);
        assert_eq!(watcher.tick(56), Seen::Fresh { skipped: None });
        assert_eq!(watcher.missed(), None, "no se sabe, y se dice");

        let mut as_if_mac = Watcher::new(Cadence::OnePerCopy);
        as_if_mac.tick(51);
        assert_eq!(
            as_if_mac.tick(56),
            Seen::Fresh { skipped: Some(4) },
            "la cadencia equivocada inventa cuatro copias perdidas"
        );
    }

    #[test]
    fn every_measured_jump_is_a_single_copy() {
        for jump in [5, 9, 11, 12] {
            let mut watcher = windows();
            watcher.tick(1000);
            assert_eq!(
                watcher.tick(1000 + jump),
                Seen::Fresh { skipped: None },
                "un salto de {jump} sigue siendo una copia"
            );
        }
    }

    #[test]
    fn a_counter_that_jumps_backwards_still_reports_each_poll() {
        let mut watcher = windows();
        watcher.tick(1000);
        assert_eq!(watcher.tick(0), Seen::Fresh { skipped: None });
        assert_eq!(watcher.tick(1005), Seen::Fresh { skipped: None });
        assert_eq!(watcher.missed(), None);
    }

    #[test]
    fn our_own_write_is_still_discarded_exactly() {
        let mut watcher = windows();
        watcher.tick(100);
        watcher.wrote(112);
        assert_eq!(watcher.tick(112), Seen::Ours);
        assert_eq!(watcher.tick(124), Seen::Fresh { skipped: None });
    }

    #[test]
    fn a_still_counter_is_still_not_an_event() {
        let mut watcher = windows();
        watcher.tick(7);
        assert_eq!(watcher.tick(7), Seen::Nothing);
    }

    #[test]
    fn the_watcher_says_which_counter_it_is_reading() {
        assert_eq!(windows().cadence(), Cadence::Opaque);
        assert_eq!(
            Watcher::new(Cadence::OnePerCopy).cadence(),
            Cadence::OnePerCopy
        );
    }
}

#[cfg(test)]
mod properties {
    use super::*;
    use proptest::prelude::*;

    fn ticks() -> impl Strategy<Value = Vec<i64>> {
        prop::collection::vec(0i64..40, 1..60)
    }

    proptest! {
        #[test]
        fn every_change_is_either_seen_or_counted(seq in ticks()) {
            let mut watcher = Watcher::new(Cadence::OnePerCopy);
            let mut emitted: u64 = 0;
            for count in &seq {
                if let Seen::Fresh { .. } = watcher.tick(*count) {
                    emitted += 1;
                }
            }

            let mut climbed: i64 = 0;
            let mut restarts: u64 = 0;
            for pair in seq.windows(2) {
                if pair[1] > pair[0] {
                    climbed += pair[1] - pair[0];
                } else if pair[1] < pair[0] {
                    restarts += 1;
                }
            }

            prop_assert_eq!(climbed as u64 + restarts, emitted + watcher.missed().unwrap());
        }

        #[test]
        fn a_still_counter_never_fires(
            start in 0i64..1000,
            repeats in 1usize..20,
            opaque in any::<bool>(),
        ) {
            let cadence = if opaque { Cadence::Opaque } else { Cadence::OnePerCopy };
            let mut watcher = Watcher::new(cadence);
            watcher.tick(start);
            for _ in 0..repeats {
                prop_assert_eq!(watcher.tick(start), Seen::Nothing);
            }
            prop_assert_eq!(watcher.missed().unwrap_or(0), 0);
        }

        #[test]
        fn what_was_missed_never_shrinks(seq in ticks()) {
            let mut watcher = Watcher::new(Cadence::OnePerCopy);
            let mut floor = Some(0);
            for count in seq {
                watcher.tick(count);
                prop_assert!(watcher.missed() >= floor);
                floor = watcher.missed();
            }
        }

        #[test]
        fn an_opaque_counter_never_invents_a_number(
            seq in prop::collection::vec(0i64..100_000, 1..40),
        ) {
            let mut watcher = Watcher::new(Cadence::Opaque);
            for count in seq {
                if let Seen::Fresh { skipped } = watcher.tick(count) {
                    prop_assert_eq!(skipped, None);
                }
                prop_assert_eq!(watcher.missed(), None);
            }
        }

        #[test]
        fn the_cadence_changes_the_count_never_the_event(seq in ticks()) {
            let mut counted = Watcher::new(Cadence::OnePerCopy);
            let mut opaque = Watcher::new(Cadence::Opaque);
            for count in seq {
                let one = counted.tick(count);
                let other = opaque.tick(count);
                prop_assert_eq!(
                    matches!(one, Seen::Fresh { .. }),
                    matches!(other, Seen::Fresh { .. })
                );
                prop_assert_eq!(one == Seen::Ours, other == Seen::Ours);
                prop_assert_eq!(one == Seen::Nothing, other == Seen::Nothing);
            }
        }

        #[test]
        fn only_the_exact_registered_count_is_ours(start in 0i64..1000, gap in 1i64..10) {
            let mut watcher = Watcher::new(Cadence::OnePerCopy);
            watcher.tick(start);
            watcher.wrote(start + gap);
            let landed = watcher.tick(start + gap);
            prop_assert_eq!(landed, Seen::Ours);
            prop_assert_eq!(watcher.tick(start + gap), Seen::Nothing);
            prop_assert_eq!(watcher.tick(start + gap + 1), Seen::Fresh { skipped: Some(0) });
        }

        #[test]
        fn a_write_that_never_lands_swallows_nothing(start in 0i64..1000) {
            let mut watcher = Watcher::new(Cadence::OnePerCopy);
            watcher.tick(start);
            watcher.wrote(start + 99);
            prop_assert_eq!(watcher.tick(start + 1), Seen::Fresh { skipped: Some(0) });
            prop_assert_ne!(watcher.tick(start + 99), Seen::Ours);
        }
    }
}
