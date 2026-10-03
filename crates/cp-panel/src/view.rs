use crate::age::age_text;
use crate::{Card, Chip};
use cp_core::kind::Kind;
use cp_core::paste_as::Form;
use cp_core::search::Excerpt;
use cp_core::token::Claims;
use cp_store::{Facet, Listed};

pub type MetaOfOne = std::collections::HashMap<String, String>;

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
        return crate::say::pick("No encontrado", "Not found");
    }
    let stale = claims
        .and_then(|claims| claims.expired_by(now / 1_000))
        .unwrap_or(false);
    if stale {
        crate::say::pick("CADUCADO", "EXPIRED")
    } else {
        ""
    }
}

fn claims_in(row: &Listed) -> Option<Claims> {
    (row.kind == Some(Kind::Token))
        .then(|| cp_core::token::claims_of(&row.preview))
        .flatten()
}

pub fn card_of(row: &Listed, now: i64, meta: Option<&MetaOfOne>) -> Card {
    let kind = row.kind.map(Kind::as_str).unwrap_or("text");
    let claims = claims_in(row);
    let token = (row.kind == Some(Kind::Token))
        .then(|| crate::token::said_of(&row.preview, now, crate::say::in_english()))
        .flatten()
        .unwrap_or_default();
    let (clock, measures) = crate::media::said_in(meta);
    let link = (row.kind == Some(Kind::Link))
        .then(|| crate::link::parts_of(&row.preview))
        .flatten();
    let folder = matches!(row.kind, Some(Kind::Folder | Kind::File))
        .then(|| crate::folder::parts_of(&row.preview))
        .flatten();
    let papers = (row.kind == Some(Kind::File))
        .then(|| crate::papers::papers_of(&row.preview))
        .flatten();
    let shape = (row.kind == Some(Kind::Json))
        .then(|| crate::shape::said_of(&row.preview, crate::say::in_english()))
        .flatten()
        .unwrap_or_else(|| crate::shape::Said {
            root: String::new(),
            counted: String::new(),
            keys: String::new(),
        });
    let body = body_of(row);
    let (lead, hit, tail) = match &row.snippet {
        Some(snippet) => parts_of(&snippet.excerpt),
        None => (String::new(), String::new(), String::new()),
    };
    let headline = headline_of(row, &body, folder.as_ref());
    let knows = Knows {
        shape: &shape,
        token: &token,
        folder: folder.as_ref(),
        papers: papers.as_ref(),
        clock: &clock,
        measures: &measures,
    };
    let aside = aside_of(row, &knows, meta);
    let (headline, aside) = top_line(headline, aside, label_of(row.kind));
    let source = row.app.clone().unwrap_or_default();
    let times = if row.paste_count > 1 {
        row.paste_count.to_string()
    } else {
        String::new()
    };
    Card {
        id: row.id as i32,
        kind: kind.into(),
        under: under_line(&aside, &source, &times).into(),
        under_named: under_line(&headline, &source, &times).into(),
        headline: headline.into(),
        name: row.label.clone().unwrap_or_default().into(),
        claims: !token.names.is_empty(),
        claim_names: token.names.into(),
        claim_values: token.values.into(),
        title: label_of(row.kind).into(),
        source: source.into(),
        times: times.clone().into(),
        age: age_text(now, touched_of(row)).into(),
        lines: open_lines_of(row, &body, now),
        shut_px: Default::default(),
        open_px: Default::default(),
        body_lines: lines_of(&body),
        squeezed: squeezed_of(&body).into(),
        shape_root: shape.root.into(),
        shape_said: shape.counted.into(),
        shape_keys: shape.keys.into(),
        media_clock: clock.into(),
        media_measures: measures.into(),
        papers_format: papers
            .as_ref()
            .map(|one| one.format.clone())
            .unwrap_or_default()
            .into(),
        papers_family: papers.as_ref().map_or("plain", |one| one.family).into(),
        folder_name: folder
            .as_ref()
            .map(|one| one.name.clone())
            .unwrap_or_default()
            .into(),
        folder_parent: folder
            .as_ref()
            .map(|one| one.parent.clone())
            .unwrap_or_default()
            .into(),
        folder_said: crate::folder::said_in(meta, crate::say::in_english()).into(),
        link_domain: link
            .as_ref()
            .map(|one| one.domain.clone())
            .unwrap_or_default()
            .into(),
        heads_group: false,
        group_said: Default::default(),
        link_path: link
            .as_ref()
            .map(|one| one.path.clone())
            .unwrap_or_default()
            .into(),
        body: body.into(),
        found: !hit.is_empty(),
        badge: badge_of(claims.as_ref()).into(),
        alert: alert_of(row, claims.as_ref(), now).into(),
        lead: lead.into(),
        hit: hit.into(),
        tail: tail.into(),
        mono: matches!(row.kind, Some(Kind::Json | Kind::Code | Kind::Token)),
        can_open: crate::opening::can_open(row.kind, &paths_of(row))
            || crate::opening::can_open_link(row.kind, &row.preview),
        paint: shown_as_colour(row).unwrap_or_default(),
        paints: shown_as_colour(row).is_some(),
        thumb: slint::Image::default(),
        has_thumb: row.thumb_path.is_some(),
        pinned: row.pinned,
        broken: row.broken_since.is_some(),
    }
}

