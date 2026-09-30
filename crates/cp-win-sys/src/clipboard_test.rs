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
fn a_write_waits_for_the_reads_that_are_not_its_own() {
    let base = readers_now();
    let _held = reading();
    assert!(
        alone(base).is_none(),
        "a write cleared the way while a read it does not own was still counted"
    );
}
