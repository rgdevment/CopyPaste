use super::*;

#[test]
fn the_three_the_rewrite_had_dropped() {
    assert_eq!(classify_file("video.flv", false), Kind::Video);
    assert_eq!(classify_file("favicon.ico", false), Kind::Image);
    assert_eq!(classify_text("mailto:alguien@ejemplo.test"), Kind::Link);
}

#[test]
fn what_the_rewrite_added_survives() {
    for (name, kind) in [
        ("captura.heic", Kind::Image),
        ("captura.avif", Kind::Image),
        ("pelicula.m4v", Kind::Video),
        ("pelicula.mpeg", Kind::Video),
        ("audio.opus", Kind::Audio),
        ("audio.aiff", Kind::Audio),
    ] {
        assert_eq!(classify_file(name, false), kind, "«{name}»");
    }
}

#[test]
fn a_bare_mail_scheme_is_not_a_link() {
    assert_eq!(classify_text("mailto:"), Kind::Text);
    assert_eq!(classify_text("MAILTO:alguien@ejemplo.test"), Kind::Link);
}

#[test]
fn an_address_without_the_scheme_is_still_an_address() {
    assert_eq!(classify_text("alguien@ejemplo.test"), Kind::Email);
}

#[test]
fn a_windows_path_is_classified_by_its_last_segment() {
    for (path, kind) in [
        (r"C:\Users\ana\Imagenes\foto.PNG", Kind::Image),
        (r"C:\fotos.2024\captura.jpg", Kind::Image),
        (r"C:\version.1.2\programa", Kind::File),
        (r"\servidor\compartido\clip.MKV", Kind::Video),
        (r"C:\sin_extension", Kind::File),
    ] {
        assert_eq!(classify_file(path, false), kind, "«{path}»");
    }
}

#[test]
fn a_folder_named_like_a_file_is_still_a_folder() {
    assert_eq!(classify_file(r"C:\copias\respaldo.zip", true), Kind::Folder);
    assert_eq!(classify_file("fotos.png", true), Kind::Folder);
}

#[test]
fn a_name_that_is_only_a_dot_has_no_extension() {
    assert_eq!(classify_file(".png", false), Kind::Image);
    assert_eq!(classify_file(".", false), Kind::File);
    assert_eq!(classify_file("", false), Kind::File);
}

#[test]
fn a_local_windows_path_is_not_a_link() {
    assert_eq!(classify_text(r"C:\Users\ana\documento.txt"), Kind::Text);
}
