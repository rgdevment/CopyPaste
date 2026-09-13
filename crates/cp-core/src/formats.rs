/// Qué hacer con un tipo que la fuente ofreció.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Take {
    /// Se copian los bytes.
    Payload,
    /// Se anota que existía, con su tamaño, sin pedir los datos.
    Presence,
    /// No se toca nunca.
    Never,
}

/// La categoría que se deduce **del formato**. La clasificación fina —si ese
/// texto es un correo, un color o código— la hace `crate::kind`, que mira el
/// contenido.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Text,
    Image,
    Files,
}

/// Los nombres de los formatos son de cada plataforma —UTIs en macOS,
/// `CF_*` y cadenas registradas en Windows— pero las reglas que se les
/// aplican son las mismas. El catálogo son los datos; la lógica vive aquí y
/// se escribe una sola vez.
pub struct Catalog {
    /// Pedir sus datos cuelga la aplicación.
    pub hangs: &'static [&'static str],
    /// Arrastran megabytes sin aportar nada recuperable.
    pub wasteful: &'static [&'static str],
    /// Se copian enteros.
    pub wanted: &'static [&'static str],
    /// Nombre viejo y su equivalente moderno, mismo contenido.
    pub aliases: &'static [(&'static str, &'static str)],
    /// Su sola presencia significa «no registres esto».
    pub concealed: &'static [&'static str],
    /// Prefijos de tipos generados, que nunca llevan payload útil.
    pub opaque_prefixes: &'static [&'static str],
    pub text: &'static [&'static str],
    pub files: &'static [&'static str],
    /// Representaciones de imagen, de la más barata a la más cara.
    pub images_by_preference: &'static [&'static str],
}

impl Catalog {
    pub fn canonical<'a>(&self, id: &'a str) -> &'a str {
        self.aliases
            .iter()
            .find(|(legacy, _)| *legacy == id)
            .map_or(id, |(_, modern)| {
                // El alias apunta a una cadena estática, que vive más que 'a.
                *modern
            })
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

    /// Se decide por el tipo, nunca por el contenido: hay gestores que ponen
    /// el secreto en claro dentro del payload, así que leerlo para decidir ya
    /// sería haberlo leído.
    pub fn is_concealed(&self, offered: &[&str]) -> bool {
        offered.iter().any(|id| self.concealed.contains(id))
    }

    /// El tipo mostrado es una clasificación sobre el conjunto, nunca una
    /// elección que descarte lo demás.
    pub fn classify(&self, offered: &[&str]) -> Option<Family> {
        let known: Vec<&str> = offered.iter().map(|id| self.canonical(id)).collect();
        if known.iter().any(|id| self.files.contains(id)) {
            return Some(Family::Files);
        }
        if known
            .iter()
            .any(|id| self.images_by_preference.contains(id))
        {
            return Some(Family::Image);
        }
        if known.iter().any(|id| self.text.contains(id)) {
            return Some(Family::Text);
        }
        None
    }

    pub fn preferred_image(&self, offered: &[&str]) -> Option<&'static str> {
        let known: Vec<&str> = offered.iter().map(|id| self.canonical(id)).collect();
        self.images_by_preference
            .iter()
            .find(|wanted| known.contains(wanted))
            .copied()
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
        opaque_prefixes: &["dyn."],
        text: &["plain/text", "rich/text"],
        files: &["one/file"],
        images_by_preference: &["cheap/image", "costly/image"],
    };

    const NOTHING: Catalog = Catalog {
        hangs: &[],
        wasteful: &[],
        wanted: &[],
        aliases: &[],
        concealed: &[],
        opaque_prefixes: &[],
        text: &[],
        files: &[],
        images_by_preference: &[],
    };

    #[test]
    fn an_empty_catalogue_notes_everything_and_promises_nothing() {
        assert_eq!(NOTHING.decide("cualquier/cosa"), Take::Presence);
        assert_eq!(NOTHING.decide(""), Take::Presence);
        assert_eq!(NOTHING.classify(&["cualquier/cosa"]), None);
        assert_eq!(NOTHING.preferred_image(&["cualquier/cosa"]), None);
        assert!(!NOTHING.is_concealed(&["cualquier/cosa"]));
        assert_eq!(NOTHING.canonical("x"), "x");
    }

    #[test]
    fn nothing_offered_at_all_is_not_an_item() {
        assert_eq!(PROBE.classify(&[]), None);
        assert_eq!(PROBE.preferred_image(&[]), None);
        assert!(!PROBE.is_concealed(&[]));
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
            opaque_prefixes: &[],
            text: &["plain/text"],
            files: &[],
            images_by_preference: &["costly/image"],
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
        assert!(PROBE.is_concealed(&["plain/text", "secret/marker"]));
        assert!(!PROBE.is_concealed(&["plain/text"]));
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
            Just("one/file".to_string()),
            "[a-z]{0,12}/[a-z]{0,12}",
            "dyn\\.[a-z0-9]{0,20}",
            Just(String::new()),
        ]
    }

    proptest! {
        /// Un alias no puede apuntar a otro alias: si el nombre moderno
        /// volviera a traducirse, el ítem se guardaría bajo un tercer nombre
        /// y la deduplicación dejaría de reconocerlo.
        #[test]
        fn canonical_is_idempotent(id in any_id()) {
            let once = PROBE.canonical(&id).to_string();
            prop_assert_eq!(PROBE.canonical(&once), once.as_str());
        }

        /// Lo que cuelga no se pide jamás, se llame como se llame.
        #[test]
        fn what_hangs_is_never_payload(id in any_id()) {
            if PROBE.hangs.contains(&PROBE.canonical(&id)) {
                prop_assert_eq!(PROBE.decide(&id), Take::Never);
            }
        }

        /// Decidir dos veces sobre lo mismo da lo mismo.
        #[test]
        fn deciding_is_deterministic(id in any_id()) {
            prop_assert_eq!(PROBE.decide(&id), PROBE.decide(&id));
        }

        /// La imagen elegida siempre es una de las que se ofrecieron.
        #[test]
        fn the_chosen_image_was_on_offer(ids in prop::collection::vec(any_id(), 0..8)) {
            let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
            if let Some(chosen) = PROBE.preferred_image(&refs) {
                let canonical: Vec<&str> = refs.iter().map(|id| PROBE.canonical(id)).collect();
                prop_assert!(canonical.contains(&chosen));
            }
        }

        /// Clasificar no inventa: si dice que son archivos, había un archivo.
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
    }
}
