use crate::Card;
use crate::view::{body_of, card_of, lines_of};
use cp_store::{Cursor, Filter, Listed, Store};
use slint::{Model, ModelNotify, ModelTracker};
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

pub const PAGE: usize = 120;
const AHEAD: usize = 40;

#[derive(Debug, Clone, Copy)]
pub struct Metrics {
    pub tall: f32,
    pub plain: f32,
    pub found: f32,
    pub frame: f32,
    pub line: f32,
}

pub fn reveal(top: f32, span: f32, scroll: f32, viewport: f32) -> f32 {
    if viewport <= 0.0 {
        return scroll;
    }
    let seen = -scroll;
    if top < seen {
        -top
    } else if top + span > seen + viewport {
        viewport - top - span
    } else {
        scroll
    }
}

pub struct Rows {
    store: Rc<Store>,
    filter: Filter,
    now: i64,
    metrics: Metrics,
    rows: RefCell<Vec<Listed>>,
    cards: RefCell<Vec<Option<Card>>>,
    tops: RefCell<Vec<f32>>,
    open: Cell<Option<usize>>,
    next: Cell<Option<Cursor>>,
    exhausted: Cell<bool>,
    loading: Cell<bool>,
    notify: ModelNotify,
    weak: RefCell<Weak<Rows>>,
}

impl Rows {
    pub fn open(store: Rc<Store>, filter: Filter, now: i64, metrics: Metrics) -> Rc<Self> {
        let rows = Rc::new(Self {
            store,
            filter,
            now,
            metrics,
            rows: RefCell::new(Vec::new()),
            cards: RefCell::new(Vec::new()),
            tops: RefCell::new(vec![0.0]),
            open: Cell::new(None),
            next: Cell::new(None),
            exhausted: Cell::new(false),
            loading: Cell::new(false),
            notify: ModelNotify::default(),
            weak: RefCell::new(Weak::new()),
        });
        *rows.weak.borrow_mut() = Rc::downgrade(&rows);
        rows.load_page();
        rows
    }

    pub fn loaded(&self) -> usize {
        self.rows.borrow().len()
    }

    pub fn open_at(&self, index: Option<usize>) {
        if self.open.get() == index {
            return;
        }
        if let Some(was) = self.open.get() {
            let back = self.base_of(was);
            self.resize(was, back);
        }
        self.open.set(index);
        if let Some(now) = index
            && !self.tall_at(now)
        {
            let open = self.open_of(now);
            self.resize(now, open);
        }
    }

    fn open_of(&self, index: usize) -> f32 {
        let rows = self.rows.borrow();
        let Some(row) = rows.get(index) else {
            return self.metrics.plain;
        };
        self.metrics.frame + lines_of(&body_of(row)) as f32 * self.metrics.line
    }

    fn tall_at(&self, index: usize) -> bool {
        self.rows
            .borrow()
            .get(index)
            .is_some_and(|row| row.thumb_path.is_some())
    }

    fn base_of(&self, index: usize) -> f32 {
        self.rows
            .borrow()
            .get(index)
            .map_or(self.metrics.plain, |row| self.height_of(row))
    }

    pub fn span_of(&self, index: usize) -> Option<(f32, f32)> {
        let tops = self.tops.borrow();
        let top = *tops.get(index)?;
        let next = *tops.get(index + 1)?;
        Some((top, next - top))
    }

    fn height_of(&self, row: &Listed) -> f32 {
        if row.thumb_path.is_some() {
            self.metrics.tall
        } else if row.snippet.is_some() {
            self.metrics.found
        } else {
            self.metrics.plain
        }
    }

    fn resize(&self, index: usize, height: f32) {
        let mut tops = self.tops.borrow_mut();
        let (Some(&top), Some(&next)) = (tops.get(index), tops.get(index + 1)) else {
            return;
        };
        let delta = height - (next - top);
        if delta == 0.0 {
            return;
        }
        for top in tops.iter_mut().skip(index + 1) {
            *top += delta;
        }
    }

