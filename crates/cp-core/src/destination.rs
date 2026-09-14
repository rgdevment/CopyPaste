#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Destination {
    pub pid: i32,
    pub bundle_id: Option<String>,
}

#[derive(Debug, Default)]
pub struct Tracker {
    ours: i32,
    last_foreign: Option<Destination>,
}

impl Tracker {
    pub fn new(our_pid: i32) -> Self {
        Self {
            ours: our_pid,
            last_foreign: None,
        }
    }

    pub fn saw(&mut self, pid: i32, bundle_id: Option<&str>) {
        if pid == self.ours {
            return;
        }
        self.last_foreign = Some(Destination {
            pid,
            bundle_id: bundle_id.map(str::to_owned),
        });
    }

    pub fn destination(&self) -> Option<&Destination> {
        self.last_foreign.as_ref()
    }

    pub fn gone(&mut self, pid: i32) {
        if self.last_foreign.as_ref().is_some_and(|one| one.pid == pid) {
            self.last_foreign = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn we_are_never_our_own_destination() {
        let mut tracker = Tracker::new(100);
        tracker.saw(100, Some("dev.rgdevment.copypaste"));
        assert!(tracker.destination().is_none());
    }

    #[test]
    fn hiding_the_panel_does_not_lose_the_target() {
        let mut tracker = Tracker::new(100);
        tracker.saw(200, Some("com.apple.TextEdit"));
        tracker.saw(100, Some("dev.rgdevment.copypaste"));
        assert_eq!(
            tracker.destination().map(|one| one.pid),
            Some(200),
            "el destino sigue siendo quien estaba antes del panel"
        );
    }

    #[test]
    fn the_newest_foreign_application_wins() {
        let mut tracker = Tracker::new(100);
        tracker.saw(200, Some("com.apple.TextEdit"));
        tracker.saw(300, Some("com.apple.Safari"));
        assert_eq!(tracker.destination().map(|one| one.pid), Some(300));
    }

    #[test]
    fn two_windows_of_the_same_application_are_told_apart_by_pid() {
        let mut tracker = Tracker::new(100);
        tracker.saw(200, Some("com.apple.Terminal"));
        tracker.saw(201, Some("com.apple.Terminal"));
        let destination = tracker.destination().expect("hay destino");
        assert_eq!(destination.pid, 201, "la instancia concreta, no el bundle");
        assert_eq!(destination.bundle_id.as_deref(), Some("com.apple.Terminal"));
    }

    #[test]
    fn an_application_without_a_bundle_is_still_a_destination() {
        let mut tracker = Tracker::new(100);
        tracker.saw(200, None);
        assert_eq!(tracker.destination().map(|one| one.pid), Some(200));
    }

    #[test]
    fn a_closed_target_stops_being_one() {
        let mut tracker = Tracker::new(100);
        tracker.saw(200, Some("com.apple.TextEdit"));
        tracker.gone(200);
        assert!(tracker.destination().is_none());
    }

    #[test]
    fn another_application_closing_changes_nothing() {
        let mut tracker = Tracker::new(100);
        tracker.saw(200, Some("com.apple.TextEdit"));
        tracker.gone(999);
        assert_eq!(tracker.destination().map(|one| one.pid), Some(200));
    }

    #[test]
    fn nothing_seen_means_nowhere_to_paste() {
        let tracker = Tracker::new(100);
        assert!(tracker.destination().is_none());
    }
}
