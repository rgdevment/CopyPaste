use objc2_app_kit::NSPasteboard;
use objc2_foundation::{MainThreadMarker, NSString};

/// El pasteboard del sistema.
///
/// `NSPasteboard` tiene un fallo de concurrencia documentado con file
/// promises, así que solo se toca desde el hilo principal. El
/// `MainThreadMarker` lo vuelve imposible de incumplir en compilación: el
/// hilo vigilante sondea el contador y el salto al principal se limita a
/// copiar bytes.
pub struct Pasteboard {
    inner: objc2::rc::Retained<NSPasteboard>,
}

impl Pasteboard {
    pub fn general(_mtm: MainThreadMarker) -> Self {
        Self {
            inner: NSPasteboard::generalPasteboard(),
        }
    }

    /// Recibo del estado. Sube exactamente de uno en uno por escritura, así
    /// que un salto mayor que uno significa copias perdidas, y eso se cuenta
    /// en lugar de ignorarse.
    pub fn change_count(&self) -> isize {
        self.inner.changeCount()
    }

    pub fn types(&self) -> Vec<String> {
        let Some(types) = self.inner.types() else {
            return Vec::new();
        };
        types.iter().map(|one| one.to_string()).collect()
    }

    /// Devuelve `None` tanto si el tipo no está como si el proveedor lo
    /// anunció y no entregó nada, que ocurre de verdad: medido el 12/09/2026,
    /// `com.apple.linkpresentation.metadata` y `fndf` hacen exactamente eso.
    pub fn data(&self, uti: &str) -> Option<Vec<u8>> {
        let name = NSString::from_str(uti);
        let data = self.inner.dataForType(&name)?;
        Some(data.to_vec())
    }

    pub fn write_text(&self, text: &str) -> bool {
        self.inner.clearContents();
        let name = NSString::from_str("public.utf8-plain-text");
        let value = NSString::from_str(text);
        self.inner.setString_forType(&value, &name)
    }
}
