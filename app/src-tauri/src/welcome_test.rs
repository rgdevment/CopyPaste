use super::{Greeting, behind, decide, told};

fn said(all: &[&str]) -> Vec<String> {
    all.iter().map(|one| (*one).to_owned()).collect()
}

#[test]
fn a_first_install_takes_the_tour() {
    assert_eq!(
        decide(true, false, None, "3.0.0", &said(&["3.0.0"])),
        Greeting::Tour { former: false }
    );
}

#[test]
fn a_first_install_over_the_2x_is_offered_its_history() {
    assert_eq!(
        decide(true, true, Some("2.9.0"), "3.0.0", &[]),
        Greeting::Tour { former: true }
    );
}

#[test]
fn an_update_shows_only_what_was_not_seen_newest_first() {
    let all = said(&["3.0.0", "3.0.1", "3.1.0", "3.2.0"]);
    assert_eq!(
        decide(false, false, Some("3.0.0"), "3.1.0", &all),
        Greeting::News {
            versions: said(&["3.1.0", "3.0.1"])
        }
    );
}

#[test]
fn the_same_version_twice_says_nothing() {
    assert_eq!(
        decide(false, false, Some("3.1.0"), "3.1.0", &said(&["3.1.0"])),
        Greeting::Nothing
    );
}

#[test]
fn going_back_a_version_says_nothing() {
    assert_eq!(
        decide(false, false, Some("3.2.0"), "3.1.0", &said(&["3.1.0"])),
        Greeting::Nothing
    );
}

#[test]
fn an_update_with_nothing_written_for_it_says_nothing() {
    assert_eq!(
        decide(false, false, Some("3.0.0"), "3.0.1", &said(&["3.0.0"])),
        Greeting::Nothing
    );
}

#[test]
fn a_candidate_counts_as_its_own_version() {
    let all = said(&["3.1.0-rc.1", "3.1.0"]);
    assert_eq!(
        decide(false, false, Some("3.0.0"), "3.1.0-rc.1", &all),
        Greeting::News {
            versions: said(&["3.1.0-rc.1"])
        }
    );
}

#[test]
fn a_first_run_cut_short_before_the_welcome_closed_takes_the_tour_again() {
    assert_eq!(
        decide(false, true, None, "3.0.0-rc1", &said(&["3.0.0"])),
        Greeting::Tour { former: true }
    );
}

#[test]
fn a_candidate_never_welcomed_is_not_left_with_nothing_because_the_news_is_ahead() {
    assert_eq!(
        decide(false, false, None, "3.0.0-rc2", &said(&["3.0.0"])),
        Greeting::Tour { former: false }
    );
}

#[test]
fn a_welcome_nobody_can_read_is_treated_as_never_given() {
    assert_eq!(
        decide(false, false, Some("ayer"), "3.0.0", &said(&["3.0.0"])),
        Greeting::Tour { former: false }
    );
}

#[test]
fn a_running_version_that_is_not_semver_says_nothing() {
    assert_eq!(
        decide(false, false, Some("3.0.0"), "dev", &said(&["3.0.0"])),
        Greeting::Nothing
    );
}

#[test]
fn only_a_welcome_behind_the_running_version_is_moved() {
    assert!(behind(None, "3.0.0"));
    assert!(behind(Some("2.9.0"), "3.0.0"));
    assert!(!behind(Some("3.0.0"), "3.0.0"));
    assert!(!behind(Some("3.1.0"), "3.0.0"));
    assert!(!behind(None, "dev"));
}

#[test]
fn every_version_in_the_news_is_one_semver_reads() {
    let all = told();
    assert!(!all.is_empty(), "the news file says nothing");
    for one in &all {
        assert!(
            semver::Version::parse(one).is_ok(),
            "«{one}» is not a version"
        );
    }
}

#[test]
fn a_build_made_by_hand_never_marks_a_published_version_as_seen() {
    let all = said(&["3.0.0", "3.0.0-rc2"]);
    for welcomed in ["3.0.0-rc1", "3.0.0-rc5", "3.0.0"] {
        assert_eq!(
            decide(false, false, Some(welcomed), "3.0.0-dev", &all),
            Greeting::Nothing,
            "{welcomed}"
        );
        assert!(!behind(Some(welcomed), "3.0.0-dev"), "{welcomed}");
    }
    assert_eq!(
        decide(false, false, Some("3.0.0-dev"), "3.0.0", &all),
        Greeting::News {
            versions: said(&["3.0.0", "3.0.0-rc2"])
        },
        "the real release still tells what it brings after a build made by hand"
    );
}