pub fn headline_of(row: &Listed, body: &str, folder: Option<&crate::folder::Parts>) -> String {
    match row.kind {
        Some(Kind::File | Kind::Folder) => match folder {
            Some(parts) => parts.name.clone(),
            None => squeezed_of(body),
        },
        _ => squeezed_of(body),
    }
}

pub fn open_lines_of(row: &Listed, body: &str, now: i64) -> i32 {
    let claims = if row.kind == Some(Kind::Token) {
        let seen = crate::token::rows_in(&row.preview, now, crate::say::in_english());
        i32::try_from(seen).unwrap_or(0)
    } else {
        0
    };
    if claims > 0 {
        return 1 + claims + 1;
    }
    lines_of(body)
}

pub fn touched_of(row: &Listed) -> i64 {
    row.last_used_at.unwrap_or(0).max(row.modified_at)
}

pub fn under_line(said: &str, source: &str, times: &str) -> String {
    [said.trim(), source.trim(), &used_text(times)]
        .into_iter()
        .filter(|one| !one.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

fn used_text(times: &str) -> String {
    if times.trim().is_empty() {
        return String::new();
    }
    format!("×{}", times.trim())
}

pub fn top_line(headline: String, aside: String, label: &str) -> (String, String) {
    if !headline.trim().is_empty() {
        return (headline, aside);
    }
    if !aside.trim().is_empty() {
        return (aside, String::new());
    }
    (label.to_owned(), String::new())
}

pub struct Knows<'a> {
    pub shape: &'a crate::shape::Said,
    pub token: &'a crate::token::Said,
    pub folder: Option<&'a crate::folder::Parts>,
    pub papers: Option<&'a crate::papers::Papers>,
    pub clock: &'a str,
    pub measures: &'a str,
}

pub fn aside_of(row: &Listed, knows: &Knows<'_>, meta: Option<&MetaOfOne>) -> String {
    let Knows {
        shape,
        token,
        folder,
        papers,
        clock,
        measures,
    } = knows;
    let joined = |parts: &[&str]| -> String {
        parts
            .iter()
            .map(|one| one.trim())
            .filter(|one| !one.is_empty())
            .collect::<Vec<_>>()
            .join(" · ")
    };
    match row.kind {
        Some(Kind::Json) => joined(&[&shape.root, &shape.counted]),
        Some(Kind::Video | Kind::Audio) => joined(&[clock, measures]),
        Some(Kind::Image) => joined(&[measures]),
        Some(Kind::File) => {
            let format = papers.map(|one| one.format.as_str()).unwrap_or_default();
            let parent = folder.map(|one| one.parent.as_str()).unwrap_or_default();
            joined(&[format, parent.trim_end_matches(['\\', '/'])])
        }
        Some(Kind::Folder) => {
            let parent = folder.map(|one| one.parent.as_str()).unwrap_or_default();
            joined(&[
                parent.trim_end_matches(['\\', '/']),
                &crate::folder::said_in(meta, crate::say::in_english()),
            ])
        }
        Some(Kind::Token) => joined(&[&token.who, &token.life]),
        Some(Kind::Text | Kind::Code) => more_than_shown(row),
        _ => String::new(),
    }
}

fn more_than_shown(row: &Listed) -> String {
    let lines = row.preview.lines().count();
    if lines <= 1 {
        return String::new();
    }
    let rest = lines - 1;
    if crate::say::in_english() {
        format!("+{rest} more lines")
    } else {
        format!("+{rest} líneas más")
    }
}

pub const NAME_ROOM: usize = 80;

pub fn name_worth_keeping(said: &str) -> Option<String> {
    use unicode_segmentation::UnicodeSegmentation;
    let whole = said.split_whitespace().collect::<Vec<_>>().join(" ");
    let name: String = whole.graphemes(true).take(NAME_ROOM).collect();
    (!name.is_empty()).then_some(name)
}

pub fn paths_of(row: &Listed) -> Vec<String> {
    row.preview
        .lines()
        .next()
        .map(|one| vec![one.to_owned()])
        .unwrap_or_default()
}

pub fn shown_as_colour(row: &Listed) -> Option<slint::Color> {
    if row.kind != Some(Kind::Color) {
        return None;
    }
    let said = cp_core::paint::rgba_of(row.preview.lines().next()?)?;
    Some(slint::Color::from_argb_u8(
        said.alpha, said.red, said.green, said.blue,
    ))
}

pub fn squeezed_of(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut spaced = true;
    for one in text.chars() {
        if one.is_whitespace() {
            if !spaced {
                out.push(' ');
                spaced = true;
            }
            continue;
        }
        out.push(one);
        spaced = false;
    }
    if out.ends_with(' ') {
        out.pop();
    }
    out
}

pub fn body_of(row: &Listed) -> String {
    match &row.snippet {
        Some(snippet) => snippet.excerpt.plain(),
        None => row.preview.clone(),
    }
}

pub fn label_of(kind: Option<Kind>) -> &'static str {
    let (es, en) = label_pair(kind);
    crate::say::pick(es, en)
}

