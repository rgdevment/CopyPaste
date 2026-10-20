use crate::Chooser;
use crate::handing::{Handed, PLAIN, forms_of, newest, paste_as};
use crate::here;
use crate::note::note;
use cp_store::{Listed, Store};
use slint::{ComponentHandle, ModelRc};
use std::cell::{Cell, OnceCell};
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::time::Duration;

const COMPLAINS_FOR: Duration = Duration::from_millis(2_200);

pub struct Aim {
    pub store: Rc<Store>,
    pub engine: Option<Rc<crate::engine::Engine>>,
    pub kept: crate::kept::Kept,
}

pub type Reach = Rc<dyn Fn() -> Aim>;

pub struct Quick {
    chooser: OnceCell<Chooser>,
    reach: Reach,
    back: Cell<usize>,
    ahead: Arc<AtomicIsize>,
    showing: Cell<Option<i64>>,
}

impl Quick {
    pub fn new(reach: Reach, ahead: Arc<AtomicIsize>) -> Rc<Self> {
        Rc::new(Self {
            chooser: OnceCell::new(),
            reach,
            back: Cell::new(0),
            ahead,
            showing: Cell::new(None),
        })
    }

    pub fn plain(self: &Rc<Self>) {
        self.hide();
        let ahead = self.ahead.load(Ordering::Relaxed);
        if ahead == 0 {
            note("nothing to paste plain into: no other window is in front");
            return;
        }
        let aim = (self.reach)();
        let Some((row, _)) = newest(&aim.store, 0) else {
            note("nothing to paste plain: the history is empty");
            return;
        };
        let towards = crate::landing::Towards::Elsewhere;
        let handed = paste_as(&aim.store, aim.engine.as_deref(), row.id, PLAIN, towards);
        if handed != Handed::Written {
            self.explain(handed.said());
            return;
        }
        let later = Rc::downgrade(self);
        here::paste_into(
            ahead,
            || {},
            move |sent| match sent {
                here::Sent::Done => {}
                here::Sent::Nobody => note("no window was left to paste plain into"),
                here::Sent::Degraded(why) => {
                    let said = crate::excuse::why_not(why, crate::say::in_english());
                    note(&format!(
                        "pasted plain onto the clipboard, unpasted: {why:?}"
                    ));
                    if let Some(quick) = later.upgrade() {
                        quick.explain(said);
                    }
                }
            },
        );
    }

    pub fn choose(self: &Rc<Self>) {
        let Some(chooser) = self.chooser() else {
            return;
        };
        self.back.set(0);
        if !self.fill(chooser) {
            note("nothing to paste as: the history is empty");
            return;
        }
        crate::app::dress_theme(chooser.global::<crate::Theme>(), (self.reach)().kept);
        chooser.set_complaint(Default::default());
        chooser.set_hints(
            crate::say::pick(
                "1-9 · ⏎ pegar · ←→ otro · esc",
                "1-9 · ⏎ paste · ←→ another · esc",
            )
            .into(),
        );
        appear(chooser);
    }

    pub fn ground(&self) {
        if let Some(handle) = self
            .chooser
            .get()
            .and_then(|chooser| crate::app::handle_of(chooser.window()))
        {
            here::ground(handle);
        }
    }

    fn explain(self: &Rc<Self>, said: &str) {
        self.choose();
        if let Some(chooser) = self.chooser.get().filter(|one| one.window().is_visible()) {
            complain(chooser, said);
        }
    }

    fn hide(&self) {
        if let Some(chooser) = self.chooser.get() {
            let _ = chooser.hide();
        }
    }

    fn chooser(self: &Rc<Self>) -> Option<&Chooser> {
        if let Some(chooser) = self.chooser.get() {
            return Some(chooser);
        }
        let chooser = match Chooser::new() {
            Ok(chooser) => chooser,
            Err(why) => {
                note(&format!("the paste as card could not be drawn: {why}"));
                return None;
            }
        };
        wire(self, &chooser);
        Some(self.chooser.get_or_init(|| chooser))
    }

