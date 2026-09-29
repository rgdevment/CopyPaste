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
