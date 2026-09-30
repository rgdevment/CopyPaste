use super::*;

#[test]
fn the_names_a_setting_would_use_map_to_a_backdrop() {
    assert_eq!(Backdrop::from_name("mica"), Some(Backdrop::Mica));
    assert_eq!(Backdrop::from_name("acrylic"), Some(Backdrop::Acrylic));
    assert_eq!(Backdrop::from_name("none"), Some(Backdrop::None));
    assert_eq!(Backdrop::from_name("blur"), None);
}

#[test]
fn a_window_that_does_not_exist_is_refused_not_crashed_on() {
    assert!(!apply(0, Backdrop::Mica, true));
    assert!(!apply(0x7fff_ffff, Backdrop::None, false));
}
