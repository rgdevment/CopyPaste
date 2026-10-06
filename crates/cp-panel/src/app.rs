use crate::here;
use crate::landing::{Landing, Towards};
use crate::model::{Metrics, Rows, reveal};
use crate::note::note;
use crate::opening::Reached;
use crate::reaching::reach_for;
use crate::showing::{NEXT_FRAME, SLOW, appear, leave_when_left, place, vanish};
use crate::view::{AS_IS, as_is_label, label_of_form, shorthand_of};
use crate::view::{chips_of, compact, count_text, empty_of, form_of, harvest, label_of, sweeten};
use crate::{Chip, FormRow, Options, Panel};
use cp_core::kind::Kind;
use cp_store::{Clock, Filter, Store};
use slint::{ComponentHandle, Model, ModelRc};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicIsize, AtomicU64, Ordering};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

const CHIP_STEP: f32 = 78.0;
const KEPT_IN_VIEW: usize = 2;
const SETTLES: Duration = Duration::from_millis(70);
const RESTS: Duration = Duration::from_millis(110);
const LOOKS: Duration = Duration::from_millis(250);
const SETTLES_SHEET: Duration = Duration::from_millis(260);

#[derive(Clone)]
pub struct App {
    ui: slint::Weak<Panel>,
    state: Rc<RefCell<State>>,
}

struct State {
    store: Rc<Store>,
    engine: Option<Rc<crate::engine::Engine>>,
    ahead: Arc<AtomicIsize>,
    query: String,
    tags: Vec<String>,
    pinned: bool,
    keeping: bool,
    ways: std::collections::HashMap<&'static str, String>,
    rows: Option<Rc<Rows>>,
    options: Options,
    metrics: Metrics,
    asking: Asking,
    typing: slint::Timer,
    leaving: slint::Timer,
    pointing: slint::Timer,
    arming: slint::Timer,
    last_refresh: Duration,
    kept: crate::kept::Shelf,
    generation: Arc<AtomicU64>,
    counter: mpsc::Sender<Request>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Asking {
    Forms(i64),
    Kinds,
    Ways,
}

struct Request {
    generation: u64,
    base: Filter,
    full: Filter,
    keys: Vec<String>,
}

impl App {
    pub fn start(store: Store, options: Options) -> Result<(Panel, Self), slint::PlatformError> {
        let panel = Panel::new()?;
        let kept = crate::kept::read();
        crate::say::adopt_english(kept.english);
        crate::view::dress_words(&panel);
        let theme = panel.global::<crate::Theme>();
        let metrics = Metrics {
            head: theme.get_row_head(),
            tall: theme.get_row_thumb(),
            body_json: theme.get_body_json(),
            body_link: theme.get_body_link(),
            body_folder: theme.get_body_folder(),
            body_papers: theme.get_body_papers(),
            body_media: theme.get_body_media(),
            plain: theme.get_row_plain(),
            mixed: theme.get_row_mixed(),
            json: theme.get_row_json(),
            found: theme.get_row_found(),
            frame: theme.get_row_frame(),
            line: theme.get_line(),
        };
        let generation = Arc::new(AtomicU64::new(0));
        let counter = spawn_counter(options.db.clone(), panel.as_weak(), generation.clone());
        let state = Rc::new(RefCell::new(State {
            store: Rc::new(store),
            engine: None,
            ahead: Arc::new(AtomicIsize::new(0)),
            query: String::new(),
            tags: Vec::new(),
            pinned: false,
            keeping: false,
            ways: std::collections::HashMap::new(),
            rows: None,
            options,
            metrics,
            asking: Asking::Kinds,
            typing: slint::Timer::default(),
            leaving: slint::Timer::default(),
            pointing: slint::Timer::default(),
            arming: slint::Timer::default(),
            last_refresh: Duration::ZERO,
            kept: crate::kept::Shelf::new(kept),
            generation,
            counter,
        }));
        let app = Self {
            ui: panel.as_weak(),
            state,
        };
        if app.state.borrow().options.flat {
            panel.global::<crate::Theme>().set_shadow_blur(0.0);
        }
        app.wire(&panel);
        dress_theme(&panel, kept.light(here::system_is_light()));
        refresh(&panel, &app.state);
        Ok((panel, app))
    }

    pub fn close(&self) {
        let engine = self.state.borrow().engine.clone();
        if let Some(engine) = engine {
            engine.close();
        }
    }

    pub fn run(&self, panel: &Panel) -> Result<(), slint::PlatformError> {
        let serving = self.state.borrow().options.serve;
        let _awake = serving.then(here::keep_awake);
        if !serving {
            panel.show()?;
            self.dress(panel);
            appear(panel);
            panel.invoke_focus_search();
        }
        if !self.state.borrow().options.measure {
            self.keep_watch(panel.as_weak());
        }
        if let Some(dir) = self.state.borrow().options.signals.clone() {
            watch_signals(panel.as_weak(), dir, self.state.borrow().kept.clone());
        }
        if serving {
            let state = self.state.borrow();
            let shelf = state.kept.clone();
            leave_when_left(panel, move || shelf.get().hides);
            listen(
                panel.as_weak(),
                state.ahead.clone(),
                state.options.backdrop.clone(),
                state.kept.clone(),
            );
        }
        if self.state.borrow().options.measure {
            crate::measure::run(self);
        }
        slint::run_event_loop_until_quit()
    }

