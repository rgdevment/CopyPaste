use super::{spanish, worded};
use tauri::image::Image;

const WINDOWS: &[u8] = include_bytes!("../icons/tray/windows-32.png");
const MACOS: &[u8] = include_bytes!("../icons/tray/macos@2x.png");

fn ink(png: &[u8]) -> (u32, usize) {
    let art = Image::from_bytes(png).expect("the icon is a png");
    let pixels = art.rgba();
    let (sum, seen) = pixels
        .as_chunks::<4>()
        .0
        .iter()
        .fold((0u64, 0usize), |(sum, seen), px| {
            if px[3] < 128 {
                return (sum, seen);
            }
            let grey = (px[0] as u64 * 299 + px[1] as u64 * 587 + px[2] as u64 * 114) / 1000;
            (sum + grey, seen + 1)
        });
    assert!(seen > 0, "the icon is transparent through and through");
    ((sum / seen as u64) as u32, seen)
}

#[test]
fn the_bar_icon_on_macos_is_pale_because_the_system_paints_it_itself() {
    let (grey, _) = ink(MACOS);
    assert!(grey > 200, "the macOS template has to be pale: {grey}");
}

#[test]
fn only_a_locale_that_starts_with_es_gets_spanish() {
    assert!(!spanish(Some("en")));
    assert!(!spanish(Some("en-GB")));
    assert!(spanish(Some("es")));
    assert!(spanish(Some("es-CL")));
    assert!(spanish(Some("ES-mx")));
    assert!(!spanish(Some("pt-BR")));
    assert!(!spanish(Some("fr")));
    assert!(!spanish(Some("de-DE")));
}

#[test]
fn the_windows_icon_is_dark_enough_for_a_light_bar() {
    let (grey, seen) = ink(WINDOWS);
    assert!(seen > 64, "the Windows icon has hardly any ink: {seen}");
    assert!(grey < 200, "on a light bar it would not show: {grey}");
}

#[test]
fn the_words_come_in_the_order_the_menu_was_built() {
    for spanish in [true, false] {
        let said = worded(spanish);
        assert_eq!(said.len(), 4, "reword zips these against the live items");
        assert!(
            said[0].to_lowercase().contains("panel"),
            "the first item shows the panel: {}",
            said[0]
        );
        assert!(
            said[2].to_lowercase().contains("panel"),
            "the third item restarts it: {}",
            said[2]
        );
        assert!(
            said[3].to_lowercase().contains("copypaste"),
            "the last one leaves: {}",
            said[3]
        );
    }
}

#[test]
fn the_two_tongues_say_different_things_item_by_item() {
    let es = worded(true);
    let en = worded(false);
    for (one, other) in es.iter().zip(en.iter()) {
        assert_ne!(one, other, "«{one}» was left untranslated");
    }
}
