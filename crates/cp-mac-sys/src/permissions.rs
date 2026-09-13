//! Las tres sondas del pegado en macOS, que no son la misma cosa.

// SAFETY: declaraciones de funciones C de los frameworks del sistema. Las
// firmas se corresponden con las de CoreGraphics y HIToolbox.
unsafe extern "C" {
    fn CGPreflightPostEventAccess() -> bool;
    fn CGRequestPostEventAccess() -> bool;
    fn AXIsProcessTrusted() -> bool;
    fn IsSecureEventInputEnabled() -> bool;
}

/// Si se pueden postear eventos de teclado.
///
/// Es la sonda correcta para lo que hace CopyPaste, y **no** es
/// `AXIsProcessTrusted`: son dos servicios distintos de TCC, con dos filas
/// separadas, que pueden divergir. La 2.x consulta el equivocado.
pub fn can_post_events() -> bool {
    // SAFETY: la función no toma argumentos ni devuelve punteros.
    unsafe { CGPreflightPostEventAccess() }
}

/// Pide el permiso, lo que muestra el diálogo del sistema la primera vez.
///
/// En una segunda llamada, con la decisión ya registrada en TCC, **no vuelve
/// a preguntar**: por eso un botón de «comprobar de nuevo» no puede arreglar
/// un permiso obsoleto, y hay que decirle al usuario que lo quite y lo
/// vuelva a añadir.
pub fn request_post_events() -> bool {
    // SAFETY: la función no toma argumentos ni devuelve punteros.
    unsafe { CGRequestPostEventAccess() }
}

/// Si el proceso está en la lista de Accesibilidad. Solo hace falta para el
/// respaldo de pegar por el menú Editar, no para el ⌘V sintético.
pub fn is_accessibility_trusted() -> bool {
    // SAFETY: la función no toma argumentos ni devuelve punteros.
    unsafe { AXIsProcessTrusted() }
}

/// Si algún proceso tiene el input seguro activado.
///
/// Medido el 12/09/2026: con esto activo, el ⌘V sintético **sí llega** y
/// `AXPress` sobre el menú **también**. Lo que rompe es la activación. Por
/// eso esto se usa para avisar y jamás para abortar: abortar dejaría sin
/// pegar precisamente a quien tiene el flag pegado por una aplicación ajena.
pub fn is_secure_input_enabled() -> bool {
    // SAFETY: la función no toma argumentos ni devuelve punteros.
    unsafe { IsSecureEventInputEnabled() }
}

/// Lo que hace falta para pegar, resumido.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Readiness {
    pub can_post: bool,
    pub accessibility: bool,
    pub secure_input: bool,
}

impl Readiness {
    pub fn probe() -> Self {
        Self {
            can_post: can_post_events(),
            accessibility: is_accessibility_trusted(),
            secure_input: is_secure_input_enabled(),
        }
    }

    /// El pegado se intenta siempre que se puedan postear eventos. El input
    /// seguro no lo impide y la accesibilidad solo abre el respaldo.
    pub fn can_paste(&self) -> bool {
        self.can_post
    }

    /// El respaldo por el menú Editar necesita el segundo permiso, y además
    /// solo sirve con el destino ya al frente.
    pub fn can_use_menu_fallback(&self) -> bool {
        self.accessibility
    }
}
