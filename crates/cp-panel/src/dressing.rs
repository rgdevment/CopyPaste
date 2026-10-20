use crate::here;
use crate::landing::Towards;
use cp_core::item::Item;
use cp_core::kind::Kind;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Copied {
    pub sequence: i64,
    pub id: i64,
    pub picture: bool,
}

impl Copied {
    pub fn of(before: Option<i64>, after: Option<i64>, id: i64, item: &Item) -> Option<Self> {
        let sequence = before.filter(|was| Some(*was) == after)?;
        Some(Self {
            sequence,
            id,
            picture: item.kind == Some(Kind::Image),
        })
    }
}

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

pub fn step(worn: Option<Worn>, terminal: bool, now: Option<i64>, copied: Option<Copied>) -> Step {
    match (worn, terminal) {
        (Some(_), true) => Step::Stay,
        (Some(was), false) => Step::Undress(was.id),
        (None, false) => Step::Stay,
        (None, true) => match copied {
            Some(copy) if copy.picture && Some(copy.sequence) == now => Step::Dress(copy.id),
            _ => Step::Stay,
        },
    }
}

pub trait Closet {
    fn copied(&self) -> Option<Copied>;
    fn copied_again(&self, sequence: i64);
    fn place(&self, id: i64, towards: Towards) -> bool;
    fn sequence(&self) -> Option<i64>;
}

pub struct Dressing<C: Closet> {
    closet: C,
    worn: Cell<Option<Worn>>,
    listening: RefCell<Option<here::Listening>>,
}

impl<C: Closet + 'static> Dressing<C> {
    pub fn start(closet: C) -> Rc<Self> {
        let dressing = Rc::new(Self {
            closet,
            worn: Cell::new(None),
            listening: RefCell::new(None),
        });
        let weak = Rc::downgrade(&dressing);
        let listening = here::on_foreground(move |window| {
            let later = weak.clone();
            slint::Timer::single_shot(Duration::ZERO, move || {
                if let Some(dressing) = later.upgrade() {
                    dressing.heard(window);
                }
            });
        });
        dressing.listening.replace(listening);
        dressing
    }

    pub fn listening(&self) -> bool {
        self.listening.borrow().is_some()
    }

    fn heard(&self, window: isize) {
        self.turn(
            here::towards(window) == Towards::Terminal,
            self.closet.sequence(),
        );
    }

    fn turn(&self, terminal: bool, now: Option<i64>) {
        let worn = still_worn(self.worn.get(), now);
        self.worn.set(worn);
        match step(worn, terminal, now, self.closet.copied()) {
            Step::Stay => {}
            Step::Dress(id) => {
                if self.closet.place(id, Towards::Terminal) {
                    let sequence = self.closet.sequence();
                    self.worn
                        .set(sequence.map(|sequence| Worn { sequence, id }));
                }
            }
            Step::Undress(id) => {
                self.worn.set(None);
                if self.closet.place(id, Towards::Elsewhere)
                    && let Some(sequence) = self.closet.sequence()
                {
                    self.closet.copied_again(sequence);
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "dressing_test.rs"]
mod tests;
