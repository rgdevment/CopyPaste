//! Enviar ⌘V, con los tres detalles que deciden si llega o no.

use std::ffi::c_void;

type CGEventSourceRef = *const c_void;
type CGEventRef = *const c_void;

// SAFETY: firmas de CoreGraphics tal como las declara su cabecera.
unsafe extern "C" {
    fn CGEventSourceCreate(state_id: i32) -> CGEventSourceRef;
    fn CGEventSourceFlagsState(state_id: i32) -> u64;
    fn CGEventSourceSetLocalEventsFilterDuringSuppressionState(
        source: CGEventSourceRef,
        filter: u32,
        state: i32,
    );
    fn CGEventCreateKeyboardEvent(
        source: CGEventSourceRef,
        keycode: u16,
        key_down: bool,
    ) -> CGEventRef;
    fn CGEventSetFlags(event: CGEventRef, flags: u64);
    fn CGEventPost(tap: u32, event: CGEventRef);
    fn CFRelease(item: *const c_void);
}

/// `kCGEventSourceStateCombinedSessionState`.
const COMBINED_SESSION: i32 = 0;
/// `kCGHIDEventTap`: entra lo más cerca posible del hardware, que es lo más
/// compatible aunque también lo más visible para otras utilidades.
const HID_TAP: u32 = 0;
const MASK_COMMAND: u64 = 0x0010_0000;
/// El bit device-dependent del ⌘ izquierdo. Hay aplicaciones que lo exigen.
const LEFT_COMMAND: u64 = 0x0000_0008;
const PERMIT_ALL: u32 = 3;
const SUPPRESSION_INTERVAL: i32 = 0;

/// Las teclas modificadoras que están **físicamente** pulsadas ahora.
///
/// Se pregunta a la fuente de eventos y no al teclado de la aplicación: un
/// atajo global llega sin pasar por ella, así que preguntarle da una respuesta
/// ciega. Es el hallazgo 22.
pub fn physical_modifiers() -> u64 {
    // SAFETY: la función solo lee el estado global de modificadores.
    unsafe { CGEventSourceFlagsState(COMBINED_SESSION) }
}

/// Si hay algún modificador del atajo todavía pulsado.
pub fn modifiers_still_held() -> bool {
    const ANY: u64 = 0x000e_0000;
    physical_modifiers() & ANY != 0
}

pub struct Keystroke {
    source: CGEventSourceRef,
}

impl Keystroke {
    pub fn new() -> Option<Self> {
        // SAFETY: devuelve +1 y este tipo lo libera al caer.
        let source = unsafe { CGEventSourceCreate(COMBINED_SESSION) };
        if source.is_null() {
            return None;
        }
        // SAFETY: `source` acaba de comprobarse no nulo.
        unsafe {
            CGEventSourceSetLocalEventsFilterDuringSuppressionState(
                source,
                PERMIT_ALL,
                SUPPRESSION_INTERVAL,
            )
        };
        Some(Self { source })
    }

    /// Manda la tecla con Command. Entre presionar y soltar van 9 ms: con
    /// separación cero, Chromium deduplica y el pegado «a veces no funciona».
    pub fn command(&self, keycode: u16) -> bool {
        let flags = MASK_COMMAND | LEFT_COMMAND;
        // Los dos eventos se crean **antes** de postear ninguno: postear el de
        // presionar y fallar luego al soltar deja la tecla hundida con
        // Command encima, y la máquina inutilizable hasta que alguien la toque.
        let (Some(down), Some(up)) = (
            self.event(keycode, true, flags),
            self.event(keycode, false, flags),
        ) else {
            return false;
        };

        // SAFETY: `down` es un evento válido recién creado.
        unsafe { CGEventPost(HID_TAP, down) };
        // SAFETY: ya se posteó, se suelta.
        unsafe { CFRelease(down) };

        std::thread::sleep(std::time::Duration::from_millis(9));

        // SAFETY: `up` es un evento válido recién creado.
        unsafe { CGEventPost(HID_TAP, up) };
        // SAFETY: ya se posteó, se suelta.
        unsafe { CFRelease(up) };
        true
    }

    fn event(&self, keycode: u16, down: bool, flags: u64) -> Option<CGEventRef> {
        // SAFETY: `self.source` sigue vivo mientras exista este tipo.
        let event = unsafe { CGEventCreateKeyboardEvent(self.source, keycode, down) };
        if event.is_null() {
            return None;
        }
        // SAFETY: `event` acaba de comprobarse no nulo.
        unsafe { CGEventSetFlags(event, flags) };
        Some(event)
    }
}

impl Drop for Keystroke {
    fn drop(&mut self) {
        // SAFETY: `source` vino de una función Create y no se ha soltado.
        unsafe { CFRelease(self.source) };
    }
}
