use super::*;
use std::cell::Cell;
use std::rc::Rc;

fn heard_by(seen: &Rc<Cell<isize>>) -> impl Fn(isize) + 'static {
    let seen = seen.clone();
    move |window| seen.set(window)
}

#[test]
fn a_window_coming_forward_reaches_whoever_listens_and_stops_once_they_stop() {
    let seen = Rc::new(Cell::new(0isize));
    let listening = listen(heard_by(&seen)).expect("the system listens");
    let hook = listening.0;
    told(hook, HWND(7 as *mut _));
    assert_eq!(seen.get(), 7);
    drop(listening);
    told(hook, HWND(9 as *mut _));
    assert_eq!(seen.get(), 7, "nobody hears after the listening stopped");
}

#[test]
fn one_listener_stopping_never_silences_another() {
    let first = Rc::new(Cell::new(0isize));
    let second = Rc::new(Cell::new(0isize));
    let a = listen(heard_by(&first)).expect("the system listens");
    let b = listen(heard_by(&second)).expect("the system listens twice");
    let hook = b.0;
    drop(a);
    told(hook, HWND(5 as *mut _));
    assert_eq!(second.get(), 5);
    assert_eq!(first.get(), 0);
}

#[test]
fn the_system_calls_back_through_the_same_door() {
    let seen = Rc::new(Cell::new(0isize));
    let listening = listen(heard_by(&seen)).expect("the system listens");
    unsafe {
        changed(
            listening.0,
            EVENT_SYSTEM_FOREGROUND,
            HWND(3 as *mut _),
            0,
            0,
            0,
            0,
        )
    };
    assert_eq!(seen.get(), 3);
}
