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
const _: () = assert!(INLINE_UP_TO.is_power_of_two() && BLOB_UP_TO.is_power_of_two());

/// Dónde acaba un payload, decidido solo por su tamaño.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    Row,
    Blob,
    Refused,
}

pub fn placement(size: usize) -> Placement {
    match size {
        0..=INLINE_UP_TO => Placement::Row,
        size if size <= BLOB_UP_TO => Placement::Blob,
        _ => Placement::Refused,
    }
}

impl Payload {
    /// Decide dónde va lo que ya se leyó.
    pub fn stored(bytes: Vec<u8>) -> Self {
        match placement(bytes.len()) {
            Placement::Row => Payload::Inline(bytes),
            Placement::Blob => Payload::Blob(bytes),
            Placement::Refused => Payload::TooBig { size: bytes.len() },
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
    pub kind: Option<crate::kind::Kind>,
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
    fn the_three_placements_have_exact_boundaries() {
        assert_eq!(placement(0), Placement::Row);
        assert_eq!(placement(INLINE_UP_TO), Placement::Row);
        assert_eq!(placement(INLINE_UP_TO + 1), Placement::Blob);
        assert_eq!(placement(BLOB_UP_TO), Placement::Blob);
        assert_eq!(placement(BLOB_UP_TO + 1), Placement::Refused);
        assert_eq!(placement(usize::MAX), Placement::Refused);
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

#[cfg(test)]
mod properties {
    use super::*;
    use proptest::prelude::*;

    fn rank(placement: Placement) -> u8 {
        match placement {
            Placement::Row => 0,
            Placement::Blob => 1,
            Placement::Refused => 2,
        }
    }

    proptest! {
        /// Cuanto más grande, más lejos se guarda. Nunca al revés.
        #[test]
        fn bigger_never_lands_closer(a in 0usize..usize::MAX, b in 0usize..usize::MAX) {
            let (small, big) = if a <= b { (a, b) } else { (b, a) };
            prop_assert!(rank(placement(small)) <= rank(placement(big)));
        }

        /// Lo que se guarda conserva su tamaño, esté donde esté.
        #[test]
        fn the_size_survives_the_decision(len in 0usize..200_000) {
            let payload = Payload::stored(vec![0u8; len]);
            prop_assert_eq!(payload.size(), Some(len));
        }

        /// Solo lo que de verdad se guardó cuenta para el peso del ítem.
        #[test]
        fn only_real_bytes_are_counted(lens in prop::collection::vec(0usize..2000, 0..12)) {
            let expected: usize = lens.iter().sum();
            let item = Item {
                kind: None,
                formats: lens
                    .iter()
                    .enumerate()
                    .map(|(at, len)| Format {
                        id: format!("t{at}"),
                        payload: Payload::Inline(vec![0u8; *len]),
                    })
                    .chain(std::iter::once(Format {
                        id: "announced".into(),
                        payload: Payload::Announced { size: Some(999_999) },
                    }))
                    .chain(std::iter::once(Format {
                        id: "absent".into(),
                        payload: Payload::Absent,
                    }))
                    .collect(),
            };
            prop_assert_eq!(item.stored_bytes(), expected);
        }
    }
}
