use super::*;
use std::cell::RefCell;

const TERMINAL: isize = 1;
const CHAT: isize = 2;

fn item(kind: Kind) -> Item {
    Item {
        kind: Some(kind),
        formats: Vec::new(),
    }
}

#[test]
fn only_a_picture_read_while_nothing_else_was_copied_is_remembered() {
    let image = item(Kind::Image);
    assert_eq!(
        Copied::of(Some(7), Some(7), 3, &image),
        Some(Copied { sequence: 7, id: 3 })
    );
    assert_eq!(Copied::of(Some(7), Some(8), 3, &image), None);
    assert_eq!(Copied::of(None, None, 3, &image), None);
    assert_eq!(Copied::of(Some(7), Some(7), 3, &item(Kind::Text)), None);
}

#[test]
fn the_steps_put_the_path_on_in_a_terminal_and_take_it_off_elsewhere() {
    let copy = Some(Copied { sequence: 7, id: 3 });
    let worn = Some(Worn { sequence: 9, id: 3 });
    assert_eq!(step(None, true, copy), Step::Dress(3));
    assert_eq!(step(None, false, copy), Step::Stay);
    assert_eq!(step(None, true, None), Step::Stay);
    assert_eq!(step(worn, true, copy), Step::Stay);
    assert_eq!(step(worn, false, None), Step::Undress(3));
}

#[test]
fn what_is_worn_or_copied_counts_only_while_the_clipboard_still_holds_it() {
    let worn = Some(Worn { sequence: 9, id: 3 });
    let copy = Some(Copied { sequence: 7, id: 3 });
    assert_eq!(still_worn(worn, Some(9)), worn);
    assert_eq!(still_worn(worn, Some(10)), None);
    assert_eq!(still_worn(worn, None), None);
    assert_eq!(still_copied(copy, Some(7)), copy);
    assert_eq!(still_copied(copy, Some(8)), None);
    assert_eq!(still_copied(None, Some(7)), None);
}

#[derive(Default)]
struct Shelf {
    sequence: Cell<i64>,
    front: Cell<isize>,
    placed: RefCell<Vec<(i64, Towards, i64)>>,
    refuses: Cell<bool>,
    blind: Cell<bool>,
    looked: Cell<usize>,
}

impl Closet for Rc<Shelf> {
    fn place(&self, id: i64, towards: Towards, over: i64) -> Option<i64> {
        if self.refuses.get() || over != self.sequence.get() {
            return None;
        }
        self.placed.borrow_mut().push((id, towards, over));
        self.sequence.set(self.sequence.get() + 1);
        Some(self.sequence.get())
    }

    fn sequence(&self) -> Option<i64> {
        (!self.blind.get()).then(|| self.sequence.get())
    }

    fn front(&self) -> isize {
        self.front.get()
    }

    fn terminal(&self, window: isize) -> bool {
        self.looked.set(self.looked.get() + 1);
        window == TERMINAL
    }
}

fn wardrobe(copied: Option<Copied>) -> (Rc<Shelf>, Dressing<Rc<Shelf>>, Latest) {
    let shelf = Rc::new(Shelf::default());
    shelf.sequence.set(7);
    let latest: Latest = Arc::new(Mutex::new(copied));
    let dressing = Dressing {
        closet: shelf.clone(),
        latest: latest.clone(),
        worn: Cell::new(None),
        listening: None,
    };
    (shelf, dressing, latest)
}

fn picture() -> Option<Copied> {
    Some(Copied { sequence: 7, id: 3 })
}

