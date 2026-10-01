use crate::Card;
use crate::view::{body_of, card_of, lines_of, was_found};
use cp_store::{Cursor, Filter, Listed, Store};
use slint::{Model, ModelNotify, ModelTracker};
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

pub const PAGE: usize = 120;
const AHEAD: usize = 40;

#[derive(Debug, Clone, Copy)]
pub struct Metrics {
    pub head: f32,
    pub tall: f32,
    pub plain: f32,
    pub json: f32,
    pub found: f32,
    pub frame: f32,
    pub line: f32,
}

const EDGE: f32 = 8.0;

pub fn heads_group(filter: &Filter, rows: &[Listed], index: usize) -> bool {
    if filter.order != cp_store::Order::ByGroup {
        return false;
    }
    let Some(row) = rows.get(index) else {
        return false;
    };
    if !crate::group::shown(&row.group) {
        return false;
    }
    match index.checked_sub(1).and_then(|before| rows.get(before)) {
        Some(before) => before.group != row.group,
        None => true,
    }
}

pub fn meta_keys_for(filter: &Filter) -> &'static [&'static str] {
    match crate::layout::layout_for(&filter.kinds) {
        crate::layout::Layout::Video | crate::layout::Layout::Audio => &crate::media::KEYS,
        crate::layout::Layout::Folder => &crate::folder::KEYS,
        _ => &[],
    }
}

pub fn shut_height_for(filter: &Filter, metrics: &Metrics, plain_way: bool) -> f32 {
    if plain_way {
        return metrics.plain;
    }
    match crate::layout::layout_for(&filter.kinds) {
        crate::layout::Layout::Json => metrics.json,
        _ => metrics.plain,
    }
}

pub fn reveal(top: f32, span: f32, scroll: f32, viewport: f32) -> f32 {
    if viewport <= 0.0 {
        return scroll;
    }
    let seen = -scroll;
    if top < seen {
        (EDGE - top).min(0.0)
    } else if top + span > seen + viewport {
        viewport - top - span - EDGE
    } else {
        scroll
    }
}

pub struct Rows {
    store: Rc<Store>,
    filter: Filter,
    now: i64,
    metrics: Metrics,
    plain_way: bool,
    rows: RefCell<Vec<Listed>>,
    meta: RefCell<cp_store::MetaByItem>,
    cards: RefCell<Vec<Option<Card>>>,
    tops: RefCell<Vec<f32>>,
    open: Cell<Option<usize>>,
    thumbless: RefCell<std::collections::HashSet<usize>>,
    next: RefCell<Option<Cursor>>,
    exhausted: Cell<bool>,
    loading: Cell<bool>,
    notify: ModelNotify,
    weak: RefCell<Weak<Rows>>,
}

impl Rows {
    pub fn open(
        store: Rc<Store>,
        filter: Filter,
        now: i64,
        metrics: Metrics,
        plain_way: bool,
    ) -> Rc<Self> {
        let rows = Rc::new(Self {
            store,
            filter,
            now,
            metrics,
            plain_way,
            rows: RefCell::new(Vec::new()),
            meta: RefCell::new(cp_store::MetaByItem::new()),
            cards: RefCell::new(Vec::new()),
            tops: RefCell::new(vec![0.0]),
            open: Cell::new(None),
            thumbless: RefCell::new(std::collections::HashSet::new()),
            next: RefCell::new(None),
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
        let head = if heads_group(&self.filter, &rows, index) {
            self.metrics.head
        } else {
            0.0
        };
        self.open_of_row(row) + head
    }

    fn open_of_row(&self, row: &Listed) -> f32 {
        self.metrics.frame + lines_of(&body_of(row)) as f32 * self.metrics.line
    }

    fn tall_at(&self, index: usize) -> bool {
        !self.thumbless.borrow().contains(&index)
            && self
                .rows
                .borrow()
                .get(index)
                .is_some_and(|row| row.thumb_path.is_some())
    }

    fn base_of(&self, index: usize) -> f32 {
        let rows = self.rows.borrow();
        rows.get(index)
            .map_or(self.metrics.plain, |row| self.height_at(index, row))
    }

    pub fn span_of(&self, index: usize) -> Option<(f32, f32)> {
        let tops = self.tops.borrow();
        let top = *tops.get(index)?;
        let next = *tops.get(index + 1)?;
        Some((top, next - top))
    }

    fn height_at(&self, index: usize, row: &Listed) -> f32 {
        let shut = if self.thumbless.borrow().contains(&index) {
            if was_found(row) {
                self.metrics.found
            } else {
                self.shut_height()
            }
        } else {
            self.height_of(row)
        };
        let rows = self.rows.borrow();
        if heads_group(&self.filter, &rows, index) {
            shut + self.metrics.head
        } else {
            shut
        }
    }

    fn height_of(&self, row: &Listed) -> f32 {
        if row.thumb_path.is_some() {
            self.metrics.tall
        } else if was_found(row) {
            self.metrics.found
        } else {
            self.shut_height()
        }
    }

    fn shut_height(&self) -> f32 {
        shut_height_for(&self.filter, &self.metrics, self.plain_way)
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
        let page = match self
            .store
            .list(&self.filter, PAGE, self.next.borrow().clone())
        {
            Ok(page) => page,
            Err(why) => {
                crate::note::note(&format!("la lista no se pudo leer: {why}"));
                self.exhausted.set(true);
                return 0;
            }
        };
        let added = page.rows.len();
        if page.next.is_none() {
            self.exhausted.set(true);
        }
        *self.next.borrow_mut() = page.next;
        let start = self.rows.borrow().len();
        self.cards.borrow_mut().extend((0..added).map(|_| None));
        let keys = meta_keys_for(&self.filter);
        if !keys.is_empty() {
            let ids: Vec<i64> = page.rows.iter().map(|one| one.id).collect();
            match self.store.meta_for(&ids, keys) {
                Ok(found) => self.meta.borrow_mut().extend(found),
                Err(why) => crate::note::note(&format!("los metadatos no se pudieron leer: {why}")),
            }
        }
        self.rows.borrow_mut().extend(page.rows);
        {
            let rows = self.rows.borrow();
            let mut tops = self.tops.borrow_mut();
            let mut at = tops.last().copied().unwrap_or(0.0);
            for index in start..rows.len() {
                at += self.height_at(index, &rows[index]);
                tops.push(at);
            }
        }
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
        let row_wanted_a_thumb = row.thumb_path.is_some();
        let heads = heads_group(&self.filter, &rows, index);
        let without_thumb = if heads { self.metrics.head } else { 0.0 }
            + if self.open.get() == Some(index) {
                self.open_of_row(row)
            } else if was_found(row) {
                self.metrics.found
            } else {
                self.shut_height()
            };
        let meta = self.meta.borrow();
        let mut card = card_of(row, self.now, meta.get(&row.id));
        drop(meta);
        if heads {
            card.heads_group = true;
            card.group_said = row.group.clone().into();
        }
        if let Some(path) = &row.thumb_path
            && let Ok(image) = slint::Image::load_from_path(std::path::Path::new(path))
        {
            card.thumb = image;
        } else {
            card.has_thumb = false;
        }
        drop(rows);
        if !card.has_thumb {
            if row_wanted_a_thumb {
                self.thumbless.borrow_mut().insert(index);
            }
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
#[path = "model_test.rs"]
mod tests;
