//! Qué tecla física hay que presionar para que salga una letra concreta.
//!
//! El keycode `0x09` es la posición de la V en QWERTY. Medido y corregido en
//! el expediente: AZERTY, Colemak y los layouts no latinos funcionan igual
//! porque la capa de Comando conmuta a una distribución latina, y **solo
//! Dvorak plano falla**. La resolución correcta no es «busca el keycode que
//! produce V» sino «busca el que la produce **con Command pulsado**», que
//! cubre los cinco casos de una vez, incluido Dvorak-QWERTY ⌘.

use std::ffi::c_void;

type CFStringRef = *const c_void;
type CFDataRef = *const c_void;
type CFArrayRef = *const c_void;
type TISInputSourceRef = *const c_void;

// SAFETY: firmas de Carbon y CoreFoundation tal como las declaran sus
// cabeceras. `kTISPropertyUnicodeKeyLayoutData` es una constante global.
unsafe extern "C" {
    static kTISPropertyUnicodeKeyLayoutData: CFStringRef;
    fn TISCopyCurrentKeyboardLayoutInputSource() -> TISInputSourceRef;
    fn TISGetInputSourceProperty(source: TISInputSourceRef, key: CFStringRef) -> CFDataRef;
    fn CFDataGetBytePtr(data: CFDataRef) -> *const u8;
    fn CFRelease(item: *const c_void);
    fn CFRetain(item: *const c_void) -> *const c_void;
    static kTISPropertyInputSourceID: CFStringRef;
    fn TISCreateInputSourceList(properties: *const c_void, include_all: bool) -> CFArrayRef;
    fn CFArrayGetCount(array: CFArrayRef) -> isize;
    fn CFArrayGetValueAtIndex(array: CFArrayRef, index: isize) -> *const c_void;
    fn CFStringGetCString(string: CFStringRef, buffer: *mut u8, size: isize, encoding: u32)
    -> bool;
    fn LMGetKbdType() -> u8;
    fn UCKeyTranslate(
        layout: *const u8,
        virtual_key: u16,
        key_action: u16,
        modifier_state: u32,
        keyboard_type: u32,
        options: u32,
        dead_keys: *mut u32,
        max_len: usize,
        actual_len: *mut usize,
        out: *mut u16,
    ) -> i32;
}

const ACTION_DOWN: u16 = 0;
const NO_DEAD_KEYS: u32 = 1;
/// `UCKeyTranslate` quiere los modificadores desplazados ocho bits, así que
/// `cmdKey` (0x0100) llega como 1.
const COMMAND_DOWN: u32 = 1;
const HIGHEST_KEYCODE: u16 = 127;

/// Guarda vivo el layout mientras se consulta.
struct Layout {
    source: TISInputSourceRef,
    bytes: *const u8,
}

/// Los identificadores de los layouts que el expediente manda probar.
pub const DVORAK: &str = "com.apple.keylayout.Dvorak";
pub const DVORAK_COMMAND_QWERTY: &str = "com.apple.keylayout.DVORAK-QWERTYCMD";
pub const AZERTY: &str = "com.apple.keylayout.French";
pub const QWERTZ: &str = "com.apple.keylayout.German";
pub const SPANISH_ISO: &str = "com.apple.keylayout.Spanish-ISO";
pub const COLEMAK: &str = "com.apple.keylayout.Colemak";
pub const ABC: &str = "com.apple.keylayout.ABC";

const UTF8: u32 = 0x0800_0100;

fn source_id(source: TISInputSourceRef) -> Option<String> {
    // SAFETY: `source` viene de la lista del sistema y la clave es constante.
    let value = unsafe { TISGetInputSourceProperty(source, kTISPropertyInputSourceID) };
    if value.is_null() {
        return None;
    }
    let mut buffer = [0u8; 256];
    // SAFETY: el buffer existe y se declara su tamaño real.
    let ok = unsafe { CFStringGetCString(value, buffer.as_mut_ptr(), buffer.len() as isize, UTF8) };
    if !ok {
        return None;
    }
    let end = buffer.iter().position(|byte| *byte == 0)?;
    String::from_utf8(buffer[..end].to_vec()).ok()
}

/// Los layouts de teclado instalados, por su identificador.
pub fn installed_layouts() -> Vec<String> {
    // SAFETY: pasar null pide la lista completa; devuelve +1 y se libera abajo.
    let list = unsafe { TISCreateInputSourceList(std::ptr::null(), true) };
    if list.is_null() {
        return Vec::new();
    }
    // SAFETY: `list` no es nulo.
    let count = unsafe { CFArrayGetCount(list) };
    let mut found = Vec::new();
    for index in 0..count {
        // SAFETY: el índice está dentro del rango que acaba de devolver.
        let source = unsafe { CFArrayGetValueAtIndex(list, index) };
        if let Some(id) = source_id(source) {
            found.push(id);
        }
    }
    // SAFETY: `list` vino de una función Create, así que hay que soltarlo.
    unsafe { CFRelease(list) };
    found
}

