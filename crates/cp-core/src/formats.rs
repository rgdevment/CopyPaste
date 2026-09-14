#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Take {
    Payload,
    Presence,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Text,
    Image,
    Files,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    Marked(&'static str),
    Declined(&'static str),
}

impl Refusal {
    pub fn marker(self) -> &'static str {
        match self {
            Refusal::Marked(id) | Refusal::Declined(id) => id,
        }
    }
}

pub struct Catalog {
    pub hangs: &'static [&'static str],
    pub wasteful: &'static [&'static str],
    pub wanted: &'static [&'static str],
    pub aliases: &'static [(&'static str, &'static str)],
    pub concealed: &'static [&'static str],
    pub denied_when_zero: &'static [&'static str],
    pub opaque_prefixes: &'static [&'static str],
    pub text: &'static [&'static str],
    pub files: &'static [&'static str],
    pub images_by_preference: &'static [&'static str],
    pub equivalents: &'static [&'static [&'static str]],
    pub embeddable: &'static [&'static str],
}

impl Catalog {
    pub fn canonical<'a>(&self, id: &'a str) -> &'a str {
        self.aliases
            .iter()
            .find(|(legacy, _)| *legacy == id)
            .map_or(id, |(_, modern)| *modern)
    }

    pub fn decide(&self, id: &str) -> Take {
        let id = self.canonical(id);
        if self.hangs.contains(&id) {
            return Take::Never;
        }
        if self.wasteful.contains(&id) || self.opaque_prefixes.iter().any(|p| id.starts_with(p)) {
            return Take::Presence;
        }
        if self.wanted.contains(&id) {
            return Take::Payload;
        }
        Take::Presence
    }

    pub fn refusal(&self, offered: &[&str]) -> Option<Refusal> {
        offered.iter().find_map(|id| {
            self.concealed
                .iter()
                .find(|marker| *marker == id)
                .map(|marker| Refusal::Marked(marker))
        })
    }

    pub fn declines(&self, id: &str, value: &[u8]) -> Option<Refusal> {
        let marker = self.denied_when_zero.iter().find(|marker| **marker == id)?;
        let [a, b, c, d, ..] = value else {
            return Some(Refusal::Declined(marker));
        };
        (u32::from_le_bytes([*a, *b, *c, *d]) == 0).then_some(Refusal::Declined(marker))
    }

    pub fn classify(&self, offered: &[&str]) -> Option<Family> {
        let known: Vec<&str> = offered.iter().map(|id| self.canonical(id)).collect();
        if known.iter().any(|id| self.files.contains(id)) {
            return Some(Family::Files);
        }
        let has_image = known
            .iter()
            .any(|id| self.images_by_preference.contains(id));
        let has_text = known.iter().any(|id| self.text.contains(id));
        let courtesy = has_text && known.iter().any(|id| self.embeddable.contains(id));
        if has_image && !courtesy {
            return Some(Family::Image);
        }
        if has_text {
            return Some(Family::Text);
        }
        has_image.then_some(Family::Image)
    }

    pub fn preferred_image(&self, offered: &[&str]) -> Option<&'static str> {
        self.cheapest(self.images_by_preference, offered)
    }

    pub fn costlier_twin(&self, id: &str, offered: &[&str]) -> bool {
        let id = self.canonical(id);
        std::iter::once(self.images_by_preference)
            .chain(self.equivalents.iter().copied())
            .filter(|group| group.contains(&id))
            .any(|group| self.cheapest(group, offered).is_some_and(|best| best != id))
    }

    fn cheapest(&self, group: &'static [&'static str], offered: &[&str]) -> Option<&'static str> {
        let known: Vec<&str> = offered.iter().map(|id| self.canonical(id)).collect();
        group.iter().find(|wanted| known.contains(wanted)).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) const PROBE: Catalog = Catalog {
        hangs: &["hangs/forever"],
        wasteful: &["huge/icon"],
        wanted: &[
            "plain/text",
            "rich/text",
            "cheap/image",
            "costly/image",
            "one/file",
        ],
        aliases: &[("old/text", "plain/text")],
        concealed: &["secret/marker"],
        denied_when_zero: &["may/record"],
        opaque_prefixes: &["dyn."],
        text: &["plain/text", "rich/text"],
        files: &["one/file"],
        images_by_preference: &["cheap/image", "costly/image"],
        equivalents: &[&["cheap/markup", "costly/markup"]],
        embeddable: &["embedded/document"],
    };

    const NOTHING: Catalog = Catalog {
        hangs: &[],
        wasteful: &[],
        wanted: &[],
        aliases: &[],
        concealed: &[],
        denied_when_zero: &[],
        opaque_prefixes: &[],
        text: &[],
        files: &[],
        images_by_preference: &[],
        equivalents: &[],
        embeddable: &[],
    };

    #[test]
    fn an_empty_catalogue_notes_everything_and_promises_nothing() {
        assert_eq!(NOTHING.decide("cualquier/cosa"), Take::Presence);
        assert_eq!(NOTHING.decide(""), Take::Presence);
        assert_eq!(NOTHING.classify(&["cualquier/cosa"]), None);
        assert_eq!(NOTHING.preferred_image(&["cualquier/cosa"]), None);
        assert_eq!(NOTHING.refusal(&["cualquier/cosa"]), None);
        assert_eq!(NOTHING.declines("cualquier/cosa", &[0, 0, 0, 0]), None);
        assert!(!NOTHING.costlier_twin("cualquier/cosa", &["otra/cosa"]));
        assert_eq!(NOTHING.canonical("x"), "x");
    }

    #[test]
    fn nothing_offered_at_all_is_not_an_item() {
        assert_eq!(PROBE.classify(&[]), None);
        assert_eq!(PROBE.preferred_image(&[]), None);
        assert_eq!(PROBE.refusal(&[]), None);
    }

    #[test]
    fn an_empty_type_name_is_just_unknown() {
        assert_eq!(PROBE.decide(""), Take::Presence);
        assert_eq!(PROBE.canonical(""), "");
    }

    #[test]
    fn what_hangs_is_never_asked_for() {
        assert_eq!(PROBE.decide("hangs/forever"), Take::Never);
    }

    #[test]
    fn the_expensive_and_the_unknown_are_only_noted() {
        assert_eq!(PROBE.decide("huge/icon"), Take::Presence);
        assert_eq!(PROBE.decide("dyn.abc123"), Take::Presence);
        assert_eq!(PROBE.decide("who/knows"), Take::Presence);
    }

    #[test]
    fn wasteful_beats_wanted_when_a_type_is_in_both() {
        const GREEDY: Catalog = Catalog {
            hangs: &[],
            wasteful: &["costly/image"],
            wanted: &["costly/image", "plain/text"],
            aliases: &[],
            concealed: &[],
            denied_when_zero: &[],
            opaque_prefixes: &[],
            text: &["plain/text"],
            files: &[],
            images_by_preference: &["costly/image"],
            equivalents: &[],
            embeddable: &[],
        };
        assert_eq!(
            GREEDY.decide("costly/image"),
            Take::Presence,
            "estar en la lista de deseados no salva a un tipo que derrocha"
        );
        assert_eq!(GREEDY.decide("plain/text"), Take::Payload);
    }

    #[test]
    fn a_legacy_name_is_the_modern_one() {
        assert_eq!(PROBE.canonical("old/text"), "plain/text");
        assert_eq!(PROBE.decide("old/text"), Take::Payload);
    }

    #[test]
    fn the_cheapest_representation_wins() {
        assert_eq!(
            PROBE.preferred_image(&["costly/image", "cheap/image"]),
            Some("cheap/image")
        );
        assert_eq!(
            PROBE.preferred_image(&["costly/image"]),
            Some("costly/image")
        );
        assert_eq!(PROBE.preferred_image(&["plain/text"]), None);
    }

    #[test]
    fn a_secret_is_recognised_by_the_type_alone() {
        assert_eq!(
            PROBE.refusal(&["plain/text", "secret/marker"]),
            Some(Refusal::Marked("secret/marker"))
        );
        assert_eq!(PROBE.refusal(&["plain/text"]), None);
    }

    #[test]
    fn classifying_never_discards_the_rest() {
        let mixed = ["plain/text", "cheap/image", "one/file"];
        assert_eq!(PROBE.classify(&mixed), Some(Family::Files));
        let kept: Vec<&str> = mixed
            .iter()
            .filter(|id| PROBE.decide(id) == Take::Payload)
            .copied()
            .collect();
        assert_eq!(
            kept.len(),
            3,
            "clasificar no es elegir uno y tirar el resto"
        );
    }

    #[test]
    fn an_image_offered_with_its_text_is_still_an_image() {
        assert_eq!(
            PROBE.classify(&["plain/text", "cheap/image"]),
            Some(Family::Image)
        );
    }
}

