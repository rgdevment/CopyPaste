use super::*;

#[test]
fn a_copy_the_store_keeps_is_updated_by_the_store() {
    assert_eq!(
        chosen(
            Some(Path::new(
                r"C:\Program Files\WindowsApps\CopyPaste\cp-gui.exe"
            )),
            UNBREWED
        ),
        Route::Store
    );
    assert!(!self_installs(Route::Store));
}

const BREWED: fn(&Path) -> bool = |at| at == Path::new("/opt/homebrew/Caskroom/copypaste");
const UNBREWED: fn(&Path) -> bool = |_| false;

#[test]
fn the_cask_moves_the_bundle_out_of_the_caskroom_and_the_receipt_still_tells() {
    let at = Path::new("/Applications/CopyPaste.app/Contents/MacOS/CopyPaste");
    if cfg!(target_os = "macos") {
        assert_eq!(
            chosen(Some(at), BREWED),
            Route::Brew,
            "the running path says Applications, the receipt says brew"
        );
    }
    assert_eq!(
        chosen(Some(at), UNBREWED),
        Route::Download,
        "no receipt, no brew"
    );
    assert!(!self_installs(Route::Brew));
}

#[test]
fn a_copy_running_from_the_caskroom_itself_is_brew_too() {
    assert_eq!(
        chosen(
            Some(Path::new(
                "/opt/homebrew/Caskroom/copypaste/3.0.0/CopyPaste.app/Contents/MacOS/CopyPaste"
            )),
            UNBREWED
        ),
        Route::Brew
    );
}

#[test]
fn a_copy_somewhere_else_is_not_brew_however_many_casks_are_installed() {
    assert_eq!(
        chosen(
            Some(Path::new(
                "/Users/quien/Downloads/CopyPaste.app/Contents/MacOS/CopyPaste"
            )),
            BREWED
        ),
        Route::Download,
        "another copy being brewed says nothing about the one running"
    );
    assert_eq!(chosen(None, BREWED), Route::Download);
    assert!(self_installs(Route::Download));
}

#[test]
fn a_copy_running_from_somewhere_read_only_knows_it_cannot_replace_itself() {
    if cfg!(target_os = "macos") {
        assert!(mounted(Some(Path::new("/Volumes/CopyPaste/CopyPaste.app"))));
        assert!(
            mounted(Some(Path::new(
                "/private/var/folders/xy/AppTranslocation/1E2/d/CopyPaste.app"
            ))),
            "Gatekeeper's copy is read only too"
        );
        assert!(!mounted(Some(Path::new("/Applications/CopyPaste.app"))));
    }
    assert!(!mounted(None));
}

#[test]
fn a_download_that_does_not_come_from_our_releases_is_refused() {
    assert!(ours(
        "https://github.com/rgdevment/CopyPaste/releases/download/v3.0.1/copypaste.exe"
    ));
    assert!(ours("https://objects.githubusercontent.com/whatever"));
    assert!(!ours("https://evil.example.com/copypaste.exe"));
    assert!(!ours("http://github.com/rgdevment/CopyPaste"));
    assert!(!ours("https://github.com.evil.example.com/x"));
    assert!(!ours("github.com/rgdevment"));
    assert!(!ours(""));
    assert!(
        !ours("https://github.com/someone-else/tool/releases/download/v1/x.exe"),
        "github.com alone is not enough: it has to be this project's releases"
    );
}

#[test]
fn a_copy_from_the_beta_cask_is_told_its_own_command() {
    assert_eq!(
        chosen(
            Some(Path::new(
                "/opt/homebrew/Caskroom/copypaste-beta/3.0.0-rc2/CopyPaste.app/Contents/MacOS/CopyPaste"
            )),
            UNBREWED
        ),
        Route::BrewBeta
    );
    let beta: fn(&Path) -> bool = |at| at == Path::new("/usr/local/Caskroom/copypaste-beta");
    let at = Path::new("/Applications/CopyPaste.app/Contents/MacOS/CopyPaste");
    if cfg!(target_os = "macos") {
        assert_eq!(chosen(Some(at), beta), Route::BrewBeta);
    }
    assert!(!self_installs(Route::BrewBeta));
}

#[test]
fn a_clock_put_back_does_not_stop_the_next_look() {
    let now = 1_000_000;
    assert!(due(None, now));
    assert!(due(Some(now - APART), now));
    assert!(!due(Some(now - APART + 1), now));
    assert!(due(Some(now + 5_000), now), "a look in the future is stale");
}

#[test]
fn only_a_higher_version_is_worth_offering() {
    assert!(worth_offering("3.0.1", "3.0.0"));
    assert!(!worth_offering("3.0.0", "3.0.0"));
    assert!(!worth_offering("2.9.9", "3.0.0"));
    assert!(!worth_offering("not a version", "3.0.0"));
    assert!(!worth_offering("3.0.1", "neither is this"));
}

#[test]
fn a_candidate_is_not_offered_over_the_stable_it_came_from() {
    assert!(!worth_offering("3.0.0-rc.1", "3.0.0"));
    assert!(worth_offering("3.0.0", "3.0.0-rc.1"));
}

#[test]
fn only_one_install_can_be_under_way_at_a_time() {
    let alone = Installing::default();
    let first = alone.claim();
    assert!(first.is_some());
    assert!(alone.claim().is_none(), "a second one is turned away");
    drop(first);
    assert!(alone.claim().is_some(), "and the next one may go");
}

