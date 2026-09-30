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