impl Layout {
    /// Un layout concreto por su identificador, **sin activarlo**: así se
    /// puede comprobar Dvorak sin tocarle el teclado a nadie.
    fn named(wanted: &str) -> Option<Self> {
        // SAFETY: pasar null pide la lista completa; devuelve +1.
        let list = unsafe { TISCreateInputSourceList(std::ptr::null(), true) };
        if list.is_null() {
            return None;
        }
        // SAFETY: `list` no es nulo.
        let count = unsafe { CFArrayGetCount(list) };
        let mut chosen = None;
        for index in 0..count {
            // SAFETY: el índice está dentro del rango devuelto.
            let source = unsafe { CFArrayGetValueAtIndex(list, index) };
            if source_id(source).as_deref() == Some(wanted) {
                chosen = Some(source);
                break;
            }
        }
        let result = chosen.and_then(|source| {
            // SAFETY: `source` pertenece al array, que sigue vivo aquí.
            let data =
                unsafe { TISGetInputSourceProperty(source, kTISPropertyUnicodeKeyLayoutData) };
            if data.is_null() {
                return None;
            }
            // SAFETY: `data` no es nulo mientras el array viva.
            let bytes = unsafe { CFDataGetBytePtr(data) };
            (!bytes.is_null()).then_some((source, bytes))
        });
        match result {
            Some((source, bytes)) => {
                // SAFETY: se retiene el source para que sobreviva al array.
                unsafe { CFRetain(source) };
                // SAFETY: el array ya no hace falta.
                unsafe { CFRelease(list) };
                Some(Self { source, bytes })
            }
            None => {
                // SAFETY: el array vino de una función Create.
                unsafe { CFRelease(list) };
                None
            }
        }
    }

    fn current() -> Option<Self> {
        // SAFETY: devuelve una referencia con +1 que este tipo libera al caer.
        let source = unsafe { TISCopyCurrentKeyboardLayoutInputSource() };
        if source.is_null() {
            return None;
        }
        // SAFETY: `source` no es nulo y la clave es la constante del sistema.
        let data = unsafe { TISGetInputSourceProperty(source, kTISPropertyUnicodeKeyLayoutData) };
        if data.is_null() {
            // SAFETY: `source` vino de una función Copy, así que hay que soltarlo.
            unsafe { CFRelease(source) };
            return None;
        }
        // SAFETY: `data` no es nulo y pertenece al input source, que sigue vivo.
        let bytes = unsafe { CFDataGetBytePtr(data) };
        if bytes.is_null() {
            // SAFETY: mismo motivo que arriba.
            unsafe { CFRelease(source) };
            return None;
        }
        Some(Self { source, bytes })
    }

    fn character_for(&self, keycode: u16, modifiers: u32) -> Option<char> {
        let mut dead_keys: u32 = 0;
        let mut produced: usize = 0;
        let mut buffer = [0u16; 4];
        // SAFETY: el layout sigue vivo, el buffer tiene el tamaño que se
        // declara, y los dos punteros de salida apuntan a locales válidas.
        let status = unsafe {
            UCKeyTranslate(
                self.bytes,
                keycode,
                ACTION_DOWN,
                modifiers,
                u32::from(keyboard_type()),
                NO_DEAD_KEYS,
                &mut dead_keys,
                buffer.len(),
                &mut produced,
                buffer.as_mut_ptr(),
            )
        };
        if status != 0 || produced == 0 {
            return None;
        }
        char::decode_utf16(buffer[..produced].iter().copied())
            .next()?
            .ok()
    }
}

impl Drop for Layout {
    fn drop(&mut self) {
        // SAFETY: `source` vino de `TISCopyCurrentKeyboardLayoutInputSource`,
        // que entrega +1, y no se ha liberado antes.
        unsafe { CFRelease(self.source) };
    }
}

fn keyboard_type() -> u8 {
    // SAFETY: la función no toma argumentos y devuelve un entero.
    unsafe { LMGetKbdType() }
}

/// El keycode que produce `wanted` con Command pulsado, en el layout activo.
///
/// Devuelve `None` si el layout no puede producir esa letra, y entonces el
/// llamante debe quedarse con el keycode físico de QWERTY antes que no pegar.
pub fn keycode_with_command(wanted: char) -> Option<u16> {
    let layout = Layout::current()?;
    (0..=HIGHEST_KEYCODE).find(|code| {
        layout
            .character_for(*code, COMMAND_DOWN)
            .is_some_and(|got| got.eq_ignore_ascii_case(&wanted))
    })
}

/// Lo mismo, en un layout concreto, sin activarlo. Devuelve `None` si ese
/// layout no está instalado.
pub fn keycode_with_command_in(layout: &str, wanted: char) -> Option<u16> {
    let layout = Layout::named(layout)?;
    (0..=HIGHEST_KEYCODE).find(|code| {
        layout
            .character_for(*code, COMMAND_DOWN)
            .is_some_and(|got| got.eq_ignore_ascii_case(&wanted))
    })
}

/// Lo que se usa si no se puede resolver nada: la posición de la V en QWERTY.
pub const QWERTY_V: u16 = 0x09;
