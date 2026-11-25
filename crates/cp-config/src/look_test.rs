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

fn family(name: &str, mono: bool) -> Family {
    Family {
        name: name.to_owned(),
        local: name.to_owned(),
        mono,
        symbol: false,
    }
}

fn symbol(name: &str) -> Family {
    Family {
        symbol: true,
        ..family(name, false)
    }
}

fn here(families: Vec<Family>, system: Option<&str>) -> Installed {
    Installed {
        families,
        system: system.map(str::to_owned),
    }
}

fn windows_with(families: Vec<Family>, system: Option<&str>) -> Choices {
    let none = |_: &Path| false;
    choices(
        &WINDOWS,
        &here(families, system),
        Path::new("/fonts"),
        &none,
    )
}

fn ids(list: &[Offered]) -> Vec<&str> {
    list.iter().map(|one| one.id.as_str()).collect()
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}

#[test]
fn a_legacy_font_is_offered_when_its_file_is_on_disk() {
    let dir = Path::new("/fonts");
    let only_avenir = |path: &Path| path.ends_with("Avenir Next.ttc");
    let offered = choices(&MAC, &Installed::default(), dir, &only_avenir);
    assert_eq!(ids(&offered.texts), ["avenir-next"]);
    assert_eq!(offered.text.label, "SF Pro");
    assert!(offered.codes.is_empty());
    assert_eq!(offered.accents.len(), Accent::ALL.len());
    assert_eq!(offered.accents[0].light, "#4F46E5");
    assert_eq!(offered.accents[0].dark, "#A5B4FC");
    assert_eq!(offered.accents[0].light_selected, "#E9E9FB");
    assert_eq!(offered.accents[0].dark_selected, "#282C46");
}

#[test]
fn a_legacy_font_is_offered_when_its_family_is_installed_without_the_file() {
    let offered = windows_with(
        vec![family("Calibri", false), family("Cascadia Mono", true)],
        None,
    );
    assert_eq!(ids(&offered.texts), ["calibri", "Cascadia Mono"]);
    assert_eq!(ids(&offered.codes), ["cascadia-mono"]);
}

#[test]
fn the_old_ids_still_resolve_to_their_families() {
    let windows = windows_with(
        vec![
            family("Segoe UI Variable Text", false),
            family("Calibri", false),
            family("Arial", false),
            family("Cascadia Mono", true),
        ],
        None,
    );
    assert_eq!(
        windows.text_family(Some("segoe-ui-variable")),
        "Segoe UI Variable Text"
    );
    assert_eq!(windows.text_family(Some("calibri")), "Calibri");
    assert_eq!(windows.text_family(Some("arial")), "Arial");
    assert_eq!(windows.code_family(Some("cascadia-mono")), "Cascadia Mono");
    let mac = choices(
        &MAC,
        &here(
            vec![family("Avenir Next", false), family(".SF NS Mono", true)],
            None,
        ),
        Path::new("/fonts"),
        &|_: &Path| false,
    );
    assert_eq!(mac.text_family(Some("avenir-next")), "Avenir Next");
}

#[test]
fn a_font_that_is_not_installed_or_unknown_falls_back_to_the_default() {
    let offered = windows_with(vec![family("Arial", false), family("Consolas", true)], None);
    assert_eq!(offered.text_family(Some("calibri")), "Segoe UI");
    assert_eq!(offered.text_family(Some("Comic Sans MS")), "Segoe UI");
    assert_eq!(offered.text_family(None), "Segoe UI");
    assert_eq!(offered.text_family(Some("arial")), "Arial");
    assert_eq!(offered.code_family(Some("cascadia-mono")), "Consolas");
    assert_eq!(offered.code_family(Some("nope")), "Consolas");
    assert_eq!(offered.code_family(None), "Consolas");
}

#[test]
fn the_code_list_has_only_monospaced_families_and_never_the_default_twice() {
    let offered = windows_with(
        vec![
            family("Consolas", true),
            family("Cascadia Mono", true),
            family("Fira Code", true),
            family("Arial", false),
        ],
        None,
    );
    assert_eq!(ids(&offered.codes), ["cascadia-mono", "Fira Code"]);
    assert!(
        offered
            .codes
            .iter()
            .all(|one| !one.family.eq_ignore_ascii_case("Consolas"))
    );
    assert_eq!(offered.code.family, "Consolas");
}

#[test]
fn symbol_and_dotted_families_are_never_offered() {
    let offered = windows_with(
        vec![
            symbol("Anything Flagged"),
            family(".SF NS Display", false),
            family("Marlett", false),
            family("Segoe MDL2 Assets", false),
            family("Segoe Fluent Icons", false),
            family("Webdings", false),
            family("Wingdings", false),
            family("Wingdings 2", false),
            family("Wingdings 3", false),
            family("Symbol", false),
            family("Segoe UI Symbol", false),
            family("Fira Code", true),
        ],
        None,
    );
    assert_eq!(ids(&offered.texts), ["Fira Code", "Segoe UI Symbol"]);
    assert_eq!(ids(&offered.codes), ["Fira Code"]);
    assert!(!offerable(&symbol("Fine Name")));
    assert!(!offerable(&family(".Hidden", false)));
    assert!(offerable(&family("Fine Name", false)));
}

