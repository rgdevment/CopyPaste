use crate::model::{Metrics, Rows, reveal};
use crate::view::{ALL, PINNED, chips_of, count_text};
use crate::{Options, Panel};
use cp_core::kind::Kind;
use cp_store::{Filter, Store};
use slint::{ComponentHandle, ModelRc};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct App {
    ui: slint::Weak<Panel>,
    state: Rc<RefCell<State>>,
}

struct State {
    store: Rc<Store>,
    query: String,
    chip: String,
    rows: Option<Rc<Rows>>,
    options: Options,
    metrics: Metrics,
    last_refresh: Duration,
    generation: Arc<AtomicU64>,
    counter: mpsc::Sender<Request>,
}

struct Request {
    generation: u64,
    base: Filter,
    chip: String,
}

impl App {
    pub fn start(store: Store, options: Options) -> Result<(Panel, Self), slint::PlatformError> {
        let panel = Panel::new()?;
        let theme = panel.global::<crate::Theme>();
        let metrics = Metrics {
            tall: theme.get_row_thumb(),
            plain: theme.get_row_plain(),
        };
        let generation = Arc::new(AtomicU64::new(0));
        let counter = spawn_counter(options.db.clone(), panel.as_weak(), generation.clone());
        let state = Rc::new(RefCell::new(State {
            store: Rc::new(store),
            query: String::new(),
            chip: ALL.into(),
            rows: None,
            options,
            metrics,
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
        refresh(&panel, &app.state);
        Ok((panel, app))
    }

    pub fn run(&self, panel: &Panel) -> Result<(), slint::PlatformError> {
        panel.show()?;
        self.dress(panel);
        panel.invoke_focus_search();
        if let Some(dir) = self.state.borrow().options.signals.clone() {
            watch_signals(panel.as_weak(), dir);
        }
        if self.state.borrow().options.measure {
            crate::measure::run(self);
        }
        slint::run_event_loop_until_quit()
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
        self.state.borrow_mut().chip = key.to_owned();
        self.refresh();
    }

    fn wire(&self, panel: &Panel) {
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_search(move |query| {
            state.borrow_mut().query = query.to_string();
            if let Some(ui) = ui.upgrade() {
                refresh(&ui, &state);
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_chip_chosen(move |key| {
            state.borrow_mut().chip = key.to_string();
            if let Some(ui) = ui.upgrade() {
                refresh(&ui, &state);
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_pin(move |id, on| {
            let now = now_ms();
            let _ = state.borrow().store.set_pinned(i64::from(id), on, now);
            if let Some(ui) = ui.upgrade() {
                refresh(&ui, &state);
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_remove(move |id| {
            let _ = state.borrow().store.mark_deleted(i64::from(id), now_ms());
            if let Some(ui) = ui.upgrade() {
                refresh(&ui, &state);
            }
        });
        let ui = self.ui.clone();
        let state = self.state.clone();
        panel.on_paste(move |id| {
            let handed = hand_over(&state.borrow().store, i64::from(id));
            if handed && let Some(ui) = ui.upgrade() {
                let _ = ui.hide();
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
        #[cfg(target_os = "windows")]
        {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            let handle = panel.window().window_handle();
            if let Ok(raw) = HasWindowHandle::window_handle(&handle)
                && let RawWindowHandle::Win32(win32) = raw.as_raw()
            {
                let wanted = self.state.borrow().options.backdrop.clone();
                let backdrop = cp_win_sys::backdrop::Backdrop::from_name(&wanted)
                    .unwrap_or(cp_win_sys::backdrop::Backdrop::Mica);
                cp_win_sys::backdrop::apply(win32.hwnd.get(), backdrop, true);
            }
        }
        #[cfg(not(target_os = "windows"))]
        let _ = panel;
    }
}

fn refresh(ui: &Panel, state: &Rc<RefCell<State>>) {
    let started = Instant::now();
    let now = now_ms();
    let (store, filter, base, chip, metrics) = {
        let state = state.borrow();
        (
            state.store.clone(),
            filter_of(&state),
            base_filter_of(&state),
            state.chip.clone(),
            state.metrics,
        )
    };
    let rows = Rows::open(store, filter, now, metrics);
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
        chip,
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
        let Ok(store) = Store::open(&db) else {
            return;
        };
        while let Ok(mut request) = rx.recv() {
            while let Ok(newer) = rx.try_recv() {
                request = newer;
            }
            if request.generation != generation.load(Ordering::SeqCst) {
                continue;
            }
            let total = store.count_matching(&request.base).unwrap_or(0);
            let pinned = store
                .count_matching(&Filter {
                    pinned_only: true,
                    ..request.base.clone()
                })
                .unwrap_or(0);
            let facets = store.facets(&request.base).unwrap_or_default();
            if request.generation != generation.load(Ordering::SeqCst) {
                continue;
            }
            let shown = match request.chip.as_str() {
                ALL => total,
                PINNED => pinned,
                kind => facets
                    .iter()
                    .find(|facet| facet.kind.as_str() == kind)
                    .map_or(0, |facet| facet.count),
            };
            let chips = chips_of(total, pinned, &facets, &request.chip);
            let footer = count_text(shown);
            let _ = ui.upgrade_in_event_loop(move |panel| {
                panel.set_chips(ModelRc::from(Rc::new(slint::VecModel::from(chips))));
                panel.set_count_text(footer.into());
            });
        }
    });
    tx
}

fn base_filter_of(state: &State) -> Filter {
    let query = state.query.trim();
    Filter {
        query: (!query.is_empty()).then(|| query.to_owned()),
        ..Default::default()
    }
}

fn filter_of(state: &State) -> Filter {
    let mut filter = base_filter_of(state);
    match state.chip.as_str() {
        ALL => {}
        PINNED => filter.pinned_only = true,
        kind => {
            if let Some(kind) = Kind::from_name(kind) {
                filter.kinds = vec![kind];
            }
        }
    }
    filter
}

fn hand_over(store: &Store, id: i64) -> bool {
    let Ok(Some(item)) = store.item(id) else {
        return false;
    };
    #[cfg(target_os = "windows")]
    {
        let Some(clipboard) = cp_win_sys::clipboard::Clipboard::open() else {
            return false;
        };
        let written = matches!(
            cp_win::restore::to_clipboard(&clipboard, &item),
            cp_win::restore::Restored::Written { .. }
        );
        if written {
            let _ = store.record_paste(id, now_ms());
        }
        written
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = item;
        false
    }
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn watch_signals(ui: slint::Weak<Panel>, dir: std::path::PathBuf) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(5));
            for (name, show) in [("show", true), ("hide", false)] {
                let flag = dir.join(name);
                if flag.exists() {
                    let _ = std::fs::remove_file(&flag);
                    let _ = ui.upgrade_in_event_loop(move |ui| {
                        if show {
                            let _ = ui.show();
                            ui.invoke_focus_search();
                        } else {
                            let _ = ui.hide();
                        }
                    });
                }
            }
        }
    });
}