#[test]
fn into_a_terminal_and_out_again_puts_the_path_on_and_takes_it_off() {
    let (shelf, dressing, latest) = wardrobe(picture());
    dressing.look(TERMINAL);
    assert_eq!(dressing.worn.get(), Some(Worn { sequence: 8, id: 3 }));
    dressing.look(TERMINAL);
    assert_eq!(
        shelf.placed.borrow().len(),
        1,
        "a second terminal changes nothing"
    );
    dressing.look(CHAT);
    assert_eq!(
        *shelf.placed.borrow(),
        [(3, Towards::Terminal, 7), (3, Towards::Elsewhere, 8)]
    );
    assert_eq!(dressing.worn.get(), None);
    assert_eq!(
        *latest.lock().expect("unpoisoned"),
        Some(Copied { sequence: 9, id: 3 }),
        "the picture alone is known as the copy again"
    );
    dressing.look(TERMINAL);
    assert_eq!(
        shelf.placed.borrow().len(),
        3,
        "and it dresses again next time"
    );
}

#[test]
fn a_picture_that_arrives_while_a_terminal_is_in_front_is_dressed_right_away() {
    let (shelf, dressing, latest) = wardrobe(None);
    shelf.front.set(TERMINAL);
    dressing.arrived();
    assert!(shelf.placed.borrow().is_empty());
    *latest.lock().expect("unpoisoned") = picture();
    dressing.arrived();
    assert_eq!(*shelf.placed.borrow(), [(3, Towards::Terminal, 7)]);
}

#[test]
fn copying_text_never_asks_which_window_is_in_front() {
    let (shelf, dressing, _) = wardrobe(None);
    dressing.look(TERMINAL);
    dressing.look(CHAT);
    assert_eq!(shelf.looked.get(), 0);
}

#[test]
fn a_copy_made_inside_the_terminal_is_left_alone_on_the_way_out() {
    let (shelf, dressing, _) = wardrobe(picture());
    dressing.look(TERMINAL);
    shelf.sequence.set(20);
    dressing.look(CHAT);
    assert_eq!(shelf.placed.borrow().len(), 1);
    assert_eq!(dressing.worn.get(), None);
}

#[test]
fn a_dress_the_clipboard_refuses_leaves_nothing_worn() {
    let (shelf, dressing, _) = wardrobe(picture());
    shelf.refuses.set(true);
    dressing.look(TERMINAL);
    assert_eq!(dressing.worn.get(), None);
    assert!(shelf.placed.borrow().is_empty());
}

#[test]
fn a_failed_undress_is_tried_again_at_the_next_window() {
    let (shelf, dressing, _) = wardrobe(picture());
    dressing.look(TERMINAL);
    shelf.refuses.set(true);
    dressing.look(CHAT);
    assert_eq!(dressing.worn.get(), Some(Worn { sequence: 8, id: 3 }));
    shelf.refuses.set(false);
    dressing.look(CHAT);
    assert_eq!(dressing.worn.get(), None);
    assert_eq!(
        shelf.placed.borrow().last(),
        Some(&(3, Towards::Elsewhere, 8))
    );
}

#[test]
fn a_clipboard_that_cannot_be_counted_is_never_touched() {
    let (shelf, dressing, _) = wardrobe(picture());
    shelf.blind.set(true);
    dressing.look(TERMINAL);
    assert!(shelf.placed.borrow().is_empty());
}

#[test]
fn quitting_takes_the_path_off_only_while_it_is_still_on() {
    let (shelf, dressing, _) = wardrobe(picture());
    dressing.put_back();
    assert!(shelf.placed.borrow().is_empty());
    dressing.look(TERMINAL);
    dressing.put_back();
    assert_eq!(
        shelf.placed.borrow().last(),
        Some(&(3, Towards::Elsewhere, 8))
    );
    let (shelf, dressing, _) = wardrobe(picture());
    dressing.look(TERMINAL);
    shelf.sequence.set(30);
    dressing.put_back();
    assert_eq!(
        shelf.placed.borrow().len(),
        1,
        "someone else's copy is left alone"
    );
}

#[test]
fn starting_listens_for_windows_where_the_system_can_tell() {
    let dressing = Dressing::start(Rc::new(Shelf::default()), Arc::new(Mutex::new(None)));
    assert_eq!(dressing.listening(), cfg!(target_os = "windows"));
}
