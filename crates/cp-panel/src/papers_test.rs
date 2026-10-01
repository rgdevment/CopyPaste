use super::*;

#[test]
fn a_windows_path_gives_the_format_of_its_last_segment() {
    let found = papers_of(r"D:\Mario\Contratos\Presupuesto Q4 2026.xlsx").expect("has one");
    assert_eq!(found.format, "XLSX");
    assert_eq!(found.family, "sheets");
}

#[test]
fn a_unix_path_and_a_bare_name_read_the_same() {
    assert_eq!(
        papers_of("/home/mario/acta.odt").expect("one").format,
        "ODT"
    );
    assert_eq!(papers_of("acta.odt").expect("one").format, "ODT");
}

#[test]
fn the_four_families_the_view_colours_are_told_apart() {
    assert_eq!(papers_of("a.xlsx").expect("one").family, "sheets");
    assert_eq!(papers_of("a.csv").expect("one").family, "sheets");
    assert_eq!(papers_of("a.docx").expect("one").family, "words");
    assert_eq!(papers_of("a.pptx").expect("one").family, "slides");
    assert_eq!(papers_of("a.pdf").expect("one").family, "pages");
}

#[test]
fn a_format_nobody_coloured_still_shows_its_name() {
    let found = papers_of("respaldo.7z").expect("one");
    assert_eq!(found.format, "7Z");
    assert_eq!(found.family, "plain", "no colour, but it says what it is");
}

#[test]
fn the_case_of_the_extension_does_not_change_the_family() {
    assert_eq!(papers_of("A.XLSX").expect("one").family, "sheets");
    assert_eq!(papers_of("A.XlSx").expect("one").format, "XLSX");
}

#[test]
fn a_name_with_dots_in_it_takes_only_the_last_piece() {
    let found = papers_of("informe.final.v3.docx").expect("one");
    assert_eq!(found.format, "DOCX");
}

#[test]
fn what_has_no_extension_has_no_format() {
    assert!(papers_of("").is_none());
    assert!(papers_of("sin_extension").is_none());
    assert!(papers_of(r"D:\carpeta\sin_extension").is_none());
    assert!(papers_of("acaba.en.punto.").is_none());
}

#[test]
fn something_that_only_looks_like_an_extension_is_refused() {
    assert!(
        papers_of("una frase con. un punto suelto").is_none(),
        "a space after the dot is not an extension"
    );
    assert!(
        papers_of("archivo.extensionlarguisima").is_none(),
        "eight characters is already generous"
    );
}

#[test]
fn only_the_first_line_is_read_because_that_is_the_first_file() {
    let found = papers_of("primero.pdf\nsegundo.xlsx").expect("one");
    assert_eq!(found.format, "PDF");
}

#[test]
fn a_trailing_carriage_return_does_not_become_part_of_the_format() {
    assert_eq!(
        papers_of("acta.pdf\r\notro.docx").expect("one").format,
        "PDF"
    );
}
