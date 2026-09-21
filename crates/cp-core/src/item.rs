#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    Inline(Vec<u8>),
    Blob(Vec<u8>),
    TooBig { size: usize },
    Announced { size: Option<usize> },
    Absent,
}

pub const INLINE_UP_TO: usize = 64 * 1024;
pub const BLOB_UP_TO: usize = 64 * 1024 * 1024;

const _: () = assert!(INLINE_UP_TO < BLOB_UP_TO);
const _: () = assert!(INLINE_UP_TO.is_power_of_two() && BLOB_UP_TO.is_power_of_two());

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub kind: Option<crate::kind::Kind>,
    pub formats: Vec<Format>,
}

pub const SYNTHETIC_TEXT: &str = "text/plain";

pub const SYNTHETIC_IMAGE: &str = "image/png";

pub const SYNTHETIC_JPEG: &str = "image/jpeg";

impl Item {
    pub fn plain(text: &str) -> Self {
        Self {
            kind: None,
            formats: vec![Format {
                id: SYNTHETIC_TEXT.into(),
                payload: Payload::Inline(text.as_bytes().to_vec()),
            }],
        }
    }

    pub fn fingerprint(&self) -> u64 {
        let mut mixed: Vec<u8> = Vec::new();
        let mut ordered: Vec<&Format> = self.formats.iter().collect();
        ordered.sort_by(|a, b| a.id.cmp(&b.id));
        for format in ordered {
            match &format.payload {
                Payload::Inline(bytes) | Payload::Blob(bytes) => {
                    mixed.extend_from_slice(format.id.as_bytes());
                    mixed.push(0);
                    mixed.extend_from_slice(bytes);
                    mixed.push(0);
                }
                _ => {}
            }
        }
        crate::hash::content_hash(&mixed)
    }

    pub fn needs_blob_store(&self) -> bool {
        self.oversized_format().is_some()
    }

    pub fn oversized_format(&self) -> Option<(String, usize)> {
        self.formats.iter().find_map(|one| match &one.payload {
            Payload::Blob(bytes) => Some((one.id.clone(), bytes.len())),
            _ => None,
        })
    }

    pub fn format(&self, id: &str) -> Option<&Format> {
        self.formats.iter().find(|one| one.id == id)
    }

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
    fn two_items_that_look_alike_but_differ_have_different_identities() {
        let one = Item {
            kind: None,
            formats: vec![Format {
                id: "public.png".into(),
                payload: Payload::Inline(vec![1, 2, 3]),
            }],
        };
        let other = Item {
            kind: None,
            formats: vec![Format {
                id: "public.png".into(),
                payload: Payload::Inline(vec![1, 2, 4]),
            }],
        };
        assert_ne!(
            one.fingerprint(),
            other.fingerprint(),
            "dos capturas distintas con el mismo preview vacío"
        );
    }

    #[test]
    fn the_identity_does_not_depend_on_the_order_of_the_formats() {
        let a = Format {
            id: "public.rtf".into(),
            payload: Payload::Inline(vec![9]),
        };
        let b = Format {
            id: "public.utf8-plain-text".into(),
            payload: Payload::Inline(vec![8]),
        };
        let one = Item {
            kind: None,
            formats: vec![a.clone(), b.clone()],
        };
        let other = Item {
            kind: None,
            formats: vec![b, a],
        };
        assert_eq!(one.fingerprint(), other.fingerprint());
    }

    #[test]
    fn what_was_only_announced_does_not_change_the_identity() {
        let with_note = Item {
            kind: None,
            formats: vec![
                Format {
                    id: "public.utf8-plain-text".into(),
                    payload: Payload::Inline(b"hola".to_vec()),
                },
                Format {
                    id: "com.apple.icns".into(),
                    payload: Payload::Announced { size: Some(999) },
                },
            ],
        };
        let without = Item {
            kind: None,
            formats: vec![Format {
                id: "public.utf8-plain-text".into(),
                payload: Payload::Inline(b"hola".to_vec()),
            }],
        };
        assert_eq!(with_note.fingerprint(), without.fingerprint());
    }

    #[test]
    fn an_item_says_when_it_needs_a_blob_store() {
        let small = Item {
            kind: None,
            formats: vec![Format {
                id: "t".into(),
                payload: Payload::Inline(vec![0; 10]),
            }],
        };
        let big = Item {
            kind: None,
            formats: vec![Format {
                id: "t".into(),
                payload: Payload::Blob(vec![0; 10]),
            }],
        };
        assert!(!small.needs_blob_store());
        assert!(big.needs_blob_store());
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
        #[test]
        fn bigger_never_lands_closer(a in 0usize..usize::MAX, b in 0usize..usize::MAX) {
            let (small, big) = if a <= b { (a, b) } else { (b, a) };
            prop_assert!(rank(placement(small)) <= rank(placement(big)));
        }

        #[test]
        fn the_size_survives_the_decision(len in 0usize..200_000) {
            let payload = Payload::stored(vec![0u8; len]);
            prop_assert_eq!(payload.size(), Some(len));
        }

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
