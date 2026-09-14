/// Lo que el vigilante ve en un sondeo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seen {
    /// El contador no se movió.
    Nothing,
    /// Se movió por nuestra propia escritura.
    Ours,
    /// Hay contenido nuevo. `skipped` son las copias que ocurrieron entre
    /// este sondeo y el anterior y que ya no se pueden recuperar: el sistema
    /// solo guarda la última. No se ignoran, se cuentan.
    Fresh { skipped: u64 },
}

/// Sondea el contador de secuencia del portapapeles —`changeCount` en macOS,
/// `GetClipboardSequenceNumber` en Windows— y decide qué ha pasado.
///
/// Medido el 12/09/2026: en macOS el contador sube exactamente de uno en uno
/// por escritura, incluso con cinco escrituras separadas por 5 ms, así que un
/// salto mayor que uno significa copias que no se vieron.
#[derive(Debug, Default)]
pub struct Watcher {
    last: Option<i64>,
    ours: Option<i64>,
    missed: u64,
}

impl Watcher {
    /// Registra el contador que dejó nuestra propia escritura, para
    /// descartar exactamente esa y no una ventana de tiempo alrededor.
    pub fn wrote(&mut self, count: i64) {
        self.ours = Some(count);
    }

    pub fn tick(&mut self, count: i64) -> Seen {
        let Some(last) = self.last else {
            // El primer sondeo solo fija el punto de partida: lo que hubiera
            // antes de arrancar no es una copia que hayamos perdido.
            self.last = Some(count);
            return Seen::Nothing;
        };
        if count == last {
            return Seen::Nothing;
        }
        self.last = Some(count);
        if self.ours == Some(count) {
            self.ours = None;
            return Seen::Ours;
        }
        // El contador retrocede al reiniciarse la sesión de ventanas, y eso
        // no es una ráfaga de copias perdidas: el `max` deja ese caso en cero.
        let skipped = count.saturating_sub(last).saturating_sub(1).max(0) as u64;
        self.missed = self.missed.saturating_add(skipped);
        Seen::Fresh { skipped }
    }

    /// Cuántas copias se sabe que ocurrieron y no se pudieron capturar.
    pub fn missed(&self) -> u64 {
        self.missed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_counter_at_its_limits_does_not_overflow() {
        let mut watcher = Watcher::default();
        watcher.tick(i64::MIN);
        // La resta de los dos extremos no cabe en i64.
        let seen = watcher.tick(i64::MAX);
        assert!(matches!(seen, Seen::Fresh { .. }));

        let mut other = Watcher::default();
        other.tick(i64::MAX);
        assert_eq!(other.tick(i64::MAX), Seen::Nothing);
        assert!(matches!(other.tick(i64::MIN), Seen::Fresh { skipped: 0 }));
    }

    #[test]
    fn repeated_giant_jumps_do_not_wrap_the_counter_of_losses() {
        let mut watcher = Watcher::default();
        watcher.tick(0);
        for step in 1..=4 {
            watcher.tick(i64::MAX / 4 * step);
        }
        assert!(watcher.missed() > 0, "algo se perdió, y se sabe");
    }

    #[test]
    fn a_negative_counter_is_handled_like_any_other() {
        let mut watcher = Watcher::default();
        watcher.tick(-100);
        assert_eq!(watcher.tick(-99), Seen::Fresh { skipped: 0 });
        assert_eq!(watcher.tick(-95), Seen::Fresh { skipped: 3 });
    }

    #[test]
    fn the_first_look_only_sets_the_mark() {
        let mut watcher = Watcher::default();
        assert_eq!(watcher.tick(42), Seen::Nothing);
        assert_eq!(watcher.missed(), 0);
    }

    #[test]
    fn a_still_counter_is_not_an_event() {
        let mut watcher = Watcher::default();
        watcher.tick(7);
        assert_eq!(watcher.tick(7), Seen::Nothing);
    }

    #[test]
    fn one_step_is_one_copy_with_nothing_lost() {
        let mut watcher = Watcher::default();
        watcher.tick(7);
        assert_eq!(watcher.tick(8), Seen::Fresh { skipped: 0 });
        assert_eq!(watcher.missed(), 0);
    }

    #[test]
    fn a_burst_is_counted_not_ignored() {
        let mut watcher = Watcher::default();
        watcher.tick(10);
        assert_eq!(watcher.tick(14), Seen::Fresh { skipped: 3 });
        assert_eq!(watcher.missed(), 3, "tres copias ocurrieron y se perdieron");
    }

    #[test]
    fn our_own_write_is_discarded_exactly() {
        let mut watcher = Watcher::default();
        watcher.tick(20);
        watcher.wrote(21);
        assert_eq!(watcher.tick(21), Seen::Ours);
        assert_eq!(watcher.missed(), 0);
    }

    #[test]
    fn a_real_copy_right_after_ours_is_not_swallowed() {
        let mut watcher = Watcher::default();
        watcher.tick(20);
        watcher.wrote(21);
        assert_eq!(watcher.tick(21), Seen::Ours);
        assert_eq!(
            watcher.tick(22),
            Seen::Fresh { skipped: 0 },
            "la ventana ciega de la 2.x se comía esta"
        );
    }

    #[test]
    fn a_counter_that_goes_backwards_is_a_new_session_not_a_burst() {
        let mut watcher = Watcher::default();
        watcher.tick(5000);
        assert_eq!(watcher.tick(3), Seen::Fresh { skipped: 0 });
        assert_eq!(watcher.missed(), 0);
    }
}

#[cfg(test)]
mod properties {
    use super::*;
    use proptest::prelude::*;

