use super::tests::PROBE;
use super::*;
use proptest::prelude::*;

fn any_id() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("plain/text".to_string()),
        Just("old/text".to_string()),
        Just("hangs/forever".to_string()),
        Just("huge/icon".to_string()),
        Just("cheap/image".to_string()),
        Just("costly/image".to_string()),
        Just("cheap/markup".to_string()),
        Just("costly/markup".to_string()),
        Just("embedded/document".to_string()),
        Just("one/file".to_string()),
        "[a-z]{0,12}/[a-z]{0,12}",
        "dyn\\.[a-z0-9]{0,20}",
        Just(String::new()),
    ]
}

proptest! {
    #[test]
    fn canonical_is_idempotent(id in any_id()) {
        let once = PROBE.canonical(&id).to_string();
        prop_assert_eq!(PROBE.canonical(&once), once.as_str());
    }

    #[test]
    fn what_hangs_is_never_payload(id in any_id()) {
        if PROBE.hangs.contains(&PROBE.canonical(&id)) {
            prop_assert_eq!(PROBE.decide(&id), Take::Never);
        }
    }

    #[test]
    fn deciding_is_deterministic(id in any_id()) {
        prop_assert_eq!(PROBE.decide(&id), PROBE.decide(&id));
    }

    #[test]
    fn the_chosen_image_was_on_offer(ids in prop::collection::vec(any_id(), 0..8)) {
        let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
        if let Some(chosen) = PROBE.preferred_image(&refs) {
            let canonical: Vec<&str> = refs.iter().map(|id| PROBE.canonical(id)).collect();
            prop_assert!(canonical.contains(&chosen));
        }
    }

    #[test]
    fn a_classification_is_backed_by_a_type(ids in prop::collection::vec(any_id(), 0..8)) {
        let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
        let canonical: Vec<&str> = refs.iter().map(|id| PROBE.canonical(id)).collect();
        match PROBE.classify(&refs) {
            Some(Family::Files) => {
                prop_assert!(canonical.iter().any(|id| PROBE.files.contains(id)))
            }
            Some(Family::Image) => prop_assert!(
                canonical
                    .iter()
                    .any(|id| PROBE.images_by_preference.contains(id))
            ),
            Some(Family::Text) => {
                prop_assert!(canonical.iter().any(|id| PROBE.text.contains(id)))
            }
            None => {}
        }
    }

    #[test]
    fn exactly_one_of_each_group_is_kept(ids in prop::collection::vec(any_id(), 0..8)) {
        let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
        for group in std::iter::once(PROBE.images_by_preference)
            .chain(PROBE.equivalents.iter().copied())
        {
            let kept: Vec<&&str> = group
                .iter()
                .filter(|id| refs.iter().any(|one| PROBE.canonical(one) == **id))
                .filter(|id| !PROBE.costlier_twin(id, &refs))
                .collect();
            let offered = group
                .iter()
                .any(|id| refs.iter().any(|one| PROBE.canonical(one) == *id));
            prop_assert_eq!(kept.len(), usize::from(offered));
        }
    }

    #[test]
    fn only_a_zero_in_a_declared_marker_refuses(
        id in any_id(),
        value in prop::collection::vec(any::<u8>(), 0..8),
    ) {
        match PROBE.declines(&id, &value) {
            Some(Refusal::Declined(marker)) => {
                prop_assert!(PROBE.denied_when_zero.contains(&marker));
                if let [a, b, c, d, ..] = value[..] {
                    prop_assert_eq!(u32::from_le_bytes([a, b, c, d]), 0);
                }
            }
            Some(other) => prop_assert!(false, "not a value-based refusal: {:?}", other),
            None => {}
        }
    }

    #[test]
    fn the_class_does_not_depend_on_the_order_offered(
        ids in prop::collection::vec(any_id(), 0..8),
    ) {
        let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
        let mut backwards = refs.clone();
        backwards.reverse();
        prop_assert_eq!(PROBE.classify(&refs), PROBE.classify(&backwards));
    }
}
