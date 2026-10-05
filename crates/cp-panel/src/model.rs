use crate::Card;
use crate::view::{body_of, card_of, was_found};
use cp_store::{Cursor, Filter, Listed, Store};
use slint::{Model, ModelNotify, ModelTracker};
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

pub const PAGE: usize = 120;
const AHEAD: usize = 40;

#[derive(Debug, Clone, Copy)]
pub struct Metrics {
    pub body_json: f32,
    pub body_link: f32,
    pub body_folder: f32,
    pub body_papers: f32,
    pub body_media: f32,
    pub head: f32,
    pub tall: f32,
    pub plain: f32,
    pub mixed: f32,
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

pub fn by_time(filter: &Filter, plain_way: bool) -> bool {
    one_height_for_all(filter, plain_way)
        && filter.order == cp_store::Order::Recent
        && filter
            .query
            .as_deref()
            .is_none_or(|said| said.trim().is_empty())
        && filter.label_query.is_none()
        && !filter.pinned_only
}

pub const MIXED_KEYS: [&str; 4] = [
    crate::media::DURATION,
    crate::media::WIDTH,
    crate::media::HEIGHT,
    crate::folder::ENTRIES,
];

pub fn meta_keys_for(filter: &Filter) -> &'static [&'static str] {
    match crate::layout::layout_for(&filter.kinds) {
        crate::layout::Layout::Video | crate::layout::Layout::Audio => &crate::media::KEYS,
        crate::layout::Layout::Folder => &crate::folder::KEYS,
        crate::layout::Layout::Everything | crate::layout::Layout::Image => &MIXED_KEYS,
        _ => &[],
    }
}

pub fn shut_height_for(filter: &Filter, metrics: &Metrics, plain_way: bool) -> f32 {
    if plain_way {
        return metrics.plain;
    }
    match crate::layout::layout_for(&filter.kinds) {
        crate::layout::Layout::Json => metrics.json,
        crate::layout::Layout::Everything => metrics.mixed,
        _ => metrics.plain,
    }
}