#[cfg(test)]
mod windows_needs {
    use super::tests::PROBE;
    use super::*;

    #[test]
    fn a_spreadsheet_is_text_even_when_it_offers_a_picture_of_itself() {
        let excel = ["plain/text", "cheap/image", "embedded/document"];
        assert_eq!(PROBE.classify(&excel), Some(Family::Text));
    }

    #[test]
    fn an_image_with_its_address_alongside_is_still_an_image() {
        let browser = ["plain/text", "cheap/image"];
        assert_eq!(PROBE.classify(&browser), Some(Family::Image));
    }

    #[test]
    fn an_embeddable_marker_without_text_does_not_hide_the_image() {
        assert_eq!(
            PROBE.classify(&["cheap/image", "embedded/document"]),
            Some(Family::Image)
        );
    }

    #[test]
    fn files_still_win_over_everything() {
        assert_eq!(
            PROBE.classify(&["one/file", "plain/text", "cheap/image", "embedded/document"]),
            Some(Family::Files)
        );
    }

    #[test]
    fn the_costlier_wrapping_of_the_same_text_is_only_noted() {
        let firefox = ["costly/markup", "cheap/markup", "plain/text"];
        assert!(PROBE.costlier_twin("costly/markup", &firefox));
        assert!(!PROBE.costlier_twin("cheap/markup", &firefox));
    }