    fn fill(&self, chooser: &Chooser) -> bool {
        let aim = (self.reach)();
        let back = self.back.get();
        let Some((row, older)) = newest(&aim.store, back) else {
            return false;
        };
        chooser.set_rows(ModelRc::from(Rc::new(slint::VecModel::from(forms_of(
            &aim.store, row.id,
        )))));
        chooser.set_current(0);
        chooser.set_kind(row.kind.map_or("text", cp_core::kind::Kind::as_str).into());
        chooser.set_subject(subject_of(&row).into());
        chooser.set_said_when(
            crate::age::age_text(crate::app::now_ms(), crate::view::touched_of(&row)).into(),
        );
        chooser.set_newer(back > 0);
        chooser.set_older(older);
        self.showing.set(Some(row.id));
        true
    }

    fn step(&self, chooser: &Chooser, by: i32) {
        let was = self.back.get();
        let Some(back) = was.checked_add_signed(by as isize) else {
            return;
        };
        self.back.set(back);
        if !self.fill(chooser) {
            self.back.set(was);
        }
        place(chooser);
    }

    fn chosen(self: &Rc<Self>, chooser: &Chooser, key: &str) {
        let Some(id) = self.showing.get() else {
            return;
        };
        let aim = (self.reach)();
        let ahead = self.ahead.load(Ordering::Relaxed);
        let handed = paste_as(
            &aim.store,
            aim.engine.as_deref(),
            id,
            key,
            here::towards(ahead),
        );
        if handed != Handed::Written {
            complain(chooser, handed.said());
            return;
        }
        let hiding = chooser.as_weak();
        let hide = move || {
            if let Some(chooser) = hiding.upgrade() {
                let _ = chooser.hide();
            }
        };
        let later = Rc::downgrade(self);
        slint::Timer::single_shot(Duration::ZERO, move || {
            here::paste_into(ahead, hide, move |sent| delivered(&later, sent));
        });
    }
}

fn delivered(later: &Weak<Quick>, sent: here::Sent) {
    let Some(chooser) = later
        .upgrade()
        .and_then(|quick| quick.chooser.get().map(ComponentHandle::clone_strong))
    else {
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
            appear(&chooser);
            complain(&chooser, said);
        }
    }
}

fn wire(quick: &Rc<Quick>, chooser: &Chooser) {
    let (weak, own) = (Rc::downgrade(quick), chooser.as_weak());
    chooser.on_chosen(move |key| {
        if let (Some(quick), Some(chooser)) = (weak.upgrade(), own.upgrade()) {
            quick.chosen(&chooser, key.as_str());
        }
    });
    let (weak, own) = (Rc::downgrade(quick), chooser.as_weak());
    chooser.on_step(move |by| {
        if let (Some(quick), Some(chooser)) = (weak.upgrade(), own.upgrade()) {
            quick.step(&chooser, by);
        }
    });
    let weak: Weak<Quick> = Rc::downgrade(quick);
    chooser.on_dismiss(move || {
        if let Some(quick) = weak.upgrade() {
            quick.hide();
            here::hand_back(quick.ahead.load(Ordering::Relaxed));
        }
    });
    leave_when_left(chooser);
}

fn appear(chooser: &Chooser) {
    place(chooser);
    if chooser.show().is_err() {
        return;
    }
    crate::showing::rescaled(chooser.window(), kept_zoom(chooser));
    place(chooser);
    if let Some(handle) = crate::app::handle_of(chooser.window()) {
        here::forward(handle);
    }
    chooser.invoke_focus_keys();
}

fn complain(chooser: &Chooser, said: &str) {
    chooser.set_complaint(said.into());
    let weak = chooser.as_weak();
    slint::Timer::single_shot(COMPLAINS_FOR, move || {
        if let Some(chooser) = weak.upgrade() {
            chooser.set_complaint(Default::default());
        }
    });
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
    let folder = matches!(
        row.kind,
        Some(cp_core::kind::Kind::File | cp_core::kind::Kind::Folder)
    )
    .then(|| crate::folder::parts_of(&row.preview))
    .flatten();
    let said = crate::view::headline_of(row, &crate::view::body_of(row), folder.as_ref());
    if said.is_empty() {
        crate::view::label_of(row.kind).to_owned()
    } else {
        said
    }
}

fn place(chooser: &Chooser) {
    let zoom = f64::from(kept_zoom(chooser));
    let size = (
        f64::from(chooser.get_wide()) * zoom,
        f64::from(chooser.get_tall()) * zoom,
    );
    crate::showing::beside_pointer(chooser.window(), size);
}

#[cfg(test)]
#[path = "quick_test.rs"]
mod tests;
