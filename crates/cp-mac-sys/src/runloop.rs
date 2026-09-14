//! Esperar sin bloquear el hilo principal.

use std::ffi::c_void;

type CFStringRef = *const c_void;

// SAFETY: firmas de CoreFoundation tal como las declara su cabecera.
unsafe extern "C" {
    static kCFRunLoopDefaultMode: CFStringRef;
    fn CFRunLoopRunInMode(
        mode: CFStringRef,
        seconds: f64,
        return_after_source_handled: bool,
    ) -> i32;
}

/// Deja correr el run loop del hilo actual durante un rato.
///
/// Es lo que hay que usar en lugar de `thread::sleep` cuando se espera **a
/// algo que este mismo hilo tiene que procesar**. Dormir el hilo principal
/// mientras se espera a que el panel termine de ocultarse y el destino
/// recupere el primer plano es esperar a un trabajo que nadie va a hacer,
/// porque quien lo haría está dormido.
pub fn pump(seconds: f64) {
    // SAFETY: el modo es la constante del sistema y la llamada solo cede el
    // hilo al run loop durante el tiempo indicado.
    unsafe { CFRunLoopRunInMode(kCFRunLoopDefaultMode, seconds, false) };
}
