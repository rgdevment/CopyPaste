use super::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn picture(sequence: i64, id: i64) -> Option<Copied> {
    Some(Copied {
        sequence,
        id,
        picture: true,
    })
}

fn item(kind: Kind) -> Item {
    Item {
        kind: Some(kind),
        formats: Vec::new(),
    }
}

#[test]
fn a_copy_is_remembered_only_when_nothing_was_copied_while_it_was_read() {
    let image = item(Kind::Image);
    assert_eq!(Copied::of(Some(7), Some(7), 3, &image), picture(7, 3));
    assert_eq!(Copied::of(Some(7), Some(8), 3, &image), None);
    assert_eq!(Copied::of(None, None, 3, &image), None);
    let words = Copied::of(Some(7), Some(7), 3, &item(Kind::Text)).expect("remembered");
    assert!(!words.picture);
}

#[test]
fn a_picture_still_on_the_clipboard_gets_its_path_in_a_terminal() {
    assert_eq!(step(None, true, Some(7), picture(7, 3)), Step::Dress(3));
}

#[test]
fn nothing_is_dressed_outside_a_terminal_or_once_something_newer_was_copied() {
    assert_eq!(step(None, false, Some(7), picture(7, 3)), Step::Stay);
    assert_eq!(step(None, true, Some(8), picture(7, 3)), Step::Stay);
    assert_eq!(step(None, true, None, picture(7, 3)), Step::Stay);
    assert_eq!(step(None, true, Some(7), None), Step::Stay);
    let words = Some(Copied {
        sequence: 7,
        id: 3,
        picture: false,
    });
    assert_eq!(step(None, true, Some(7), words), Step::Stay);
}

#[test]
fn leaving_the_terminal_takes_the_path_off_and_moving_between_terminals_keeps_it() {
    let worn = Some(Worn { sequence: 9, id: 3 });
    assert_eq!(step(worn, false, Some(9), picture(7, 3)), Step::Undress(3));
    assert_eq!(step(worn, true, Some(9), picture(7, 3)), Step::Stay);
}

#[test]
fn a_dress_is_forgotten_as_soon_as_anyone_else_writes_the_clipboard() {
    let worn = Some(Worn { sequence: 9, id: 3 });
    assert_eq!(still_worn(worn, Some(9)), worn);
    assert_eq!(still_worn(worn, Some(10)), None);
    assert_eq!(still_worn(worn, None), None);
    assert_eq!(still_worn(None, Some(9)), None);
    assert_eq!(
        step(still_worn(worn, Some(10)), false, Some(10), picture(7, 3)),
        Step::Stay,
        "someone else's copy is never undressed"
    );
}

#[derive(Default)]
struct Shelf {
    copied: Cell<Option<Copied>>,
    sequence: Cell<i64>,
    placed: RefCell<Vec<(i64, Towards)>>,
    refuses: Cell<bool>,
    blind: Cell<bool>,
}

impl Closet for Rc<Shelf> {
    fn copied(&self) -> Option<Copied> {
        self.copied.get()
    }

    fn copied_again(&self, sequence: i64) {
        if let Some(mut copy) = self.copied.get() {
            copy.sequence = sequence;
            self.copied.set(Some(copy));
        }
    }

    fn place(&self, id: i64, towards: Towards) -> bool {
        if self.refuses.get() {
            return false;
        }
        self.placed.borrow_mut().push((id, towards));
        self.sequence.set(self.sequence.get() + 1);
        true
    }

    fn sequence(&self) -> Option<i64> {
        (!self.blind.get()).then(|| self.sequence.get())
    }
}

fn wardrobe(sequence: i64, copied: Option<Copied>) -> (Rc<Shelf>, Dressing<Rc<Shelf>>) {
    let shelf = Rc::new(Shelf::default());
    shelf.sequence.set(sequence);
    shelf.copied.set(copied);
    let dressing = Dressing {
        closet: shelf.clone(),
        worn: Cell::new(None),
        listening: RefCell::new(None),
    };
    (shelf, dressing)
}

#[test]
fn into_a_terminal_and_out_again_puts_the_path_on_and_takes_it_off() {
    let (shelf, dressing) = wardrobe(7, picture(7, 3));
    dressing.turn(true, shelf.sequence());
    assert_eq!(dressing.worn.get(), Some(Worn { sequence: 8, id: 3 }));
    dressing.turn(true, shelf.sequence());
    assert_eq!(
        shelf.placed.borrow().len(),
        1,
        "a second terminal changes nothing"
    );
    dressing.turn(false, shelf.sequence());
    assert_eq!(
        *shelf.placed.borrow(),
        [(3, Towards::Terminal), (3, Towards::Elsewhere)]
    );
    assert_eq!(dressing.worn.get(), None);
    assert_eq!(
        shelf.copied.get(),
        picture(9, 3),
        "the picture is known again"
    );
    dressing.turn(true, shelf.sequence());
    assert_eq!(
        shelf.placed.borrow().len(),
        3,
        "and it dresses again next time"
    );
}

#[test]
fn a_copy_made_inside_the_terminal_is_left_alone_on_the_way_out() {
    let (shelf, dressing) = wardrobe(7, picture(7, 3));
    dressing.turn(true, shelf.sequence());
    shelf.sequence.set(20);
    dressing.turn(false, shelf.sequence());
    assert_eq!(shelf.placed.borrow().len(), 1);
    assert_eq!(dressing.worn.get(), None);
}

#[test]
fn a_write_the_clipboard_refuses_leaves_nothing_worn() {
    let (shelf, dressing) = wardrobe(7, picture(7, 3));
    shelf.refuses.set(true);
    dressing.turn(true, shelf.sequence());
    assert_eq!(dressing.worn.get(), None);
    shelf.refuses.set(false);
    dressing.turn(true, shelf.sequence());
    shelf.refuses.set(true);
    dressing.turn(false, shelf.sequence());
    assert_eq!(dressing.worn.get(), None);
    assert_eq!(
        shelf.copied.get(),
        picture(7, 3),
        "nothing claims the clipboard went back"
    );
}

#[test]
fn a_clipboard_that_cannot_be_counted_is_never_dressed() {
    let (shelf, dressing) = wardrobe(7, picture(7, 3));
    shelf.blind.set(true);
    dressing.turn(true, shelf.sequence());
    assert!(shelf.placed.borrow().is_empty());
    shelf.blind.set(false);
    dressing.turn(true, shelf.sequence());
    shelf.blind.set(true);
    dressing.turn(true, Some(8));
    assert_eq!(dressing.worn.get(), Some(Worn { sequence: 8, id: 3 }));
}

#[test]
fn a_window_that_is_no_terminal_coming_forward_takes_the_path_off() {
    let (shelf, dressing) = wardrobe(7, picture(7, 3));
    dressing.turn(true, shelf.sequence());
    dressing.heard(0);
    assert_eq!(shelf.placed.borrow().last(), Some(&(3, Towards::Elsewhere)));
}

#[test]
fn starting_listens_for_windows_where_the_system_can_tell() {
    let dressing = Dressing::start(Rc::new(Shelf::default()));
    assert_eq!(dressing.listening(), cfg!(target_os = "windows"));
}
