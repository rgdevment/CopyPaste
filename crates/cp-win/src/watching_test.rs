use super::*;

#[test]
fn the_period_keeps_up_with_a_person_without_spinning() {
    assert!(
        EVERY <= Duration::from_millis(250),
        "una copia no puede tardar en verse"
    );
    assert!(
        EVERY >= Duration::from_millis(16),
        "ni sondear mas rapido que la pantalla"
    );
}

#[test]
fn stopping_is_what_drop_does_and_it_waits_for_the_thread() {
    let watching = Watching::every(Duration::from_millis(5), || {});
    assert!(watching.thread.is_some());
    drop(watching);
}

#[test]
fn a_long_period_does_not_make_stopping_take_that_long() {
    let watching = Watching::every(Duration::from_secs(3600), || {});
    let started = std::time::Instant::now();
    drop(watching);
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "cerrar la aplicacion no puede esperar al siguiente sondeo"
    );
}
