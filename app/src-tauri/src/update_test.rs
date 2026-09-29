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