    /// Una secuencia de sondeos del contador.
    fn ticks() -> impl Strategy<Value = Vec<i64>> {
        prop::collection::vec(0i64..40, 1..60)
    }

    proptest! {
        /// Nada se pierde ni se inventa. Cada evento emitido corresponde a un
        /// avance del contador o a un retroceso —que es una sesión nueva, no
        /// una ráfaga—, y lo que avanzó sin emitirse está contado como perdido.
        #[test]
        fn every_change_is_either_seen_or_counted(seq in ticks()) {
            let mut watcher = Watcher::default();
            let mut emitted: u64 = 0;
            for count in &seq {
                if let Seen::Fresh { .. } = watcher.tick(*count) {
                    emitted += 1;
                }
            }

            // El primer sondeo solo fija la marca: lo anterior a arrancar no
            // es una copia perdida.
            let mut climbed: i64 = 0;
            let mut restarts: u64 = 0;
            for pair in seq.windows(2) {
                if pair[1] > pair[0] {
                    climbed += pair[1] - pair[0];
                } else if pair[1] < pair[0] {
                    restarts += 1;
                }
            }

            prop_assert_eq!(climbed as u64 + restarts, emitted + watcher.missed());
        }

        /// Un contador que no avanza nunca produce un evento.
        #[test]
        fn a_still_counter_never_fires(start in 0i64..1000, repeats in 1usize..20) {
            let mut watcher = Watcher::default();
            watcher.tick(start);
            for _ in 0..repeats {
                prop_assert_eq!(watcher.tick(start), Seen::Nothing);
            }
            prop_assert_eq!(watcher.missed(), 0);
        }

        /// Lo perdido solo puede crecer.
        #[test]
        fn what_was_missed_never_shrinks(seq in ticks()) {
            let mut watcher = Watcher::default();
            let mut floor = 0;
            for count in seq {
                watcher.tick(count);
                prop_assert!(watcher.missed() >= floor);
                floor = watcher.missed();
            }
        }

        /// Solo el contador exacto que registramos se descarta como nuestro, y
        /// solo una vez.
        #[test]
        fn only_the_exact_registered_count_is_ours(start in 0i64..1000, gap in 1i64..10) {
            let mut watcher = Watcher::default();
            watcher.tick(start);
            watcher.wrote(start + gap);
            let landed = watcher.tick(start + gap);
            prop_assert_eq!(landed, Seen::Ours);
            // El mismo valor otra vez ya no es nuestro: es que alguien copió.
            prop_assert_eq!(watcher.tick(start + gap), Seen::Nothing);
            prop_assert_eq!(watcher.tick(start + gap + 1), Seen::Fresh { skipped: 0 });
        }

        /// Registrar una escritura que nunca llega no puede tragarse una copia
        /// ajena posterior.
        #[test]
        fn a_write_that_never_lands_swallows_nothing(start in 0i64..1000) {
            let mut watcher = Watcher::default();
            watcher.tick(start);
            watcher.wrote(start + 99);
            prop_assert_eq!(watcher.tick(start + 1), Seen::Fresh { skipped: 0 });
        }
    }
}
