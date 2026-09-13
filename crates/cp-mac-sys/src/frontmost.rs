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
