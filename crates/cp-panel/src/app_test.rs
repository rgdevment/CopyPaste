use super::*;

#[test]
fn choosing_a_theme_wins_over_what_the_system_wants() {
    assert!(light_for(cp_config::Theme::Light, false));
    assert!(!light_for(cp_config::Theme::Dark, true));
}

#[test]
fn leaving_it_to_the_system_follows_the_system_both_ways() {
    assert!(light_for(cp_config::Theme::System, true));
    assert!(!light_for(cp_config::Theme::System, false));
}