#[test]
fn what_is_kept_survives_being_written_and_read_again() {
    let one = Kept {
        checked_at: Some(1_700_000_000),
        found: Some("3.0.1".to_owned()),
        from: Some("3.0.0".to_owned()),
    };
    let said = serde_json::to_string(&one).expect("written");
    let back: Kept = serde_json::from_str(&said).expect("read");
    assert_eq!(back.checked_at, one.checked_at);
    assert_eq!(back.found, one.found);

    let empty: Kept = serde_json::from_str("{}").expect("an empty one still reads");
    assert!(empty.checked_at.is_none() && empty.found.is_none());
}

#[test]
fn a_store_copy_never_claims_to_have_looked() {
    let looked = Looked {
        route: Route::Store,
        looked: false,
        ready: None,
    };
    assert!(
        !looked.looked,
        "saying nothing is not saying it is up to date"
    );
}

const TODAY: &str = r#"{"schema":1,"latest":"0.0.0","latestPrerelease":"3.0.0-rc2"}"#;

fn offer(here: &str, manifest: &str) -> Option<String> {
    newer(here, manifest).expect("a manifest that reads")
}

#[test]
fn a_candidate_is_offered_the_next_candidate_while_no_stable_exists() {
    assert_eq!(offer("3.0.0-rc1", TODAY).as_deref(), Some("3.0.0-rc2"));
    assert_eq!(offer("3.0.0-rc2", TODAY), None, "it is already there");
}

#[test]
fn a_stable_copy_is_never_walked_onto_a_candidate() {
    assert_eq!(offer("3.0.0", TODAY), None);
    let hostile = r#"{"schema":1,"latest":"3.1.0-rc1"}"#;
    assert_eq!(
        offer("3.0.0", hostile),
        None,
        "a candidate written into the stable field is still a candidate"
    );
}

#[test]
fn a_candidate_takes_the_stable_release_that_passes_it() {
    let shipped = r#"{"schema":1,"latest":"3.0.0","latestPrerelease":"3.0.0-rc2"}"#;
    assert_eq!(offer("3.0.0-rc2", shipped).as_deref(), Some("3.0.0"));
    let both = r#"{"schema":1,"latest":"3.0.0","latestPrerelease":"3.1.0-rc1"}"#;
    assert_eq!(offer("3.0.0-rc2", both).as_deref(), Some("3.1.0-rc1"));
    assert_eq!(
        offer("3.0.0", both),
        None,
        "the stable copy stays on its track"
    );
    let behind = r#"{"schema":1,"latest":"3.0.1","latestPrerelease":"3.0.1-rc1"}"#;
    assert_eq!(
        offer("3.0.0-rc2", behind).as_deref(),
        Some("3.0.1"),
        "a candidate older than the stable one is passed over"
    );
}

#[test]
fn a_stable_copy_is_offered_the_next_stable() {
    let next = r#"{"schema":1,"latest":"3.0.1"}"#;
    assert_eq!(offer("3.0.0", next).as_deref(), Some("3.0.1"));
    assert_eq!(offer("3.0.1", next), None);
    assert_eq!(
        offer("3.1.0", next),
        None,
        "an older number is never offered"
    );
}

#[test]
fn semver_reads_a_dotted_or_two_digit_candidate_as_older_which_is_why_tags_stop_at_rc9() {
    let dotted = r#"{"schema":1,"latest":"0.0.0","latestPrerelease":"3.0.0-rc.3"}"#;
    assert_eq!(offer("3.0.0-rc2", dotted), None);
    let ten = r#"{"schema":1,"latest":"0.0.0","latestPrerelease":"3.0.0-rc10"}"#;
    assert_eq!(offer("3.0.0-rc2", ten), None);
}

#[test]
fn a_manifest_that_does_not_read_is_an_error_and_not_an_answer() {
    for broken in [
        "not json",
        r#"{"schema":1}"#,
        r#"{"schema":2,"latest":"3.0.0"}"#,
        r#"{"schema":1,"latest":"three"}"#,
        r#"{"latest":"3.0.0"}"#,
    ] {
        assert_eq!(newer("3.0.0-rc1", broken), Err(UNREADABLE), "{broken}");
    }
    assert_eq!(newer("dev", TODAY), Ok(None));
}

#[test]
fn what_was_kept_is_offered_again_only_under_the_same_rule() {
    assert!(allowed("3.0.0-rc3", "3.0.0-rc2"));
    assert!(allowed("3.0.1", "3.0.0"));
    assert!(
        !allowed("3.1.0-rc1", "3.0.1"),
        "a candidate a former candidate found is not offered to the stable copy that replaced it"
    );
    assert!(!allowed("3.0.0", "3.0.0"));
    assert!(!allowed("2.9.0", "3.0.0"));
}

#[test]
fn a_candidate_downloads_from_the_candidates_first_and_a_stable_only_from_the_stable() {
    assert_eq!(feeds_for("3.0.0-rc2"), vec![CANDIDATE, LATEST]);
    assert_eq!(feeds_for("3.0.0"), vec![LATEST]);
    assert_eq!(feeds_for("nonsense"), vec![LATEST]);
    for one in [MANIFEST, LATEST, CANDIDATE] {
        assert!(one.starts_with("https://raw.githubusercontent.com/rgdevment/CopyPaste/manifest/"));
    }
}

#[test]
fn a_copy_refuses_an_update_not_signed_for_its_version() {
    let conf: serde_json::Value =
        serde_json::from_str(include_str!("../tauri.conf.json")).expect("tauri.conf.json reads");
    assert_eq!(
        conf["plugins"]["updater"]["requireSignedVersion"],
        serde_json::Value::Bool(true)
    );
}
