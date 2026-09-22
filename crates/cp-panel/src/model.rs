use crate::Card;
use crate::view::card_of;
use cp_store::{Cursor, Filter, Listed, Store};
use slint::{Model, ModelNotify, ModelTracker};
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

pub const PAGE: usize = 120;
const AHEAD: usize = 40;

pub struct Rows {
    store: Rc<Store>,
    filter: Filter,
    now: i64,
    rows: RefCell<Vec<Listed>>,
    cards: RefCell<Vec<Option<Card>>>,
    next: Cell<Option<Cursor>>,
    exhausted: Cell<bool>,
    loading: Cell<bool>,
    notify: ModelNotify,
    weak: RefCell<Weak<Rows>>,
}

impl Rows {
    pub fn open(store: Rc<Store>, filter: Filter, now: i64) -> Rc<Self> {
        let rows = Rc::new(Self {
            store,
            filter,
            now,
            rows: RefCell::new(Vec::new()),
            cards: RefCell::new(Vec::new()),
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

    fn load_page(&self) -> usize {
        if self.exhausted.get() {
            return 0;
        }
        let Ok(page) = self.store.list(&self.filter, PAGE, self.next.get()) else {
            self.exhausted.set(true);
            return 0;
        };
        let added = page.rows.len();
        self.next.set(page.next);
        if page.next.is_none() {
            self.exhausted.set(true);
        }
        let start = self.rows.borrow().len();
        self.cards.borrow_mut().extend((0..added).map(|_| None));
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
        let mut card = card_of(row, self.now);
        if let Some(path) = &row.thumb_path
            && let Ok(image) = slint::Image::load_from_path(std::path::Path::new(path))
        {
            card.thumb = image;
        } else {
            card.has_thumb = false;
        }
        drop(rows);
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

    fn store_with(count: usize) -> Rc<Store> {
        let store = Store::in_memory().expect("esquema");
        for at in 0..count {
            store
                .insert_text(&format!("u{at}"), &format!("elemento {at}"), at as i64)
                .expect("insert");
        }
        Rc::new(store)
    }

    #[test]
    fn the_first_page_comes_with_the_model_and_the_rest_waits_for_the_view() {
        let rows = Rows::open(store_with(300), Filter::default(), 1_000);
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
        let rows = Rows::open(store_with(3), Filter::default(), 5_000);
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
        let rows = Rows::open(store_with(300), Filter::default(), 0);
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
        let rows = Rows::open(store_with(2), Filter::default(), 0);
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
        let rows = Rows::open(store_with(10), filter, 0);
        assert_eq!(rows.row_count(), 0);
        assert!(rows.exhausted.get());
    }
}
