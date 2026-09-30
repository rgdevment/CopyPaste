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
    assert!(
        chips[0].selected,
        "the chosen one is marked wherever it lands"
    );
    assert!(!chips[1].selected);
    assert_eq!(chips[0].count.as_str(), "3");
    assert!(
        chips
            .iter()
            .all(|one| one.key != "all" && one.key != "pinned"),
        "todo y anclados no son tipos y no viven aquí"
    );

    assert!(
        chips_of(&[], &[]).is_empty(),
        "with no facets there is no row"
    );
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
    assert!(hit.is_empty(), "with no hit there is nothing to highlight");
    assert!(tail.is_empty());
}

#[test]
fn a_long_lead_is_cut_from_the_left_so_the_hit_does_not_fall_off() {
    let long = "una entrada muy larga que empuja el término lejos del principio ";
    let (lead, hit, _) = parts_of(&excerpt_of(&[(long, false), ("término", true)]));
    assert!(lead.starts_with('…'), "it is cut from the left: {lead}");
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
    assert_eq!(
        lines_of("#FF8800"),
        2,
        "what is short does not grow when it opens"
    );
    assert_eq!(lines_of(""), 2);
    assert_eq!(lines_of(&"a".repeat(57)), 2);
    assert_eq!(
        lines_of(&"a".repeat(58)),
        2,
        "two lines are still the floor"
    );
    assert_eq!(lines_of(&"a".repeat(57 * 3)), 3);
    assert_eq!(lines_of(&"a".repeat(57 * 20)), 7, "and there is a ceiling");
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
fn every_form_answers_to_its_key() {
    for form in Form::ALL {
        assert_eq!(form_of(form.as_str()), Some(form));
    }
    assert_eq!(form_of("no-existe"), None);
}

#[test]
fn nothing_the_panel_says_is_left_untranslated() {
    for form in Form::ALL {
        let (es, en) = form_pair(form);
        assert!(
            !es.is_empty() && !en.is_empty(),
            "{form:?} was left without a name"
        );
    }
    for kind in [None, Some(Kind::Text), Some(Kind::Code), Some(Kind::Image)] {
        let (es, en) = label_pair(kind);
        assert!(
            !es.is_empty() && !en.is_empty(),
            "{kind:?} was left without a name"
        );
    }
}

#[test]
fn what_is_written_in_english_is_not_the_spanish_copied_over() {
    let english: Vec<&str> = Form::ALL
        .into_iter()
        .map(|form| form_pair(form).1)
        .collect();
    let shared = Form::ALL
        .into_iter()
        .filter(|form| {
            let (es, en) = form_pair(*form);
            es == en
        })
        .count();
    assert!(shared <= 3, "too many forms left untranslated: {shared}");
    assert!(english.iter().all(|said| !said.contains('ó')));
}

#[test]
fn an_empty_list_explains_itself_in_both_tongues() {
    for english in [false, true] {
        let (title, hint) = empty_in(english, "", false, false);
        assert!(!title.is_empty() && !hint.is_empty());
        let (asked, _) = empty_in(english, "perdido", false, false);
        assert!(asked.contains("perdido"), "{asked}");
    }
    assert_ne!(
        empty_in(false, "", true, false).0,
        empty_in(true, "", true, false).0
    );
}

#[test]
fn the_count_reads_the_way_each_tongue_counts() {
    assert_eq!(count_in(false, 0), "sin elementos");
    assert_eq!(count_in(false, 1), "1 elemento");
    assert_eq!(count_in(false, 7), "7 elementos");
    assert_eq!(count_in(true, 0), "nothing kept");
    assert_eq!(count_in(true, 1), "1 item");
    assert_eq!(count_in(true, 7), "7 items");
}

#[test]
fn every_form_has_a_name_for_the_sheet() {
    for form in Form::ALL {
        assert!(!label_of_form(form).is_empty(), "{form:?} has no name");
    }
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
    assert!(hint.contains("por palabras"), "a comma is not a word");

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
