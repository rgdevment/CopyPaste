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
            hides: false
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
    };
    other.set(wanted);
    assert_eq!(one.get(), wanted);
}
