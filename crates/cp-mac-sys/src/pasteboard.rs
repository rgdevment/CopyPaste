use objc2_app_kit::{NSPasteboard, NSPasteboardItem, NSPasteboardWriting};
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
    pub fn change_count(&self) -> i64 {
        self.inner.changeCount() as i64
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

    /// Los datos de un tipo, **de cada ítem** del portapapeles.
    ///
    /// `dataForType:` sobre el pasteboard plano devuelve solo el primer ítem,
    /// así que copiar tres archivos en Finder daba una sola ruta. La API por
    /// ítems es la que ve el conjunto: es lo que hace la 2.x con
    /// `readObjects(forClasses:)`.
    pub fn data_per_item(&self, uti: &str) -> Vec<Vec<u8>> {
        let Some(items) = self.inner.pasteboardItems() else {
            return Vec::new();
        };
        let name = NSString::from_str(uti);
        items
            .iter()
            .filter_map(|item| item.dataForType(&name).map(|data| data.to_vec()))
            .collect()
    }

    /// Cuántos ítems distintos hay en el portapapeles.
    pub fn item_count(&self) -> usize {
        self.inner.pasteboardItems().map_or(0, |items| items.len())
    }

    /// Escribe varios ítems, que es como el sistema representa «tres
    /// archivos» y no «un archivo con tres rutas dentro».
    pub fn write_items(&self, items: &[Vec<(&str, &str)>]) -> bool {
        self.inner.clearContents();
        let written: Vec<objc2::rc::Retained<NSPasteboardItem>> = items
            .iter()
            .map(|entries| {
                let item = NSPasteboardItem::new();
                for (uti, value) in entries {
                    let name = NSString::from_str(uti);
                    let text = NSString::from_str(value);
                    item.setString_forType(&text, &name);
                }
                item
            })
            .collect();
        let refs: Vec<&objc2::runtime::ProtocolObject<dyn NSPasteboardWriting>> = written
            .iter()
            .map(|item| objc2::runtime::ProtocolObject::from_ref(&**item))
            .collect();
        let array = objc2_foundation::NSArray::from_slice(&refs);
        self.inner.writeObjects(&array)
    }

    /// Escribe datos binarios, que es como llega una imagen de verdad.
    pub fn write_data(&self, uti: &str, bytes: &[u8]) -> bool {
        self.inner.clearContents();
        let name = NSString::from_str(uti);
        let data = objc2_foundation::NSData::with_bytes(bytes);
        self.inner.setData_forType(Some(&data), &name)
    }

    /// Escribe varios formatos del **mismo** ítem, en una sola operación.
    ///
    /// Uno por uno no vale: cada `clearContents` empieza de cero y dejaría el
    /// portapapeles con el último formato escrito en lugar de con el conjunto.
    pub fn write_all(&self, entries: &[(&str, &[u8])]) -> bool {
        self.inner.clearContents();
        entries.iter().all(|(uti, bytes)| {
            let name = NSString::from_str(uti);
            let data = objc2_foundation::NSData::with_bytes(bytes);
            self.inner.setData_forType(Some(&data), &name)
        })
    }

    pub fn write_text(&self, text: &str) -> bool {
        self.inner.clearContents();
        let name = NSString::from_str("public.utf8-plain-text");
        let value = NSString::from_str(text);
        self.inner.setString_forType(&value, &name)
    }

    /// Escribe varios tipos a la vez, para poder reproducir lo que hace una
    /// aplicación real —incluido un gestor de contraseñas marcando su copia—.
    pub fn write_types(&self, entries: &[(&str, &str)]) -> bool {
        self.inner.clearContents();
        entries.iter().all(|(uti, value)| {
            let name = NSString::from_str(uti);
            let text = NSString::from_str(value);
            self.inner.setString_forType(&text, &name)
        })
    }
}

/// El contador de cambios, leído **sin** el marcador de hilo principal.
///
/// El invariante 1 pide que el vigilante sondee desde su propio hilo y que el
/// salto al principal se limite a copiar bytes. Esto es lo que hace posible
/// esa mitad: leer el contador es una consulta de un entero y no materializa
/// ninguna representación, así que no dispara el `NSPasteboardItemDataProvider`
/// de nadie ni toca el camino de file promises, que es donde está el fallo de
/// concurrencia documentado de `NSPasteboard`.
///
/// Todo lo demás —tipos y datos— sigue exigiendo el hilo principal, y el tipo
/// `Pasteboard` lo obliga en compilación. Esa exigencia es **nuestra**, no de
/// la API: `objc2` expone estas llamadas como seguras, así que el marcador es
/// lo único que impide que alguien lea datos desde un hilo cualquiera.
pub fn change_count_from_any_thread() -> i64 {
    // `NSInteger` es isize; el núcleo habla i64 porque en Windows el contador
    // es un DWORD. La conversión se hace aquí, en la frontera.
    objc2_app_kit::NSPasteboard::generalPasteboard().changeCount() as i64
}
