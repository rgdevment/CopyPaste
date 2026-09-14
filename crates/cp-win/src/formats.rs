use cp_core::formats::Catalog;

/// Los tipos de Windows. Las reglas que se les aplican viven en `cp-core`;
/// esto son solo los datos, y su gemelo de macOS tiene la misma forma.
///
/// Los `CF_*` son enteros; aquí se nombran por su constante, y la capa de
/// sistema traduce. Los demás llegan con su nombre registrado tal cual.
pub const CATALOG: Catalog = Catalog {
    // Ninguno confirmado todavía. En Windows el renderizado diferido no es
    // propiedad de un tipo sino de quien copió —`rdpclip` difiere todo—, así
    // que la defensa no es una lista: es el reloj de quien pide los bytes.
    hangs: &[],
    wasteful: &[
        "Embed Source",
        "Native",
        "OwnerLink",
        "ObjectLink",
        "Link Source",
        "Link Source Descriptor",
        "Link",
        "Object Descriptor",
        "Ole Private Data",
        "DataObject",
        "DataObjectAttributes",
        "DataObjectAttributesRequiringElevation",
        "Shell Object Offsets",
        "AsyncFlag",
        "ZoneIdentifier",
        "CF_ENHMETAFILE",
        "CF_METAFILEPICT",
        "CF_SYLK",
        "CF_DIF",
        "Biff12",
        "Biff8",
        "Biff5",
        "XML Spreadsheet",
        // Un `HBITMAP` de GDI, no un bloque de memoria: medido, `GlobalSize`
        // no devuelve nada sobre él.
        "CF_BITMAP",
        // El descriptor se anota y el contenido no se puede pedir por el API
        // plano del portapapeles, que es el hallazgo de los archivos virtuales.
        "FileGroupDescriptorW",
        "FileContents",
        "Shell IDList Array",
        "Chromium internal source RFH token",
    ],
    wanted: &[
        "CF_UNICODETEXT",
        "Rich Text Format",
        "HTML Format",
        "PNG",
        "CF_DIBV5",
        "CF_DIB",
        "CF_HDROP",
        "Csv",
        "Preferred DropEffect",
        "Chromium internal source URL",
        "UniformResourceLocatorW",
        "text/uri-list",
    ],
    // `FileName` y `FileNameW` **no** son alias de `CF_HDROP`: llevan una sola
    // ruta cuando se copiaron varias, así que tomarlos por equivalentes perdería
    // archivos. Van en `wasteful`, que los anota sin pedirlos.
    aliases: &[],
    concealed: &["ExcludeClipboardContentFromMonitorProcessing"],
    denied_when_zero: &["CanIncludeInClipboardHistory"],
    opaque_prefixes: &[],
    text: &["CF_UNICODETEXT", "Rich Text Format", "HTML Format", "Csv"],
    files: &["CF_HDROP"],
    images_by_preference: &["PNG", "CF_DIBV5", "CF_DIB"],
    equivalents: &[
        // El Unicode primero: los otros dos son el mismo texto pasado a la
        // página de códigos del sistema, con la pérdida que eso trae.
        &["CF_UNICODETEXT", "CF_TEXT", "CF_OEMTEXT"],
        // El de Windows lleva la cabecera de offsets y pesa la mitad.
        &["HTML Format", "text/html"],
    ],
    embeddable: &[
        "Embed Source",
        "Native",
        "Object Descriptor",
        "Link Source Descriptor",
        "Biff12",
        "Biff8",
        "Biff5",
        "XML Spreadsheet",
        "Csv",
    ],
};

#[cfg(test)]
mod tests {
    use super::CATALOG;
    use cp_core::formats::{Family, Refusal, Take};

    /// Las siete fuentes se midieron el 14/09/2026 sobre Windows 11 26200,
    /// enumerando todos los tipos que cada aplicación ofrecía de verdad.
    const WORD: &[&str] = &[
        "DataObject",
        "Object Descriptor",
        "Rich Text Format",
        "HTML Format",
        "CF_TEXT",
        "CF_UNICODETEXT",
        "CF_ENHMETAFILE",
        "CF_METAFILEPICT",
        "Embed Source",
        "Native",
        "OwnerLink",
        "Link Source",
        "Link Source Descriptor",
        "ObjectLink",
        "Ole Private Data",
        "CF_LOCALE",
        "CF_OEMTEXT",
    ];

