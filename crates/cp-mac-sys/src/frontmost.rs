use objc2_app_kit::NSWorkspace;

/// Quién está al frente ahora mismo.
///
/// Es la **única** sonda válida para saber si le hemos robado el primer plano
/// al destino: medido el 12/09/2026, `NSApplication::isActive` pasa a `true`
/// con un panel no-activador visible, y `AXFocusedApplication` también nos
/// señala, mientras que esta sigue devolviendo la aplicación de destino.
pub fn frontmost() -> Option<(i32, Option<String>)> {
    let app = NSWorkspace::sharedWorkspace().frontmostApplication()?;
    let pid = app.processIdentifier();
    let bundle = app.bundleIdentifier().map(|id| id.to_string());
    Some((pid, bundle))
}

/// El pid de este proceso, para que el seguidor sepa a quién ignorar.
pub fn our_pid() -> i32 {
    std::process::id() as i32
}
use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication};

/// Trae al frente la aplicación con ese pid.
///
/// El valor que devuelve la API **no es de fiar**: medido el 12/09/2026, con
/// Secure Input activo devuelve `true` y la aplicación no pasa al frente. Se
/// devuelve igualmente por si sirve de pista, pero quien llama tiene que
/// comprobar el resultado mirando quién está al frente de verdad.
pub fn bring_to_front(pid: i32) -> bool {
    let Some(app) = NSRunningApplication::runningApplicationWithProcessIdentifier(pid) else {
        return false;
    };
    // `ActivateAllWindows` es lo que hace `open -a` y lo que la 2.x pedía:
    // traer la aplicación entera, no solo su ventana principal.
    app.activateWithOptions(NSApplicationActivationOptions::ActivateAllWindows)
}

/// Si el proceso sigue vivo. Un destino que se cerró entre la captura y el
/// pegado no es un fallo de foco, y confundirlos da un diagnóstico inútil.
pub fn is_alive(pid: i32) -> bool {
    NSRunningApplication::runningApplicationWithProcessIdentifier(pid).is_some()
}
