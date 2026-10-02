use super::*;

#[test]
fn the_backoff_covers_most_of_a_second_without_a_busy_loop() {
    assert_eq!(BACKOFF_MS.first(), Some(&0), "el primer intento no espera");
    let total: u64 = BACKOFF_MS.iter().sum();
    assert!(
        (700..=900).contains(&total),
        "la cobertura total es de {total} ms"
    );
    assert!(
        BACKOFF_MS.windows(2).all(|pair| pair[0] <= pair[1]),
        "el retroceso no puede acortarse"
    );
}

#[test]
fn a_write_that_got_through_is_the_only_one_allowed_to_replace() {
    let Some(writing) = Clipboard::to_write() else {
        return;
    };
    assert!(writing.opened_to_write());
    drop(writing);
    let Some(reading) = Clipboard::open() else {
        return;
    };
    assert!(
        !reading.opened_to_write(),
        "a clipboard opened to read claims it may replace"
    );
}

#[test]
fn a_read_is_only_called_stuck_once_it_is_past_the_ceiling() {
    let floor = STUCK_AFTER.as_millis() as u64;
    assert_eq!(
        stuck_for(1, 1_000, 1_000 + floor - 1),
        None,
        "still working"
    );
    assert_eq!(
        stuck_for(1, 1_000, 1_000 + floor),
        Some(STUCK_AFTER),
        "past the ceiling it is stuck"
    );
}

#[test]
fn nothing_is_stuck_when_no_read_is_counted() {
    assert_eq!(stuck_for(0, 1_000, 9_999_999), None);
    assert_eq!(stuck_for(1, 0, 9_999_999), None, "nobody wrote a start");
}

#[test]
fn a_clock_that_went_backwards_is_not_read_as_an_eternity() {
    assert_eq!(stuck_for(1, 9_000, 1_000), None);
}

#[test]
fn a_second_read_waits_for_the_first_to_close_instead_of_sharing_it() {
    let (opened, heard) = std::sync::mpsc::channel();
    let first = std::thread::spawn(move || {
        let held = loop {
            if let Some(held) = Clipboard::open() {
                break held;
            }
        };
        opened.send(()).expect("the test is listening");
        std::thread::sleep(std::time::Duration::from_millis(200));
        let closing = std::time::Instant::now();
        drop(held);
        closing
    });
    heard.recv().expect("the first read opened");
    let second = loop {
        if Clipboard::open().is_some() {
            break std::time::Instant::now();
        }
    };
    let closed = first.join().expect("the first read finished");
    assert!(
        second >= closed,
        "the second read opened {:?} before the first one closed",
        closed - second
    );
}
