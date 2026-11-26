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

thread_local! {
    static PLATFORM: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

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
    tapped: Rc<RefCell<Vec<(i32, bool)>>>,
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
    if !PLATFORM.replace(true) {
        i_slint_backend_testing::init_no_event_loop();
    }
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
    let tapped = Rc::new(RefCell::new(Vec::new()));
    let counted = tapped.clone();
    panel.on_paste_clicked(move |id, plain| counted.borrow_mut().push((id, plain)));
    tick(600);
    Bench {
        panel,
        cards,
        pasted,
        moved,
        tapped,
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
    let list_top = panel.get_list_top();
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
            let floor = bench.panel.get_list_top() + bench.panel.get_viewport_height();
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

fn one_click_bench(count: i32) -> Bench {
    let bench = bench(count);
    bench.panel.set_one_click(true);
    tick(100);
    bench
}

fn shifted_click(panel: &Panel, at: LogicalPosition) {
    let window = panel.window();
    let shift: slint::SharedString = slint::platform::Key::Shift.into();
    window.dispatch_event(WindowEvent::KeyPressed {
        text: shift.clone(),
    });
    click(panel, at, 60);
    window.dispatch_event(WindowEvent::KeyReleased { text: shift });
}

fn grid_bench(count: i32) -> (Bench, f32) {
    let bench = one_click_bench(count);
    let top = bench.panel.get_list_top();
    bench.panel.set_layout("image".into());
    bench.panel.set_way("grid".into());
    let lines = (0..(count + 1) / 2).collect::<Vec<i32>>();
    bench.panel.set_grid_lines(VecModel::from_slice(&lines));
    tick(100);
    (bench, top)
}

fn grid_cell(bench: &Bench, top: f32, line: usize, left: bool) -> LogicalPosition {
    let cell = bench.panel.global::<crate::Theme>().get_cell();
    let y = top + line as f32 * cell + cell / 2.0 - 4.0;
    LogicalPosition::new(if left { 120.0 } else { 360.0 }, y)
}

#[test]
fn with_one_click_on_a_single_click_pastes_the_card_and_opens_nothing() {
    let bench = one_click_bench(8);
    click(&bench.panel, centre_of(&bench, 3), 60);
    tick(600);
    assert_eq!(*bench.tapped.borrow(), [(3, false)]);
    assert!(bench.pasted.borrow().is_empty());
    assert!(bench.moved.borrow().is_empty());
    assert_eq!(open_one(&bench.panel), None);
}

#[test]
fn with_one_click_on_shift_and_click_asks_for_plain() {
    let bench = one_click_bench(8);
    shifted_click(&bench.panel, centre_of(&bench, 2));
    tick(600);
    assert_eq!(*bench.tapped.borrow(), [(2, true)]);
    assert!(bench.pasted.borrow().is_empty());
}

#[test]
fn with_one_click_on_a_click_after_a_shift_click_is_not_plain() {
    let bench = one_click_bench(8);
    shifted_click(&bench.panel, centre_of(&bench, 2));
    tick(600);
    click(&bench.panel, centre_of(&bench, 4), 60);
    assert_eq!(*bench.tapped.borrow(), [(2, true), (4, false)]);
}

#[test]
fn with_one_click_on_dragging_a_card_past_the_start_does_not_paste_it() {
    let bench = one_click_bench(8);
    let window = bench.panel.window();
    let from = centre_of(&bench, 3);
    let to = LogicalPosition::new(from.x + 30.0, from.y);
    window.dispatch_event(WindowEvent::PointerMoved { position: from });
    window.dispatch_event(WindowEvent::PointerPressed {
        position: from,
        button: PointerEventButton::Left,
    });
    tick(60);
    window.dispatch_event(WindowEvent::PointerMoved { position: to });
    tick(60);
    window.dispatch_event(WindowEvent::PointerReleased {
        position: to,
        button: PointerEventButton::Left,
    });
    tick(600);
    assert!(bench.tapped.borrow().is_empty());
    assert!(bench.pasted.borrow().is_empty());
}

#[test]
fn with_one_click_on_a_click_while_the_sheet_is_open_only_closes_it() {
    let bench = one_click_bench(8);
    bench.panel.set_sheet_open(true);
    click(&bench.panel, centre_of(&bench, 3), 60);
    tick(600);
    assert!(!bench.panel.get_sheet_open());
    assert!(bench.tapped.borrow().is_empty());
    assert!(bench.pasted.borrow().is_empty());
}

#[test]
fn while_the_panel_lingers_a_click_fires_nothing() {
    let bench = one_click_bench(8);
    bench.panel.set_lingering(true);
    tick(100);
    click(&bench.panel, centre_of(&bench, 3), 60);
    tick(600);
    assert!(bench.tapped.borrow().is_empty());
    assert!(bench.pasted.borrow().is_empty());
    assert!(bench.moved.borrow().is_empty());
}

#[test]
fn with_one_click_on_a_click_on_the_image_grid_pastes_the_cell() {
    let (bench, top) = grid_bench(6);
    click(&bench.panel, grid_cell(&bench, top, 1, true), 60);
    tick(600);
    assert_eq!(*bench.tapped.borrow(), [(2, false)]);
    click(&bench.panel, grid_cell(&bench, top, 2, false), 60);
    tick(600);
    assert_eq!(*bench.tapped.borrow(), [(2, false), (5, false)]);
    assert!(bench.pasted.borrow().is_empty());
}

#[test]
fn with_one_click_on_a_double_click_on_the_image_grid_clicks_twice_and_never_pastes_the_old_way() {
    let (bench, top) = grid_bench(6);
    let at = grid_cell(&bench, top, 1, false);
    click(&bench.panel, at, 60);
    tick(100);
    click(&bench.panel, at, 60);
    tick(600);
    assert_eq!(bench.tapped.borrow().len(), 2);
    assert!(bench.pasted.borrow().is_empty());
}

#[test]
fn with_one_click_off_a_single_click_only_opens_the_card() {
    let bench = bench(8);
    click(&bench.panel, centre_of(&bench, 3), 60);
    tick(600);
    assert!(bench.tapped.borrow().is_empty());
    assert!(bench.pasted.borrow().is_empty());
    assert_eq!(open_one(&bench.panel), Some(3));
}

#[test]
fn with_one_click_off_a_double_click_on_the_image_grid_still_pastes() {
    let bench = bench(6);
    let top = bench.panel.get_list_top();
    bench.panel.set_layout("image".into());
    bench.panel.set_way("grid".into());
    let lines = [0, 1, 2];
    bench.panel.set_grid_lines(VecModel::from_slice(&lines));
    tick(100);
    let at = grid_cell(&bench, top, 1, true);
    click(&bench.panel, at, 60);
    tick(100);
    click(&bench.panel, at, 60);
    tick(600);
    assert!(bench.tapped.borrow().is_empty());
    assert_eq!(*bench.pasted.borrow(), [2]);
}

fn theme_of(panel: &Panel) -> crate::Theme<'_> {
    panel.global::<crate::Theme>()
}

#[test]
fn the_lit_edge_is_the_accent_only_with_contrast() {
    let bench = bench(1);
    let theme = theme_of(&bench.panel);
    theme.set_contrast(false);
    assert_eq!(theme.get_lit_edge(), theme.get_card_edge());
    let swatch_edge = theme.get_selected_edge();
    theme.set_contrast(true);
    assert_eq!(theme.get_lit_edge(), theme.get_accent());
    assert_eq!(
        theme.get_selected_edge(),
        swatch_edge,
        "contrast leaves the swatch and chip borders alone"
    );
}

#[test]
fn contrast_darkens_the_text_outlines_the_cards_and_thickens_the_type() {
    let bench = bench(1);
    let theme = theme_of(&bench.panel);
    theme.set_contrast(false);
    let (body, edge, weight, mid, bar, light_body) = (
        theme.get_body(),
        theme.get_card_edge(),
        theme.get_fw_title(),
        theme.get_fw_mid(),
        theme.get_lit_bar(),
        theme.get_fw_body(),
    );
    assert_eq!(edge.alpha(), 0, "no outline without contrast");
    assert_eq!((light_body, mid, weight), (400, 500, 600));
    assert_eq!(bar, 3.0);

    theme.set_contrast(true);
    assert_ne!(theme.get_body(), body);
    assert_ne!(theme.get_card_edge().alpha(), 0, "cards get an outline");
    assert_eq!(theme.get_fw_body(), 500);
    assert_eq!(theme.get_fw_mid(), 600);
    assert_eq!(theme.get_fw_title(), 700);
    assert_eq!(theme.get_lit_bar(), 4.0);
}

#[test]
fn the_gap_between_rows_follows_listing() {
    let bench = bench(1);
    let theme = theme_of(&bench.panel);
    theme.set_listing(false);
    assert_eq!(theme.get_row_gap(), 6.0);
    theme.set_listing(true);
    assert_eq!(theme.get_row_gap(), 2.0);
    assert_eq!(theme.get_row_listed(), 32.0);
}

use crate::odd;

const AREAS: [(f64, f64); 2] = [(1920.0, 1032.0), (1366.0, 728.0)];

const TEXT_SIZES: [cp_config::TextSize; 4] = [
    cp_config::TextSize::Small,
    cp_config::TextSize::Normal,
    cp_config::TextSize::Large,
    cp_config::TextSize::Larger,
];

const DENSITIES: [cp_config::Density; 4] = [
    cp_config::Density::Compact,
    cp_config::Density::Normal,
    cp_config::Density::Comfortable,
    cp_config::Density::List,
];

const KINDS: [cp_core::kind::Kind; 4] = [
    cp_core::kind::Kind::Text,
    cp_core::kind::Kind::Link,
    cp_core::kind::Kind::Json,
    cp_core::kind::Kind::File,
];

fn odd_cards(density: cp_config::Density) -> Vec<Card> {
    let mut id = 0;
    let mut cards = Vec::new();
    for kind in KINDS {
        for (_, text) in odd::texts() {
            id += 1;
            let row = cp_store::Listed {
                id,
                modified_at: 1_000_000,
                created_at: 1_000_000,
                kind: Some(kind),
                preview: text,
                app: Some("Mail".into()),
                label: None,
                color: 0,
                thumb_path: None,
                paste_count: 2,
                last_used_at: None,
                broken_since: None,
                pinned: id % 7 == 0,
                group: String::new(),
                snippet: None,
            };
            let mut card = crate::view::card_of(&row, 2_000_000, None, density.shut_lines());
            card.shut_px = if density.lists() { 32.0 } else { SHUT };
            card.open_px = OPEN;
            cards.push(card);
        }
    }
    cards
}

fn dressed_bench(
    size: cp_config::PanelSize,
    density: cp_config::Density,
    contrast: bool,
    light: bool,
) -> Bench {
    let bench = bench(0);
    let theme = theme_of(&bench.panel);
    theme.set_contrast(contrast);
    theme.set_light(light);
    theme.set_listing(density.lists());
    let (width, height) = size.wanted();
    theme.set_wanted_width(width);
    theme.set_wanted_height(height.unwrap_or(0.0));
    bench.cards.set_vec(odd_cards(density));
    tick(300);
    bench
}

fn failures_of_the_fit(
    bench: &Bench,
    size: cp_config::PanelSize,
    label: &str,
    failed: &mut Vec<String>,
) {
    let theme = theme_of(&bench.panel);
    let wanted = size.wanted();
    for area in AREAS {
        for text in TEXT_SIZES {
            let zoom = text.zoom();
            let (width, height) =
                crate::sizing::fitted((wanted.0, wanted.1.unwrap_or(0.0)), zoom, area);
            let (shown_width, shown_height) = (f64::from(width * zoom), f64::from(height * zoom));
            let fits = width > 0.0
                && height > 0.0
                && shown_width <= area.0 - 16.0 + 1e-3
                && shown_height <= area.1 - 16.0 + 1e-3
                && (wanted.1.is_some() || (shown_height - (area.1 - 16.0)).abs() < 1e-2);
            if !fits {
                failed.push(format!(
                    "{label} {area:?} {text:?}: {shown_width} x {shown_height}"
                ));
            }
            theme.set_zoom(zoom);
            theme.set_width(width);
            theme.set_height(height);
            bench
                .panel
                .window()
                .set_size(slint::LogicalSize::new(width, height));
            tick(50);
            if !bench.panel.get_viewport_height().is_finite() {
                failed.push(format!("{label} {area:?} {text:?}: no viewport"));
            }
        }
    }
}

fn failures_of_the_theme(
    bench: &Bench,
    density: cp_config::Density,
    contrast: bool,
    label: &str,
    failed: &mut Vec<String>,
) {
    let theme = theme_of(&bench.panel);
    let (listing, lit, edge, accent) = (
        theme.get_row_gap(),
        theme.get_lit_edge(),
        theme.get_card_edge(),
        theme.get_accent(),
    );
    let weights = (
        theme.get_fw_body(),
        theme.get_fw_mid(),
        theme.get_fw_title(),
    );
    let body = theme.get_body();
    theme.set_contrast(!contrast);
    let other_body = theme.get_body();
    theme.set_contrast(contrast);
    let checks = [
        (
            listing == if density.lists() { 2.0 } else { 6.0 },
            "row-gap",
        ),
        (theme.get_row_listed() == 32.0, "row-listed"),
        (lit == if contrast { accent } else { edge }, "lit-edge"),
        ((edge.alpha() != 0) == contrast, "card-edge"),
        (
            weights
                == if contrast {
                    (500, 600, 700)
                } else {
                    (400, 500, 600)
                },
            "weights",
        ),
        (body != other_body, "body"),
        (
            bench.cards.row_count() == KINDS.len() * odd::texts().len(),
            "cards",
        ),
    ];
    for (held, what) in checks {
        if !held {
            failed.push(format!("{label}: {what}"));
        }
    }
}

fn every_look_of(size: cp_config::PanelSize) -> Vec<String> {
    let mut failed = Vec::new();
    for density in DENSITIES {
        for contrast in [false, true] {
            for light in [false, true] {
                let label = format!("{size:?} {density:?} contrast={contrast} light={light}");
                let bench = dressed_bench(size, density, contrast, light);
                failures_of_the_theme(&bench, density, contrast, &label, &mut failed);
                failures_of_the_fit(&bench, size, &label, &mut failed);
            }
        }
    }
    failed
}

#[test]
fn a_normal_panel_holds_odd_cards_in_every_density_contrast_and_light() {
    let failed = every_look_of(cp_config::PanelSize::Normal);
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

#[test]
fn a_large_panel_holds_odd_cards_in_every_density_contrast_and_light() {
    let failed = every_look_of(cp_config::PanelSize::Large);
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

#[test]
fn a_tall_panel_holds_odd_cards_in_every_density_contrast_and_light() {
    let failed = every_look_of(cp_config::PanelSize::Tall);
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}
