use super::*;

#[test]
fn every_block_is_reserved_before_the_clipboard_is_emptied() {
    let entries: Vec<(u32, &[u8])> = vec![(1, b"uno".as_slice()), (2, b"dos".as_slice())];
    let ready = reserved(&entries).expect("la memoria se consigue");
    assert_eq!(ready.len(), 2, "lo que se pidio, reservado y sin entregar");
    release(&ready);
}

#[test]
fn nothing_to_write_reserves_nothing() {
    assert!(
        reserved(&[]).is_none(),
        "there is nothing to hand over, and replace refuses an empty list before it ever asks"
    );
}

#[test]
fn text_survives_the_round_trip() {
    for original in ["hola", "", "accents: ñáéíóú", "emoji: 🦀", "日本語"] {
        let bytes = utf16_of(original);
        assert_eq!(text_of(&bytes).as_deref(), Some(original), "«{original}»");
    }
}

#[test]
fn the_encoding_ends_where_the_terminator_says() {
    let bytes = utf16_of("ab");
    assert_eq!(bytes.len(), 6, "two units plus the zero");
    assert_eq!(&bytes[4..], &[0, 0]);
}

#[test]
fn what_comes_after_the_terminator_is_not_text() {
    let mut bytes = utf16_of("hola");
    bytes.extend_from_slice(&utf16_of("basura"));
    assert_eq!(text_of(&bytes).as_deref(), Some("hola"));
}

#[test]
fn an_odd_number_of_bytes_does_not_panic() {
    assert_eq!(text_of(&[0x68]), Some(String::new()));
    assert_eq!(text_of(&[]), Some(String::new()));
}

#[test]
fn a_lone_surrogate_is_refused_instead_of_mangled() {
    let broken = [0x00u8, 0xD8, 0x41, 0x00];
    assert_eq!(text_of(&broken), None);
}
