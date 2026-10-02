use super::*;

const TOKEN: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJjbG91ZGZsYXJlIiwic3ViIjoicm9kcmlnbyIsImV4cCI6MTc5MDk1MjkxNn0.firma";
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
        group: String::new(),
        snippet: None,
    }
}

#[test]
fn a_row_becomes_the_card_the_panel_draws() {
    let card = card_of(&row(Some(Kind::Json)), 1_000_000 + 9 * 60_000, None);
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
    let card = card_of(&hit, 2_000_000, None);
    assert_eq!(card.body.as_str(), "…la reunión del jueves…");
    assert!(!card.mono);
    assert_eq!(card.times.as_str(), "3");
}

#[test]
fn what_was_pasted_once_or_never_carries_no_count() {
    let mut once = row(None);
    once.paste_count = 1;
    once.app = None;
    let card = card_of(&once, 1_000_000, None);
    assert_eq!(card.times.as_str(), "");
    assert_eq!(
        card.source.as_str(),
        "",
        "a card from nowhere says nothing, because a dash beside a separator reads as noise"
    );
    assert_eq!(card.under.as_str(), "");
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
    let card = card_of(&row, 1_000_000, None);
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

#[test]
fn a_closed_card_shows_content_where_the_indentation_was() {
    let pretty = "{\n  \"annotations\": [\n    {\n      \"id\": 1,\n      \"note\": \"revisar\"\n    }\n  ]\n}";
    let squeezed = squeezed_of(pretty);
    assert!(
        squeezed.starts_with("{ \"annotations\": [ { \"id\": 1, \"note\": \"revisar\" }"),
        "«{squeezed}»"
    );
    assert!(!squeezed.contains('\n'));
    assert!(!squeezed.contains("  "), "no run of spaces survives");
}

#[test]
fn squeezing_leaves_a_single_line_alone() {
    assert_eq!(
        squeezed_of("ya viene en una linea"),
        "ya viene en una linea"
    );
    assert_eq!(squeezed_of(""), "");
    assert_eq!(squeezed_of("   \n\t  "), "");
}

#[test]
fn squeezing_keeps_the_words_and_their_order() {
    let said = "primero\n\n\tsegundo   tercero\r\ncuarto ";
    assert_eq!(squeezed_of(said), "primero segundo tercero cuarto");
}

#[test]
fn a_colour_card_carries_the_colour_it_is() {
    let mut said = row(Some(Kind::Color));
    said.preview = "#FF8800".into();
    let card = card_of(&said, 1_000_000, None);
    assert!(card.paints, "the only type whose content is its appearance");
    assert_eq!(card.paint.red(), 0xFF);
    assert_eq!(card.paint.green(), 0x88);
    assert_eq!(card.paint.blue(), 0x00);
    assert_eq!(card.paint.alpha(), 0xFF);
}

#[test]
fn the_colour_is_read_from_the_first_line_whatever_shape_it_has() {
    for (said, red, green, blue) in [
        ("#F80", 0xFF, 0x88, 0x00),
        ("rgb(1, 2, 3)", 1, 2, 3),
        ("hsl(120, 100%, 50%)", 0, 255, 0),
        ("#FF8800\ntrailing noise", 0xFF, 0x88, 0x00),
    ] {
        let mut one = row(Some(Kind::Color));
        one.preview = said.into();
        let card = card_of(&one, 1_000_000, None);
        assert!(card.paints, "«{said}»");
        assert_eq!(
            (card.paint.red(), card.paint.green(), card.paint.blue()),
            (red, green, blue),
            "«{said}»"
        );
    }
}

#[test]
fn nothing_else_pretends_to_be_a_colour() {
    for kind in [Kind::Text, Kind::Json, Kind::Link, Kind::File, Kind::Token] {
        let mut said = row(Some(kind));
        said.preview = "#FF8800".into();
        assert!(
            !card_of(&said, 1_000_000, None).paints,
            "{kind:?} holds text that looks like a colour, and is not one"
        );
    }
    let mut unreadable = row(Some(Kind::Color));
    unreadable.preview = "not a colour at all".into();
    assert!(
        !card_of(&unreadable, 1_000_000, None).paints,
        "and a colour nobody can parse draws no swatch"
    );
}

#[test]
fn a_file_card_knows_its_name_apart_from_its_folder() {
    let mut said = row(Some(Kind::File));
    said.preview = r"D:\Mario\Downloads\b774f629-9376-419f-9a7b-d390e7f775c2.pdf".into();
    let card = card_of(&said, 1_000_000, None);
    assert_eq!(
        card.folder_name.as_str(),
        "b774f629-9376-419f-9a7b-d390e7f775c2.pdf",
        "the name is what says which file it is, so it cannot be the part that gets cut"
    );
    assert_eq!(card.folder_parent.as_str(), r"D:\Mario\Downloads\");
    assert_eq!(card.papers_format.as_str(), "PDF");
}

#[test]
fn a_file_with_no_extension_still_says_which_file_it_is() {
    let mut said = row(Some(Kind::File));
    said.preview = r"D:\Mario\LICENSE".into();
    let card = card_of(&said, 1_000_000, None);
    assert_eq!(card.folder_name.as_str(), "LICENSE");
    assert_eq!(
        card.papers_format.as_str(),
        "",
        "there is no format to show, so the card draws a glyph instead of an empty box"
    );
}

#[test]
fn the_second_line_says_what_each_type_already_knows() {
    let mut json = row(Some(Kind::Json));
    json.preview = r#"{"a": 1, "b": {"c": 2}}"#.into();
    let said = card_of(&json, 1_000_000, None).under.to_string();
    assert!(
        !said.is_empty(),
        "JSON knows its keys and its depth: «{said}»"
    );

    let mut file = row(Some(Kind::File));
    file.preview = r"D:\Mario\Downloads\one.yml".into();
    let said = card_of(&file, 1_000_000, None).under.to_string();
    assert!(said.contains("YML"), "«{said}»");
    assert!(said.contains("Downloads"), "«{said}»");
    assert!(
        !said.ends_with('\\') && !said.ends_with('/'),
        "the trailing separator is noise: «{said}»"
    );

    let mut folder = row(Some(Kind::Folder));
    folder.preview = r"D:\Mario\Downloads".into();
    let said = card_of(&folder, 1_000_000, None).under.to_string();
    assert!(said.contains("Mario"), "the folder it lives in: «{said}»");
}

#[test]
fn the_first_line_of_a_file_is_its_name_not_its_path() {
    let mut said = row(Some(Kind::File));
    said.preview = r"D:\Mario\Downloads\a-very-long-name.yml".into();
    assert_eq!(
        card_of(&said, 1_000_000, None).headline.as_str(),
        "a-very-long-name.yml"
    );
    let mut folder = row(Some(Kind::Folder));
    folder.preview = r"D:\Mario\Downloads".into();
    assert_eq!(
        card_of(&folder, 1_000_000, None).headline.as_str(),
        "Downloads"
    );
}

#[test]
fn a_text_of_one_line_says_nothing_more_and_a_long_one_says_how_much_more() {
    let mut one = row(Some(Kind::Text));
    one.preview = "just one line".into();
    assert_eq!(
        card_of(&one, 1_000_000, None).under.as_str(),
        "Mail · ×3",
        "there is nothing the card is hiding, so only where it came from and how often it was used"
    );
    let mut many = row(Some(Kind::Text));
    many.preview = "first\nsecond\nthird".into();
    let said = card_of(&many, 1_000_000, None).under.to_string();
    assert!(said.contains('2'), "two more lines are waiting: «{said}»");
}

#[test]
fn what_needs_no_second_line_gets_none() {
    for kind in [Kind::Uuid, Kind::Ip, Kind::Email, Kind::Phone, Kind::Color] {
        let mut said = row(Some(kind));
        said.preview = "something".into();
        assert_eq!(
            card_of(&said, 1_000_000, None).under.as_str(),
            "Mail · ×3",
            "{kind:?}: the glyph already says what it is, so only the app and the count are left"
        );
    }
}

#[test]
fn a_name_of_only_spaces_clears_the_name() {
    assert_eq!(super::name_worth_keeping("   	  "), None);
    assert_eq!(super::name_worth_keeping(""), None);
}

#[test]
fn a_name_is_one_line_however_it_was_pasted() {
    assert_eq!(
        super::name_worth_keeping(
            "  claves   de
  produccion 
"
        ),
        Some("claves de produccion".to_owned()),
        "a label reads on one line, so the line breaks of a paste do not survive it"
    );
}

#[test]
fn a_name_longer_than_the_row_is_cut_to_the_room_there_is() {
    let said = "a".repeat(super::NAME_ROOM + 40);
    let kept = super::name_worth_keeping(&said).expect("a name");
    assert_eq!(kept.chars().count(), super::NAME_ROOM);
}

#[test]
fn a_name_is_cut_by_letters_and_not_by_bytes() {
    let said = "ñ".repeat(super::NAME_ROOM + 5);
    let kept = super::name_worth_keeping(&said).expect("a name");
    assert_eq!(
        kept.chars().count(),
        super::NAME_ROOM,
        "cutting a two byte letter in half would not even be text"
    );
}

#[test]
fn a_card_with_nothing_to_quote_puts_what_it_knows_on_the_first_line() {
    let (first, second) = super::top_line(String::new(), "476×576".to_owned(), "Imagen");
    assert_eq!(first, "476×576");
    assert_eq!(
        second, "",
        "una captura no tiene texto que citar, y una primera linea vacia se lee como una tarjeta rota"
    );
}

#[test]
fn a_card_with_something_to_quote_keeps_both_lines_where_they_were() {
    let (first, second) =
        super::top_line("git add -A".to_owned(), "+3 líneas más".to_owned(), "Texto");
    assert_eq!(first, "git add -A");
    assert_eq!(second, "+3 líneas más");
}

#[test]
fn a_first_line_of_only_spaces_counts_as_empty() {
    let (first, second) = super::top_line(
        "   
 "
        .to_owned(),
        "2 claves".to_owned(),
        "JSON",
    );
    assert_eq!(first, "2 claves");
    assert_eq!(second, "");
}

#[test]
fn the_card_asks_for_the_same_lines_the_model_reserves() {
    let store = cp_store::Store::in_memory().expect("esquema");
    let token =
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJhIiwic3ViIjoiYiIsImV4cCI6MX0.firma";
    store.insert_text("t", token, 1).expect("insert");
    let page = store
        .list(&cp_store::Filter::default(), 10, None)
        .expect("pagina");
    let row = page.rows.first().expect("una fila");
    let body = super::body_of(row);
    let card = super::card_of(row, 2, None);
    assert_eq!(
        card.lines,
        super::open_lines_of(row, &body),
        "el delegado dibuja con card.lines y el modelo reserva con la misma cuenta: si se separan, la tarjeta se corta"
    );
    assert!(
        card.lines > super::lines_of(&body),
        "un token con claves pide mas alto que su texto suelto"
    );
}

#[test]
fn the_second_line_keeps_the_app_and_what_the_card_says() {
    assert_eq!(super::under_line("476×576", "Orca", ""), "476×576 · Orca");
}

#[test]
fn a_card_from_nowhere_does_not_drag_a_dangling_separator() {
    assert_eq!(
        super::under_line("476×576", "", ""),
        "476×576",
        "a card with no application behind it used to read «476×576 · —», and a dash is not a name"
    );
    assert_eq!(super::under_line("", "Orca", ""), "Orca");
    assert_eq!(super::under_line("  ", "  ", ""), "");
}

#[test]
fn a_named_card_still_says_what_it_holds() {
    let under = super::under_line("cargo build --workspace", "Orca", "");
    assert!(
        under.starts_with("cargo build --workspace"),
        "naming a card must not hide what is in it: {under}"
    );
}

#[test]
fn the_age_a_card_shows_is_the_one_the_list_is_sorted_by() {
    let mut said = row(Some(Kind::Image));
    said.modified_at = 1_000;
    said.last_used_at = Some(500_000);
    assert_eq!(
        super::touched_of(&said),
        500_000,
        "the list orders by MAX(modified_at, last_used_at), so a card that shows only modified_at \
         sits above one that reads newer and the order looks made up"
    );

    said.last_used_at = None;
    assert_eq!(super::touched_of(&said), 1_000);

    said.last_used_at = Some(1);
    assert_eq!(
        super::touched_of(&said),
        1_000,
        "pasting something long after copying it must not make it look older"
    );
}

#[test]
fn the_lines_an_open_card_asks_for_are_its_body_plus_its_table_and_a_gap() {
    let mut text = row(Some(Kind::Text));
    text.preview = "una sola linea".into();
    let body = super::body_of(&text);
    assert_eq!(
        super::open_lines_of(&text, &body),
        super::lines_of(&body),
        "with no table there is nothing to add"
    );

    let mut token = row(Some(Kind::Token));
    token.preview = TOKEN.into();
    let body = super::body_of(&token);
    let rows = i32::try_from(crate::token::rows_in(TOKEN)).expect("a few");
    assert!(rows > 0);
    assert_eq!(
        super::open_lines_of(&token, &body),
        super::lines_of(&body) + rows + 1,
        "the table takes a line per claim and one more for the gap above it"
    );
}

#[test]
fn only_a_token_is_read_as_a_token() {
    let mut said = row(Some(Kind::Text));
    said.preview = TOKEN.into();
    let card = card_of(&said, 1_000_000, None);
    assert!(
        !card.claims,
        "a text that happens to look like a JWT is still a text, and must not grow a table"
    );
    said.kind = Some(Kind::Token);
    assert!(card_of(&said, 1_000_000, None).claims);
}

#[test]
fn what_can_be_opened_is_a_path_or_a_link_and_not_everything_else() {
    let mut file = row(Some(Kind::File));
    file.preview = r"D:\Mario\uno.yml".into();
    assert!(card_of(&file, 1_000_000, None).can_open);

    let mut link = row(Some(Kind::Link));
    link.preview = "https://ejemplo.cl/a".into();
    assert!(card_of(&link, 1_000_000, None).can_open);

    let mut plain = row(Some(Kind::Text));
    plain.preview = "nada que abrir".into();
    assert!(
        !card_of(&plain, 1_000_000, None).can_open,
        "a button that cannot do anything is worse than no button"
    );
}

#[test]
fn each_type_puts_its_own_fact_on_the_second_line() {
    let mut video = row(Some(Kind::Video));
    video.preview = "clip.mp4".into();
    let meta = [(crate::media::DURATION.to_owned(), "90000".to_owned())]
        .into_iter()
        .collect::<super::MetaOfOne>();
    let said = card_of(&video, 1_000_000, Some(&meta)).under.to_string();
    assert!(said.contains("1:30"), "a video says its clock: «{said}»");

    let mut image = row(Some(Kind::Image));
    image.preview = String::new();
    let meta = [
        (crate::media::WIDTH.to_owned(), "800".to_owned()),
        (crate::media::HEIGHT.to_owned(), "600".to_owned()),
    ]
    .into_iter()
    .collect::<super::MetaOfOne>();
    let said = card_of(&image, 1_000_000, Some(&meta));
    assert!(
        said.headline.as_str().contains("800"),
        "an image has nothing to quote, so its measures take the first line: «{}»",
        said.headline
    );

    let mut token = row(Some(Kind::Token));
    token.preview = TOKEN.into();
    let said = card_of(&token, 1_000_000, None).under.to_string();
    assert!(
        said.contains("rodrigo") && said.contains("cloudflare"),
        "a token says who it is for and who signed it: «{said}»"
    );
}

#[test]
fn a_card_pasted_more_than_once_carries_its_count_on_the_second_line() {
    assert_eq!(
        super::under_line("476×576", "Orca", "3"),
        "476×576 · Orca · ×3",
        "the count only shows for what was pasted again, so it earns its place"
    );
    assert_eq!(super::under_line("", "", "7"), "×7");
    assert_eq!(
        super::under_line("476×576", "Orca", ""),
        "476×576 · Orca",
        "pasted once or never says nothing"
    );
}

#[test]
fn the_count_reaches_the_general_list_and_not_only_the_views_by_kind() {
    let mut said = row(Some(Kind::Text));
    said.preview = "algo que pegas mucho".into();
    said.paste_count = 4;
    let card = card_of(&said, 1_000_000, None);
    assert_eq!(card.times.as_str(), "4");
    assert!(
        card.under.as_str().contains("×4"),
        "the general list has no head row, so a count that lives only there is a count nobody \
         sees: «{}»",
        card.under
    );
}
