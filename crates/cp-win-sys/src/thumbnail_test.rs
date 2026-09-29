use super::*;

#[test]
fn a_file_that_is_not_there_has_no_thumbnail() {
    assert_eq!(
        dib_of_file(std::path::Path::new(r"C:\no-existe.png"), SIDE),
        None
    );
}

#[test]
fn a_side_of_zero_is_refused() {
    let dir = std::env::temp_dir();
    assert_eq!(dib_of_file(&dir, 0), None);
    assert_eq!(dib_of_file(&dir, -1), None);
}

#[test]
fn a_tiny_square_at_full_size_is_the_generic_icon() {
    assert!(
        is_an_icon(32, 32, SIDE),
        "el shell devuelve el icono del tipo"
    );
    assert!(is_an_icon(SMALLEST, SMALLEST, SIDE));
}

#[test]
fn a_wide_thumbnail_is_not_an_icon() {
    assert!(
        !is_an_icon(256, 57, SIDE),
        "una imagen apaisada tiene un lado corto y sigue siendo una miniatura"
    );
    assert!(!is_an_icon(57, 256, SIDE));
}

#[test]
fn a_real_thumbnail_is_not_an_icon() {
    assert!(!is_an_icon(SMALLEST + 1, SMALLEST + 1, SIDE));
    assert!(!is_an_icon(256, 144, SIDE));
}

#[test]
fn asking_for_a_small_image_never_calls_it_an_icon() {
    assert!(
        !is_an_icon(32, 32, 32),
        "if small was asked for, small is fine"
    );
}
