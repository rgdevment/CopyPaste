use crate::age::age_text;
use crate::{Card, Chip};
use cp_core::kind::Kind;
use cp_store::{Facet, Listed};

pub const ALL: &str = "all";
pub const PINNED: &str = "pinned";

pub fn card_of(row: &Listed, now: i64) -> Card {
    let kind = row.kind.map(Kind::as_str).unwrap_or("text");
    Card {
        id: row.id as i32,
        kind: kind.into(),
        title: label_of(row.kind).into(),
        source: row.app.clone().unwrap_or_else(|| "—".into()).into(),
        times: if row.paste_count > 1 {
            row.paste_count.to_string().into()
        } else {
            "".into()
        },
        age: age_text(now, row.modified_at).into(),
        body: body_of(row).into(),
        mono: matches!(row.kind, Some(Kind::Json | Kind::Code | Kind::Token)),
        thumb: slint::Image::default(),
        has_thumb: row.thumb_path.is_some(),
        pinned: row.pinned,
        broken: row.broken_since.is_some(),
    }
}

fn body_of(row: &Listed) -> String {
    match &row.snippet {
        Some(snippet) => snippet.excerpt.plain(),
        None => row.preview.clone(),
    }
}

pub fn label_of(kind: Option<Kind>) -> &'static str {
    match kind {
        Some(Kind::Text) | None => "Texto",
        Some(Kind::Code) => "Código",
        Some(Kind::Json) => "JSON",
        Some(Kind::Link) => "Enlace",
        Some(Kind::Email) => "Correo",
        Some(Kind::Phone) => "Teléfono",
        Some(Kind::Color) => "Color",
        Some(Kind::Ip) => "IP",
        Some(Kind::Uuid) => "UUID",
        Some(Kind::Image) => "Imagen",
        Some(Kind::File) => "Archivo",
        Some(Kind::Folder) => "Carpeta",
        Some(Kind::Audio) => "Audio",
        Some(Kind::Video) => "Video",
        Some(Kind::Token) => "Token",
    }
}

pub fn chips_of(total: i64, pinned: i64, facets: &[Facet], selected: &str) -> Vec<Chip> {
    let mut chips = vec![
        chip(ALL, "Todo", total, selected),
        chip(PINNED, "Anclados", pinned, selected),
    ];
    let mut sorted: Vec<&Facet> = facets.iter().filter(|f| f.count > 0).collect();
    sorted.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then(a.kind.as_str().cmp(b.kind.as_str()))
    });
    for facet in sorted {
        chips.push(chip(
            facet.kind.as_str(),
            label_of(Some(facet.kind)),
            facet.count,
            selected,
        ));
    }
    chips
}

fn chip(key: &str, label: &str, count: i64, selected: &str) -> Chip {
    Chip {
        key: key.into(),
        label: label.into(),
        count: compact(count).into(),
        selected: key == selected,
    }
}

pub fn compact(count: i64) -> String {
    if count >= 10_000 {
        format!("{}k", count / 1_000)
    } else if count >= 1_000 {
        format!("{:.1}k", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}

pub fn count_text(count: i64) -> String {
    match count {
        0 => "sin elementos".into(),
        1 => "1 elemento".into(),
        n => format!("{n} elementos"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cp_core::search::{Excerpt, Segment};
    use cp_store::{FoundIn, Snippet};

    fn row(kind: Option<Kind>) -> Listed {
        Listed {
            id: 7,
            modified_at: 1_000_000,
            created_at: 1_000_000,
            kind,
            preview: "hola mundo".into(),
            app: Some("Mail".into()),
            label: None,
            color: 0,
            thumb_path: None,
            paste_count: 3,
            last_used_at: None,
            broken_since: None,
            pinned: true,
            snippet: None,
        }
    }

    #[test]
    fn a_row_becomes_the_card_the_panel_draws() {
        let card = card_of(&row(Some(Kind::Json)), 1_000_000 + 9 * 60_000);
        assert_eq!(card.id, 7);
        assert_eq!(card.kind.as_str(), "json");
        assert_eq!(card.title.as_str(), "JSON");
        assert_eq!(card.source.as_str(), "Mail");
        assert_eq!(card.times.as_str(), "3");
        assert_eq!(card.age.as_str(), "9 min");
        assert_eq!(card.body.as_str(), "hola mundo");
        assert!(card.mono);
        assert!(!card.has_thumb);
        assert!(card.pinned);
    }

    #[test]
    fn a_search_hit_shows_its_excerpt_instead_of_the_preview() {
        let mut hit = row(Some(Kind::Text));
        hit.snippet = Some(Snippet {
            found_in: FoundIn::Text,
            excerpt: Excerpt {
                segments: vec![
                    Segment {
                        text: "…la ".into(),
                        matched: false,
                    },
                    Segment {
                        text: "reunión".into(),
                        matched: true,
                    },
                    Segment {
                        text: " del jueves…".into(),
                        matched: false,
                    },
                ],
            },
        });
        let card = card_of(&hit, 2_000_000);
        assert_eq!(card.body.as_str(), "…la reunión del jueves…");
        assert!(!card.mono);
        assert_eq!(card.times.as_str(), "3");
    }

    #[test]
    fn what_was_pasted_once_or_never_carries_no_count() {
        let mut once = row(None);
        once.paste_count = 1;
        once.app = None;
        let card = card_of(&once, 1_000_000);
        assert_eq!(card.times.as_str(), "");
        assert_eq!(card.source.as_str(), "—");
        assert_eq!(card.title.as_str(), "Texto");
    }

    #[test]
    fn chips_lead_with_all_and_pinned_then_the_kinds_by_count() {
        let facets = [
            Facet {
                kind: Kind::Image,
                count: 2,
            },
            Facet {
                kind: Kind::Text,
                count: 3,
            },
            Facet {
                kind: Kind::Code,
                count: 0,
            },
            Facet {
                kind: Kind::File,
                count: 2,
            },
        ];
        let chips = chips_of(14, 1, &facets, "text");
        let keys: Vec<&str> = chips.iter().map(|c| c.key.as_str()).collect();
        assert_eq!(keys, ["all", "pinned", "text", "file", "image"]);
        assert!(chips[2].selected);
        assert!(!chips[0].selected);
        assert_eq!(chips[0].count.as_str(), "14");
    }

    #[test]
    fn big_counts_are_shortened_so_the_pill_stays_a_pill() {
        assert_eq!(compact(999), "999");
        assert_eq!(compact(1_250), "1.2k");
        assert_eq!(compact(50_000), "50k");
        assert_eq!(count_text(0), "sin elementos");
        assert_eq!(count_text(1), "1 elemento");
        assert_eq!(count_text(50_000), "50000 elementos");
    }
}