    const EXCEL: &[&str] = &[
        "DataObject",
        "CF_ENHMETAFILE",
        "CF_METAFILEPICT",
        "CF_BITMAP",
        "Biff12",
        "Biff8",
        "Biff5",
        "CF_SYLK",
        "CF_DIF",
        "XML Spreadsheet",
        "HTML Format",
        "CF_UNICODETEXT",
        "CF_TEXT",
        "Csv",
        "Rich Text Format",
        "Embed Source",
        "Native",
        "OwnerLink",
        "Object Descriptor",
        "Link Source",
        "Link Source Descriptor",
        "Link",
        "ExcludeClipboardContentFromMonitorProcessing",
        "ObjectLink",
        "Ole Private Data",
        "CF_LOCALE",
        "CF_OEMTEXT",
        "CF_DIB",
        "CF_DIBV5",
    ];

    const FIREFOX: &[&str] = &[
        "DataObject",
        "text/html",
        "HTML Format",
        "text/_moz_htmlcontext",
        "text/_moz_htmlinfo",
        "CF_UNICODETEXT",
        "CF_TEXT",
        "text/x-moz-url-priv",
        "Ole Private Data",
        "CF_LOCALE",
        "CF_OEMTEXT",
    ];

    const CHROME: &[&str] = &[
        "HTML Format",
        "CF_UNICODETEXT",
        "Chromium internal source RFH token",
        "Chromium internal source URL",
        "CF_LOCALE",
        "CF_TEXT",
        "CF_OEMTEXT",
    ];

    const EXPLORER: &[&str] = &[
        "DataObject",
        "Shell IDList Array",
        "DataObjectAttributes",
        "DataObjectAttributesRequiringElevation",
        "Shell Object Offsets",
        "Preferred DropEffect",
        "AsyncFlag",
        "CF_HDROP",
        "FileName",
        "FileContents",
        "FileNameW",
        "FileGroupDescriptorW",
        "ZoneIdentifier",
        "Ole Private Data",
    ];

    const SNIP: &[&str] = &[
        "DataObject",
        "CF_BITMAP",
        "PNG",
        "CanUploadToCloudClipboard",
        "CanIncludeInClipboardHistory",
        "Ole Private Data",
        "CF_DIB",
        "CF_DIBV5",
    ];

    const TERMINAL: &[&str] = &["CF_UNICODETEXT", "CF_LOCALE", "CF_TEXT", "CF_OEMTEXT"];

