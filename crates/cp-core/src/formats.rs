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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
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
    pub fn classify(&self, offered: &[&str]) -> Option<Kind> {
        let known: Vec<&str> = offered.iter().map(|id| self.canonical(id)).collect();
        if known.iter().any(|id| self.files.contains(id)) {
            return Some(Kind::Files);
        }
        if known
            .iter()
            .any(|id| self.images_by_preference.contains(id))
        {
            return Some(Kind::Image);
        }
        if known.iter().any(|id| self.text.contains(id)) {
            return Some(Kind::Text);
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

    const PROBE: Catalog = Catalog {
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
        assert_eq!(PROBE.classify(&mixed), Some(Kind::Files));
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
            Some(Kind::Image)
        );
    }
}
