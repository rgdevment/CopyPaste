pub fn sequence(raw: u32) -> Option<i64> {
    (raw != 0).then_some(i64::from(raw))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cp_core::watch::{Cadence, Seen, Watcher};

    #[test]
    fn a_zero_is_not_a_counter() {
        assert_eq!(sequence(0), None);
    }

    #[test]
    fn any_other_value_is_one() {
        assert_eq!(sequence(1), Some(1));
        assert_eq!(sequence(915), Some(915));
        assert_eq!(sequence(u32::MAX), Some(i64::from(u32::MAX)));
    }

    #[test]
    fn the_upper_half_of_the_range_does_not_turn_negative() {
        for raw in [0x8000_0000u32, 0xFFFF_FFFF] {
            let counted = sequence(raw).expect("no es cero");
            assert!(counted > 0, "{raw} salió como {counted}");
        }
    }

    #[test]
    fn locking_the_screen_without_copying_adds_nothing() {
        let mut watcher = Watcher::new(Cadence::Opaque);
        let mut fresh = 0;
        for raw in [915u32, 0, 0, 915] {
            let Some(counted) = sequence(raw) else {
                continue;
            };
            if let Seen::Fresh { .. } = watcher.tick(counted) {
                fresh += 1;
            }
        }
        assert_eq!(fresh, 0, "nadie copió, y nada se registró");
    }

    #[test]
    fn a_copy_made_across_the_lock_is_still_seen() {
        let mut watcher = Watcher::new(Cadence::Opaque);
        let mut fresh = 0;
        for raw in [915u32, 0, 920] {
            let Some(counted) = sequence(raw) else {
                continue;
            };
            if let Seen::Fresh { .. } = watcher.tick(counted) {
                fresh += 1;
            }
        }
        assert_eq!(fresh, 1);
    }
}