    fn kept(offered: &[&str]) -> Vec<&'static str> {
        offered
            .iter()
            .filter(|id| CATALOG.decide(id) == Take::Payload)
            .filter(|id| !CATALOG.costlier_twin(id, offered))
            .map(|id| {
                CATALOG
                    .wanted
                    .iter()
                    .find(|one| **one == CATALOG.canonical(id))
                    .copied()
                    .unwrap_or("?")
            })
            .collect()
    }

    #[test]
    fn word_keeps_its_styles_and_leaves_the_ole_object_alone() {
        assert_eq!(CATALOG.classify(WORD), Some(Family::Text));
        let kept = kept(WORD);
        assert!(kept.contains(&"Rich Text Format"), "{kept:?}");
        assert!(kept.contains(&"HTML Format"), "{kept:?}");
        assert!(kept.contains(&"CF_UNICODETEXT"), "{kept:?}");
        // 41.833 bytes de RTF y 39.321 de HTML por un párrafo, y ni un byte
        // del objeto incrustado, que los arrastra por decenas de megas.
        for heavy in ["Embed Source", "Native", "CF_ENHMETAFILE"] {
            assert_eq!(CATALOG.decide(heavy), Take::Presence, "{heavy}");
        }
    }

    /// El fallo que Windows estrena: Excel adjunta una imagen del rango —medido,
    /// 258.380 bytes de `CF_DIBV5` por 500 de texto— y con la regla de macOS
    /// copiar celdas se guardaría como una captura de pantalla.
    #[test]
    fn a_spreadsheet_is_text_and_not_a_picture_of_itself() {
        assert_eq!(CATALOG.classify(EXCEL), Some(Family::Text));
    }

    /// Y Excel pide no ser registrado en **cada** copia, incluso de dos celdas
    /// y con la aplicación abierta a la vista. Se obedece, y se dice quién lo
    /// pidió: un descarte mudo es indistinguible de un fallo.
    #[test]
    fn excel_asks_not_to_be_recorded_and_says_so_by_name() {
        assert_eq!(
            CATALOG.refusal(EXCEL),
            Some(Refusal::Marked(
                "ExcludeClipboardContentFromMonitorProcessing"
            ))
        );
        assert_eq!(CATALOG.refusal(WORD), None, "Word no lo pide");
    }

    #[test]
    fn a_password_manager_that_says_zero_is_obeyed() {
        assert_eq!(
            CATALOG.declines("CanIncludeInClipboardHistory", &[0, 0, 0, 0]),
            Some(Refusal::Declined("CanIncludeInClipboardHistory"))
        );
        assert_eq!(
            CATALOG.declines("CanIncludeInClipboardHistory", &[1, 0, 0, 0]),
            None,
            "la captura de pantalla dice que sí y se guarda"
        );
    }

    /// Firefox ofrece la misma selección dos veces: 568.458 bytes en
    /// `text/html` y 284.561 en `HTML Format`. Se guarda una.
    #[test]
    fn a_browser_selection_is_not_stored_twice() {
        assert_eq!(CATALOG.classify(FIREFOX), Some(Family::Text));
        assert!(CATALOG.costlier_twin("text/html", FIREFOX));
        assert!(!CATALOG.costlier_twin("HTML Format", FIREFOX));
    }

    /// El texto degradado a la página de códigos del sistema no se guarda
    /// nunca: es el mismo contenido con pérdida, y en Word llega **antes** que
    /// el Unicode, así que quedarse con el primero sería quedarse con el malo.
    #[test]
    fn the_ansi_halves_of_the_text_are_never_kept() {
        for source in [WORD, EXCEL, FIREFOX, CHROME, TERMINAL] {
            for degraded in ["CF_TEXT", "CF_OEMTEXT"] {
                assert!(
                    CATALOG.costlier_twin(degraded, source),
                    "{degraded} se guardó pudiendo guardar el Unicode"
                );
            }
            assert!(!CATALOG.costlier_twin("CF_UNICODETEXT", source));
        }
    }

    /// Una captura ofrece 13.127.103 bytes en total. Con el PNG basta: 164.311.
    #[test]
    fn a_screen_capture_costs_its_png_and_not_its_bitmap() {
        assert_eq!(CATALOG.classify(SNIP), Some(Family::Image));
        assert_eq!(CATALOG.preferred_image(SNIP), Some("PNG"));
        assert!(CATALOG.costlier_twin("CF_DIB", SNIP));
        assert!(CATALOG.costlier_twin("CF_DIBV5", SNIP));
        assert!(!CATALOG.costlier_twin("PNG", SNIP));
    }

    /// Sin PNG gana el `CF_DIBV5`, nunca el `CF_DIB`: el clásico pierde el
    /// canal alfa y son 84 bytes de diferencia.
    #[test]
    fn without_a_png_the_bitmap_with_alpha_wins() {
        let paint = ["DataObject", "CF_BITMAP", "CF_DIB", "CF_DIBV5"];
        assert_eq!(CATALOG.preferred_image(&paint), Some("CF_DIBV5"));
        assert!(CATALOG.costlier_twin("CF_DIB", &paint));
        assert_eq!(CATALOG.classify(&paint), Some(Family::Image));
    }

    /// El explorador ofrece catorce tipos y solo dos hacen falta: las rutas y
    /// si fue copia o corte. `FileName` y `FileNameW` llevan una sola ruta
    /// cuando se copiaron varias, así que guardarlos perdería archivos.
    #[test]
    fn the_explorer_needs_only_the_paths_and_the_effect() {
        assert_eq!(CATALOG.classify(EXPLORER), Some(Family::Files));
        let kept = kept(EXPLORER);
        assert!(kept.contains(&"CF_HDROP"), "{kept:?}");
        assert!(kept.contains(&"Preferred DropEffect"), "{kept:?}");
        assert_eq!(kept.len(), 2, "y nada más: {kept:?}");
        for partial in ["FileName", "FileNameW"] {
            assert_eq!(CATALOG.decide(partial), Take::Presence, "{partial}");
        }
    }

    /// Los archivos virtuales se anotan y no se piden: `FileContents` no llega
    /// por el API plano del portapapeles, y pedirlo sería colgarse esperando.
    #[test]
    fn virtual_files_are_noted_and_never_asked_for() {
        for virtualised in ["FileGroupDescriptorW", "FileContents"] {
            assert_eq!(CATALOG.decide(virtualised), Take::Presence);
        }
    }

    /// Una terminal ofrece lo mínimo, y lo mínimo sigue siendo un ítem.
    #[test]
    fn the_plainest_copy_there_is_still_an_item() {
        assert_eq!(CATALOG.classify(TERMINAL), Some(Family::Text));
        assert_eq!(kept(TERMINAL), vec!["CF_UNICODETEXT"]);
    }

    /// El navegador guarda de dónde salió, que es contexto que la 2.x tiraba.
    #[test]
    fn a_browser_copy_keeps_where_it_came_from() {
        assert_eq!(
            CATALOG.decide("Chromium internal source URL"),
            Take::Payload
        );
        assert_eq!(
            CATALOG.decide("Chromium internal source RFH token"),
            Take::Presence,
            "el testigo interno no es contexto de nadie"
        );
    }

    /// Lo que ninguna fuente medida ofrece tampoco se pide: el catálogo no
    /// promete nada sobre lo que no conoce.
    #[test]
    fn the_unknown_is_only_noted() {
        for unknown in ["ApplicationXYZ", "", "algo/inventado"] {
            assert_eq!(CATALOG.decide(unknown), Take::Presence);
        }
    }

    /// Ninguna de las siete fuentes cae en «no se sabe qué es esto».
    #[test]
    fn every_measured_source_gets_a_class() {
        for (name, source) in [
            ("Word", WORD),
            ("Excel", EXCEL),
            ("Firefox", FIREFOX),
            ("Chrome", CHROME),
            ("Explorador", EXPLORER),
            ("Recortes", SNIP),
            ("Terminal", TERMINAL),
        ] {
            assert!(CATALOG.classify(source).is_some(), "{name}");
        }
    }

    /// Un tipo no puede estar a la vez en lo que se copia y en lo que se anota:
    /// la lista larga gana en `decide` y el catálogo mentiría sobre sí mismo.
    #[test]
    fn nothing_is_both_wanted_and_wasteful() {
        for id in CATALOG.wanted {
            assert!(
                !CATALOG.wasteful.contains(id),
                "«{id}» está en las dos listas"
            );
        }
    }

    /// Todo lo que decide una familia tiene que poder copiarse; si no, la
    /// clasificación prometería un contenido que nunca se guardó.
    #[test]
    fn everything_that_names_a_family_can_be_copied() {
        for id in CATALOG.text.iter().chain(CATALOG.files) {
            assert_eq!(CATALOG.decide(id), Take::Payload, "«{id}»");
        }
        for id in CATALOG.images_by_preference {
            assert_eq!(CATALOG.decide(id), Take::Payload, "«{id}»");
        }
    }

    /// El primero de cada grupo de equivalentes es el que se guarda, así que
    /// tiene que ser uno de los que se copian.
    #[test]
    fn the_preferred_of_each_group_is_one_that_gets_copied() {
        for group in CATALOG.equivalents {
            let first = group.first().expect("un grupo vacío no desempata nada");
            assert_eq!(CATALOG.decide(first), Take::Payload, "«{first}»");
        }
    }
}
