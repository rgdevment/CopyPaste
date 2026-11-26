use super::*;

fn held(found: Option<&str>, staged: Option<&str>, tried: Option<&str>) -> Kept {
    Kept {
        checked_at: Some(1),
        found: found.map(str::to_owned),
        from: Some("3.5.1".to_owned()),
        shop: None,
        staged: staged.map(str::to_owned),
        tried: tried.map(str::to_owned),
        ..Kept::default()
    }
}

const NOW: u64 = 1_000_000;
const A_DAY: u64 = 24 * 60 * 60;

#[test]
fn a_new_version_is_downloaded_ahead_once() {
    assert_eq!(
        worth_staging(&held(Some("3.6.0"), None, None), "3.5.1", NOW).as_deref(),
        Some("3.6.0")
    );
    assert_eq!(
        worth_staging(&held(Some("3.6.0"), Some("3.6.0"), None), "3.5.1", NOW),
        None
    );
    assert_eq!(
        worth_staging(&held(Some("3.6.0"), None, Some("3.6.0")), "3.5.1", NOW),
        None
    );
}

#[test]
fn what_waits_installs_on_start_only_once_and_only_when_wanted() {
    let waiting = held(Some("3.6.0"), Some("3.6.0"), None);
    assert_eq!(
        to_apply(&waiting, "3.5.1", true, true).as_deref(),
        Some("3.6.0")
    );
    assert_eq!(to_apply(&waiting, "3.5.1", false, true), None);
    assert_eq!(to_apply(&waiting, "3.5.1", true, false), None);
    assert_eq!(to_apply(&waiting, "3.6.0", true, true), None);
    let tried = held(Some("3.6.0"), Some("3.6.0"), Some("3.6.0"));
    assert_eq!(to_apply(&tried, "3.5.1", true, true), None);
}

#[test]
fn nothing_is_downloaded_ahead_for_what_was_not_found_here_or_is_not_allowed() {
    assert_eq!(worth_staging(&held(None, None, None), "3.5.1", NOW), None);
    assert_eq!(
        worth_staging(&held(Some("3.6.0"), None, None), "3.5.0", NOW),
        None
    );
    let mut elsewhere = held(Some("3.6.0"), None, None);
    elsewhere.from = None;
    assert_eq!(worth_staging(&elsewhere, "3.5.1", NOW), None);
    assert_eq!(
        worth_staging(&held(Some("3.5.1"), None, None), "3.5.1", NOW),
        None
    );
    assert_eq!(
        worth_staging(&held(Some("3.6.0-rc1"), None, None), "3.5.1", NOW),
        None
    );
}

#[test]
fn another_version_staged_or_tried_does_not_stop_a_newer_one() {
    assert_eq!(
        worth_staging(
            &held(Some("3.7.0"), Some("3.6.0"), Some("3.6.0")),
            "3.5.1",
            NOW
        )
        .as_deref(),
        Some("3.7.0")
    );
}

#[test]
fn a_copy_on_candidates_stages_candidates() {
    let mut one = held(Some("3.6.0-rc1"), None, None);
    one.from = Some("3.5.0-rc2".to_owned());
    assert_eq!(
        worth_staging(&one, "3.5.0-rc2", NOW).as_deref(),
        Some("3.6.0-rc1")
    );
}

#[test]
fn nothing_installs_on_start_without_a_package_from_this_very_version() {
    assert_eq!(to_apply(&held(None, None, None), "3.5.1", true, true), None);
    let mut other = held(Some("3.6.0"), Some("3.6.0"), None);
    other.from = None;
    assert_eq!(to_apply(&other, "3.5.1", true, true), None);
}

#[test]
fn what_waits_is_not_installed_when_it_is_no_longer_ahead_or_allowed() {
    let behind = held(None, Some("3.5.1"), None);
    assert_eq!(to_apply(&behind, "3.5.1", true, true), None);
    let candidate = held(None, Some("3.6.0-rc1"), None);
    assert_eq!(to_apply(&candidate, "3.5.1", true, true), None);
}

#[test]
fn what_was_tried_in_another_version_does_not_block_this_one() {
    let waiting = held(None, Some("3.6.0"), Some("3.5.9"));
    assert_eq!(
        to_apply(&waiting, "3.5.1", true, true).as_deref(),
        Some("3.6.0")
    );
}

fn failed_at(version: &str, at: u64) -> Kept {
    Kept {
        attempt: Some(Attempt {
            version: version.to_owned(),
            at,
        }),
        ..held(Some("3.6.0"), None, None)
    }
}

#[test]
fn a_download_that_failed_lately_is_not_tried_again_for_a_day() {
    assert_eq!(
        worth_staging(&failed_at("3.6.0", NOW - 60), "3.5.1", NOW),
        None
    );
    assert_eq!(
        worth_staging(&failed_at("3.6.0", NOW - A_DAY + 1), "3.5.1", NOW),
        None
    );
    assert_eq!(
        worth_staging(&failed_at("3.6.0", NOW - A_DAY), "3.5.1", NOW).as_deref(),
        Some("3.6.0")
    );
}

#[test]
fn a_clock_set_back_does_not_hold_a_download_for_good() {
    assert_eq!(
        worth_staging(&failed_at("3.6.0", NOW + 10), "3.5.1", NOW).as_deref(),
        Some("3.6.0")
    );
}

#[test]
fn a_failure_of_another_version_does_not_hold_this_one() {
    assert_eq!(
        worth_staging(&failed_at("3.5.9", NOW - 60), "3.5.1", NOW).as_deref(),
        Some("3.6.0")
    );
}

#[test]
fn only_a_missing_connection_counts_as_being_offline() {
    use tauri_plugin_updater::Error;
    assert!(unreachable(&Error::Network("no route".to_owned())));
    assert!(!unreachable(&Error::ReleaseNotFound));
    assert!(!unreachable(&Error::TargetNotFound(
        "windows-x86_64".to_owned()
    )));
    let unreadable = serde_json::from_str::<u8>("x").expect_err("not a number");
    assert!(!unreachable(&Error::Serialization(unreadable)));
}

#[test]
fn the_third_start_without_a_connection_gives_up() {
    assert!(!gives_up_after(None));
    assert!(!gives_up_after(Some(0)));
    assert!(!gives_up_after(Some(2)));
    assert!(gives_up_after(Some(3)));
    assert!(gives_up_after(Some(4)));
}