fn label_pair(kind: Option<Kind>) -> (&'static str, &'static str) {
    match kind {
        Some(Kind::Text) | None => ("Texto", "Text"),
        Some(Kind::Code) => ("Código", "Code"),
        Some(Kind::Json) => ("JSON", "JSON"),
        Some(Kind::Link) => ("Enlace", "Link"),
        Some(Kind::Email) => ("Correo", "Email"),
        Some(Kind::Phone) => ("Teléfono", "Phone"),
        Some(Kind::Color) => ("Color", "Colour"),
        Some(Kind::Ip) => ("IP", "IP"),
        Some(Kind::Uuid) => ("UUID", "UUID"),
        Some(Kind::Image) => ("Imagen", "Image"),
        Some(Kind::File) => ("Archivo", "File"),
        Some(Kind::Folder) => ("Carpeta", "Folder"),
        Some(Kind::Audio) => ("Audio", "Audio"),
        Some(Kind::Video) => ("Video", "Video"),
        Some(Kind::Token) => ("Token", "Token"),
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

pub fn label_of_form(form: Form) -> &'static str {
    let (es, en) = form_pair(form);
    crate::say::pick(es, en)
}

fn form_pair(form: Form) -> (&'static str, &'static str) {
    match form {
        Form::PlainText => ("En texto plano", "As plain text"),
        Form::Markdown => ("Como Markdown", "As Markdown"),
        Form::JsonPretty => ("Formateado", "Formatted"),
        Form::JsonMinified => ("Minificado", "Minified"),
        Form::JsonKeys => ("Solo las claves", "Keys only"),
        Form::JsonTable => ("Como tabla", "As a table"),
        Form::ColorHex => ("En hex", "In hex"),
        Form::ColorRgb => ("En rgb()", "In rgb()"),
        Form::ColorHsl => ("En hsl()", "In hsl()"),
        Form::ColorName => ("Por su nombre", "By its name"),
        Form::LinkMarkdown => ("En Markdown", "In Markdown"),
        Form::LinkDomain => ("Solo el dominio", "Domain only"),
        Form::LinkTitled => ("Con su título", "With its title"),
        Form::CodeOneLine => ("Sin saltos de línea", "Without line breaks"),
        Form::CodeBlock => ("Bloque Markdown", "Markdown block"),
        Form::CodeDedented => ("Sin indentación", "Without indentation"),
        Form::TokenHeader => ("Como cabecera", "As a header"),
        Form::TokenClaims => ("Su contenido", "Its contents"),
        Form::TokenCurl => ("Como curl", "As curl"),
        Form::ImageJpeg => ("Como JPEG", "As JPEG"),
        Form::ImageOcr => ("El texto que leyó", "The text it read"),
        Form::Path => ("Su ruta", "Its path"),
        Form::FileName => ("Su nombre", "Its name"),
        Form::TextQuote => ("Como cita", "As a quote"),
        Form::TextUpper => ("TODO EN MAYÚSCULAS", "ALL CAPS"),
        Form::TextLower => ("todo en minúsculas", "all lowercase"),
    }
}

pub const AS_IS: &str = "as-is";

pub fn shorthand_of(form: Form) -> Option<&'static str> {
    match form {
        Form::ColorHex => Some("hex"),
        Form::ColorRgb => Some("rgb"),
        Form::ColorHsl => Some("hsl"),
        Form::ColorName => Some(crate::say::pick("nombre", "name")),
        _ => None,
    }
}

