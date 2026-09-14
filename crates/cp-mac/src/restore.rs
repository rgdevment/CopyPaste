use cp_core::item::{Item, Payload};
use cp_mac_sys::pasteboard::Pasteboard;

/// Devuelve un ítem del historial al portapapeles, con **todos** los formatos
/// que se guardaron de él.
///
/// Es la otra mitad del pegado y la razón de que el ítem guarde el conjunto
/// entero: restaurar solo el texto de algo que se copió de Word haría que
/// pegarlo perdiera los estilos que sí se habían capturado.
///
/// Se escribe **antes** de tocar el foco. Así, si algo falla después, el peor
/// resultado posible sigue siendo «está en tu portapapeles, pégalo tú».
pub fn to_pasteboard(pb: &Pasteboard, item: &Item) -> Restored {
    let writable: Vec<(&str, &[u8])> = item
        .formats
        .iter()
        .filter_map(|format| match &format.payload {
            Payload::Inline(bytes) | Payload::Blob(bytes) => {
                Some((format.id.as_str(), bytes.as_slice()))
            }
            // Lo que solo se anotó no tiene bytes que devolver, y lo que no
            // cupo tampoco: decirlo es mejor que escribir un ítem a medias
            // sin que nadie se entere.
            _ => None,
        })
        .collect();

    if writable.is_empty() {
        return Restored::NothingToWrite;
    }

    let entries: Vec<(&str, &[u8])> = writable.clone();
    if pb.write_all(&entries) {
        Restored::Written {
            formats: writable.len(),
            incomplete: writable.len() != item.formats.len(),
        }
    } else {
        Restored::Failed
    }
}

/// Deja en el portapapeles **solo** el texto plano del ítem.
///
/// Pegar sin formato es una opción de **esta** vez, no un cambio en lo
/// guardado: el ítem conserva su RTF, su HTML y su RTFD intactos, y la
/// siguiente vez se puede pegar con estilos. La 2.x tiene aquí un fallo
/// —escribe el texto plano y no restaura lo enriquecido, así que el ítem
/// queda mutilado a partir de entonces—, y es el hallazgo 27 del mapa.
///
/// Tampoco duplica: no crea un ítem nuevo «en versión plana».
pub fn to_pasteboard_as_plain_text(pb: &Pasteboard, item: &Item) -> Restored {
    let Some(text) = item
        .formats
        .iter()
        .find_map(|format| match &format.payload {
            Payload::Inline(bytes) | Payload::Blob(bytes) if format.id == PLAIN_TEXT => {
                Some(bytes.as_slice())
            }
            _ => None,
        })
    else {
        return Restored::NothingToWrite;
    };

    if pb.write_all(&[(PLAIN_TEXT, text)]) {
        Restored::Written {
            formats: 1,
            // Es intencionado: se pidió solo el texto. No es un ítem a medias.
            incomplete: false,
        }
    } else {
        Restored::Failed
    }
}

const PLAIN_TEXT: &str = "public.utf8-plain-text";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Restored {
    /// Se escribieron `formats` formatos. `incomplete` avisa de que el ítem
    /// tenía alguno más que no se pudo devolver.
    Written {
        formats: usize,
        incomplete: bool,
    },
    /// El ítem no tiene ningún byte que devolver.
    NothingToWrite,
    Failed,
}
