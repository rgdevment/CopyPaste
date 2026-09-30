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
    pub page_archives: &'static [&'static str],
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

    pub fn decide_in(&self, family: Option<Family>, id: &str) -> Take {
        let id = self.canonical(id);
        if family == Some(Family::Files) && self.images_by_preference.contains(&id) {
            return Take::Presence;
        }
        if family == Some(Family::Image) && self.page_archives.contains(&id) {
            return Take::Presence;
        }
        self.decide(id)
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
#[path = "formats_test.rs"]
mod tests;

#[cfg(test)]
#[path = "formats_windows_needs_test.rs"]
mod windows_needs;

#[cfg(test)]
#[path = "formats_properties_test.rs"]
mod properties;
