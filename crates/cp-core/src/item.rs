/// Qué se guardó de un formato concreto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    /// Cabe en la fila.
    Inline(Vec<u8>),
    /// Va a disco, direccionado por contenido.
    Blob(Vec<u8>),
    /// Existía y era demasiado grande. Se anota el tamaño y se dice en la
    /// tarjeta: nunca un descarte silencioso.
    TooBig { size: usize },
    /// La fuente lo ofrecía y no se le pidió, porque cuelga o porque
    /// derrocha. El tamaño solo se conoce donde la plataforma lo regala.
    Announced { size: Option<usize> },
    /// Se pidió y no entregó nada. Ocurre de verdad: medido el 12/09/2026,
    /// `com.apple.linkpresentation.metadata` y `fndf` hacen exactamente esto.
    Absent,
}

pub const INLINE_UP_TO: usize = 64 * 1024;
pub const BLOB_UP_TO: usize = 64 * 1024 * 1024;

const _: () = assert!(INLINE_UP_TO < BLOB_UP_TO);

impl Payload {
    /// Decide dónde va lo que ya se leyó.
    pub fn stored(bytes: Vec<u8>) -> Self {
        match bytes.len() {
            0..=INLINE_UP_TO => Payload::Inline(bytes),
            len if len <= BLOB_UP_TO => Payload::Blob(bytes),
            len => Payload::TooBig { size: len },
        }
    }

    pub fn size(&self) -> Option<usize> {
        match self {
            Payload::Inline(bytes) | Payload::Blob(bytes) => Some(bytes.len()),
            Payload::TooBig { size } => Some(*size),
            Payload::Announced { size } => *size,
            Payload::Absent => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Format {
    pub id: String,
    pub payload: Payload,
}

/// Un ítem guarda **el conjunto** de formatos que la fuente ofreció. El tipo
/// mostrado es una clasificación sobre ese conjunto, nunca una elección que
/// descarte el resto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub kind: Option<crate::formats::Kind>,
    pub formats: Vec<Format>,
}

impl Item {
    pub fn format(&self, id: &str) -> Option<&Format> {
        self.formats.iter().find(|one| one.id == id)
    }

    /// Lo que de verdad ocupa, sin contar lo que no se guardó.
    pub fn stored_bytes(&self) -> usize {
        self.formats
            .iter()
            .filter_map(|one| match &one.payload {
                Payload::Inline(bytes) | Payload::Blob(bytes) => Some(bytes.len()),
                _ => None,
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_things_stay_in_the_row() {
        let small = vec![0u8; 1024];
        assert!(matches!(Payload::stored(small), Payload::Inline(_)));
    }

    #[test]
    fn the_boundary_belongs_to_the_row() {
        assert!(matches!(
            Payload::stored(vec![0u8; INLINE_UP_TO]),
            Payload::Inline(_)
        ));
        assert!(matches!(
            Payload::stored(vec![0u8; INLINE_UP_TO + 1]),
            Payload::Blob(_)
        ));
    }

    #[test]
    fn what_does_not_fit_is_recorded_not_dropped() {
        let huge = Payload::TooBig {
            size: BLOB_UP_TO + 1,
        };
        assert_eq!(huge.size(), Some(BLOB_UP_TO + 1), "el tamaño se conserva");
    }

    #[test]
    fn an_announced_type_without_a_size_is_still_a_format() {
        let announced = Format {
            id: "com.apple.icns".into(),
            payload: Payload::Announced { size: None },
        };
        let item = Item {
            kind: None,
            formats: vec![announced],
        };
        assert!(item.format("com.apple.icns").is_some());
        assert_eq!(item.stored_bytes(), 0, "anotado no es guardado");
    }

    #[test]
    fn only_what_was_really_kept_counts_towards_the_size() {
        let item = Item {
            kind: None,
            formats: vec![
                Format {
                    id: "a".into(),
                    payload: Payload::Inline(vec![0u8; 100]),
                },
                Format {
                    id: "b".into(),
                    payload: Payload::Blob(vec![0u8; 900]),
                },
                Format {
                    id: "c".into(),
                    payload: Payload::TooBig { size: 999_999 },
                },
                Format {
                    id: "d".into(),
                    payload: Payload::Absent,
                },
            ],
        };
        assert_eq!(item.stored_bytes(), 1000);
    }
}
