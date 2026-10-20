use super::*;
use cp_core::kind::Kind;
use slint::Model;
use slint::platform::{Key, WindowEvent};

thread_local! {
    static PLATFORM: Cell<bool> = const { Cell::new(false) };
}

fn listed(kind: Kind, preview: &str, label: Option<&str>) -> Listed {
    Listed {
        id: 1,
        modified_at: 0,
        created_at: 0,
        kind: Some(kind),
        preview: preview.into(),
        app: None,
        label: label.map(str::to_owned),
        color: 0,
        thumb_path: None,
        paste_count: 0,
        last_used_at: None,
        broken_since: None,
        pinned: false,
        group: String::new(),
        snippet: None,
    }
}

struct Bench {
    quick: Rc<Quick>,
    store: Rc<Store>,
    ahead: Arc<AtomicIsize>,
}

fn bench(copied: &[&str]) -> Bench {
    if !PLATFORM.replace(true) {
        i_slint_backend_testing::init_no_event_loop();
    }
    let store = Rc::new(Store::in_memory().expect("a store in memory"));
    for (at, text) in copied.iter().enumerate() {
        store
            .insert_text(&format!("copy-{at}"), text, 1_000 + at as i64)
            .expect("the copy is kept");
    }
    let ahead = Arc::new(AtomicIsize::new(0));
    let held = store.clone();
    let quick = Quick::new(
        Rc::new(move || Aim {
            store: held.clone(),
            engine: None,
            kept: crate::kept::resolve(None),
        }),
        ahead.clone(),
    );
    Bench {
        quick,
        store,
        ahead,
    }
}

fn card(bench: &Bench) -> Chooser {
    bench
        .quick
        .chooser
        .get()
        .map(ComponentHandle::clone_strong)
        .expect("the card was drawn")
}

fn press(chooser: &Chooser, text: impl Into<slint::SharedString>) {
    let text = text.into();
    chooser
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    chooser
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text });
}

fn tick(ms: u64) {
    i_slint_backend_testing::mock_elapsed_time(Duration::from_millis(ms));
}

#[test]
fn the_card_is_headed_by_the_name_it_was_given() {
    let row = listed(Kind::Json, "{\"a\":1}", Some("  config  "));
    assert_eq!(subject_of(&row), "config");
}

#[test]
fn without_a_name_the_card_is_headed_by_what_was_copied_on_one_line() {
    let row = listed(Kind::Text, "hola\n   mundo  ", Some("   "));
    assert_eq!(subject_of(&row), "hola mundo");
}

#[test]
fn a_file_is_headed_by_its_name_as_the_panel_heads_it() {
    let row = listed(
        Kind::File,
        "C:\\Users\\ana\\Documents\\informe-final.docx",
        None,
    );
    let body = crate::view::body_of(&row);
    let folder = crate::folder::parts_of(&row.preview);
    assert_eq!(
        subject_of(&row),
        crate::view::headline_of(&row, &body, folder.as_ref())
    );
    assert!(
        !subject_of(&row).contains("Documents"),
        "{}",
        subject_of(&row)
    );
}

#[test]
fn an_empty_history_draws_the_card_but_never_shows_it() {
    let bench = bench(&[]);
    bench.quick.choose();
    assert!(!card(&bench).window().is_visible());
}

#[test]
fn the_card_opens_on_the_newest_copy_with_every_way_to_paste_it() {
    let bench = bench(&["older", "newest"]);
    bench.quick.choose();
    let chooser = card(&bench);
    assert!(chooser.window().is_visible());
    assert_eq!(chooser.get_subject(), "newest");
    assert_eq!(chooser.get_kind(), "text");
    assert_eq!(chooser.get_current(), 0);
    assert!(chooser.get_older());
    assert!(!chooser.get_newer());
    assert!(chooser.get_rows().row_count() > 0);
    assert_eq!(chooser.get_complaint(), "");
    assert!(!chooser.get_hints().is_empty());
    assert!(chooser.get_tall() > 0.0);
}

#[test]
fn the_arrows_walk_back_through_the_history_and_stop_at_both_ends() {
    let bench = bench(&["older", "newest"]);
    bench.quick.choose();
    let chooser = card(&bench);
    press(&chooser, Key::LeftArrow);
    assert_eq!(chooser.get_subject(), "older");
    assert!(chooser.get_newer());
    assert!(!chooser.get_older());
    press(&chooser, Key::LeftArrow);
    assert_eq!(chooser.get_subject(), "older");
    bench.quick.step(&chooser, 1);
    assert_eq!(
        chooser.get_subject(),
        "older",
        "past the oldest it stays put"
    );
    press(&chooser, Key::RightArrow);
    assert_eq!(chooser.get_subject(), "newest");
    bench.quick.step(&chooser, -1);
    assert_eq!(
        chooser.get_subject(),
        "newest",
        "before the newest it stays put"
    );
}

#[test]
fn up_and_down_move_through_the_ways_without_leaving_them() {
    let bench = bench(&["newest"]);
    bench.quick.choose();
    let chooser = card(&bench);
    let last = chooser.get_rows().row_count() as i32 - 1;
    press(&chooser, Key::UpArrow);
    assert_eq!(chooser.get_current(), 0);
    for _ in 0..=last + 2 {
        press(&chooser, Key::DownArrow);
    }
    assert_eq!(chooser.get_current(), last);
}

