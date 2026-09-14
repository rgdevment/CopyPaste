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

/// El nombre que el usuario ve, no el identificador.
///
/// Es «Safari», no «com.apple.Safari». Guardar el bundle y enseñárselo a
/// alguien que escribe «safari» en el buscador sería no encontrarlo.
pub fn app_name(pid: i32) -> Option<String> {
    let app = NSRunningApplication::runningApplicationWithProcessIdentifier(pid)?;
    app.localizedName().map(|name| name.to_string())
}

/// Si las rutas de un ítem de archivos siguen existiendo.
///
/// Copiar la ruta de algo que después se mueve o se borra es corriente, y la
/// diferencia entre un historial útil y uno que miente es avisar de ello.
pub fn missing_paths(file_urls: &str) -> Vec<String> {
    file_urls
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter(|url| {
            let path = url
                .strip_prefix("file://")
                .map(percent_decoded)
                .unwrap_or_else(|| (*url).to_string());
            !std::path::Path::new(&path).exists()
        })
        .map(str::to_owned)
        .collect()
}

/// Las rutas llegan con los espacios y los acentos escapados.
fn percent_decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == b'%' && at + 2 < bytes.len() {
            let pair = std::str::from_utf8(&bytes[at + 1..at + 3]).ok();
            if let Some(byte) = pair.and_then(|hex| u8::from_str_radix(hex, 16).ok()) {
                out.push(byte);
                at += 3;
                continue;
            }
        }
        out.push(bytes[at]);
        at += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
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
