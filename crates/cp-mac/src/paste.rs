use cp_core::destination::Destination;
use cp_core::paste::{Attempt, Failure, Focus, Next, ORDER, Phase};
use cp_mac_sys::frontmost;
use cp_mac_sys::keyboard::{self, QWERTY_V};
use cp_mac_sys::keystroke::{self, Keystroke};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Se envió la pulsación. No significa que la aplicación la haya usado:
    /// eso solo lo sabe quien mire el destino.
    Sent { took: std::time::Duration },
    /// El contenido está en el portapapeles y el usuario puede pegarlo.
    Degraded(Failure),
}

pub struct Paster {
    keys: Keystroke,
    keycode: u16,
}

impl Paster {
    pub fn new() -> Option<Self> {
        Some(Self {
            keys: Keystroke::new()?,
            // Se resuelve una vez y se guarda: recorrer el layout entero en
            // cada pegado sería trabajo repetido para un dato que solo cambia
            // al cambiar de teclado.
            keycode: keyboard::keycode_with_command('v').unwrap_or(QWERTY_V),
        })
    }

    pub fn keycode(&self) -> u16 {
        self.keycode
    }

    /// La secuencia completa, en el orden que fija `cp_core::paste::ORDER`.
    ///
    /// El portapapeles ya tiene que estar escrito: es la fase cero y ocurre
    /// antes de que esto se llame, para que cualquier fallo de aquí en
    /// adelante degrade a «está en tu portapapeles».
    pub fn paste_into(&self, target: &Destination, hide_panel: impl FnOnce()) -> Outcome {
        let started = std::time::Instant::now();
        let mut attempt = Attempt::default();

        hide_panel();

        while self.focus_of(target) == Focus::Elsewhere {
            if attempt.on_failure(Failure::NotForeground) != Next::Retry {
                return Outcome::Degraded(Failure::NotForeground);
            }
            std::thread::sleep(std::time::Duration::from_millis(60));
        }

        // Los modificadores del atajo pueden seguir pulsados: el hotkey llega
        // en key-down. Se espera a que se suelten, con techo.
        let waiting = std::time::Instant::now();
        while keystroke::modifiers_still_held() && waiting.elapsed().as_millis() < 120 {
            std::thread::sleep(std::time::Duration::from_millis(4));
        }

        attempt.sending();
        if self.keys.command(self.keycode) {
            Outcome::Sent {
                took: started.elapsed(),
            }
        } else {
            Outcome::Degraded(Failure::SendDenied)
        }
    }

    /// Medido: no existe pegado sin activación, así que lo único que importa
    /// es si el destino está al frente. La sonda válida es `frontmost`, no
    /// `NSApplication::isActive`.
    fn focus_of(&self, target: &Destination) -> Focus {
        match frontmost::frontmost() {
            Some((pid, _)) if pid == target.pid => Focus::OnTarget,
            Some(_) => Focus::Elsewhere,
            None => Focus::Unknown,
        }
    }
}

/// Las fases que esta implementación recorre, para que la prueba compruebe
/// que no se ha saltado ninguna ni las ha reordenado.
pub fn phases() -> &'static [Phase] {
    ORDER
}
