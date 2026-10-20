use crate::Chooser;
use crate::handing::{forms_of, newest, paste_as, paste_plain};
use crate::here;
use crate::note::note;
use cp_store::{Listed, Store};
use slint::{ComponentHandle, ModelRc};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

const WIDE: f64 = 340.0;
const ROW: f64 = 34.0;
const FRAME: f64 = 80.0;
const COMPLAINS_FOR: Duration = Duration::from_millis(2_200);

pub struct Aim {
    pub store: Rc<Store>,
    pub engine: Option<Rc<crate::engine::Engine>>,
    pub kept: crate::kept::Kept,
}

pub type Reach = Rc<dyn Fn() -> Aim>;

pub struct Quick {
    chooser: Chooser,
    reach: Reach,
    back: Cell<usize>,
    ahead: Cell<isize>,
    showing: Cell<Option<i64>>,
}

impl Quick {
    pub fn start(reach: Reach) -> Result<Rc<Self>, slint::PlatformError> {
        let quick = Rc::new(Self {
            chooser: Chooser::new()?,
            reach,
            back: Cell::new(0),
            ahead: Cell::new(0),
            showing: Cell::new(None),
        });
        wire(&quick);
        Ok(quick)
    }

    pub fn plain(&self, ahead: isize) {
        let aim = (self.reach)();
        let Some(row) = newest(&aim.store, 0) else {
            note("nothing to paste plain: the history is empty");
            return;
        };
        if !paste_plain(&aim.store, aim.engine.as_deref(), row.id) {
            return;
        }
        here::paste_into(
            ahead,
            || {},
            |sent| {
                if let here::Sent::Degraded(why) = sent {
                    note(&format!(
                        "pasted plain onto the clipboard, unpasted: {why:?}"
                    ));
                }
            },
        );
    }

    pub fn choose(&self, ahead: isize) {
        self.ahead.set(ahead);
        self.back.set(0);
        if !self.fill() {
            note("nothing to paste as: the history is empty");
            return;
        }
        let kept = (self.reach)().kept;
        crate::app::dress_theme(self.chooser.global::<crate::Theme>(), kept);
        self.chooser.set_complaint(Default::default());
        self.chooser.set_hints(
            crate::say::pick(
                "1-9 · ⏎ pegar · ←→ otro · esc",
                "1-9 · ⏎ paste · ←→ another · esc",
            )
            .into(),
        );
        self.appear();
    }

    fn appear(&self) {
        place(&self.chooser);
        if self.chooser.show().is_err() {
            return;
        }
        crate::showing::rescaled(self.chooser.window(), kept_zoom(&self.chooser));
        place(&self.chooser);
        if let Some(handle) = crate::app::handle_of(self.chooser.window()) {
            here::forward(handle);
        }
        self.chooser.invoke_focus_keys();
    }

    fn fill(&self) -> bool {
        let aim = (self.reach)();
        let back = self.back.get();
        let Some(row) = newest(&aim.store, back) else {
            return false;
        };
        let rows = forms_of(&aim.store, row.id);
        self.chooser
            .set_rows(ModelRc::from(Rc::new(slint::VecModel::from(rows))));
        self.chooser.set_current(0);
        self.chooser
            .set_kind(row.kind.map_or("text", cp_core::kind::Kind::as_str).into());
        self.chooser.set_subject(subject_of(&row).into());
        self.chooser.set_said_when(
            crate::age::age_text(crate::app::now_ms(), crate::view::touched_of(&row)).into(),
        );
        self.chooser.set_newer(back > 0);
        self.chooser
            .set_older(newest(&aim.store, back + 1).is_some());
        self.showing.set(Some(row.id));
        true
    }

    fn step(&self, by: i32) {
        let was = self.back.get();
        let Some(back) = was.checked_add_signed(by as isize) else {
            return;
        };
        self.back.set(back);
        if !self.fill() {
            self.back.set(was);
            self.fill();
        }
        place(&self.chooser);
    }

