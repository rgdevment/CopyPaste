use crate::model::{Metrics, Rows, reveal};
use crate::note::note;
#[cfg(target_os = "windows")]
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
const OUT: Duration = Duration::from_millis(130);
const NEXT_FRAME: Duration = Duration::from_millis(16);
const SETTLES: Duration = Duration::from_millis(70);
const HOVERS: Duration = Duration::from_millis(55);
const LOOKS: Duration = Duration::from_millis(250);
const SETTLES_SHEET: Duration = Duration::from_millis(260);
const AFTER_ROLLING: Duration = Duration::from_millis(220);
const JUST_ROLLED: Duration = Duration::from_millis(260);

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
    rows: Option<Rc<Rows>>,
    options: Options,
    metrics: Metrics,
    asking: Asking,
    typing: slint::Timer,
    leaving: slint::Timer,
    pointing: slint::Timer,
    arming: slint::Timer,
    rolled: Instant,
    last_refresh: Duration,
    generation: Arc<AtomicU64>,
    counter: mpsc::Sender<Request>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Asking {
    Forms(i64),
    Kinds,
    Keys,
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
        let theme = panel.global::<crate::Theme>();
        let metrics = Metrics {
            tall: theme.get_row_thumb(),
            plain: theme.get_row_plain(),
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
            rows: None,
            options,
            metrics,
            asking: Asking::Kinds,
            typing: slint::Timer::default(),
            leaving: slint::Timer::default(),
            pointing: slint::Timer::default(),
            arming: slint::Timer::default(),
            rolled: Instant::now() - JUST_ROLLED,
            last_refresh: Duration::ZERO,
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
        dress_theme(&panel);
        refresh(&panel, &app.state);
        Ok((panel, app))
    }

    pub fn run(&self, panel: &Panel) -> Result<(), slint::PlatformError> {
        let serving = self.state.borrow().options.serve;
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
            watch_signals(panel.as_weak(), dir);
        }
        if serving {
            let state = self.state.borrow();
            listen(
                panel.as_weak(),
                state.ahead.clone(),
                state.options.backdrop.clone(),
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
            Err(why) => note(&format!("nadie vigila el portapapeles: {why}")),
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
        toggle_tag(&self.state, key);
        self.refresh();
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
                let state = state.clone();
                state.clone().borrow().typing.start(
                    slint::TimerMode::SingleShot,
                    SETTLES,
                    move || {
                        if let Some(ui) = later.upgrade() {
                            refresh(&ui, &state);
                        }
                    },
                );
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_chip_chosen(move |key| {
            toggle_tag(&state, key.as_str());
            if let Some(ui) = ui.upgrade() {
                blink(&ui);
                refresh(&ui, &state);
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
                note(&format!("no se pudo anclar {id}: {why}"));
            }
            if let Some(ui) = ui.upgrade() {
                keeping_place(&ui, &state);
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_remove(move |id| {
            if let Err(why) = state.borrow().store.mark_deleted(i64::from(id), now_ms()) {
                note(&format!("no se pudo borrar {id}: {why}"));
            }
            if let Some(ui) = ui.upgrade() {
                keeping_place(&ui, &state);
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_paste(move |id| {
            let (store, engine) = {
                let state = state.borrow();
                (state.store.clone(), state.engine.clone())
            };
            let handed = hand_over(&store, engine.as_deref(), i64::from(id));
            if let Some(ui) = ui.upgrade() {
                if handed {
                    deliver(&ui, &state);
                } else {
                    complain(&ui, BUSY);
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
            open_sheet(&ui, "PEGAR COMO", rows);
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
            open_sheet(&ui, "FILTRAR POR TIPO", rows);
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
                state.borrow_mut().rolled = Instant::now();
                ui.set_hovered(-1);
                return;
            }
            let waits = if state.borrow().rolled.elapsed() < JUST_ROLLED {
                AFTER_ROLLING
            } else {
                HOVERS
            };
            let later = ui.as_weak();
            state
                .borrow()
                .pointing
                .start(slint::TimerMode::SingleShot, waits, move || {
                    if let Some(ui) = later.upgrade() {
                        ui.set_hovered(index);
                    }
                });
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_reopened(move || {
            if let Some(ui) = ui.upgrade() {
                refresh(&ui, &state);
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
                dress_theme(&ui);
                refresh(&ui, &state);
                watch_leaving(&ui, &state);
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_emptied(move || {
            let store = state.borrow().store.clone();
            match store.clear_all_unpinned(now_ms()) {
                Ok(gone) => note(&format!("se vaciaron {gone} elementos sin anclar")),
                Err(why) => note(&format!("no se pudo vaciar: {why}")),
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
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_ask_keys(move || {
            let Some(ui) = ui.upgrade() else {
                return;
            };
            state.borrow_mut().asking = Asking::Keys;
            ui.set_sheet_anchor(0.0);
            ui.set_sheet_span(0.0);
            ui.set_sheet_subject(Default::default());
            open_sheet(&ui, "ATAJOS", keys_sheet());
            arm_sheet(&state, &ui);
        });
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
                    let (store, engine) = {
                        let state = state.borrow();
                        (state.store.clone(), state.engine.clone())
                    };
                    if paste_as(&store, engine.as_deref(), id, key.as_str()) {
                        deliver(&ui, &state);
                    } else {
                        complain(&ui, BUSY);
                    }
                }
                Asking::Kinds => {
                    toggle_tag(&state, key.as_str());
                    blink(&ui);
                    refresh(&ui, &state);
                }
                Asking::Keys => {}
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_paste_as(move |id, key| {
            let (store, engine) = {
                let state = state.borrow();
                (state.store.clone(), state.engine.clone())
            };
            let done = paste_as(&store, engine.as_deref(), i64::from(id), key.as_str());
            if let Some(ui) = ui.upgrade() {
                ui.set_sheet_open(false);
                if done {
                    deliver(&ui, &state);
                } else {
                    complain(&ui, BUSY);
                }
            }
        });
        let ui = self.ui.clone();
        panel.on_dismiss(move || {
            if let Some(ui) = ui.upgrade() {
                let _ = ui.hide();
            }
        });
        panel.on_edit(|_| {});
    }

    fn refresh(&self) {
        if let Some(panel) = self.ui.upgrade() {
            refresh(&panel, &self.state);
        }
    }

    fn dress(&self, panel: &Panel) {
        dress(panel, &self.state.borrow().options.backdrop);
    }
}

fn dress(panel: &Panel, wanted: &str) {
    #[cfg(target_os = "windows")]
    {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let handle = panel.window().window_handle();
        if let Ok(raw) = HasWindowHandle::window_handle(&handle)
            && let RawWindowHandle::Win32(win32) = raw.as_raw()
        {
            let backdrop = cp_win_sys::backdrop::Backdrop::from_name(wanted)
                .unwrap_or(cp_win_sys::backdrop::Backdrop::Mica);
            cp_win_sys::backdrop::apply(win32.hwnd.get(), backdrop, !wants_light());
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (panel, wanted);
    }
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
    ui.set_chips_scroll(0.0);
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
    let rows = Rows::open(store, filter, now, metrics);
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
    std::thread::spawn(move || {
        let store = match Store::open(&db) {
            Ok(store) => store,
            Err(why) => {
                note(&format!("el contador no pudo abrir el almacén: {why}"));
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
                panel.set_pinned_count(anchored.into());
                panel.set_pinned_on(only_anchored);
            });
        }
    });
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
    filter
}

fn toggle_tag(state: &Rc<RefCell<State>>, key: &str) {
    let mut state = state.borrow_mut();
    match state.tags.iter().position(|one| one == key) {
        Some(at) => {
            state.tags.remove(at);
        }
        None => state.tags.push(key.to_owned()),
    }
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

fn hand_over(store: &Store, engine: Option<&crate::engine::Engine>, id: i64) -> bool {
    let item = match store.item(id) {
        Ok(Some(item)) => item,
        Ok(None) => {
            note(&format!("pegar {id}: ya no está en el almacén"));
            return false;
        }
        Err(why) => {
            note(&format!("pegar {id}: {why}"));
            return false;
        }
    };
    #[cfg(target_os = "windows")]
    {
        let Some(clipboard) = cp_win_sys::clipboard::Clipboard::open() else {
            note(&format!("pegar {id}: el portapapeles no se dejó abrir"));
            return false;
        };
        let how = cp_win::restore::to_clipboard(&clipboard, &item);
        let written = matches!(how, cp_win::restore::Restored::Written { .. });
        if written {
            mark(engine);
            drop(clipboard);
            if let Err(why) = store.record_paste(id, now_ms()) {
                note(&format!("pegado {id} sin anotar: {why}"));
            }
        } else {
            note(&format!("pegar {id}: la escritura salió {how:?}"));
        }
        written
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = item;
        false
    }
}

#[cfg(target_os = "windows")]
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

fn keys_sheet() -> Vec<FormRow> {
    const KEYS: [(&str, &str); 12] = [
        ("Enter", "pegar lo seleccionado"),
        ("Shift + Enter", "pegar en plano"),
        ("Alt + Enter  ·  Ctrl + Enter", "pegar como…"),
        ("Flechas", "moverse por la lista"),
        ("Clic", "abrir la tarjeta; otro clic la cierra"),
        ("Doble clic", "pegar esa tarjeta"),
        ("Tab  ·  Shift + Tab", "recorrer los filtros"),
        ("#imagen  ·  #carpeta", "filtrar por tipo desde el buscador"),
        ("Retroceso", "quitar la última etiqueta"),
        ("Supr", "borrar la seleccionada"),
        ("Ctrl + P", "anclar o desanclar"),
        ("Esc", "cerrar el panel"),
    ];
    KEYS.iter()
        .map(|(keys, what)| FormRow {
            key: Default::default(),
            label: (*what).into(),
            preview: (*keys).into(),
        })
        .collect()
}

fn open_sheet(ui: &Panel, title: &str, rows: Vec<FormRow>) {
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

thread_local! {
    static CURTAIN: slint::Timer = slint::Timer::default();
}

fn appear(ui: &Panel) {
    ui.set_shown(0.0);
    let weak = ui.as_weak();
    CURTAIN.with(|timer| {
        timer.stop();
        timer.start(slint::TimerMode::SingleShot, NEXT_FRAME, move || {
            if let Some(ui) = weak.upgrade() {
                ui.set_shown(1.0);
            }
        });
    });
}

fn vanish(ui: &Panel) {
    ui.set_sheet_open(false);
    ui.set_shown(0.0);
    let weak = ui.as_weak();
    CURTAIN.with(|timer| {
        timer.stop();
        timer.start(slint::TimerMode::SingleShot, OUT, move || {
            if let Some(ui) = weak.upgrade() {
                let _ = ui.hide();
            }
        });
    });
}

const BUSY: &str = "no se pudo pegar: el portapapeles está ocupado";
const NOT_THERE: &str = "está copiado, pero no se pudo pegar ahí";

fn complain(ui: &Panel, said: &str) {
    ui.set_count_text(said.into());
    let weak = ui.as_weak();
    slint::Timer::single_shot(Duration::from_millis(2_200), move || {
        if let Some(ui) = weak.upgrade() {
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

#[cfg(target_os = "windows")]
fn forms_of(store: &Store, id: i64) -> Vec<FormRow> {
    let Ok(Some(item)) = store.item(id) else {
        return Vec::new();
    };
    let ocr = store.ocr_text(id).ok().flatten();
    let content = cp_win::content::content_of(&item, ocr.as_deref());
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

#[cfg(not(target_os = "windows"))]
fn forms_of(_store: &Store, _id: i64) -> Vec<FormRow> {
    Vec::new()
}

#[cfg(target_os = "windows")]
fn paste_as(store: &Store, engine: Option<&crate::engine::Engine>, id: i64, key: &str) -> bool {
    if key == AS_IS {
        return hand_over(store, engine, id);
    }
    let Some(form) = form_of(key) else {
        note(&format!("forma desconocida: {key}"));
        return false;
    };
    let Ok(Some(item)) = store.item(id) else {
        return false;
    };
    let ocr = store.ocr_text(id).ok().flatten();
    let content = cp_win::content::content_of(&item, ocr.as_deref());
    let Some(rendered) = cp_core::paste_as::render(form, &content) else {
        note(&format!(
            "pegar como {key} sobre {id}: la forma no dio nada"
        ));
        return false;
    };
    let made = rendered.into_item();
    let Some(clipboard) = cp_win_sys::clipboard::Clipboard::open() else {
        return false;
    };
    let written = matches!(
        cp_win::restore::to_clipboard(&clipboard, &made),
        cp_win::restore::Restored::Written { .. }
    );
    if written {
        mark(engine);
        drop(clipboard);
        if let Err(why) = store.record_paste(id, now_ms()) {
            note(&format!("pegado {id} sin anotar: {why}"));
        }
    }
    written
}

#[cfg(not(target_os = "windows"))]
fn paste_as(_store: &Store, _engine: Option<&crate::engine::Engine>, _id: i64, key: &str) -> bool {
    let _ = form_of(key);
    false
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn mark(engine: Option<&crate::engine::Engine>) {
    let Some(engine) = engine else {
        return;
    };
    if !engine.ours() {
        note("no se pudo marcar como nuestra la escritura del portapapeles");
    }
}

fn deliver(ui: &Panel, state: &Rc<RefCell<State>>) {
    let ahead = state.borrow().ahead.swap(0, Ordering::Relaxed);
    #[cfg(target_os = "windows")]
    if let Some(target) = cp_win_sys::frontmost::target_at(ahead) {
        let weak = ui.as_weak();
        let sent = cp_win::paste::paste_into(&target, move || {
            if let Some(ui) = weak.upgrade() {
                ui.set_sheet_open(false);
                let _ = ui.hide();
            }
        });
        if let cp_win::paste::Outcome::Degraded(why) = sent {
            note(&format!("queda en el portapapeles, sin pegar: {why:?}"));
            if ui.show().is_ok() {
                forward(ui);
                appear(ui);
                complain(ui, NOT_THERE);
            }
        }
        return;
    }
    let _ = ahead;
    vanish(ui);
}

fn forward(panel: &Panel) {
    #[cfg(target_os = "windows")]
    {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let handle = panel.window().window_handle();
        if let Ok(raw) = HasWindowHandle::window_handle(&handle)
            && let RawWindowHandle::Win32(win32) = raw.as_raw()
            && let Some(target) = cp_win_sys::frontmost::target_at(win32.hwnd.get())
        {
            cp_win_sys::frontmost::bring_forward(target.window);
        }
    }
    #[cfg(not(target_os = "windows"))]
    let _ = panel;
}

fn light_for(asked: cp_config::Theme, the_system_is_light: bool) -> bool {
    match asked {
        cp_config::Theme::Light => true,
        cp_config::Theme::Dark => false,
        cp_config::Theme::System => the_system_is_light,
    }
}

fn wants_light() -> bool {
    #[cfg(target_os = "windows")]
    {
        let asked = cp_win_sys::paths::data_dir()
            .and_then(|dir| cp_config::read(&cp_config::at(&dir)).ok())
            .map_or(cp_config::Theme::System, |kept| kept.theme);
        light_for(asked, cp_win_sys::theme::wants_light().unwrap_or(false))
    }
    #[cfg(not(target_os = "windows"))]
    {
        light_for(cp_config::Theme::System, false)
    }
}

fn dress_theme(ui: &Panel) {
    ui.global::<crate::Theme>().set_light(wants_light());
}

fn hides_when_left() -> bool {
    #[cfg(target_os = "windows")]
    {
        cp_win_sys::paths::data_dir()
            .and_then(|dir| cp_config::read(&cp_config::at(&dir)).ok())
            .is_none_or(|kept| kept.hides_when_left)
    }
    #[cfg(not(target_os = "windows"))]
    {
        true
    }
}

fn watch_leaving(ui: &Panel, state: &Rc<RefCell<State>>) {
    let held = state.borrow();
    held.leaving.stop();
    if !hides_when_left() {
        return;
    }
    let weak = ui.as_weak();
    let mine = state.clone();
    let mut was_ours = false;
    held.leaving
        .start(slint::TimerMode::Repeated, LOOKS, move || {
            let Some(ui) = weak.upgrade() else {
                return;
            };
            if !ui.window().is_visible() {
                return;
            }
            if ahead_now() == 0 {
                was_ours = true;
                return;
            }
            if was_ours {
                mine.borrow().leaving.stop();
                vanish(&ui);
            }
        });
}

fn ahead_now() -> isize {
    #[cfg(target_os = "windows")]
    {
        cp_win_sys::frontmost::ahead()
    }
    #[cfg(not(target_os = "windows"))]
    {
        0
    }
}

const ORDER_UP_TO: u64 = 64;

fn listen(ui: slint::Weak<Panel>, ahead: Arc<AtomicIsize>, backdrop: String) {
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
                    ahead.store(ahead_now(), Ordering::Relaxed);
                    let dressed = backdrop.clone();
                    let _ = ui.upgrade_in_event_loop(move |panel| {
                        if panel.show().is_err() {
                            return;
                        }
                        dress(&panel, &dressed);
                        forward(&panel);
                        panel.invoke_fresh_start();
                        appear(&panel);
                        panel.invoke_focus_search();
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
                _ => note("llegó una orden que no entiendo"),
            }
        }
        let _ = ui.upgrade_in_event_loop(|_| {
            let _ = slint::quit_event_loop();
        });
    });
}

fn watch_signals(ui: slint::Weak<Panel>, dir: std::path::PathBuf) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(40));
            for (name, show) in [("show", true), ("hide", false)] {
                let flag = dir.join(name);
                if flag.exists() {
                    let _ = std::fs::remove_file(&flag);
                    let _ = ui.upgrade_in_event_loop(move |ui| {
                        if show {
                            let _ = ui.show();
                            ui.invoke_reopened();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choosing_a_theme_wins_over_what_the_system_wants() {
        assert!(light_for(cp_config::Theme::Light, false));
        assert!(!light_for(cp_config::Theme::Dark, true));
    }

    #[test]
    fn leaving_it_to_the_system_follows_the_system_both_ways() {
        assert!(light_for(cp_config::Theme::System, true));
        assert!(!light_for(cp_config::Theme::System, false));
    }
}
