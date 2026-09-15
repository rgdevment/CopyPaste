use crate::clipboard;
use crate::frontmost;

pub const MEDIUM: u32 = 0x2000;
pub const HIGH: u32 = 0x3000;

const _: () = assert!(MEDIUM < HIGH);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Readiness {
    pub reaches_window_station: bool,
    pub integrity: Option<u32>,
}

impl Readiness {
    pub fn probe() -> Self {
        Self {
            reaches_window_station: clipboard::sequence().is_some(),
            integrity: frontmost::integrity_of(std::process::id()),
        }
    }

    pub fn can_watch(&self) -> bool {
        self.reaches_window_station
    }

    pub fn can_paste_into(&self, target: u32) -> bool {
        self.integrity.is_some_and(|ours| ours >= target)
    }

    pub fn is_elevated(&self) -> bool {
        self.integrity.is_some_and(|ours| ours >= HIGH)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_cannot_reach_the_window_station_cannot_watch() {
        let blind = Readiness {
            reaches_window_station: false,
            integrity: Some(MEDIUM),
        };
        assert!(!blind.can_watch(), "sin estacion de ventanas no se vigila");
        let seeing = Readiness {
            reaches_window_station: true,
            integrity: None,
        };
        assert!(
            seeing.can_watch(),
            "y con ella si, aunque el nivel se ignore"
        );
    }

    #[test]
    fn a_target_at_our_own_level_is_reachable() {
        let ready = Readiness {
            reaches_window_station: true,
            integrity: Some(MEDIUM),
        };
        assert!(ready.can_paste_into(MEDIUM));
    }

    #[test]
    fn a_target_above_us_is_not_reachable() {
        let ready = Readiness {
            reaches_window_station: true,
            integrity: Some(MEDIUM),
        };
        assert!(!ready.can_paste_into(HIGH));
        assert!(ready.can_paste_into(MEDIUM));
    }

    #[test]
    fn without_a_level_of_our_own_nothing_is_promised() {
        let blind = Readiness {
            reaches_window_station: true,
            integrity: None,
        };
        assert!(!blind.can_paste_into(MEDIUM));
        assert!(!blind.is_elevated());
    }

    #[test]
    fn a_locked_station_stops_the_watcher_and_nothing_else() {
        let locked = Readiness {
            reaches_window_station: false,
            integrity: Some(MEDIUM),
        };
        assert!(!locked.can_watch());
        assert!(locked.can_paste_into(MEDIUM));
    }

    #[test]
    fn running_elevated_is_told_apart_from_running_normal() {
        assert!(
            Readiness {
                reaches_window_station: true,
                integrity: Some(HIGH),
            }
            .is_elevated()
        );
        assert!(
            !Readiness {
                reaches_window_station: true,
                integrity: Some(MEDIUM),
            }
            .is_elevated()
        );
    }
}
