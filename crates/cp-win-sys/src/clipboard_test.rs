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
