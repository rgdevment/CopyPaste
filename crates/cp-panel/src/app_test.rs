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

#[test]
fn only_a_file_that_is_really_gone_is_marked_as_gone() {
    use super::Landing;
    assert_eq!(super::landing_of(true, true, true), Landing::Opened);
    assert_eq!(super::landing_of(false, true, false), Landing::Gone);
    assert_eq!(
        super::landing_of(false, true, true),
        Landing::Refused,
        "the system refuses to open plenty of files that are right there: no application for the \
         extension, a share violation, SmartScreen. Marking those as gone hides them and the \
         housekeeping deletes them later"
    );
    assert_eq!(
        super::landing_of(false, false, false),
        Landing::Refused,
        "what was written out to be looked at has no path of its own to be missing from"
    );
}