    fn keep_watch(&self, ui: slint::Weak<Panel>) {
        let db = self.state.borrow().options.db.clone();
        match crate::engine::Engine::start(&db, move |_| {
            let _ = ui.upgrade_in_event_loop(|panel| {
                if panel.window().is_visible() && !panel.get_sheet_open() {
                    panel.invoke_arrived();
                }
            });
        }) {
            Ok(engine) => self.state.borrow_mut().engine = Some(Rc::new(engine)),
            Err(why) => crate::note::trouble(&format!("nadie vigila el portapapeles: {why}")),
        }
    }

    pub fn ui(&self) -> slint::Weak<Panel> {
        self.ui.clone()
    }

    pub fn last_refresh(&self) -> Duration {
        self.state.borrow().last_refresh
    }

    pub fn search(&self, query: &str) {
        self.state.borrow_mut().query = query.to_owned();
        self.refresh();
    }

    pub fn choose_chip(&self, key: &str) {
        pick_tag(&self.state, key, false);
        self.refresh();
    }

    fn wire_chrome(&self, panel: &Panel) {
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_sheet_chosen(move |key| {
            let Some(ui) = ui.upgrade() else {
                return;
            };
            ui.set_sheet_open(false);
            let asking = state.borrow().asking;
            match asking {
                Asking::Forms(id) => {
                    let (store, engine, towards) = aimed(&state);
                    if paste_as(&store, engine.as_deref(), id, key.as_str(), towards) {
                        deliver(&ui, &state);
                    } else {
                        complain(&ui, busy());
                    }
                }
                Asking::Kinds => {
                    pick_tag(&state, key.as_str(), false);
                    blink(&ui);
                    refresh(&ui, &state);
                }
                Asking::Ways => {
                    {
                        let mut state = state.borrow_mut();
                        let layout = crate::layout::layout_for(&filter_of(&state).kinds);
                        crate::ways::remember(&mut state.ways, layout, key.as_str());
                    }
                    refresh(&ui, &state);
                }
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_keep_toggled(move || {
            let now = !state.borrow().keeping;
            state.borrow_mut().keeping = now;
            if let Some(ui) = ui.upgrade() {
                ui.set_keeping(now);
            }
        });
        let ui = self.ui.clone();
        panel.on_ask_settings(move || {
            let Some(ui) = ui.upgrade() else {
                return;
            };
            ui.set_sheet_open(false);
            let _ = ui.hide();
            crate::note::tell("settings");
        });
        let ui = self.ui.clone();
        panel.on_nudge(move |dx, dy| {
            if let Some(ui) = ui.upgrade() {
                crate::showing::nudge(&ui, dx, dy);
            }
        });
    }

    fn wire(&self, panel: &Panel) {
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_search(move |query| {
            let (taken, rest) = harvest(query.as_str());
            {
                let mut state = state.borrow_mut();
                for tag in taken {
                    if !state.tags.contains(&tag) {
                        state.tags.push(tag);
                    }
                }
                state.query = rest.clone();
            }
            if let Some(ui) = ui.upgrade() {
                if rest != query.as_str() {
                    ui.set_query(rest.into());
                }
                let later = ui.as_weak();
                let mine = Rc::downgrade(&state);
                state
                    .borrow()
                    .typing
                    .start(slint::TimerMode::SingleShot, SETTLES, move || {
                        if let (Some(ui), Some(state)) = (later.upgrade(), mine.upgrade()) {
                            refresh(&ui, &state);
                        }
                    });
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_chip_chosen(move |key, adding| {
            pick_tag(&state, key.as_str(), adding);
            if let Some(ui) = ui.upgrade() {
                blink(&ui);
                refresh(&ui, &state);
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_ask_ways_for(move |_| {
            if let Some(ui) = ui.upgrade() {
                ask_ways(&ui, &state);
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_clear_filters(move || {
            {
                let mut state = state.borrow_mut();
                state.query.clear();
                state.tags.clear();
                state.pinned = false;
            }
            if let Some(ui) = ui.upgrade() {
                ui.set_query(Default::default());
                blink(&ui);
                refresh(&ui, &state);
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_drop_tag(move |key| {
            {
                let mut state = state.borrow_mut();
                if key.is_empty() {
                    state.tags.pop();
                } else if let Some(at) = state.tags.iter().position(|one| one == key.as_str()) {
                    state.tags.remove(at);
                }
            }
            if let Some(ui) = ui.upgrade() {
                blink(&ui);
                refresh(&ui, &state);
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_pin_filter(move || {
            {
                let mut state = state.borrow_mut();
                state.pinned = !state.pinned;
            }
            if let Some(ui) = ui.upgrade() {
                ui.set_pinned_on(state.borrow().pinned);
                blink(&ui);
                refresh(&ui, &state);
            }
        });

        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_cycle(move |delta| {
            let Some(ui) = ui.upgrade() else {
                return;
            };
            let chips = ui.get_chips();
            let mut keys: Vec<String> = vec![String::new()];
            keys.extend(
                (0..chips.row_count())
                    .filter_map(|index| chips.row_data(index).map(|chip| chip.key.to_string())),
            );
            let here = keys
                .iter()
                .position(|key| state.borrow().tags.first() == Some(key))
                .unwrap_or(0);
            let next = (here as i32 + delta).rem_euclid(keys.len() as i32) as usize;
            ui.set_chips_scroll(-(next.saturating_sub(KEPT_IN_VIEW + 1) as f32) * CHIP_STEP);
            {
                let mut state = state.borrow_mut();
                state.tags.clear();
                if !keys[next].is_empty() {
                    state.tags.push(keys[next].clone());
                }
            }
            blink(&ui);
            refresh(&ui, &state);
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_pin(move |id, on| {
            let now = now_ms();
            if let Err(why) = state.borrow().store.set_pinned(i64::from(id), on, now) {
                note(&format!("{id} could not be pinned: {why}"));
            }
            if let Some(ui) = ui.upgrade() {
                keeping_place(&ui, &state);
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_remove(move |id| {
            if let Err(why) = state.borrow().store.mark_deleted(i64::from(id), now_ms()) {
                note(&format!("{id} could not be deleted: {why}"));
            }
            if let Some(ui) = ui.upgrade() {
                keeping_place(&ui, &state);
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_paste(move |id| {
            let (store, engine, towards) = aimed(&state);
            let handed = hand_over(&store, engine.as_deref(), i64::from(id), towards);
            if let Some(ui) = ui.upgrade() {
                if handed {
                    deliver(&ui, &state);
                } else {
                    complain(&ui, busy());
                }
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_copy(move |id| {
            let (store, engine) = {
                let state = state.borrow();
                (state.store.clone(), state.engine.clone())
            };
            let handed = hand_over(&store, engine.as_deref(), i64::from(id), Towards::Elsewhere);
            if let Some(ui) = ui.upgrade() {
                if handed {
                    following(&ui, &state, i64::from(id));
                    complain(&ui, crate::say::pick("copiado", "copied"));
                } else {
                    complain(&ui, busy());
                }
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_moved(move |index| {
            if index < 0 {
                return;
            }
            let Some(ui) = ui.upgrade() else {
                return;
            };
            let state = state.borrow();
            let Some(rows) = state.rows.as_ref() else {
                return;
            };
            rows.open_at(ui.get_opened().then_some(index as usize));
            let Some((top, span)) = rows.span_of(index as usize) else {
                return;
            };
            ui.set_scroll_y(reveal(
                top,
                span,
                ui.get_scroll_y(),
                ui.get_viewport_height(),
            ));
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_ask_forms(move |id| {
            let Some(ui) = ui.upgrade() else {
                return;
            };
            let rows = forms_of(&state.borrow().store, i64::from(id));
            if rows.is_empty() {
                return;
            }
            state.borrow_mut().asking = Asking::Forms(i64::from(id));
            let at = ui.get_current();
            if at >= 0
                && let Some(rows) = state.borrow().rows.as_ref()
            {
                rows.open_at(ui.get_opened().then_some(at as usize));
            }
            ui.set_hovered(-1);
            if let Some(card) = (at >= 0)
                .then(|| ui.get_cards().row_data(at as usize))
                .flatten()
            {
                ui.set_sheet_subject(slint::format!("{} · {}", card.title, card.source));
                ui.set_sheet_kind(card.kind.clone());
            }
            open_sheet(&ui, crate::say::pick("PEGAR COMO", "PASTE AS"), rows);
            arm_sheet(&state, &ui);
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_ask_kinds(move || {
            let Some(ui) = ui.upgrade() else {
                return;
            };
            let chips = ui.get_chips();
            let rows: Vec<FormRow> = (0..chips.row_count())
                .filter_map(|at| chips.row_data(at))
                .map(|chip| FormRow {
                    key: chip.key.clone(),
                    label: chip.label.clone(),
                    preview: chip.count.clone(),
                })
                .collect();
            if rows.is_empty() {
                return;
            }
            state.borrow_mut().asking = Asking::Kinds;
            ui.set_sheet_anchor(0.0);
            ui.set_sheet_span(0.0);
            ui.set_sheet_subject(Default::default());
            open_sheet(
                &ui,
                crate::say::pick("FILTRAR POR TIPO", "FILTER BY KIND"),
                rows,
            );
            arm_sheet(&state, &ui);
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_stirred(move |index| {
            let Some(ui) = ui.upgrade() else {
                return;
            };
            if index < 0 {
                state.borrow().pointing.stop();
                ui.set_hovered(-1);
                return;
            }
            let later = ui.as_weak();
            state
                .borrow()
                .pointing
                .start(slint::TimerMode::SingleShot, RESTS, move || {
                    if let Some(ui) = later.upgrade() {
                        ui.set_hovered(index);
                    }
                });
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_reopened(move || {
            if let Some(ui) = ui.upgrade() {
                keeping_place(&ui, &state);
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_fresh_start(move || {
            {
                let mut state = state.borrow_mut();
                state.query.clear();
                state.tags.clear();
                state.pinned = false;
            }
            if let Some(ui) = ui.upgrade() {
                ui.set_query(Default::default());
                ui.set_sheet_open(false);
                ui.set_chips_scroll(0.0);
                dress_theme(
                    &ui,
                    state.borrow().kept.get().light(here::system_is_light()),
                );
                refresh(&ui, &state);
                back_to_the_newest(&ui, &state);
                watch_leaving(&ui, &state);
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_emptied(move || {
            let store = state.borrow().store.clone();
            match store.clear_all_unpinned(now_ms()) {
                Ok(gone) => {
                    note(&format!("{gone} unpinned items were emptied out"));
                    crate::note::tell(&format!("emptied {gone}"));
                }
                Err(why) => crate::note::trouble(&format!("no se pudo vaciar: {why}")),
            }
            if let Some(ui) = ui.upgrade() {
                refresh(&ui, &state);
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_arrived(move || {
            if let Some(ui) = ui.upgrade() {
                keeping_place(&ui, &state);
            }
        });
        self.wire_chrome(panel);
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_paste_as(move |id, key| {
            let (store, engine, towards) = aimed(&state);
            let done = paste_as(
                &store,
                engine.as_deref(),
                i64::from(id),
                key.as_str(),
                towards,
            );
            if let Some(ui) = ui.upgrade() {
                ui.set_sheet_open(false);
                if done {
                    deliver(&ui, &state);
                } else {
                    complain(&ui, busy());
                }
            }
        });
        let ui = self.ui.clone();
        panel.on_dismiss(move || {
            if let Some(ui) = ui.upgrade() {
                let _ = ui.hide();
            }
        });
        self.wire_opening(panel);
        self.wire_dragging(panel);
        self.wire_naming(panel);
    }

    fn wire_opening(&self, panel: &Panel) {
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_open_asked(move |id| {
            let store = state.borrow().store.clone();
            let said = reach_for(&store, i64::from(id));
            let Some(ui) = ui.upgrade() else {
                return;
            };
            match said {
                Reached::Opened | Reached::Working => {
                    if said == Reached::Working {
                        note(&format!("{id} is being opened, slowly"));
                    }
                    ui.set_sheet_open(false);
                    let _ = ui.hide();
                }
                Reached::NoLink => complain(&ui, cannot_open_link()),
                Reached::Missing => {
                    let now = now_ms();
                    if let Err(why) = state.borrow().store.mark_broken(i64::from(id), now) {
                        note(&format!("{id} could not be marked as gone: {why}"));
                    }
                    keeping_place(&ui, &state);
                    complain(&ui, cannot_open());
                }
                Reached::Refused => complain(&ui, cannot_open()),
            }
        });
    }

    fn wire_dragging(&self, panel: &Panel) {
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_dragged(move |id| {
            let Some(ui) = ui.upgrade() else {
                return;
            };
            let store = state.borrow().store.clone();
            let Ok(Some(item)) = store.item(i64::from(id)) else {
                note(&format!("{id} could not be looked up to drag it"));
                return;
            };
            let label = store.label_of(i64::from(id)).unwrap_or_default();
            let content = here::content_of(&item, None);
            let files = crate::dragging::files_for(
                item.kind,
                &content.paths,
                content.image,
                label.as_deref(),
                std::time::SystemTime::now(),
            );
            if files.is_empty() {
                note(&format!("{id} has no file another application could take"));
                return;
            }
            let Some(handle) = handle_of(&ui) else {
                note(&format!(
                    "{id} cannot be dragged: the panel has no native window"
                ));
                return;
            };
            let carried: Vec<&std::path::Path> = files.iter().map(|one| one.as_path()).collect();
            let went = here::drag_out(handle, &carried);
            if went != here::Dragged::Started {
                note(&format!("{id} was not dragged: {went:?}"));
            }
        });
    }

    fn wire_naming(&self, panel: &Panel) {
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_named(move |id, said| {
            let name = crate::view::name_worth_keeping(&said);
            let kept = state
                .borrow()
                .store
                .set_label(i64::from(id), name.as_deref(), now_ms());
            let landed = match &kept {
                Ok(landed) => *landed,
                Err(why) => {
                    note(&format!("{id} could not be named: {why}"));
                    false
                }
            };
            if kept.is_ok() && !landed {
                note(&format!("{id} was named but it is no longer in the store"));
            }
            if let Some(ui) = ui.upgrade() {
                ui.set_naming(-1);
                if !landed {
                    complain(&ui, cannot_name());
                }
                keeping_place(&ui, &state);
            }
        });
    }

    fn refresh(&self) {
        if let Some(panel) = self.ui.upgrade() {
            refresh(&panel, &self.state);
        }
    }

    fn dress(&self, panel: &Panel) {
        let state = self.state.borrow();
        dress(
            panel,
            &state.options.backdrop,
            state.kept.get().light(here::system_is_light()),
        );
    }
}

fn handle_of(panel: &Panel) -> Option<raw_window_handle::RawWindowHandle> {
    use raw_window_handle::HasWindowHandle;
    let handle = panel.window().window_handle();
    HasWindowHandle::window_handle(&handle)
        .ok()
        .map(|raw| raw.as_raw())
}

fn dress(panel: &Panel, wanted: &str, light: bool) {
    if let Some(handle) = handle_of(panel) {
        here::dress(handle, wanted, light);
    }
}

fn back_to_the_newest(ui: &Panel, state: &Rc<RefCell<State>>) {
    if let Some(rows) = state.borrow().rows.as_ref() {
        rows.open_at(None);
    }
    ui.set_opened(false);
    ui.set_current(if ui.get_cards().row_count() > 0 {
        0
    } else {
        -1
    });
    ui.invoke_to_the_top();
}

fn following(ui: &Panel, state: &Rc<RefCell<State>>, id: i64) {
    refresh(ui, state);
    let Some(at) = state
        .borrow()
        .rows
        .as_ref()
        .and_then(|rows| rows.index_of(id))
        .and_then(|at| i32::try_from(at).ok())
    else {
        return;
    };
    ui.set_current(at);
    ui.invoke_moved(at);
}

fn keeping_place(ui: &Panel, state: &Rc<RefCell<State>>) {
    let was = ui.get_current();
    refresh(ui, state);
    let rows = state.borrow().rows.as_ref().map_or(0, |rows| rows.loaded());
    if rows == 0 || was <= 0 {
        return;
    }
    let back = was.min(rows as i32 - 1);
    ui.set_current(back);
    ui.invoke_moved(back);
}

fn refresh(ui: &Panel, state: &Rc<RefCell<State>>) {
    let started = Instant::now();
    ui.set_sheet_open(false);
    let now = now_ms();
    let (store, filter, base, metrics) = {
        let state = state.borrow();
        (
            state.store.clone(),
            filter_of(&state),
            base_filter_of(&state),
            state.metrics,
        )
    };
    let keys = keys_of(&filter);
    let full = filter.clone();
    let layout = crate::layout::layout_for(&filter.kinds);
    let way = crate::ways::recalled(&state.borrow().ways, layout).to_owned();
    let here = crate::ways::chosen(layout, &way);
    ui.set_way(here.key.into());
    ui.set_way_said(if crate::ways::ways_of(layout).is_empty() {
        Default::default()
    } else {
        crate::ways::label_of(here, crate::say::in_english()).into()
    });
    ui.set_layout(layout.as_str().into());
    let mut kinds = filter.kinds.clone();
    kinds.dedup();
    ui.set_one_kind(kinds.len() == 1);
    ui.set_plain_way(here.plain);
    ui.set_shut(crate::model::shut_height_for(&filter, &metrics, here.plain));
    let rows = Rows::open(
        store,
        filter,
        now,
        metrics,
        here.plain,
        Some(crate::model::by_the_quarter(here::utc_offset_at)),
    );
    ui.set_opened(false);
    {
        let state = state.borrow();
        ui.set_filtered(!state.query.trim().is_empty() || !state.tags.is_empty() || state.pinned);
        ui.set_tags(ModelRc::from(Rc::new(slint::VecModel::from(tags_of(
            &state,
        )))));
    }
    if rows.loaded() == 0 {
        let (title, hint) = {
            let state = state.borrow();
            empty_of(&state.query, state.pinned, !state.tags.is_empty())
        };
        ui.set_empty_title(title.into());
        ui.set_empty_hint(hint.into());
    }
    ui.set_current(if rows.loaded() > 0 { 0 } else { -1 });
    ui.set_scroll_y(0.0);
    ui.set_cards(ModelRc::from(rows.clone()));
    ui.set_grid_lines(ModelRc::from(crate::paired::Paired::over(rows.clone())));
    let mut state = state.borrow_mut();
    state.rows = Some(rows);
    state.last_refresh = started.elapsed();
    let generation = state.generation.fetch_add(1, Ordering::SeqCst) + 1;
    let _ = state.counter.send(Request {
        generation,
        base,
        full,
        keys,
    });
    if state.options.measure {
        eprintln!(
            "refresh «{}»: list {:.1} ms",
            state.query,
            state.last_refresh.as_secs_f64() * 1000.0
        );
    }
}

fn spawn_counter(
    db: std::path::PathBuf,
    ui: slint::Weak<Panel>,
    generation: Arc<AtomicU64>,
) -> mpsc::Sender<Request> {
    let (tx, rx) = mpsc::channel::<Request>();
    let spawned = std::thread::Builder::new()
        .name(crate::note::COUNTER.to_owned())
        .spawn(move || {
            let store = match Store::open(&db) {
                Ok(store) => store,
                Err(why) => {
                    note(&format!("the counter could not open the store: {why}"));
                    return;
                }
            };
            while let Ok(mut request) = rx.recv() {
                while let Ok(newer) = rx.try_recv() {
                    request = newer;
                }
                if request.generation != generation.load(Ordering::SeqCst) {
                    continue;
                }
                let pinned = store
                    .count_matching(&Filter {
                        pinned_only: true,
                        ..request.base.clone()
                    })
                    .unwrap_or(0);
                let facets = store.facets(&request.base).unwrap_or_default();
                let shown = store.count_matching(&request.full).unwrap_or(0);
                if request.generation != generation.load(Ordering::SeqCst) {
                    continue;
                }
                let chips = chips_of(&facets, &request.keys);
                let footer = count_text(shown);
                let anchored = compact(pinned);
                let only_anchored = request.full.pinned_only;
                let mine = request.generation;
                let clock = generation.clone();
                let _ = ui.upgrade_in_event_loop(move |panel| {
                    if mine != clock.load(Ordering::SeqCst) {
                        return;
                    }
                    panel.set_chips(ModelRc::from(Rc::new(slint::VecModel::from(chips))));
                    panel.set_count_text(footer.into());
                    panel.set_complaining(false);
                    panel.set_pinned_count(anchored.into());
                    panel.set_pinned_on(only_anchored);
                });
            }
        });
    if let Err(why) = spawned {
        note(&format!("nobody counts what the history holds: {why}"));
    }
    tx
}

fn written_filter(state: &State) -> Filter {
    let now = now_ms();
    let clock = Clock {
        now,
        day_start: now - now.rem_euclid(cp_store::query::DAY),
    };
    let mut written = cp_store::parse(&sweeten(&state.query), &clock);
    if written.broken == cp_store::Broken::Hidden {
        written.broken = cp_store::Broken::Shown;
    }
    written
}

fn base_filter_of(state: &State) -> Filter {
    let mut filter = written_filter(state);
    filter.kinds.clear();
    filter
}

fn filter_of(state: &State) -> Filter {
    let mut filter = written_filter(state);
    for tag in &state.tags {
        if let Some(kind) = Kind::from_name(tag) {
            filter.kinds.push(kind);
        }
    }
    if state.pinned {
        filter.pinned_only = true;
    }
    let layout = crate::layout::layout_for(&filter.kinds);
    if layout.groups()
        && !crate::ways::chosen(layout, crate::ways::recalled(&state.ways, layout)).recent
    {
        filter.order = cp_store::Order::ByGroup;
    }
    filter
}

fn ask_ways(ui: &Panel, state: &Rc<RefCell<State>>) {
    let (keeping, way) = {
        let state = state.borrow();
        (
            state.keeping,
            crate::ways::recalled(
                &state.ways,
                crate::layout::layout_for(&filter_of(&state).kinds),
            )
            .to_owned(),
        )
    };
    if keeping {
        return;
    }
    let layout = crate::layout::layout_for(&filter_of(&state.borrow()).kinds);
    let ways = crate::ways::ways_of(layout);
    if ways.is_empty() {
        return;
    }
    let english = crate::say::in_english();
    let here = crate::ways::chosen(layout, &way);
    let rows: Vec<FormRow> = ways
        .iter()
        .map(|one| FormRow {
            key: one.key.into(),
            label: crate::ways::label_of(*one, english).into(),
            preview: if one.key == here.key {
                crate::say::pick("ahora", "now").into()
            } else {
                Default::default()
            },
        })
        .collect();
    state.borrow_mut().asking = Asking::Ways;
    ui.set_sheet_anchor(0.0);
    ui.set_sheet_span(0.0);
    ui.set_sheet_subject(Default::default());
    open_sheet(
        ui,
        crate::say::pick("CÓMO MOSTRARLO", "HOW TO SHOW IT"),
        rows,
    );
    ui.set_sheet_narrow(true);
    arm_sheet(state, ui);
}

fn pick_tag(state: &Rc<RefCell<State>>, key: &str, adding: bool) {
    let adding = adding || state.borrow().keeping;
    let next = crate::tags::after(&state.borrow().tags, key, adding);
    state.borrow_mut().tags = next;
}

fn tags_of(state: &State) -> Vec<Chip> {
    state
        .tags
        .iter()
        .filter_map(|tag| Kind::from_name(tag).map(|kind| (tag, kind)))
        .map(|(tag, kind)| Chip {
            key: tag.as_str().into(),
            label: label_of(Some(kind)).into(),
            count: Default::default(),
            selected: true,
            has_ways: false,
        })
        .collect()
}

fn keys_of(filter: &Filter) -> Vec<String> {
    filter
        .kinds
        .iter()
        .map(|kind| kind.as_str().to_owned())
        .collect()
}

type Aimed = (Rc<Store>, Option<Rc<crate::engine::Engine>>, Towards);

fn aimed(state: &Rc<RefCell<State>>) -> Aimed {
    let state = state.borrow();
    let towards = here::towards(state.ahead.load(Ordering::Relaxed));
    (state.store.clone(), state.engine.clone(), towards)
}

fn landing_for(store: &Store, item: &cp_core::item::Item, id: i64, towards: Towards) -> Landing {
    if towards == Towards::Elsewhere {
        return Landing::anywhere();
    }
    let label = (towards == Towards::Browser)
        .then(|| store.label_of(id).ok().flatten())
        .flatten();
    let content = here::content_of(item, None);
    let files = crate::dragging::files_for(
        item.kind,
        &content.paths,
        content.image,
        label.as_deref(),
        std::time::SystemTime::now(),
    );
    Landing { towards, files }
}

fn hand_over(
    store: &Store,
    engine: Option<&crate::engine::Engine>,
    id: i64,
    towards: Towards,
) -> bool {
    let item = match store.item(id) {
        Ok(Some(item)) => item,
        Ok(None) => {
            note(&format!("pasting {id}: it is no longer in the store"));
            return false;
        }
        Err(why) => {
            note(&format!("pasting {id}: {why}"));
            return false;
        }
    };
    let landing = landing_for(store, &item, id, towards);
    let landed = here::to_clipboard(&item, &landing, || starting(engine), || mark(engine));
    let written = short_of(id, landed);
    if written {
        if let Err(why) = store.record_paste(id, now_ms()) {
            note(&format!("{id} was pasted and nobody wrote it down: {why}"));
        }
    } else {
        note(&format!(
            "pasting {id}: the write never reached the clipboard"
        ));
    }
    written
}

fn glimpse(rendered: Option<cp_core::paste_as::Rendered>) -> String {
    const SHOWN: usize = 22;
    let text = match rendered {
        Some(cp_core::paste_as::Rendered::Text(text)) => text,
        Some(cp_core::paste_as::Rendered::Jpeg(bytes)) => {
            return format!("{} KB", bytes.len() / 1024);
        }
        None => return String::new(),
    };
    let flat: String = text
        .chars()
        .map(|one| if one.is_control() { ' ' } else { one })
        .collect();
    let trimmed = flat.split_whitespace().collect::<Vec<_>>().join(" ");
    if trimmed.chars().count() <= SHOWN {
        return trimmed;
    }
    let kept: String = trimmed.chars().take(SHOWN).collect();
    format!("{}…", kept.trim_end())
}

fn open_sheet(ui: &Panel, title: &str, rows: Vec<FormRow>) {
    ui.set_sheet_narrow(false);
    ui.set_sheet_at(ui.get_scroll_y());
    ui.set_sheet_armed(false);
    ui.set_sheet_title(title.into());
    ui.set_sheet_rows(ModelRc::from(Rc::new(slint::VecModel::from(rows))));
    ui.set_sheet_current(0);
    ui.set_sheet_open(true);
}

fn arm_sheet(state: &Rc<RefCell<State>>, ui: &Panel) {
    let later = ui.as_weak();
    state
        .borrow()
        .arming
        .start(slint::TimerMode::SingleShot, SETTLES_SHEET, move || {
            if let Some(ui) = later.upgrade() {
                ui.set_sheet_at(ui.get_scroll_y());
                ui.set_sheet_armed(true);
            }
        });
}

fn busy() -> &'static str {
    if crate::here::read_stuck() {
        return crate::say::pick(
            "una app dejó de responder con lo copiado; reinicia CopyPaste",
            "an app stopped answering about what it copied; restart CopyPaste",
        );
    }
    crate::say::pick(
        "no se pudo pegar: el portapapeles está ocupado",
        "could not paste: the clipboard is busy",
    )
}
fn complain(ui: &Panel, said: &str) {
    ui.set_count_text(said.into());
    ui.set_complaining(true);
    let weak = ui.as_weak();
    slint::Timer::single_shot(Duration::from_millis(2_200), move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_complaining(false);
            ui.invoke_reopened();
        }
    });
}

fn blink(ui: &Panel) {
    ui.set_fade(0.35);
    let weak = ui.as_weak();
    slint::Timer::single_shot(NEXT_FRAME, move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_fade(1.0);
        }
    });
}

fn forms_of(store: &Store, id: i64) -> Vec<FormRow> {
    let Ok(Some(item)) = store.item(id) else {
        return Vec::new();
    };
    let ocr = store.ocr_text(id).ok().flatten();
    let content = here::content_of(&item, ocr.as_deref());
    let mut rows = vec![FormRow {
        key: AS_IS.into(),
        label: as_is_label(item.kind).into(),
        preview: glimpse(
            content
                .text
                .as_deref()
                .map(|text| cp_core::paste_as::Rendered::Text(text.to_owned())),
        )
        .into(),
    }];
    rows.extend(
        cp_core::paste_as::forms_for(&content)
            .into_iter()
            .map(|form| {
                let shown = glimpse(cp_core::paste_as::render(form, &content));
                match shorthand_of(form) {
                    Some(short) => FormRow {
                        key: form.as_str().into(),
                        label: shown.into(),
                        preview: short.into(),
                    },
                    None => FormRow {
                        key: form.as_str().into(),
                        label: label_of_form(form).into(),
                        preview: shown.into(),
                    },
                }
            }),
    );
    rows
}

fn paste_as(
    store: &Store,
    engine: Option<&crate::engine::Engine>,
    id: i64,
    key: &str,
    towards: Towards,
) -> bool {
    if key == AS_IS {
        return hand_over(store, engine, id, towards);
    }
    let Some(form) = form_of(key) else {
        note(&format!("a form nobody knows: {key}"));
        return false;
    };
    let Ok(Some(item)) = store.item(id) else {
        return false;
    };
    let ocr = store.ocr_text(id).ok().flatten();
    let content = here::content_of(&item, ocr.as_deref());
    let Some(rendered) = cp_core::paste_as::render(form, &content) else {
        note(&format!(
            "pasting {id} as {key}: the form gave nothing back"
        ));
        return false;
    };
    let made = rendered.into_item();
    let written = short_of(
        id,
        here::to_clipboard(
            &made,
            &Landing::anywhere(),
            || starting(engine),
            || mark(engine),
        ),
    );
    if written && let Err(why) = store.record_paste(id, now_ms()) {
        note(&format!("{id} was pasted and nobody wrote it down: {why}"));
    }
    written
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn cannot_open() -> &'static str {
    crate::say::pick(
        "no se pudo abrir: puede que ya no esté ahí",
        "could not open it: it may not be there any more",
    )
}

fn cannot_open_link() -> &'static str {
    crate::say::pick(
        "no se pudo abrir el enlace",
        "that link could not be opened",
    )
}

fn cannot_name() -> &'static str {
    crate::say::pick(
        "no se pudo guardar el nombre",
        "that name could not be kept",
    )
}

fn short_of(id: i64, landed: here::Landed) -> bool {
    if let here::Landed::Short { placed, wanted } = landed {
        note(&format!(
            "pasting {id}: only {placed} of {wanted} formats fitted on the clipboard"
        ));
    }
    landed != here::Landed::Nothing
}

fn starting(engine: Option<&crate::engine::Engine>) {
    let Some(engine) = engine else {
        return;
    };
    if !engine.writing() {
        note("the start of the clipboard write could not be marked as ours");
    }
}

fn mark(engine: Option<&crate::engine::Engine>) {
    let Some(engine) = engine else {
        return;
    };
    if !engine.ours() {
        note("the clipboard write could not be marked as ours");
    }
}

fn deliver(ui: &Panel, state: &Rc<RefCell<State>>) {
    let ahead = state.borrow().ahead.swap(0, Ordering::Relaxed);
    let (weak, later, state) = (ui.as_weak(), ui.as_weak(), state.clone());
    let hide = move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_sheet_open(false);
            let _ = ui.hide();
        }
    };
    here::paste_into(ahead, hide, move |sent| {
        let Some(ui) = later.upgrade() else {
            return;
        };
        match sent {
            here::Sent::Nobody => vanish(&ui),
            here::Sent::Done => {}
            here::Sent::Degraded(why) => {
                let _ = state.borrow().ahead.compare_exchange(
                    0,
                    ahead,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                );
                let said = crate::excuse::why_not(why, crate::say::in_english());
                note(&format!(
                    "it stays on the clipboard, unpasted: {why:?}: {said}"
                ));
                if ui.show().is_ok() {
                    forward(&ui);
                    appear(&ui);
                    watch_leaving(&ui, &state);
                    complain(&ui, said);
                }
            }
        }
    });
}

fn forward(panel: &Panel) {
    if let Some(handle) = handle_of(panel) {
        here::forward(handle);
    }
}

fn dress_theme(ui: &Panel, light: bool) {
    ui.global::<crate::Theme>().set_light(light);
}

fn watch_leaving(ui: &Panel, state: &Rc<RefCell<State>>) {
    let held = state.borrow();
    held.leaving.stop();
    if !held.kept.get().hides {
        return;
    }
    let weak = ui.as_weak();
    let mine = Rc::downgrade(state);
    let mut was_ours = false;
    held.leaving
        .start(slint::TimerMode::Repeated, LOOKS, move || {
            let (Some(ui), Some(state)) = (weak.upgrade(), mine.upgrade()) else {
                return;
            };
            if !ui.window().is_visible() {
                state.borrow().leaving.stop();
                return;
            }
            match here::ours_up_front() {
                Some(true) => {
                    was_ours = true;
                    return;
                }
                None => return,
                Some(false) => {}
            }
            if was_ours {
                state.borrow().leaving.stop();
                vanish(&ui);
            }
        });
}

const ORDER_UP_TO: u64 = 64;

fn listen(
    ui: slint::Weak<Panel>,
    ahead: Arc<AtomicIsize>,
    backdrop: String,
    shelf: crate::kept::Shelf,
) {
    std::thread::spawn(move || {
        use std::io::{BufRead, Read};
        let input = std::io::stdin();
        let mut reader = std::io::BufReader::new(input.lock());
        let mut said = String::new();
        loop {
            said.clear();
            match (&mut reader).take(ORDER_UP_TO).read_line(&mut said) {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
            match said.trim() {
                "show" => {
                    let asked = Instant::now();
                    let in_front = here::ahead_now();
                    if in_front != 0 {
                        ahead.store(in_front, Ordering::Relaxed);
                    }
                    let kept = shelf.renew();
                    crate::say::adopt_english(kept.english);
                    let dressed = backdrop.clone();
                    let _ = ui.upgrade_in_event_loop(move |panel| {
                        place(&panel);
                        crate::view::dress_words(&panel);
                        panel.invoke_fresh_start();
                        if panel.show().is_err() {
                            return;
                        }
                        place(&panel);
                        dress(&panel, &dressed, kept.light(here::system_is_light()));
                        forward(&panel);
                        appear(&panel);
                        panel.invoke_focus_search();
                        let took = asked.elapsed();
                        if took > SLOW {
                            note(&format!("the panel took {took:?} to show"));
                        }
                    });
                }
                "empty" => {
                    let _ = ui.upgrade_in_event_loop(|panel| panel.invoke_emptied());
                }
                "hide" => {
                    ahead.store(0, Ordering::Relaxed);
                    let _ = ui.upgrade_in_event_loop(|panel| vanish(&panel));
                }
                "quit" => break,
                _ => note("an order arrived that means nothing here"),
            }
        }
        let _ = ui.upgrade_in_event_loop(|panel| {
            if let Some(handle) = handle_of(&panel) {
                here::ground(handle);
            }
            let _ = slint::quit_event_loop();
        });
    });
}

fn watch_signals(ui: slint::Weak<Panel>, dir: std::path::PathBuf, shelf: crate::kept::Shelf) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(40));
            for (name, show) in [("show", true), ("hide", false)] {
                let flag = dir.join(name);
                if flag.exists() {
                    let _ = std::fs::remove_file(&flag);
                    if show {
                        crate::say::adopt_english(shelf.renew().english);
                    }
                    let _ = ui.upgrade_in_event_loop(move |ui| {
                        if show {
                            crate::view::dress_words(&ui);
                            ui.invoke_fresh_start();
                            let _ = ui.show();
                            appear(&ui);
                            ui.invoke_focus_search();
                        } else {
                            vanish(&ui);
                        }
                    });
                }
            }
        }
    });
}
