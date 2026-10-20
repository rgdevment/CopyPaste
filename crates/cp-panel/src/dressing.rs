use crate::here;
use crate::landing::Towards;
use cp_core::item::Item;
use cp_core::kind::Kind;
use std::cell::Cell;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const SETTLES: Duration = Duration::from_millis(60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Copied {
    pub sequence: i64,
    pub id: i64,
}

impl Copied {
    pub fn of(before: Option<i64>, after: Option<i64>, id: i64, item: &Item) -> Option<Self> {
        if item.kind != Some(Kind::Image) {
            return None;
        }
        let sequence = before.filter(|was| Some(*was) == after)?;
        Some(Self { sequence, id })
    }
}

pub type Latest = Arc<Mutex<Option<Copied>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Worn {
    pub sequence: i64,
    pub id: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Stay,
    Dress(i64),
    Undress(i64),
}

pub fn still_worn(worn: Option<Worn>, now: Option<i64>) -> Option<Worn> {
    worn.filter(|was| Some(was.sequence) == now)
}

pub fn still_copied(copied: Option<Copied>, now: Option<i64>) -> Option<Copied> {
    copied.filter(|copy| Some(copy.sequence) == now)
}

pub fn step(worn: Option<Worn>, terminal: bool, copied: Option<Copied>) -> Step {
    match (worn, terminal, copied) {
        (Some(_), true, _) => Step::Stay,
        (Some(was), false, _) => Step::Undress(was.id),
        (None, true, Some(copy)) => Step::Dress(copy.id),
        (None, _, _) => Step::Stay,
    }
}

pub trait Closet {
    fn place(&self, id: i64, towards: Towards, over: i64) -> Option<i64>;
    fn sequence(&self) -> Option<i64>;
    fn front(&self) -> isize;
    fn terminal(&self, window: isize) -> bool;
}

pub struct Dressing<C: Closet> {
    closet: C,
    latest: Latest,
    worn: Cell<Option<Worn>>,
    refused: Cell<Option<Copied>>,
    settling: slint::Timer,
    listening: Option<here::Listening>,
}

impl<C: Closet + 'static> Dressing<C> {
    pub fn start(closet: C, latest: Latest) -> Rc<Self> {
        Rc::new_cyclic(|weak: &Weak<Self>| {
            let weak = weak.clone();
            let listening = here::on_foreground(move |_window| {
                if let Some(dressing) = weak.upgrade() {
                    dressing.settle();
                }
            });
            Self {
                closet,
                latest,
                worn: Cell::new(None),
                refused: Cell::new(None),
                settling: slint::Timer::default(),
                listening,
            }
        })
    }

    pub fn listening(&self) -> bool {
        self.listening.is_some()
    }

    pub fn arrived(&self) {
        self.look(self.closet.front());
    }

    fn settle(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        self.settling
            .start(slint::TimerMode::SingleShot, SETTLES, move || {
                if let Some(dressing) = weak.upgrade() {
                    dressing.arrived();
                }
            });
    }

    pub fn put_back(&self) {
        let now = self.closet.sequence();
        if let Some(was) = still_worn(self.worn.get(), now) {
            self.undress(was);
        }
    }

    fn copied(&self) -> Option<Copied> {
        self.latest.lock().ok().and_then(|held| *held)
    }

    fn look(&self, window: isize) {
        let now = self.closet.sequence();
        let worn = still_worn(self.worn.get(), now);
        self.worn.set(worn);
        let copied =
            still_copied(self.copied(), now).filter(|copy| self.refused.get() != Some(*copy));
        if worn.is_none() && copied.is_none() {
            return;
        }
        match step(worn, self.closet.terminal(window), copied) {
            Step::Stay => {}
            Step::Dress(id) => {
                if let Some(over) = now {
                    let placed = self.closet.place(id, Towards::Terminal, over);
                    self.worn.set(placed.map(|sequence| Worn { sequence, id }));
                    if placed.is_none() {
                        self.refused.set(copied);
                    }
                }
            }
            Step::Undress(_) => {
                if let Some(was) = worn {
                    self.undress(was);
                }
            }
        }
    }

    fn undress(&self, was: Worn) {
        let Some(sequence) = self.closet.place(was.id, Towards::Elsewhere, was.sequence) else {
            return;
        };
        self.worn.set(None);
        if let Ok(mut held) = self.latest.lock() {
            *held = Some(Copied {
                sequence,
                id: was.id,
            });
        }
    }
}

#[cfg(test)]
#[path = "dressing_test.rs"]
mod tests;