pub fn as_is_label(kind: Option<Kind>) -> &'static str {
    let (es, en) = match kind {
        Some(Kind::Token) => ("El token", "The token"),
        Some(Kind::File) | Some(Kind::Folder) => ("El archivo", "The file"),
        Some(Kind::Image) => ("La imagen", "The image"),
        Some(Kind::Link) => ("El enlace", "The link"),
        _ => ("Tal cual", "As it is"),
    };
    crate::say::pick(es, en)
}

pub fn form_of(key: &str) -> Option<Form> {
    Form::ALL.into_iter().find(|form| form.as_str() == key)
}

pub fn chips_of(facets: &[Facet], selected: &[String]) -> Vec<Chip> {
    let mut kinds: Vec<&Facet> = facets.iter().filter(|facet| facet.count > 0).collect();
    kinds.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
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
    let of_kind = cp_core::kind::Kind::from_name(key)
        .map(|kind| crate::layout::layout_for(&[kind]))
        .unwrap_or(crate::layout::Layout::Everything);
    Chip {
        key: key.into(),
        label: label.into(),
        count: compact(count).into(),
        selected,
        has_ways: !crate::ways::ways_of(of_kind).is_empty(),
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
    empty_in(crate::say::in_english(), query, pinned, kinds)
}

fn empty_in(english: bool, query: &str, pinned: bool, kinds: bool) -> (String, String) {
    let say = |es: &'static str, en: &'static str| crate::say::pick_in(english, es, en).to_owned();
    let query = query.trim();
    if query.is_empty() {
        return match (pinned, kinds) {
            (true, _) => (
                say("No hay nada anclado", "Nothing is pinned"),
                say(
                    "Ancla lo que uses seguido y se queda a mano",
                    "Pin what you use often and it stays within reach",
                ),
            ),
            (false, true) => (
                say("No hay nada de este tipo", "Nothing of this kind"),
                say(
                    "Quita el filtro para ver todo",
                    "Drop the filter to see everything",
                ),
            ),
            (false, false) => (
                say("Todavía no hay nada", "Nothing here yet"),
                say("Lo que copies aparece aquí", "What you copy shows up here"),
            ),
        };
    }
    let shown = shorten(query);
    let asked = crate::say::pick_in(english, "Nada coincide con «{}»", "Nothing matches “{}”")
        .replace("{}", &shown);
    if !query.chars().any(char::is_alphanumeric) {
        return (
            asked,
            say(
                "La búsqueda va por palabras: los signos solos no se buscan",
                "Search goes by words: punctuation alone is not searched",
            ),
        );
    }
    if pinned || kinds {
        (
            asked,
            say(
                "Prueba con menos letras o quita el filtro",
                "Try fewer letters, or drop the filter",
            ),
        )
    } else {
        (asked, say("Prueba con menos letras", "Try fewer letters"))
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
    count_in(crate::say::in_english(), count)
}

fn count_in(english: bool, count: i64) -> String {
    match (count, english) {
        (0, false) => "sin elementos".into(),
        (0, true) => "nothing kept".into(),
        (1, false) => "1 elemento".into(),
        (1, true) => "1 item".into(),
        (n, false) => format!("{n} elementos"),
        (n, true) => format!("{n} items"),
    }
}

const FOOTER_ES: &str = if cfg!(target_os = "macos") {
    "pegar · ⌥↩: más formas"
} else {
    "pegar · alt+enter: más formas"
};
const FOOTER_EN: &str = if cfg!(target_os = "macos") {
    "paste · ⌥↩: more forms"
} else {
    "paste · alt+enter: more forms"
};

pub fn dress_words(ui: &crate::Panel) {
    use slint::ComponentHandle;
    let words = ui.global::<crate::Words>();
    words.set_hint(crate::say::pick("Busca o filtra con #", "Search, or filter with #").into());
    words.set_footer(crate::say::pick(FOOTER_ES, FOOTER_EN).into());
    words.set_paste_as(crate::say::pick("pegar como", "paste as").into());
    words.set_open_it(crate::say::pick("abrir", "open").into());
    words.set_name_it(crate::say::pick("poner nombre", "give it a name").into());
    words.set_name_room(i32::try_from(NAME_ROOM).unwrap_or(i32::MAX));
    words.set_copy_it(crate::say::pick("copiar sin pegar", "copy without pasting").into());
    words.set_pin(crate::say::pick("anclar", "pin").into());
    words.set_unpin(crate::say::pick("desanclar", "unpin").into());
    words.set_remove(crate::say::pick("borrar", "delete").into());
    words.set_one_kind(crate::say::pick("uno", "one").into());
    words.set_many_kinds(crate::say::pick("varios", "many").into());
}

#[cfg(test)]
#[path = "view_test.rs"]
mod tests;
