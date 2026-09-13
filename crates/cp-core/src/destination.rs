/// Quién recibirá el pegado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Destination {
    pub pid: i32,
    pub bundle_id: Option<String>,
}

/// Recuerda la última aplicación al frente **que no éramos nosotros**.
///
/// Ocultar el panel en macOS deja a la aplicación activa, así que preguntar
/// quién está al frente en ese momento se lee a uno mismo y el pegado vuelve
/// al panel. La 2.x lo resuelve observando las activaciones; aquí basta con
/// recordar, porque el vigilante ya está mirando.
///
/// Guarda el **pid** además del bundle: identificar el destino solo por bundle
/// elige arbitrariamente entre dos ventanas de la misma aplicación, que es el
/// hallazgo 24 del mapa.
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

    /// Se llama con quien esté al frente, tantas veces como se quiera.
    pub fn saw(&mut self, pid: i32, bundle_id: Option<&str>) {
        if pid == self.ours {
            return;
        }
        self.last_foreign = Some(Destination {
            pid,
            bundle_id: bundle_id.map(str::to_owned),
        });
    }

    /// El destino del pegado: el último que no fuimos nosotros.
    pub fn destination(&self) -> Option<&Destination> {
        self.last_foreign.as_ref()
    }

    /// Cuando el destino se cierra, deja de ser un destino. Recapturar no es
    /// posible una vez que el panel tiene el foco, así que esto se sabe por
    /// el pid y no por la ventana.
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
        // El panel se muestra y nos volvemos nosotros el frente.
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
