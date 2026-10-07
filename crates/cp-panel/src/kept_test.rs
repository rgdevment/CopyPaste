use super::*;

fn config(locale: Option<&str>, theme: cp_config::Theme, hides: bool) -> cp_config::Config {
    cp_config::Config {
        locale: locale.map(str::to_owned),
        theme,
        hides_when_left: hides,
        ..cp_config::Config::default()
    }
}

#[test]
fn an_explicit_theme_beats_the_system_and_system_follows_it() {
    assert!(light_for(cp_config::Theme::Light, false));
    assert!(!light_for(cp_config::Theme::Dark, true));
    assert!(light_for(cp_config::Theme::System, true));
    assert!(!light_for(cp_config::Theme::System, false));
}

#[test]
fn without_a_config_the_panel_follows_the_system_and_hides_when_left() {
    let kept = resolve(None);
    assert!(kept.light(true));
    assert!(!kept.light(false));
    assert!(kept.hides);
}

#[test]
fn a_config_decides_language_theme_and_hiding_in_one_go() {
    let kept = resolve(Some(&config(Some("en-US"), cp_config::Theme::Dark, false)));
    assert_eq!(
        kept,
        Kept {
            english: true,
            theme: cp_config::Theme::Dark,
            hides: false,
            look: Look::default(),
        }
    );
    assert!(
        !kept.light(true),
        "a chosen dark stays dark on a light system"
    );
    let kept = resolve(Some(&config(Some("es-CL"), cp_config::Theme::Light, true)));
    assert!(!kept.english);
    assert!(
        kept.light(false),
        "a chosen light stays light on a dark system"
    );
    assert!(kept.hides);
}

#[test]
fn the_shelf_hands_out_what_was_last_set_across_clones() {
    let one = Shelf::new(resolve(None));
    let other = one.clone();
    let wanted = Kept {
        english: true,
        theme: cp_config::Theme::Light,
        hides: false,
        look: Look {
            accent: cp_config::Accent::Rose,
            ..Look::default()
        },
    };
    other.set(wanted);
    assert_eq!(one.get(), wanted);
}

#[test]
fn renewing_reads_what_the_settings_say_now() {
    let dir = tempfile::tempdir().expect("dir");
    let shelf = Shelf::new(resolve(None));
    let written = config(Some("en-GB"), cp_config::Theme::Dark, false);
    cp_config::write(&cp_config::at(dir.path()), &written).expect("written");
    let kept = shelf.renew_from(Some(dir.path()));
    assert_eq!(kept, resolve(Some(&written)));
    assert_eq!(shelf.get(), kept, "the shelf holds what was just read");
}

#[test]
fn with_nowhere_to_read_from_the_defaults_rule() {
    assert_eq!(read_from(None), resolve(None));
}

#[test]
fn without_a_config_the_panel_looks_as_it_always_did() {
    let look = resolve(None).look;
    assert_eq!(look.size, cp_config::TextSize::Normal);
    assert_eq!(look.density, cp_config::Density::Normal);
    assert_eq!(look.accent, cp_config::Accent::Indigo);
    assert_eq!(look.font, cp_config::look::fonts_here().text.family);
    assert_eq!(look.mono, cp_config::look::fonts_here().code.family);
}

#[test]
fn the_look_chosen_reaches_the_panel_and_a_missing_font_does_not() {
    let chosen = cp_config::Config {
        text_size: cp_config::TextSize::Larger,
        density: cp_config::Density::Comfortable,
        accent: cp_config::Accent::Teal,
        font: Some("avenir-next".into()),
        code_font: Some("sf-mono".into()),
        ..cp_config::Config::default()
    };
    let dir = std::path::Path::new("/fonts");
    let look = look_of(
        &chosen,
        &cp_config::look::MAC,
        dir,
        &|path: &std::path::Path| path.ends_with("Avenir Next.ttc"),
    );
    assert_eq!(look.size, cp_config::TextSize::Larger);
    assert_eq!(look.density, cp_config::Density::Comfortable);
    assert_eq!(look.accent, cp_config::Accent::Teal);
    assert_eq!(look.font, "Avenir Next");
    assert_eq!(look.mono, "Menlo");
}

#[test]
fn a_config_with_a_look_is_read_from_the_file() {
    let dir = tempfile::tempdir().expect("a folder");
    let chosen = cp_config::Config {
        accent: cp_config::Accent::Amber,
        density: cp_config::Density::Compact,
        ..cp_config::Config::default()
    };
    cp_config::write(&cp_config::at(dir.path()), &chosen).expect("writes");
    let look = read_from(Some(dir.path())).look;
    assert_eq!(look.accent, cp_config::Accent::Amber);
    assert_eq!(look.density, cp_config::Density::Compact);
}