#[test]
fn escape_closes_the_card_and_forgets_where_it_was_going() {
    let bench = bench(&["newest"]);
    bench.ahead.store(0, Ordering::Relaxed);
    bench.quick.choose();
    let chooser = card(&bench);
    press(&chooser, Key::Escape);
    assert!(!chooser.window().is_visible());
    assert_eq!(bench.ahead.load(Ordering::Relaxed), 0);
}

#[test]
fn a_way_that_cannot_be_pasted_is_told_on_the_card_and_then_let_go() {
    let bench = bench(&["newest"]);
    bench.quick.choose();
    let chooser = card(&bench);
    let showing = bench.quick.showing.get().expect("a copy is showing");
    bench
        .store
        .mark_deleted(showing, 5_000)
        .expect("the copy is gone");
    press(&chooser, "1");
    assert!(chooser.window().is_visible());
    assert_ne!(chooser.get_complaint(), "");
    tick(COMPLAINS_FOR.as_millis() as u64 + 50);
    assert_eq!(chooser.get_complaint(), "");
}

#[test]
fn enter_and_a_number_take_the_way_they_point_at() {
    let bench = bench(&["newest"]);
    bench.quick.choose();
    let chooser = card(&bench);
    let showing = bench.quick.showing.get().expect("a copy is showing");
    bench
        .store
        .mark_deleted(showing, 5_000)
        .expect("the copy is gone");
    press(&chooser, Key::Return);
    assert_ne!(chooser.get_complaint(), "");
    chooser.set_complaint(Default::default());
    press(&chooser, "9");
    assert_eq!(chooser.get_complaint(), "", "there is no ninth way");
}

#[test]
fn plain_with_no_window_behind_pastes_nothing_and_closes_the_card() {
    let bench = bench(&["newest"]);
    bench.quick.choose();
    let chooser = card(&bench);
    bench.quick.plain();
    assert!(!chooser.window().is_visible());
}

#[test]
fn plain_over_an_empty_history_leaves_the_target_untouched_by_no_card() {
    let bench = bench(&[]);
    bench.ahead.store(42, Ordering::Relaxed);
    bench.quick.plain();
    assert!(bench.quick.chooser.get().is_none());
}

#[test]
fn plain_that_finds_no_text_opens_the_card_and_says_why() {
    let bench = bench(&[""]);
    bench.ahead.store(42, Ordering::Relaxed);
    bench.quick.plain();
    let chooser = card(&bench);
    assert!(chooser.window().is_visible());
    assert_ne!(chooser.get_complaint(), "");
    assert_eq!(bench.ahead.load(Ordering::Relaxed), 42);
}

#[test]
fn a_paste_that_never_landed_brings_the_card_back_with_the_reason() {
    let bench = bench(&["newest"]);
    bench.quick.choose();
    let chooser = card(&bench);
    let _ = chooser.hide();
    delivered(
        &Rc::downgrade(&bench.quick),
        42,
        here::Sent::Degraded(cp_core::paste::Failure::TargetElevated),
    );
    assert!(chooser.window().is_visible());
    assert_ne!(chooser.get_complaint(), "");
    assert_eq!(bench.ahead.load(Ordering::Relaxed), 42);
}

#[test]
fn a_paste_with_nowhere_to_go_closes_the_card() {
    let bench = bench(&["newest"]);
    bench.quick.choose();
    let chooser = card(&bench);
    delivered(&Rc::downgrade(&bench.quick), 42, here::Sent::Nobody);
    assert!(!chooser.window().is_visible());
    bench.quick.choose();
    delivered(&Rc::downgrade(&bench.quick), 42, here::Sent::Done);
    assert!(chooser.window().is_visible());
}

#[test]
fn a_finished_paste_for_a_card_that_is_gone_is_ignored() {
    let gone = {
        let bench = bench(&["newest"]);
        Rc::downgrade(&bench.quick)
    };
    delivered(&gone, 42, here::Sent::Nobody);
    let never_drawn = bench(&["newest"]);
    delivered(&Rc::downgrade(&never_drawn.quick), 42, here::Sent::Nobody);
    assert!(never_drawn.quick.chooser.get().is_none());
}

#[test]
fn losing_the_focus_puts_the_card_away_on_the_next_turn() {
    let bench = bench(&["newest"]);
    bench.quick.choose();
    let chooser = card(&bench);
    left(&chooser);
    assert!(
        chooser.window().is_visible(),
        "not in the middle of the event"
    );
    tick(1);
    assert!(!chooser.window().is_visible());
    left(&chooser);
    tick(1);
    assert!(!chooser.window().is_visible());
}

#[test]
fn putting_the_card_down_on_quitting_is_safe_drawn_or_not() {
    let never_drawn = bench(&[]);
    never_drawn.quick.ground();
    let drawn = bench(&["newest"]);
    drawn.quick.choose();
    drawn.quick.ground();
}
