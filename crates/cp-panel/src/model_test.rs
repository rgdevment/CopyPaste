use super::*;

const SIZES: Metrics = Metrics {
    tall: 146.0,
    json: 112.0,
    plain: 86.0,
    found: 68.0,
    frame: 50.0,
    line: 18.0,
};

fn store_with(count: usize) -> Rc<Store> {
    let store = Store::in_memory().expect("esquema");
    for at in 0..count {
        store
            .insert_text(&format!("u{at}"), &format!("elemento {at}"), at as i64)
            .expect("insert");
    }
    Rc::new(store)
}

fn open(store: Rc<Store>, now: i64) -> Rc<Rows> {
    Rows::open(store, Filter::default(), now, SIZES)
}

#[test]
fn the_first_page_comes_with_the_model_and_the_rest_waits_for_the_view() {
    let rows = open(store_with(300), 1_000);
    assert_eq!(rows.row_count(), PAGE);
    assert_eq!(rows.loaded(), PAGE);
    assert_eq!(rows.row_data(0).map(|card| card.id), Some(300));
    let more = rows.load_page();
    assert_eq!(more, PAGE);
    assert_eq!(rows.row_count(), 2 * PAGE);
    assert_eq!(rows.load_page(), 60);
    assert_eq!(rows.load_page(), 0, "there are no more pages");
    assert_eq!(rows.row_count(), 300);
}

#[test]
fn a_row_is_built_once_and_read_back_from_the_cache() {
    let rows = open(store_with(3), 5_000);
    let first = rows.row_data(0).expect("fila");
    assert_eq!(first.body.as_str(), "elemento 2");
    assert_eq!(first.age.as_str(), "ahora");
    assert!(rows.cards.borrow()[0].is_some());
    let again = rows.row_data(0).expect("from the cache");
    assert_eq!((again.id, again.body.as_str()), (first.id, "elemento 2"));
    assert_eq!(rows.row_data(9).map(|card| card.id), None);
}

#[test]
fn the_next_page_is_wanted_only_near_the_end_and_only_once() {
    let rows = open(store_with(300), 0);
    assert!(!rows.needs_more(0));
    assert!(!rows.needs_more(PAGE - AHEAD - 1));
    assert!(rows.needs_more(PAGE - AHEAD));
    assert!(rows.needs_more(PAGE - 1));
    rows.loading.set(true);
    assert!(!rows.needs_more(PAGE - 1), "a page is already on its way");
    rows.loading.set(false);
    while rows.load_page() > 0 {}
    assert!(!rows.needs_more(299), "no queda nada que pedir");
}

#[test]
fn a_row_written_from_the_view_is_what_comes_back() {
    let rows = open(store_with(2), 0);
    let mut card = rows.row_data(1).expect("fila");
    card.body = "editado".into();
    rows.set_row_data(1, card);
    assert_eq!(
        rows.row_data(1).map(|c| c.body.to_string()),
        Some("editado".into())
    );
    rows.set_row_data(9, Card::default());
    assert_eq!(rows.row_count(), 2, "fuera de rango no crea filas");
}

#[test]
fn a_query_that_matches_nothing_is_an_empty_model_not_an_error() {
    let filter = Filter {
        query: Some("nada-de-esto".into()),
        ..Default::default()
    };
    let rows = Rows::open(store_with(10), filter, 0, SIZES);
    assert_eq!(rows.row_count(), 0);
    assert!(rows.exhausted.get());
}

#[test]
fn every_row_knows_where_it_starts_and_a_thumbnail_makes_it_taller() {
    let store = store_with(3);
    store.set_thumb(2, Some("miniatura.png"), 1).expect("thumb");
    let rows = open(store, 0);
    assert_eq!(rows.span_of(0), Some((0.0, SIZES.plain)));
    assert_eq!(rows.span_of(1), Some((SIZES.plain, SIZES.tall)));
    assert_eq!(
        rows.span_of(2),
        Some((SIZES.plain + SIZES.tall, SIZES.plain))
    );
    assert_eq!(rows.span_of(3), None, "no hay cuarta fila");
}

#[test]
fn a_thumbnail_that_does_not_load_gives_its_height_back_to_the_rows_below() {
    let store = store_with(3);
    store.set_thumb(3, Some("no-existe.png"), 2).expect("thumb");
    let rows = open(store, 0);
    assert_eq!(rows.span_of(0), Some((0.0, SIZES.tall)));
    let card = rows.row_data(0).expect("fila");
    assert!(!card.has_thumb, "the thumbnail is not on disk");
    assert_eq!(rows.span_of(0), Some((0.0, SIZES.plain)));
    assert_eq!(rows.span_of(1), Some((SIZES.plain, SIZES.plain)));
}

