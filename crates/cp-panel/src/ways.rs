use crate::layout::Layout;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Way {
    pub key: &'static str,
    pub es: &'static str,
    pub en: &'static str,
    pub recent: bool,
}

const KEYS: Way = Way {
    key: "keys",
    es: "Claves",
    en: "Keys",
    recent: false,
};
const RAW: Way = Way {
    key: "raw",
    es: "Crudo",
    en: "Raw",
    recent: false,
};
const GRID: Way = Way {
    key: "grid",
    es: "Rejilla",
    en: "Grid",
    recent: false,
};
const ROWS: Way = Way {
    key: "rows",
    es: "Lista",
    en: "List",
    recent: false,
};
const COVER: Way = Way {
    key: "cover",
    es: "Carátula",
    en: "Cover",
    recent: false,
};
const WAVE: Way = Way {
    key: "wave",
    es: "Onda",
    en: "Wave",
    recent: false,
};
const TIGHT: Way = Way {
    key: "tight",
    es: "Compacto",
    en: "Compact",
    recent: false,
};
const BY_GROUP: Way = Way {
    key: "by-group",
    es: "Por grupo",
    en: "By group",
    recent: false,
};
const NEWEST: Way = Way {
    key: "newest",
    es: "Recientes",
    en: "Newest",
    recent: true,
};

pub fn ways_of(layout: Layout) -> &'static [Way] {
    match layout {
        Layout::Everything => &[],
        Layout::Json => &[KEYS, RAW],
        Layout::Image => &[GRID, ROWS],
        Layout::Video => &[COVER, TIGHT],
        Layout::Audio => &[WAVE, TIGHT],
        Layout::Link | Layout::Folder | Layout::Papers => &[BY_GROUP, NEWEST],
    }
}

pub fn chosen(layout: Layout, key: &str) -> Way {
    let ways = ways_of(layout);
    ways.iter()
        .find(|one| one.key == key)
        .copied()
        .or_else(|| ways.first().copied())
        .unwrap_or(NEWEST)
}

pub fn label_of(way: Way, english: bool) -> &'static str {
    crate::say::pick_in(english, way.es, way.en)
}

#[cfg(test)]
#[path = "ways_test.rs"]
mod tests;
