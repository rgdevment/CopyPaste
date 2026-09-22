use crate::age::age_text;
use crate::{Card, Chip};
use cp_core::kind::Kind;
use cp_core::paste_as::Form;
use cp_core::search::Excerpt;
use cp_core::token::Claims;
use cp_store::{Facet, Listed};

const LEAD: usize = 30;
const PER_LINE: usize = 57;
const SHUT: i32 = 2;
const OPEN_AT_MOST: i32 = 7;

pub fn lines_of(text: &str) -> i32 {
    let wanted = text
        .lines()
        .map(|line| line.chars().count().div_ceil(PER_LINE).max(1) as i32)
        .sum::<i32>();
    wanted.clamp(SHUT, OPEN_AT_MOST)
}

pub fn was_found(row: &Listed) -> bool {
    row.snippet
        .as_ref()
        .is_some_and(|snippet| snippet.excerpt.segments.iter().any(|one| one.matched))
}

pub fn parts_of(excerpt: &Excerpt) -> (String, String, String) {
    let at = excerpt.segments.iter().position(|one| one.matched);
    let Some(at) = at else {
        return (excerpt.plain(), String::new(), String::new());
    };
    let lead: String = excerpt.segments[..at]
        .iter()
        .map(|one| one.text.as_str())
        .collect();
    let tail: String = excerpt.segments[at + 1..]
        .iter()
        .map(|one| one.text.as_str())
        .collect();
    (trim_left(&lead), excerpt.segments[at].text.clone(), tail)
}

fn trim_left(lead: &str) -> String {
    let count = lead.chars().count();
    if count <= LEAD {
        return lead.to_owned();
    }
    let kept: String = lead.chars().skip(count - LEAD).collect();
    format!("…{}", kept.trim_start())
}

pub fn badge_of(claims: Option<&Claims>) -> &'static str {
    if claims.is_some() { "JWT" } else { "" }
}

pub fn alert_of(row: &Listed, claims: Option<&Claims>, now: i64) -> &'static str {
    if row.broken_since.is_some() {
        return "No encontrado";
    }
    let stale = claims
        .and_then(|claims| claims.expired_by(now / 1_000))
        .unwrap_or(false);
    if stale { "CADUCADO" } else { "" }
}

fn claims_in(row: &Listed) -> Option<Claims> {
    (row.kind == Some(Kind::Token))
        .then(|| cp_core::token::claims_of(&row.preview))
        .flatten()
}

pub fn card_of(row: &Listed, now: i64) -> Card {
    let kind = row.kind.map(Kind::as_str).unwrap_or("text");
    let claims = claims_in(row);
    let body = body_of(row);
    let (lead, hit, tail) = match &row.snippet {
        Some(snippet) => parts_of(&snippet.excerpt),
        None => (String::new(), String::new(), String::new()),
    };
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
        lines: lines_of(&body),
        body: body.into(),
        found: !hit.is_empty(),
        badge: badge_of(claims.as_ref()).into(),
        alert: alert_of(row, claims.as_ref(), now).into(),
        lead: lead.into(),
        hit: hit.into(),
        tail: tail.into(),
        mono: matches!(row.kind, Some(Kind::Json | Kind::Code | Kind::Token)),
        thumb: slint::Image::default(),
        has_thumb: row.thumb_path.is_some(),
        pinned: row.pinned,
        broken: row.broken_since.is_some(),
    }
}