    #[test]
    fn the_only_wrapping_on_offer_is_never_the_costlier_one() {
        assert!(!PROBE.costlier_twin("costly/markup", &["costly/markup"]));
        assert!(!PROBE.costlier_twin("costly/image", &["costly/image"]));
    }

    #[test]
    fn a_type_in_no_group_has_no_twin() {
        assert!(!PROBE.costlier_twin("plain/text", &["plain/text", "cheap/markup"]));
    }

    #[test]
    fn a_marker_that_says_zero_refuses_the_copy() {
        assert_eq!(
            PROBE.declines("may/record", &[0, 0, 0, 0]),
            Some(Refusal::Declined("may/record"))
        );
        assert_eq!(PROBE.declines("may/record", &[1, 0, 0, 0]), None);
    }

    #[test]
    fn a_marker_too_short_to_read_is_obeyed_anyway() {
        for unreadable in [&[][..], &[0][..], &[0, 0, 0][..], &[1, 0, 0][..]] {
            assert_eq!(
                PROBE.declines("may/record", unreadable),
                Some(Refusal::Declined("may/record")),
                "{unreadable:?}"
            );
        }
    }

    #[test]
    fn only_a_declared_marker_is_read() {
        assert_eq!(PROBE.declines("plain/text", &[0, 0, 0, 0]), None);
        assert_eq!(PROBE.declines("secret/marker", &[0, 0, 0, 0]), None);
    }

    #[test]
    fn a_longer_marker_is_read_by_its_first_four_bytes() {
        assert_eq!(
            PROBE.declines("may/record", &[0, 0, 0, 0, 9, 9]),
            Some(Refusal::Declined("may/record"))
        );
        assert_eq!(PROBE.declines("may/record", &[1, 0, 0, 0, 0, 0]), None);
    }

    #[test]
    fn the_four_bytes_are_one_little_endian_number() {
        assert_eq!(PROBE.declines("may/record", &[0, 0, 0, 1]), None);
        assert_eq!(PROBE.declines("may/record", &[0, 1, 0, 0]), None);
    }

    #[test]
    fn either_road_to_a_refusal_names_the_marker() {
        assert_eq!(
            PROBE
                .refusal(&["plain/text", "secret/marker"])
                .map(Refusal::marker),
            Some("secret/marker")
        );
        assert_eq!(
            PROBE
                .declines("may/record", &[0, 0, 0, 0])
                .map(Refusal::marker),
            Some("may/record")
        );
    }
}

#[cfg(test)]
mod properties {
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
                Some(other) => prop_assert!(false, "no es una negativa por valor: {:?}", other),
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
}
