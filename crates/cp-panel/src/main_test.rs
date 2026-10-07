use super::*;
use slint::platform::{PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, Model, VecModel};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

const HEAD: f32 = 23.0;
const GAP: f32 = 6.0;
const SHUT: f32 = 58.0;
const OPEN: f32 = 190.0;
const BIG: f32 = 340.0;

static PLATFORM: std::sync::Once = std::sync::Once::new();

fn card(id: i32) -> Card {
    Card {
        id,
        kind: "text".into(),
        title: format!("card {id}").into(),
        body: "some words".into(),
        face: "words".into(),
        opened: "some words\nmore words\nand more".into(),
        shut_lines: 1,
        open_lines: 3,
        shut_px: SHUT,
        open_px: if id == 1 { BIG } else { OPEN },
        heads_group: id == 0,
        group_said: "Hoy".into(),
        can_drag: true,
        ..Default::default()
    }
}

struct Bench {
    panel: Panel,
    cards: Rc<VecModel<Card>>,
    pasted: Rc<RefCell<Vec<i32>>>,
    moved: Rc<RefCell<Vec<i32>>>,
}

fn tick(ms: u64) {
    i_slint_backend_testing::mock_elapsed_time(Duration::from_millis(ms));
}

fn height_of(cards: &VecModel<Card>, open: Option<usize>, index: usize) -> f32 {
    let card = cards.row_data(index).expect("a row");
    let head = if card.heads_group { HEAD } else { 0.0 };
    head + if open == Some(index) {
        card.open_px
    } else {
        card.shut_px
    }
}

fn span_of(cards: &VecModel<Card>, open: Option<usize>, index: usize) -> (f32, f32) {
    let top = (0..index).map(|i| height_of(cards, open, i)).sum();
    (top, height_of(cards, open, index))
}

fn open_one(panel: &Panel) -> Option<usize> {
    (panel.get_opened() && panel.get_current() >= 0).then(|| panel.get_current() as usize)
}

fn bench(count: i32) -> Bench {
    PLATFORM.call_once(i_slint_backend_testing::init_no_event_loop);
    let panel = Panel::new().expect("the panel builds");
    panel
        .window()
        .set_size(slint::LogicalSize::new(480.0, 620.0));
    let cards = Rc::new(VecModel::from((0..count).map(card).collect::<Vec<_>>()));
    panel.set_cards(cards.clone().into());
    panel.show().expect("the panel shows");
    let pasted = Rc::new(RefCell::new(Vec::new()));
    let seen = pasted.clone();
    panel.on_paste(move |id| seen.borrow_mut().push(id));
    let moved = Rc::new(RefCell::new(Vec::new()));
    let noted = moved.clone();
    let weak = panel.as_weak();
    let rows = cards.clone();
    panel.on_moved(move |index| {
        noted.borrow_mut().push(index);
        let panel = weak.unwrap();
        let (top, span) = span_of(&rows, open_one(&panel), index as usize);
        panel.set_scroll_y(crate::model::reveal(
            top,
            span,
            panel.get_scroll_y(),
            panel.get_viewport_height(),
        ));
    });
    tick(600);
    Bench {
        panel,
        cards,
        pasted,
        moved,
    }
}

fn centre_of(bench: &Bench, index: usize) -> LogicalPosition {
    let panel = &bench.panel;
    let (top, span) = span_of(&bench.cards, open_one(panel), index);
    let head = if bench.cards.row_data(index).expect("a row").heads_group {
        HEAD
    } else {
        0.0
    };
    let list_top = panel.global::<Theme>().get_margin() + panel.get_list_top();
    let y = list_top + panel.get_scroll_y() + top + head + (span - head - GAP) / 2.0;
    LogicalPosition::new(200.0, y)
}

fn click(panel: &Panel, at: LogicalPosition, hold: u64) {
    let window = panel.window();
    window.dispatch_event(WindowEvent::PointerMoved { position: at });
    window.dispatch_event(WindowEvent::PointerPressed {
        position: at,
        button: PointerEventButton::Left,
    });
    tick(hold);
    window.dispatch_event(WindowEvent::PointerReleased {
        position: at,
        button: PointerEventButton::Left,
    });
}

#[test]
fn a_double_click_pastes_the_card_under_it_whatever_was_open_and_however_slow() {
    the_card_opened_at_the_bottom_ends_up_whole_on_screen();
    let mut failed = Vec::new();
    for gap in [150u64, 250, 350, 400] {
        for (open_first, target) in [
            (None, 1usize),
            (None, 4),
            (None, 7),
            (Some(0usize), 3),
            (Some(6), 2),
            (Some(2), 2),
            (Some(1), 2),
            (Some(1), 1),
        ] {
            let bench = bench(12);
            if let Some(first) = open_first {
                click(&bench.panel, centre_of(&bench, first), 60);
                tick(600);
            }
            bench.moved.borrow_mut().clear();
            let at = centre_of(&bench, target);
            let floor = bench.panel.global::<Theme>().get_margin()
                + bench.panel.get_list_top()
                + bench.panel.get_viewport_height();
            assert!(at.y < floor, "card {target} is out of sight at {}", at.y);
            click(&bench.panel, at, 90);
            tick(gap);
            click(&bench.panel, at, 90);
            tick(600);
            let pasted = bench.pasted.borrow().clone();
            let moved = bench.moved.borrow().clone();
            if pasted != [target as i32] || moved.iter().any(|&one| one != target as i32) {
                failed.push(format!(
                    "{gap} ms, open {open_first:?}, card {target}: pasted {pasted:?}, clicked {moved:?}"
                ));
            }
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

fn the_card_opened_at_the_bottom_ends_up_whole_on_screen() {
    let bench = bench(12);
    let target = 7;
    click(&bench.panel, centre_of(&bench, target), 60);
    tick(600);
    let (top, span) = span_of(&bench.cards, open_one(&bench.panel), target);
    let bottom = top + span + bench.panel.get_scroll_y();
    assert!(
        bottom <= bench.panel.get_viewport_height(),
        "the open card ends at {bottom}, past {}",
        bench.panel.get_viewport_height()
    );
}

fn press(panel: &Panel, key: slint::platform::Key) {
    let window = panel.window();
    window.dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window.dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}

#[test]
fn enter_pastes_the_card_under_the_pointer_not_the_one_chosen_by_keys() {
    let bench = bench(6);
    bench.panel.invoke_focus_search();
    bench.panel.set_current(0);
    bench.panel.set_hovered(3);
    press(&bench.panel, slint::platform::Key::Return);
    assert_eq!(*bench.pasted.borrow(), [3]);
    assert_eq!(bench.panel.get_current(), 3);
}

#[test]
fn enter_with_the_pointer_away_pastes_the_card_chosen_by_keys() {
    let bench = bench(6);
    bench.panel.invoke_focus_search();
    bench.panel.set_current(2);
    bench.panel.set_hovered(-1);
    press(&bench.panel, slint::platform::Key::Return);
    assert_eq!(*bench.pasted.borrow(), [2]);
}