pub fn body_of(row: &Listed) -> String {
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

pub fn kind_from_word(word: &str) -> Option<Kind> {
    let folded: String = word
        .to_lowercase()
        .chars()
        .map(|one| match one {
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' => 'u',
            other => other,
        })
        .collect();
    let name = match folded.as_str() {
        "texto" => "text",
        "codigo" => "code",
        "enlace" | "link" => "link",
        "correo" => "email",
        "telefono" => "phone",
        "imagen" => "image",
        "archivo" => "file",
        "carpeta" => "folder",
        "video" => "video",
        "audio" => "audio",
        "color" => "color",
        other => other,
    };
    Kind::from_name(name)
}

pub fn harvest(query: &str) -> (Vec<String>, String) {
    let still_typing = !query.ends_with(char::is_whitespace);
    let tokens: Vec<&str> = query.split_whitespace().collect();
    let mut taken = Vec::new();
    let mut kept = Vec::new();
    for (at, token) in tokens.iter().enumerate() {
        let last = at + 1 == tokens.len();
        match token.strip_prefix('#').and_then(kind_from_word) {
            Some(kind) if !(last && still_typing) => taken.push(kind.as_str().to_owned()),
            _ => kept.push(*token),
        }
    }
    let mut rest = kept.join(" ");
    if !rest.is_empty() && !still_typing {
        rest.push(' ');
    }
    (taken, rest)
}

pub fn sweeten(query: &str) -> String {
    query
        .split_whitespace()
        .map(|token| {
            let (sign, rest) = match token.strip_prefix('-') {
                Some(rest) => ("-", rest),
                None => ("", token),
            };
            match rest.strip_prefix('#').and_then(kind_from_word) {
                Some(kind) => format!("{sign}k:{}", kind.as_str()),
                None => token.to_owned(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(any(target_os = "windows", test))]
pub fn label_of_form(form: Form) -> &'static str {
    match form {
        Form::PlainText => "En texto plano",
        Form::Markdown => "Como Markdown",
        Form::JsonPretty => "Formateado",
        Form::JsonMinified => "Minificado",
        Form::JsonKeys => "Solo las claves",
        Form::JsonTable => "Como tabla",
        Form::ColorHex => "En hex",
        Form::ColorRgb => "En rgb()",
        Form::ColorHsl => "En hsl()",
        Form::ColorName => "Por su nombre",
        Form::LinkMarkdown => "En Markdown",
        Form::LinkDomain => "Solo el dominio",
        Form::LinkTitled => "Con su título",
        Form::CodeOneLine => "Sin saltos de línea",
        Form::CodeBlock => "Bloque Markdown",
        Form::CodeDedented => "Sin indentación",
        Form::TokenHeader => "Como cabecera",
        Form::TokenClaims => "Su contenido",
        Form::TokenCurl => "Como curl",
        Form::ImageJpeg => "Como JPEG",
        Form::ImageOcr => "El texto que leyó",
        Form::Path => "Su ruta",
        Form::FileName => "Su nombre",
        Form::TextQuote => "Como cita",
        Form::TextUpper => "TODO EN MAYÚSCULAS",
        Form::TextLower => "todo en minúsculas",
    }
}

#[cfg(any(target_os = "windows", test))]
pub const AS_IS: &str = "as-is";

#[cfg(any(target_os = "windows", test))]
pub fn shorthand_of(form: Form) -> Option<&'static str> {
    match form {
        Form::ColorHex => Some("hex"),
        Form::ColorRgb => Some("rgb"),
        Form::ColorHsl => Some("hsl"),
        Form::ColorName => Some("nombre"),
        _ => None,
    }
}

#[cfg(any(target_os = "windows", test))]
pub fn as_is_label(kind: Option<Kind>) -> &'static str {
    match kind {
        Some(Kind::Token) => "El token",
        Some(Kind::File) | Some(Kind::Folder) => "El archivo",
        Some(Kind::Image) => "La imagen",
        Some(Kind::Link) => "El enlace",
        _ => "Tal cual",
    }
}

pub fn form_of(key: &str) -> Option<Form> {
    Form::ALL.into_iter().find(|form| form.as_str() == key)
}

pub fn chips_of(facets: &[Facet], selected: &[String]) -> Vec<Chip> {
    let chosen = |facet: &Facet| selected.iter().any(|one| one == facet.kind.as_str());
    let mut kinds: Vec<&Facet> = facets.iter().filter(|facet| facet.count > 0).collect();
    kinds.sort_by(|a, b| {
        chosen(b)
            .cmp(&chosen(a))
            .then(b.count.cmp(&a.count))
            .then(a.kind.as_str().cmp(b.kind.as_str()))
    });
    kinds
        .into_iter()
        .map(|facet| {
            let key = facet.kind.as_str();
            chip(
                key,
                label_of(Some(facet.kind)),
                facet.count,
                selected.iter().any(|one| one == key),
            )
        })
        .collect()
}

fn chip(key: &str, label: &str, count: i64, selected: bool) -> Chip {
    Chip {
        key: key.into(),
        label: label.into(),
        count: compact(count).into(),
        selected,
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

const SHOWN_QUERY: usize = 24;

pub fn empty_of(query: &str, pinned: bool, kinds: bool) -> (String, String) {
    let query = query.trim();
    if query.is_empty() {
        return match (pinned, kinds) {
            (true, _) => (
                "No hay nada anclado".into(),
                "Ancla lo que uses seguido y se queda a mano".into(),
            ),
            (false, true) => (
                "No hay nada de este tipo".into(),
                "Quita el filtro para ver todo".into(),
            ),
            (false, false) => (
                "Todavía no hay nada".into(),
                "Lo que copies aparece aquí".into(),
            ),
        };
    }
    let asked = format!("Nada coincide con «{}»", shorten(query));
    if !query.chars().any(char::is_alphanumeric) {
        return (
            asked,
            "La búsqueda va por palabras: los signos solos no se buscan".into(),
        );
    }
    if pinned || kinds {
        (asked, "Prueba con menos letras o quita el filtro".into())
    } else {
        (asked, "Prueba con menos letras".into())
    }
}

fn shorten(query: &str) -> String {
    if query.chars().count() <= SHOWN_QUERY {
        return query.to_owned();
    }
    let kept: String = query.chars().take(SHOWN_QUERY).collect();
    format!("{}…", kept.trim_end())
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
    fn every_chip_sits_where_its_count_puts_it() {
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
        let chosen = ["text".to_owned()];
        let chips = chips_of(&facets, &chosen);
        let keys: Vec<&str> = chips.iter().map(|c| c.key.as_str()).collect();
        assert_eq!(
            keys,
            ["text", "file", "image"],
            "solo tipos, del que más tiene al que menos"
        );
        assert!(chips[0].selected, "el elegido se marca donde caiga");
        assert!(!chips[1].selected);
        assert_eq!(chips[0].count.as_str(), "3");
        assert!(
            chips
                .iter()
                .all(|one| one.key != "all" && one.key != "pinned"),
            "todo y anclados no son tipos y no viven aquí"
        );

        assert!(chips_of(&[], &[]).is_empty(), "sin facetas no hay fila");
    }

    fn excerpt_of(parts: &[(&str, bool)]) -> Excerpt {
        Excerpt {
            segments: parts
                .iter()
                .map(|(text, matched)| Segment {
                    text: (*text).into(),
                    matched: *matched,
                })
                .collect(),
        }
    }

    #[test]
    fn the_excerpt_breaks_into_what_goes_before_the_hit_and_what_follows() {
        let excerpt = excerpt_of(&[
            ("confirmamos la ", false),
            ("reunión", true),
            (" del jueves a las 16:30", false),
        ]);
        assert_eq!(
            parts_of(&excerpt),
            (
                "confirmamos la ".into(),
                "reunión".into(),
                " del jueves a las 16:30".into()
            )
        );
    }

    #[test]
    fn an_excerpt_with_nothing_matched_is_all_text_and_no_hit() {
        let (lead, hit, tail) = parts_of(&excerpt_of(&[("sin coincidencias", false)]));
        assert_eq!(lead, "sin coincidencias");
        assert!(hit.is_empty(), "sin hit no hay nada que resaltar");
        assert!(tail.is_empty());
    }

    #[test]
    fn a_long_lead_is_cut_from_the_left_so_the_hit_does_not_fall_off() {
        let long = "una entrada muy larga que empuja el término lejos del principio ";
        let (lead, hit, _) = parts_of(&excerpt_of(&[(long, false), ("término", true)]));
        assert!(lead.starts_with('…'), "se corta por la izquierda: {lead}");
        assert!(lead.chars().count() <= LEAD + 1);
        assert!(lead.ends_with("lejos del principio "));
        assert_eq!(hit, "término");
    }

    #[test]
    fn a_row_that_matched_carries_its_three_parts() {
        let mut row = row(Some(Kind::Text));
        row.snippet = Some(Snippet {
            found_in: FoundIn::Text,
            excerpt: excerpt_of(&[("antes de ", false), ("esto", true), (" y después", false)]),
        });
        let card = card_of(&row, 1_000_000);
        assert!(card.found);
        assert_eq!(card.lead.as_str(), "antes de ");
        assert_eq!(card.hit.as_str(), "esto");
        assert_eq!(card.tail.as_str(), " y después");
    }

    const EXPIRED: &str =
        "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.eyJleHAiOjE3MDAwMDAwMDAsInN1YiI6ImNwLTMifQ.sig";

    #[test]
    fn a_card_wears_what_is_wrong_with_it() {
        let mut broken = row(Some(Kind::File));
        broken.broken_since = Some(10);
        assert_eq!(alert_of(&broken, None, 1_000), "No encontrado");
        assert_eq!(badge_of(None), "");

        let plain = row(Some(Kind::Text));
        assert_eq!(alert_of(&plain, None, 1_000), "");

        let mut token = row(Some(Kind::Token));
        token.preview = EXPIRED.into();
        let claims = claims_in(&token);
        assert_eq!(
            badge_of(claims.as_ref()),
            "JWT",
            "un token legible se marca"
        );
        assert_eq!(
            alert_of(&token, claims.as_ref(), 1_700_000_001_000),
            "CADUCADO",
            "pasado el exp, se dice"
        );
        assert_eq!(
            alert_of(&token, claims.as_ref(), 1_600_000_000_000),
            "",
            "antes del exp, no"
        );

        let mut fake = row(Some(Kind::Token));
        fake.preview = "no-es-un-token".into();
        assert_eq!(
            badge_of(claims_in(&fake).as_ref()),
            "",
            "lo que no se lee no lleva insignia"
        );
    }

    #[test]
    fn a_card_asks_for_as_many_lines_as_its_text_needs() {
        assert_eq!(lines_of("#FF8800"), 2, "lo corto no crece al abrirse");
        assert_eq!(lines_of(""), 2);
        assert_eq!(lines_of(&"a".repeat(57)), 2);
        assert_eq!(
            lines_of(&"a".repeat(58)),
            2,
            "dos líneas siguen siendo el mínimo"
        );
        assert_eq!(lines_of(&"a".repeat(57 * 3)), 3);
        assert_eq!(lines_of(&"a".repeat(57 * 20)), 7, "y hay un techo");
        assert_eq!(
            lines_of(
                "uno
dos
tres
cuatro"
            ),
            4,
            "los saltos cuentan"
        );
    }

    #[test]
    fn a_hash_in_the_search_box_becomes_a_kind_filter() {
        assert_eq!(sweeten("#ip"), "k:ip");
        assert_eq!(sweeten("#imagen playa"), "k:image playa");
        assert_eq!(sweeten("#Código git"), "k:code git");
        assert_eq!(sweeten("#vídeo"), "k:video");
        assert_eq!(sweeten("reunión jueves"), "reunión jueves");
        assert_eq!(
            sweeten("#loquesea"),
            "#loquesea",
            "lo que no es una clase se busca"
        );
        assert_eq!(sweeten("#carpeta #texto"), "k:folder k:text");
        assert_eq!(
            sweeten("-#imagen playa"),
            "-k:image playa",
            "y se puede excluir"
        );
        assert_eq!(
            sweeten("-reunión"),
            "-reunión",
            "el menos suelto no se toca"
        );
    }

    #[test]
    fn every_form_has_a_name_and_answers_to_its_key() {
        for form in Form::ALL {
            let label = label_of_form(form);
            assert!(!label.is_empty(), "{form:?} sin nombre");
            assert_eq!(form_of(form.as_str()), Some(form));
        }
        assert_eq!(form_of("no-existe"), None);
    }

    #[test]
    fn an_empty_list_says_why_it_is_empty() {
        assert_eq!(
            empty_of("", false, false),
            (
                "Todavía no hay nada".into(),
                "Lo que copies aparece aquí".into()
            )
        );
        assert_eq!(empty_of("  ", true, false).0, "No hay nada anclado");
        assert_eq!(empty_of("", false, true).0, "No hay nada de este tipo");

        let (title, hint) = empty_of(",", false, false);
        assert_eq!(title, "Nada coincide con «,»");
        assert!(hint.contains("por palabras"), "la coma no es una palabra");

        let (title, hint) = empty_of("reunion", false, true);
        assert_eq!(title, "Nada coincide con «reunion»");
        assert!(hint.contains("quita el filtro"));

        assert_eq!(
            empty_of("reunion", false, false).1,
            "Prueba con menos letras"
        );
        assert_eq!(
            empty_of(
                "una consulta larguísima que no cabe en la tarjeta",
                false,
                false
            )
            .0,
            "Nada coincide con «una consulta larguísima…»"
        );
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