    fn load_page(&self) -> usize {
        if self.exhausted.get() {
            return 0;
        }
        let page = match self.store.list(&self.filter, PAGE, self.next.get()) {
            Ok(page) => page,
            Err(why) => {
                eprintln!("la lista no se pudo leer: {why}");
                self.exhausted.set(true);
                return 0;
            }
        };
        let added = page.rows.len();
        self.next.set(page.next);
        if page.next.is_none() {
            self.exhausted.set(true);
        }
        let start = self.rows.borrow().len();
        self.cards.borrow_mut().extend((0..added).map(|_| None));
        {
            let mut tops = self.tops.borrow_mut();
            let mut at = tops.last().copied().unwrap_or(0.0);
            for row in &page.rows {
                at += self.height_of(row);
                tops.push(at);
            }
        }
        self.rows.borrow_mut().extend(page.rows);
        self.notify.row_added(start, added);
        added
    }

    fn needs_more(&self, index: usize) -> bool {
        !self.exhausted.get() && !self.loading.get() && index + AHEAD >= self.rows.borrow().len()
    }

    fn fetch_ahead(&self) {
        self.loading.set(true);
        let weak = self.weak.borrow().clone();
        slint::Timer::single_shot(std::time::Duration::ZERO, move || {
            if let Some(rows) = weak.upgrade() {
                rows.load_page();
                rows.loading.set(false);
            }
        });
    }

    fn materialize(&self, index: usize) -> Option<Card> {
        let cached = self.cards.borrow().get(index).cloned().flatten();
        if let Some(card) = cached {
            return Some(card);
        }
        let rows = self.rows.borrow();
        let row = rows.get(index)?;
        let without_thumb = if row.snippet.is_some() {
            self.metrics.found
        } else {
            self.metrics.plain
        };
        let mut card = card_of(row, self.now);
        if let Some(path) = &row.thumb_path
            && let Ok(image) = slint::Image::load_from_path(std::path::Path::new(path))
        {
            card.thumb = image;
        } else {
            card.has_thumb = false;
        }
        drop(rows);
        if !card.has_thumb {
            self.resize(index, without_thumb);
        }
        if let Some(slot) = self.cards.borrow_mut().get_mut(index) {
            *slot = Some(card.clone());
        }
        Some(card)
    }
}

impl Model for Rows {
    type Data = Card;

    fn row_count(&self) -> usize {
        self.rows.borrow().len()
    }

    fn row_data(&self, index: usize) -> Option<Card> {
        if self.needs_more(index) {
            self.fetch_ahead();
        }
        self.materialize(index)
    }

    fn set_row_data(&self, index: usize, card: Card) {
        if let Some(slot) = self.cards.borrow_mut().get_mut(index) {
            *slot = Some(card);
        }
        self.notify.row_changed(index);
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.notify
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZES: Metrics = Metrics {
        tall: 146.0,
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
        assert_eq!(rows.load_page(), 0, "no hay más páginas");
        assert_eq!(rows.row_count(), 300);
    }

    #[test]
    fn a_row_is_built_once_and_read_back_from_the_cache() {
        let rows = open(store_with(3), 5_000);
        let first = rows.row_data(0).expect("fila");
        assert_eq!(first.body.as_str(), "elemento 2");
        assert_eq!(first.age.as_str(), "ahora");
        assert!(rows.cards.borrow()[0].is_some());
        let again = rows.row_data(0).expect("de la caché");
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
        assert!(!rows.needs_more(PAGE - 1), "ya hay una página en camino");
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
        assert!(!card.has_thumb, "la miniatura no está en disco");
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
            "la de abajo baja lo que creció la abierta"
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
            -124.0,
            "queda encima"
        );
        assert_eq!(
            reveal(500.0, 124.0, 0.0, viewport),
            -224.0,
            "queda debajo: sube lo justo"
        );
        assert_eq!(reveal(500.0, 124.0, 0.0, 0.0), 0.0, "sin alto no se decide");
    }
}