#[test]
fn labels_are_localized_and_ids_stay_english_sorted_by_label() {
    let local = |name: &str, local: &str| Family {
        local: local.to_owned(),
        ..family(name, false)
    };
    let offered = windows_with(
        vec![
            local("Yu Mincho", "Zeta"),
            local("MS Gothic", "Mitad"),
            local("Arial Black", "Alfa"),
        ],
        None,
    );
    assert_eq!(
        ids(&offered.texts),
        ["Arial Black", "MS Gothic", "Yu Mincho"]
    );
    assert_eq!(offered.texts[1].label, "Mitad");
    assert_eq!(offered.texts[1].family, "MS Gothic");
    assert_eq!(offered.texts[1].css, "\"MS Gothic\"");
}

#[test]
fn a_family_is_found_by_either_name_without_regard_to_case() {
    let one = Family {
        local: "Mitad".to_owned(),
        ..family("Segoe UI", false)
    };
    let all = vec![one];
    assert!(installed("segoe ui", &all).is_some());
    assert!(installed("MITAD", &all).is_some());
    assert!(installed("Segoe", &all).is_none());
}

#[test]
fn with_no_choice_the_text_is_system_and_the_old_default_id_reads_as_system() {
    let offered = windows_with(vec![family("Segoe UI", false)], None);
    assert_eq!(offered.text.id, "");
    assert_eq!(offered.text.family, "Segoe UI");
    assert!(offered.plain_system);
    assert_eq!(offered.text_family(None), "Segoe UI");
    assert_eq!(offered.text_family(Some("")), "Segoe UI");
    let said = serde_json::to_value(&offered).expect("serializes");
    assert_eq!(said["plainSystem"], true);
    assert!(said["text"].get("family").is_none());
}

#[test]
fn an_installed_system_font_that_is_not_the_platform_one_becomes_the_default() {
    let offered = windows_with(
        vec![family("Segoe UI", false), family("Comic Sans MS", false)],
        Some("Comic Sans MS"),
    );
    assert!(!offered.plain_system);
    assert_eq!(offered.text.id, "");
    assert_eq!(offered.text.family, "Comic Sans MS");
    assert_eq!(offered.text.css, "\"Comic Sans MS\"");
    assert_eq!(offered.text_family(None), "Comic Sans MS");
    assert_eq!(offered.text_family(Some("unknown")), "Comic Sans MS");
    assert_eq!(offered.text_family(Some("Segoe UI")), "Segoe UI");
}

#[test]
fn a_system_font_that_is_missing_symbolic_or_segoe_keeps_the_platform_default() {
    let all = vec![family("Segoe UI", false), symbol("Odd Icons")];
    for system in [
        Some("Comic Sans MS"),
        Some("segoe ui"),
        Some("Odd Icons"),
        None,
    ] {
        let offered = windows_with(all.clone(), system);
        assert!(offered.plain_system, "{system:?}");
        assert_eq!(offered.text.family, "Segoe UI", "{system:?}");
        assert_eq!(offered.text.css, "\"Segoe UI\"", "{system:?}");
    }
}

#[test]
fn a_substitute_is_found_by_its_plain_key() {
    let entries = pairs(&[("MS Shell Dlg", "Microsoft Sans Serif")]);
    assert_eq!(
        substituted("MS Shell Dlg", &entries),
        "Microsoft Sans Serif"
    );
}

#[test]
fn a_substitute_may_carry_a_charset_in_its_key_and_its_value() {
    let entries = pairs(&[("Helv,0", "Arial,0")]);
    assert_eq!(substituted("Helv", &entries), "Arial");
}

#[test]
fn a_substitute_ignores_case_and_an_exact_key_beats_a_suffixed_one() {
    let entries = pairs(&[("ms shell dlg,0", "Suffixed"), ("MS Shell Dlg", "Exact")]);
    assert_eq!(substituted("ms SHELL dlg", &entries), "Exact");
    let only_suffixed = pairs(&[("ms shell dlg,0", "Suffixed")]);
    assert_eq!(substituted("MS Shell Dlg", &only_suffixed), "Suffixed");
}

#[test]
fn without_a_usable_substitute_the_face_is_its_own() {
    assert_eq!(substituted("Segoe UI", &[]), "Segoe UI");
    assert_eq!(
        substituted("Segoe UI", &pairs(&[("Arial", "Other")])),
        "Segoe UI"
    );
    assert_eq!(
        substituted("Segoe UI", &pairs(&[("Segoe UI", "")])),
        "Segoe UI"
    );
}

#[test]
fn a_css_family_is_quoted_and_escaped() {
    assert_eq!(css_of("Segoe UI"), "\"Segoe UI\"");
    assert_eq!(css_of("Odd\"Name"), "\"Odd\\\"Name\"");
    assert_eq!(css_of("Back\\slash"), "\"Back\\\\slash\"");
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
    assert_eq!(offered.code.label, here.code.label);
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