pub fn one_height_for_all(filter: &Filter, plain_way: bool) -> bool {
    !plain_way && crate::layout::layout_for(&filter.kinds).shows_cards()
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

pub type Clock = Rc<dyn Fn(i64) -> i64>;

pub struct Rows {
    store: Rc<Store>,
    filter: Filter,
    now: i64,
    metrics: Metrics,
    plain_way: bool,
    clock: Option<Clock>,
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
        clock: Option<Clock>,
    ) -> Rc<Self> {
        let rows = Rc::new(Self {
            store,
            filter,
            now,
            metrics,
            plain_way,
            clock,
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

    fn time_offset(&self) -> Option<&Clock> {
        self.clock
            .as_ref()
            .filter(|_| by_time(&self.filter, self.plain_way))
    }

    fn when(&self, row: &Listed, offset: &Clock) -> crate::age::When {
        let touched = crate::view::touched_of(row);
        crate::age::when_of(self.now, touched, offset(self.now), offset(touched))
    }

    fn heads(&self, rows: &[Listed], index: usize) -> bool {
        let Some(offset) = self.time_offset() else {
            return heads_group(&self.filter, rows, index);
        };
        let Some(row) = rows.get(index) else {
            return false;
        };
        match index.checked_sub(1).and_then(|before| rows.get(before)) {
            Some(before) => self.when(before, offset) != self.when(row, offset),
            None => true,
        }
    }

    pub fn loaded(&self) -> usize {
        self.rows.borrow().len()
    }

    pub fn index_of(&self, id: i64) -> Option<usize> {
        self.rows.borrow().iter().position(|row| row.id == id)
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
        let head = if self.heads(&rows, index) {
            self.metrics.head
        } else {
            0.0
        };
        let thumb = row.thumb_path.is_some() && !self.thumbless.borrow().contains(&index);
        if one_height_for_all(&self.filter, self.plain_way) {
            return self.mixed_open(row, thumb) + head;
        }
        self.open_of_row(row) + head
    }

    fn mixed_open(&self, row: &Listed, thumb: bool) -> f32 {
        use cp_core::kind::Kind;
        let room = match row.kind {
            Some(Kind::Folder) => self.metrics.body_folder,
            Some(Kind::File) => self.metrics.body_papers,
            Some(Kind::Video | Kind::Audio) => self.metrics.body_media,
            _ => 0.0,
        };
        crate::view::mixed_open_px(row, thumb, self.now, room)
    }

    fn open_of_row(&self, row: &Listed) -> f32 {
        if one_height_for_all(&self.filter, self.plain_way) {
            return self.mixed_open(row, false);
        }
        if row.thumb_path.is_some() {
            return self.metrics.tall;
        }
        let lines = crate::view::open_lines_of(row, &body_of(row), self.now);
        let own = if one_height_for_all(&self.filter, self.plain_way) {
            self.body_room(row)
        } else {
            0.0
        };
        self.metrics.frame + lines as f32 * self.metrics.line + own
    }

    fn body_room(&self, row: &Listed) -> f32 {
        use cp_core::kind::Kind;
        let tall = match row.kind {
            Some(Kind::Json) => self.metrics.body_json,
            Some(Kind::Link) => self.metrics.body_link,
            Some(Kind::Folder) => self.metrics.body_folder,
            Some(Kind::File) => self.metrics.body_papers,
            Some(Kind::Video | Kind::Audio) => self.metrics.body_media,
            _ => 0.0,
        };
        if tall > 0.0 { tall + EDGE } else { 0.0 }
    }

    fn tall_at(&self, index: usize) -> bool {
        !one_height_for_all(&self.filter, self.plain_way)
            && !self.thumbless.borrow().contains(&index)
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
        let alive = row.thumb_path.is_some() && !self.thumbless.borrow().contains(&index);
        let shut = self.shut_of(index, alive);
        let rows = self.rows.borrow();
        if self.heads(&rows, index) {
            shut + self.metrics.head
        } else {
            shut
        }
    }

    fn shut_of(&self, index: usize, has_thumb: bool) -> f32 {
        let rows = self.rows.borrow();
        let Some(row) = rows.get(index) else {
            return self.metrics.plain;
        };
        if one_height_for_all(&self.filter, self.plain_way) {
            return crate::view::face_for(row, has_thumb).shut_px();
        }
        if has_thumb {
            return self.metrics.tall;
        }
        if was_found(row) {
            return self.metrics.found;
        }
        self.shut_height()
    }

    fn open_of_row_with(&self, index: usize, has_thumb: bool) -> f32 {
        let rows = self.rows.borrow();
        let Some(row) = rows.get(index) else {
            return self.metrics.plain;
        };
        if one_height_for_all(&self.filter, self.plain_way) {
            return self.mixed_open(row, has_thumb);
        }
        if has_thumb {
            return self.metrics.tall;
        }
        let lines = crate::view::open_lines_of(row, &body_of(row), self.now);
        let own = if one_height_for_all(&self.filter, self.plain_way) {
            self.body_room(row)
        } else {
            0.0
        };
        self.metrics.frame + lines as f32 * self.metrics.line + own
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
        let heads = self.heads(&rows, index);
        let without_thumb = if heads { self.metrics.head } else { 0.0 }
            + if self.open.get() == Some(index) {
                self.open_of_row(row)
            } else if one_height_for_all(&self.filter, self.plain_way) {
                crate::view::face_for(row, false).shut_px()
            } else if was_found(row) {
                self.metrics.found
            } else {
                self.shut_height()
            };
        let meta = self.meta.borrow();
        let mut card = card_of(row, self.now, meta.get(&row.id));
        drop(meta);
        let english = crate::say::in_english();
        if let Some(offset) = self.time_offset() {
            let touched = crate::view::touched_of(row);
            card.age = crate::age::age_in_group(
                self.now,
                touched,
                offset(self.now),
                offset(touched),
                english,
            )
            .into();
            if heads {
                card.heads_group = true;
                card.group_said = crate::age::when_said(self.when(row, offset), english).into();
            }
        } else if heads {
            card.heads_group = true;
            card.group_said = row.group.clone().into();
        }
        if let Some(path) = &row.thumb_path
            && let Ok(image) = slint::Image::load_from_path(std::path::Path::new(path))
        {
            card.thumb = image;
        } else {
            card.has_thumb = false;
            let face = crate::view::face_for(row, false);
            let opened = crate::view::opened_for(row, face);
            card.face = face.as_str().into();
            card.shut_lines = face.lines();
            card.opened = opened.text.into();
            card.open_lines = opened.lines;
            card.more_said = opened.more.into();
        }
        drop(rows);
        if !card.has_thumb {
            if row_wanted_a_thumb {
                self.thumbless.borrow_mut().insert(index);
            }
            self.resize(index, without_thumb);
        }
        card.shut_px = self.shut_of(index, card.has_thumb);
        card.open_px = self.open_of_row_with(index, card.has_thumb);
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
