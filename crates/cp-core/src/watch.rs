/// Lo que el vigilante ve en un sondeo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seen {
    /// El contador no se movió.
    Nothing,
    /// Se movió por nuestra propia escritura.
    Ours,
    /// Hay contenido nuevo. `skipped` son las copias que ocurrieron entre este
    /// sondeo y el anterior y que ya no se pueden recuperar: el sistema solo
    /// guarda la última. No se ignoran, se cuentan —y donde el contador no
    /// permite contarlas, se dice que no se sabe en vez de decir cero.
    Fresh { skipped: Option<u64> },
}

/// Qué significa un salto del contador, que no es lo mismo en cada sistema.
///
/// Elegir mal es contar copias perdidas que nunca ocurrieron, así que el
/// vigilante no tiene valor por defecto: quien lo construye lo declara.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cadence {
    /// Una unidad por copia. Medido el 12/09/2026 en macOS 26.6.2: el
    /// `changeCount` sube exactamente de uno en uno por escritura, incluso con
    /// cinco escrituras separadas por 5 ms, así que un salto mayor que uno son
    /// copias que no se vieron.
    OnePerCopy,
    /// Una cantidad que depende de cuántos formatos sintetiza el sistema y de
    /// si quien copió pasó por OLE. Medido el 14/09/2026 en Windows 11 26200:
    /// `GetClipboardSequenceNumber` sube **+5** al escribir `CF_UNICODETEXT`
    /// directamente, **+12** con el mismo texto a través de OLE, **+11** con
    /// `CF_HDROP` y **+9** con una imagen. El salto dice que hubo cambio;
    /// cuántas copias hubo no lo dice.
    Opaque,
}

/// Sondea el contador de secuencia del portapapeles —`changeCount` en macOS,
/// `GetClipboardSequenceNumber` en Windows— y decide qué ha pasado.
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
        let skipped = match self.cadence {
            // El contador retrocede al reiniciarse la sesión de ventanas, y eso
            // no es una ráfaga de copias perdidas: el `max` deja ese caso en cero.
            Cadence::OnePerCopy => {
                let skipped = count.saturating_sub(last).saturating_sub(1).max(0) as u64;
                self.missed = self.missed.saturating_add(skipped);
                Some(skipped)
            }
            Cadence::Opaque => None,
        };
        Seen::Fresh { skipped }
    }

    /// Cuántas copias se sabe que ocurrieron y no se pudieron capturar.
    /// `None` donde el contador de la plataforma no permite saberlo.
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
mod tests {
    use super::*;

    fn mac() -> Watcher {
        Watcher::new(Cadence::OnePerCopy)
    }

    #[test]
    fn the_counter_at_its_limits_does_not_overflow() {
        let mut watcher = mac();
        watcher.tick(i64::MIN);
        // La resta de los dos extremos no cabe en i64.
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

/// El contador de Windows sube una cantidad que no es el número de copias.
/// Medido el 14/09/2026 en Windows 11 26200 con `GetClipboardSequenceNumber`.
#[cfg(test)]
mod windows_counter {
    use super::*;

    fn windows() -> Watcher {
        Watcher::new(Cadence::Opaque)
    }

    /// El salto medido de una copia de texto corriente. Con la cadencia de
    /// macOS, esa copia se anotaría como cuatro pérdidas que nunca existieron.
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

    /// Los cuatro saltos medidos, uno por forma de copiar. Ninguno vale uno, y
    /// entre ellos no hay proporción: por eso el número no se interpreta.
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

    /// El contador vale cero cuando el proceso no alcanza la estación de
    /// ventanas —escritorio seguro, pantalla de bloqueo—. Volver de ahí es un
    /// salto enorme desde cero, y no una ráfaga de mil copias.
    #[test]
    fn coming_back_from_a_locked_desktop_is_one_copy() {
        let mut watcher = windows();
        watcher.tick(1000);
        assert_eq!(watcher.tick(0), Seen::Fresh { skipped: None });
        assert_eq!(watcher.tick(1005), Seen::Fresh { skipped: None });
        assert_eq!(watcher.missed(), None);
    }

    /// Lo que no depende de la cadencia: nuestra propia escritura se descarta
    /// por el valor exacto, y la copia que llega justo después no se traga.
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

    /// El vigilante dice con qué cadencia se construyó: el diagnóstico tiene
    /// que poder distinguir «no hubo pérdidas» de «no se pueden contar».
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
            let mut watcher = Watcher::new(Cadence::OnePerCopy);
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

            prop_assert_eq!(climbed as u64 + restarts, emitted + watcher.missed().unwrap());
        }

        /// Un contador que no avanza nunca produce un evento, se cuente como
        /// se cuente.
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

        /// Lo perdido solo puede crecer.
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

        /// Un contador opaco no inventa una cifra jamás, salte lo que salte.
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

        /// Las dos cadencias ven **los mismos eventos**: lo único que cambia es
        /// si se puede decir cuántas copias se perdieron, nunca si hubo copia.
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

        /// Solo el contador exacto que registramos se descarta como nuestro, y
        /// solo una vez.
        #[test]
        fn only_the_exact_registered_count_is_ours(start in 0i64..1000, gap in 1i64..10) {
            let mut watcher = Watcher::new(Cadence::OnePerCopy);
            watcher.tick(start);
            watcher.wrote(start + gap);
            let landed = watcher.tick(start + gap);
            prop_assert_eq!(landed, Seen::Ours);
            // El mismo valor otra vez ya no es nuestro: es que alguien copió.
            prop_assert_eq!(watcher.tick(start + gap), Seen::Nothing);
            prop_assert_eq!(watcher.tick(start + gap + 1), Seen::Fresh { skipped: Some(0) });
        }

        /// Registrar una escritura que nunca llega no puede tragarse una copia
        /// ajena posterior.
        #[test]
        fn a_write_that_never_lands_swallows_nothing(start in 0i64..1000) {
            let mut watcher = Watcher::new(Cadence::OnePerCopy);
            watcher.tick(start);
            watcher.wrote(start + 99);
            prop_assert_eq!(watcher.tick(start + 1), Seen::Fresh { skipped: Some(0) });
        }
    }
}
