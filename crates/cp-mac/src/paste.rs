use cp_core::destination::Destination;
use cp_core::paste::{Attempt, Failure, Focus, Next, ORDER, Phase};
use cp_mac_sys::frontmost;
use cp_mac_sys::keyboard::{self, QWERTY_V};
use cp_mac_sys::keystroke::{self, Keystroke};
use cp_mac_sys::runloop;

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
}

impl Paster {
    pub fn new() -> Option<Self> {
        Some(Self {
            keys: Keystroke::new()?,
        })
    }

    /// Se resuelve en **cada** pegado, no al arrancar.
    ///
    /// Guardarlo parecía la optimización obvia y era un fallo: cambiar de
    /// distribución con la aplicación abierta dejaba el keycode viejo, y en
    /// Dvorak eso escribe otra letra. Medido, resolverlo cuesta 458 ns, así
    /// que la caché solo aportaba el fallo.
    pub fn keycode(&self) -> u16 {
        keyboard::keycode_with_command('v').unwrap_or(QWERTY_V)
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

        // Un destino que ya no existe no es un problema de foco, y decir que
        // lo es deja al usuario sin saber qué pasó.
        if !frontmost::is_alive(target.pid) {
            return Outcome::Degraded(Failure::TargetGone);
        }

        // Traerlo al frente. Con el panel no-activador esto suele ser
        // innecesario porque el destino nunca se desactivó, pero si algo se
        // interpuso hay que recuperarlo: no existe pegado sin activación.
        if self.focus_of(target) != Focus::OnTarget {
            frontmost::bring_to_front(target.pid);
        }

        while self.focus_of(target) != Focus::OnTarget {
            if !frontmost::is_alive(target.pid) {
                return Outcome::Degraded(Failure::TargetGone);
            }
            if attempt.on_failure(Failure::NotForeground) != Next::Retry {
                return Outcome::Degraded(Failure::NotForeground);
            }
            frontmost::bring_to_front(target.pid);
            self.wait(0.060);
        }

        // Los modificadores del atajo pueden seguir pulsados: el hotkey llega
        // en key-down. Se espera a que se suelten, con techo.
        let waiting = std::time::Instant::now();
        while keystroke::modifiers_still_held() && waiting.elapsed().as_millis() < 120 {
            self.wait(0.004);
        }

        attempt.sending();
        if self.keys.command(self.keycode()) {
            Outcome::Sent {
                took: started.elapsed(),
            }
        } else {
            Outcome::Degraded(Failure::SendDenied)
        }
    }

    /// Esperar **girando el run loop**, no durmiendo.
    ///
    /// Esto corre en el hilo principal, que es el mismo que tiene que
    /// procesar el `orderOut` del panel y la desactivación que este bucle
    /// está esperando. Dormirlo es esperar un trabajo que nadie hará.
    fn wait(&self, seconds: f64) {
        runloop::pump(seconds);
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