    fn chosen(&self, key: &str) {
        let Some(id) = self.showing.get() else {
            return;
        };
        let aim = (self.reach)();
        let ahead = self.ahead.get();
        if !paste_as(
            &aim.store,
            aim.engine.as_deref(),
            id,
            key,
            here::towards(ahead),
        ) {
            self.complain(crate::say::pick(
                "no se pudo pegar: el portapapeles está ocupado",
                "could not paste: the clipboard is busy",
            ));
            return;
        }
        let (hiding, later) = (self.chooser.as_weak(), self.chooser.as_weak());
        let hide = move || {
            if let Some(chooser) = hiding.upgrade() {
                let _ = chooser.hide();
            }
        };
        here::paste_into(ahead, hide, move |sent| {
            let Some(chooser) = later.upgrade() else {
                return;
            };
            match sent {
                here::Sent::Done => {}
                here::Sent::Nobody => {
                    note("no window was left to paste into, it stays on the clipboard");
                    let _ = chooser.hide();
                }
                here::Sent::Degraded(why) => {
                    let said = crate::excuse::why_not(why, crate::say::in_english());
                    note(&format!(
                        "it stays on the clipboard, unpasted: {why:?}: {said}"
                    ));
                    chooser.set_complaint(said.into());
                    let _ = chooser.show();
                }
            }
        });
    }

    fn complain(&self, said: &str) {
        self.chooser.set_complaint(said.into());
        let weak = self.chooser.as_weak();
        slint::Timer::single_shot(COMPLAINS_FOR, move || {
            if let Some(chooser) = weak.upgrade() {
                chooser.set_complaint(Default::default());
            }
        });
    }
}

fn wire(quick: &Rc<Quick>) {
    let chooser = &quick.chooser;
    let weak = Rc::downgrade(quick);
    chooser.on_chosen(move |key| {
        if let Some(quick) = weak.upgrade() {
            quick.chosen(key.as_str());
        }
    });
    let weak = Rc::downgrade(quick);
    chooser.on_step(move |by| {
        if let Some(quick) = weak.upgrade() {
            quick.step(by);
        }
    });
    let weak = Rc::downgrade(quick);
    chooser.on_dismiss(move || {
        if let Some(quick) = weak.upgrade() {
            let _ = quick.chooser.hide();
            here::hand_back(quick.ahead.get());
        }
    });
    leave_when_left(chooser);
}

fn leave_when_left(chooser: &Chooser) {
    use i_slint_backend_winit::winit::event::WindowEvent;
    use i_slint_backend_winit::{EventResult, WinitWindowAccessor};
    let weak = chooser.as_weak();
    chooser.window().on_winit_window_event(move |_, event| {
        if let WindowEvent::Focused(false) = event
            && let Some(chooser) = weak.upgrade()
            && chooser.window().is_visible()
        {
            let later = chooser.as_weak();
            slint::Timer::single_shot(Duration::ZERO, move || {
                if let Some(chooser) = later.upgrade() {
                    let _ = chooser.hide();
                }
            });
        }
        EventResult::Propagate
    });
}

fn kept_zoom(chooser: &Chooser) -> f32 {
    chooser.global::<crate::Theme>().get_zoom()
}

fn subject_of(row: &Listed) -> String {
    if let Some(label) = row.label.as_deref().filter(|one| !one.trim().is_empty()) {
        return label.trim().to_owned();
    }
    let said = crate::view::squeezed_of(&crate::view::body_of(row));
    if said.is_empty() {
        crate::view::label_of(row.kind).to_owned()
    } else {
        said
    }
}

fn tall(rows: usize) -> f64 {
    FRAME + ROW * rows as f64
}

fn place(chooser: &Chooser) {
    use slint::Model;
    let Some(pointer) = here::pointer() else {
        return;
    };
    let zoom = f64::from(kept_zoom(chooser));
    let size = (WIDE * zoom, tall(chooser.get_rows().row_count()) * zoom);
    let window = chooser.window();
    if pointer.physical {
        let scale = pointer.scale;
        let (x, y) = crate::placing::beside(
            pointer.at,
            pointer.area,
            (size.0 * scale, size.1 * scale),
            scale,
        );
        #[allow(clippy::cast_possible_truncation)]
        window.set_position(slint::PhysicalPosition::new(
            x.round() as i32,
            y.round() as i32,
        ));
    } else {
        let (x, y) = crate::placing::beside(pointer.at, pointer.area, size, 1.0);
        #[allow(clippy::cast_possible_truncation)]
        window.set_position(slint::LogicalPosition::new(x as f32, y as f32));
    }
}

#[cfg(test)]
#[path = "quick_test.rs"]
mod tests;