#[test]
fn the_open_row_grows_what_its_text_asks_and_nobody_else_pays() {
    let store = Store::in_memory().expect("esquema");
    store.insert_text("u0", "corto", 0).expect("insert");
    store
        .insert_text("u1", &"palabra ".repeat(40), 1)
        .expect("insert");
    store.insert_text("u2", "otro corto", 2).expect("insert");
    let rows = open(Rc::new(store), 0);
    let long = SIZES.frame + 6.0 * SIZES.line;

    assert_eq!(rows.span_of(1), Some((SIZES.plain, SIZES.plain)));

    rows.open_at(Some(1));
    assert_eq!(
        rows.span_of(0),
        Some((0.0, SIZES.plain)),
        "la de arriba no se mueve"
    );
    assert_eq!(
        rows.span_of(1),
        Some((SIZES.plain, long)),
        "crece lo que pide su texto"
    );
    assert_eq!(
        rows.span_of(2),
        Some((SIZES.plain + long, SIZES.plain)),
        "the one below drops by what the open one grew"
    );

    rows.open_at(Some(2));
    assert_eq!(
        rows.span_of(1),
        Some((SIZES.plain, SIZES.plain)),
        "la anterior vuelve"
    );
    assert_eq!(
        rows.span_of(2),
        Some((2.0 * SIZES.plain, SIZES.plain)),
        "un texto corto no gana nada al abrirse"
    );

    rows.open_at(None);
    assert_eq!(rows.span_of(1), Some((SIZES.plain, SIZES.plain)));
}

#[test]
fn a_thumbnail_that_failed_does_not_get_its_height_back() {
    let store = store_with(3);
    store.set_thumb(3, Some("no-existe.png"), 2).expect("thumb");
    let rows = open(store, 0);
    assert!(rows.row_data(0).is_some_and(|card| !card.has_thumb));
    assert_eq!(rows.span_of(0), Some((0.0, SIZES.plain)));

    rows.open_at(Some(0));
    rows.open_at(None);
    assert_eq!(
        rows.span_of(0),
        Some((0.0, SIZES.plain)),
        "closing it does not give back the height of a thumbnail that is not there"
    );
    assert_eq!(rows.span_of(1), Some((SIZES.plain, SIZES.plain)));
}

#[test]
fn a_row_with_a_thumbnail_does_not_open() {
    let store = store_with(2);
    store.set_thumb(2, Some("miniatura.png"), 1).expect("thumb");
    let rows = open(store, 0);
    rows.open_at(Some(0));
    assert_eq!(
        rows.span_of(0),
        Some((0.0, SIZES.tall)),
        "la miniatura ya ocupa lo suyo"
    );
}

#[test]
fn revealing_a_row_only_scrolls_when_the_row_is_out_of_sight() {
    let viewport = 400.0;
    assert_eq!(reveal(0.0, 124.0, 0.0, viewport), 0.0, "ya se ve");
    assert_eq!(
        reveal(124.0, 124.0, -124.0, viewport),
        -124.0,
        "arriba del todo"
    );
    assert_eq!(
        reveal(124.0, 124.0, -300.0, viewport),
        -116.0,
        "queda encima: baja con un respiro"
    );
    assert_eq!(
        reveal(4.0, 124.0, -20.0, viewport),
        0.0,
        "arriba del todo no se pasa de largo"
    );
    assert_eq!(
        reveal(500.0, 124.0, 0.0, viewport),
        -232.0,
        "queda debajo: sube lo justo"
    );
    assert_eq!(reveal(500.0, 124.0, 0.0, 0.0), 0.0, "sin alto no se decide");
}

#[test]
fn a_json_only_list_gives_its_rows_the_taller_shut_height() {
    let metrics = SIZES;
    let general = Filter::default();
    let only_json = Filter {
        kinds: vec![cp_core::kind::Kind::Json],
        ..Default::default()
    };
    let mixed = Filter {
        kinds: vec![cp_core::kind::Kind::Json, cp_core::kind::Kind::Text],
        ..Default::default()
    };

    assert_eq!(shut_height_for(&general, &metrics), 86.0);
    assert_eq!(shut_height_for(&only_json, &metrics), 112.0);
    assert_eq!(
        shut_height_for(&mixed, &metrics),
        86.0,
        "two kinds is the general layout, and the general height"
    );
}
