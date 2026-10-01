use super::*;

#[test]
fn the_windows_counter_is_read_as_opaque_so_losses_are_not_invented() {
    let watching = every(cp_core::watching::EVERY, || {});
    assert_eq!(
        watching.missed(),
        None,
        "el contador de Windows salta varios por copia"
    );
    assert!(watching.close());
}

#[test]
fn both_ends_of_a_write_of_ours_read_the_same_counter() {
    let watching = every(Duration::from_secs(3_600), || {});
    assert_eq!(
        watching.writing(),
        watching.ours(),
        "si uno de los dos extremos no se marca, el tramo queda abierto"
    );
    assert!(watching.close());
}
