use super::*;

const LIGHT_CARD: u32 = 0xFFFFFF;
const LIGHT_PANEL: u32 = 0xF7F8FB;
const DARK_CARD: u32 = 0x1E2132;
const DARK_PANEL: u32 = 0x1A1D2B;

fn channel(value: u32) -> f64 {
    let one = f64::from(value & 0xFF) / 255.0;
    if one <= 0.039_28 {
        one / 12.92
    } else {
        ((one + 0.055) / 1.055).powf(2.4)
    }
}

fn luminance(rgb: u32) -> f64 {
    0.2126 * channel(rgb >> 16) + 0.7152 * channel(rgb >> 8) + 0.0722 * channel(rgb)
}

fn contrast(one: u32, other: u32) -> f64 {
    let (a, b) = (luminance(one), luminance(other));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

#[test]
fn every_accent_reads_as_text_on_the_cards_of_its_own_theme() {
    for one in Accent::ALL {
        for (light, backs) in [
            (true, [LIGHT_CARD, LIGHT_PANEL]),
            (false, [DARK_CARD, DARK_PANEL]),
        ] {
            let swatch = one.swatch(light);
            for back in backs.into_iter().chain([swatch.selected]) {
                let said = contrast(swatch.accent, back);
                assert!(
                    said >= 4.5,
                    "{one:?} light={light} on {back:06X}: {said:.2}"
                );
            }
            let dim = contrast(swatch.dim, backs[0]);
            assert!(dim >= 3.0, "{one:?} light={light} dim: {dim:.2}");
        }
    }
}

#[test]
fn the_default_accent_is_the_one_the_panel_always_had() {
    assert_eq!(Accent::default(), Accent::Indigo);
    assert_eq!(
        Accent::Indigo.swatch(true),
        swatch(0x4F46E5, 0x6B63EA, 0xE9E9FB, 0xB4B1EC)
    );
    assert_eq!(
        Accent::Indigo.swatch(false),
        swatch(0xA5B4FC, 0x7C86C9, 0x282C46, 0x4A5085)
    );
}

#[test]
fn each_accent_answers_to_its_own_name() {
    let keys: Vec<_> = Accent::ALL.iter().map(|one| one.key()).collect();
    for (one, key) in Accent::ALL.iter().zip(&keys) {
        let back: Accent = serde_json::from_str(&format!("\"{key}\"")).expect("parses");
        assert_eq!(back, *one);
    }
    let mut unique = keys.clone();
    unique.dedup();
    assert_eq!(unique.len(), keys.len());
}

#[test]
fn the_sizes_grow_in_order_and_never_past_a_sixth() {
    let zooms: Vec<f32> = [
        TextSize::Small,
        TextSize::Normal,
        TextSize::Large,
        TextSize::Larger,
    ]
    .into_iter()
    .map(TextSize::zoom)
    .collect();
    assert!(zooms.windows(2).all(|pair| pair[0] < pair[1]), "{zooms:?}");
    assert!((TextSize::Normal.zoom() - 1.0).abs() < f32::EPSILON);
    assert!(
        zooms.iter().all(|one| (0.9..=1.17).contains(one)),
        "{zooms:?}"
    );
}

#[test]
fn a_closed_card_shows_one_two_or_three_lines() {
    assert_eq!(Density::Compact.shut_lines(), 1);
    assert_eq!(Density::Normal.shut_lines(), 2);
    assert_eq!(Density::Comfortable.shut_lines(), 3);
}

#[test]
fn only_the_fonts_on_disk_are_offered() {
    let dir = Path::new("/fonts");
    let only_avenir = |path: &Path| path.ends_with("Avenir Next.ttc");
    let shown = present(MAC.texts, dir, &only_avenir);
    assert_eq!(
        shown.iter().map(|one| one.id).collect::<Vec<_>>(),
        ["avenir-next"]
    );
    let offered = choices(&MAC, dir, &only_avenir);
    assert_eq!(offered.text.label, "SF Pro");
    assert_eq!(offered.texts.len(), 1);
    assert!(offered.codes.is_empty());
    assert_eq!(offered.accents.len(), Accent::ALL.len());
    assert_eq!(offered.accents[0].light, "#4F46E5");
    assert_eq!(offered.accents[0].dark, "#A5B4FC");
    assert_eq!(offered.accents[0].light_selected, "#E9E9FB");
    assert_eq!(offered.accents[0].dark_selected, "#282C46");
}

#[test]
fn a_font_that_is_missing_or_unknown_falls_back_to_the_system_one() {
    let dir = Path::new("/fonts");
    let all = |_: &Path| true;
    let none = |_: &Path| false;
    assert_eq!(
        family_of(Some("avenir-next"), MAC.text, MAC.texts, dir, &all),
        "Avenir Next"
    );
    assert_eq!(
        family_of(Some("avenir-next"), MAC.text, MAC.texts, dir, &none),
        "System Font"
    );
    assert_eq!(
        family_of(Some("comic"), MAC.text, MAC.texts, dir, &all),
        "System Font"
    );
    assert_eq!(
        family_of(None, WINDOWS.code, WINDOWS.codes, dir, &all),
        "Consolas"
    );
    assert_eq!(
        family_of(
            Some("cascadia-mono"),
            WINDOWS.code,
            WINDOWS.codes,
            dir,
            &all
        ),
        "Cascadia Mono"
    );
}

#[test]
fn the_fonts_offered_are_those_of_the_system_it_runs_on() {
    let here = fonts_here();
    if cfg!(target_os = "macos") {
        assert_eq!(here, MAC);
        assert_eq!(fonts_dir_here(), Path::new(MAC_DIR));
    } else {
        assert_eq!(here, WINDOWS);
        assert!(fonts_dir_here().ends_with("Fonts"));
    }
    let offered = choices_here();
    assert_eq!(offered.text.label, here.text.label);
}

#[test]
fn the_window_reads_the_selected_colours_in_its_own_case() {
    let said = serde_json::to_value(&choices_here().accents[0]).expect("serializes");
    assert_eq!(said["lightSelected"], "#E9E9FB");
    assert_eq!(said["darkSelected"], "#282C46");
}

#[test]
fn colours_are_written_as_the_web_writes_them() {
    assert_eq!(hex(0x4F46E5), "#4F46E5");
    assert_eq!(hex(0xFF00_0A0B), "#000A0B");
}
