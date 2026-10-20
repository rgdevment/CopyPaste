use super::*;
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn a_window_coming_forward_reaches_whoever_listens_and_stops_once_they_stop() {
    let seen = Rc::new(Cell::new(0isize));
    let heard = seen.clone();
    let listening = listen(move |window| heard.set(window)).expect("the system listens");
    told(HWND(7 as *mut _));
    assert_eq!(seen.get(), 7);
    drop(listening);
    told(HWND(9 as *mut _));
    assert_eq!(seen.get(), 7, "nobody hears after the listening stopped");
}

#[test]
fn the_system_calls_back_through_the_same_door() {
    let seen = Rc::new(Cell::new(0isize));
    let heard = seen.clone();
    let _listening = listen(move |window| heard.set(window)).expect("the system listens");
    unsafe {
        changed(
            HWINEVENTHOOK::default(),
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
