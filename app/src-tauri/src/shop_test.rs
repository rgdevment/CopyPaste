use super::*;

const NOW: &str = "3.5.1";

fn said_before(said: Said, version: Option<&str>) -> Memory {
    Memory {
        version: version.map(str::to_owned),
        said: Some(said),
    }
}

#[test]
fn the_store_s_offer_can_be_taken_whatever_is_known_of_its_number() {
    let named = verdict(
        Shelf::Waiting,
        Ahead::Is("3.6.0".into()),
        NOW,
        Memory::default(),
    );
    let ready = named.ready.clone().expect("an offer");
    assert!(ready.installs);
    assert_eq!(ready.version, "3.6.0");
    assert_eq!(named.offered_version().as_deref(), Some("3.6.0"));

    let unnamed = verdict(Shelf::Waiting, Ahead::Nothing, NOW, Memory::default());
    assert!(unnamed.ready.clone().expect("an offer").installs);
    assert_eq!(
        unnamed.offered_version(),
        None,
        "the panel names no version it does not know"
    );
}

#[test]
fn a_copy_the_store_did_not_sell_is_never_told_the_store_brings_anything() {
    let said = verdict(
        Shelf::NotSold,
        Ahead::Is("3.6.0".into()),
        NOW,
        said_before(Said::Offered, Some("3.6.0")),
    );
    assert_eq!(said.ready, None);
    assert!(!said.unanswered);
}

#[test]
fn the_download_fills_the_bar_and_the_rest_is_installing() {
    assert_eq!(step(0.4), ("getting", 50));
    assert_eq!(step(0.8), ("installing", 100));
    assert_eq!(step(-1.0), ("getting", 0));
}

fn coming_before(version: &str) -> Memory {
    said_before(Said::NotYet, Some(version))
}

fn known(version: &str) -> Ahead {
    Ahead::Is(version.to_owned())
}

fn manifest(latest: &str) -> Result<String, String> {
    Ok(format!(r#"{{"schema":1,"latest":"{latest}"}}"#))
}

#[test]
fn the_download_fills_the_bar_on_its_own() {
    assert_eq!(step(0.0), ("getting", 0));
    assert_eq!(step(0.79), ("getting", 98));
}

#[test]
fn what_the_store_counts_as_the_install_is_shown_as_installing() {
    assert_eq!(step(0.9), ("installing", 100));
    assert_eq!(step(1.0), ("installing", 100));
    assert_eq!(step(2.0), ("installing", 100));
}

#[test]
fn with_nobody_able_to_say_the_version_known_stays_while_it_is_still_ahead() {
    let ahead_of_us = verdict(Shelf::Waiting, Ahead::Unheard, NOW, coming_before("3.6.0"));
    assert_eq!(ahead_of_us.ready.expect("an offer").version, "3.6.0");

    let behind_us = verdict(Shelf::Waiting, Ahead::Unheard, NOW, coming_before("3.5.1"));
    assert_eq!(behind_us.ready.expect("an offer").version, "");
}

#[test]
fn what_already_landed_offers_nothing_to_press() {
    let landed = verdict(
        Shelf::Landed("3.6.0".into()),
        Ahead::Nothing,
        NOW,
        Memory::default(),
    );
    let ready = landed.ready.expect("it is here");
    assert!(!ready.installs);
    assert_eq!(ready.version, "3.6.0");
    assert_eq!(ready.coming, None);
    assert_eq!(landed.memory, said_before(Said::Landed, Some("3.6.0")));
}

#[test]
fn a_queue_held_up_is_told_apart_from_one_that_finishes_by_itself() {
    let moving = verdict(Shelf::Queued, known("3.6.0"), NOW, Memory::default());
    assert_eq!(
        moving.ready.expect("on its way").coming,
        Some(Coming::Queued)
    );

    let paused = verdict(Shelf::Paused, known("3.6.0"), NOW, Memory::default());
    let paused = paused.ready.expect("paused there");
    assert_eq!(paused.coming, Some(Coming::Paused));
    assert!(!paused.installs);
}

#[test]
fn a_copy_the_store_did_not_sell_forgets_what_it_was_told() {
    let said = verdict(
        Shelf::NotSold,
        known("3.6.0"),
        NOW,
        said_before(Said::Offered, Some("3.6.0")),
    );
    assert_eq!(said.memory, Memory::default());
}

#[test]
fn a_version_the_catalog_lists_is_on_its_way_even_when_the_store_says_nothing() {
    for shelf in [Shelf::Current, Shelf::Silent] {
        let said = verdict(shelf, known("3.6.0"), NOW, Memory::default());
        let ready = said.ready.expect("on its way");
        assert_eq!(ready.coming, Some(Coming::NotYet));
        assert!(!ready.installs);
        assert_eq!(said.memory, coming_before("3.6.0"));
        assert!(!said.unanswered);
    }
}

#[test]
fn a_store_that_answered_and_a_catalog_with_nothing_newer_end_the_matter() {
    let said = verdict(
        Shelf::Current,
        Ahead::Nothing,
        NOW,
        said_before(Said::Offered, Some("3.6.0")),
    );
    assert_eq!(said.ready, None);
    assert_eq!(said.memory, Memory::default());
    assert!(!said.unanswered);
}

#[test]
fn a_store_that_did_not_answer_is_reported_even_when_the_catalog_lists_nothing_newer() {
    let offer = verdict(
        Shelf::Silent,
        Ahead::Nothing,
        NOW,
        said_before(Said::Offered, Some("3.6.0")),
    );
    assert!(offer.unanswered);
    assert!(offer.ready.expect("its own offer stands").installs);

    let coming = verdict(Shelf::Silent, Ahead::Nothing, NOW, coming_before("3.6.0"));
    assert!(coming.unanswered);
    assert_eq!(coming.ready, None);
    assert_eq!(coming.memory, Memory::default());

    let nothing_before = verdict(Shelf::Silent, Ahead::Nothing, NOW, Memory::default());
    assert!(nothing_before.unanswered);
    assert_eq!(nothing_before.ready, None);
}

#[test]
fn the_store_saying_it_has_no_offer_takes_back_the_offer_it_made() {
    let said = verdict(
        Shelf::Current,
        Ahead::Unheard,
        NOW,
        said_before(Said::Offered, Some("3.6.0")),
    );
    assert_eq!(said.ready, None);
    assert_eq!(said.memory, Memory::default());

    let coming = verdict(Shelf::Current, Ahead::Unheard, NOW, coming_before("3.6.0"));
    assert!(coming.ready.expect("still on its way").coming.is_some());
    assert_eq!(coming.memory, coming_before("3.6.0"));
}

#[test]
fn a_store_that_did_not_answer_leaves_what_it_last_said_standing() {
    let said = verdict(
        Shelf::Silent,
        Ahead::Unheard,
        NOW,
        said_before(Said::Offered, Some("3.6.0")),
    );
    assert!(said.ready.expect("still standing").installs);
    assert_eq!(said.memory, said_before(Said::Offered, Some("3.6.0")));
    assert!(said.unanswered);
}

#[test]
fn what_was_last_said_ends_when_this_copy_catches_up_or_had_no_number() {
    assert_eq!(
        last_said(NOW, &said_before(Said::Offered, Some("3.5.1"))),
        None
    );
    assert_eq!(last_said(NOW, &coming_before("3.5.1")), None);
    assert_eq!(last_said(NOW, &said_before(Said::Offered, None)), None);
    assert_eq!(last_said(NOW, &Memory::default()), None);
    let no_said = Memory {
        version: Some("3.6.0".into()),
        said: None,
    };
    assert_eq!(last_said(NOW, &no_said), None);
    let offer = last_said(NOW, &said_before(Said::Offered, Some("3.6.0")));
    assert!(offer.expect("an offer").installs);
    let coming = last_said(NOW, &coming_before("3.6.0"));
    assert!(coming.expect("on its way").coming.is_some());
}

#[test]
fn what_landed_and_is_remembered_still_offers_nothing_to_press() {
    let landed = said_before(Said::Landed, Some("3.6.0"));
    let replayed = last_said(NOW, &landed).expect("still here");
    assert!(!replayed.installs);
    assert_eq!(replayed.coming, None);
}

#[test]
fn a_queue_remembered_keeps_why_it_waits() {
    let paused = said_before(Said::Paused, Some("3.6.0"));
    assert_eq!(
        last_said(NOW, &paused).expect("still paused").coming,
        Some(Coming::Paused)
    );
    let queued = said_before(Said::Queued, Some("3.6.0"));
    assert_eq!(
        last_said(NOW, &queued).expect("still queued").coming,
        Some(Coming::Queued)
    );
}

#[test]
fn only_an_offer_a_press_can_take_names_a_version() {
    let offer = |installs: bool, version: &str| Verdict {
        ready: Some(Ready {
            version: version.into(),
            installs,
            coming: None,
        }),
        memory: Memory::default(),
        unanswered: false,
    };
    assert_eq!(
        offer(true, "3.6.0").offered_version().as_deref(),
        Some("3.6.0")
    );
    assert_eq!(offer(false, "3.6.0").offered_version(), None);
    assert_eq!(offer(true, "").offered_version(), None);
    let nothing = verdict(Shelf::NotSold, Ahead::Nothing, NOW, Memory::default());
    assert_eq!(nothing.offered_version(), None);
}

#[test]
fn only_a_press_is_waited_for_and_only_when_there_may_be_something() {
    assert!(waits(true, &known("3.6.0")));
    assert!(waits(true, &Ahead::Unheard));
    assert!(!waits(true, &Ahead::Nothing));
    assert!(!waits(false, &known("3.6.0")));
    assert!(!waits(false, &Ahead::Unheard));
}

#[test]
fn a_manifest_that_reads_says_whether_a_stable_release_is_ahead() {
    assert_eq!(ahead_of(NOW, manifest("3.6.0")), known("3.6.0"));
    assert_eq!(ahead_of(NOW, manifest("3.5.1")), Ahead::Nothing);
    assert_eq!(ahead_of(NOW, manifest("3.5.0")), Ahead::Nothing);
}

#[test]
fn a_manifest_that_does_not_read_or_never_came_leaves_the_catalog_unheard() {
    assert_eq!(ahead_of(NOW, Ok("not json".into())), Ahead::Unheard);
    assert_eq!(
        ahead_of(NOW, Ok(r#"{"schema":2,"latest":"3.6.0"}"#.into())),
        Ahead::Unheard
    );
    assert_eq!(ahead_of(NOW, Err("offline".into())), Ahead::Unheard);
}

#[test]
fn the_store_is_looked_at_again_after_half_an_hour_or_when_the_clock_went_back() {
    assert!(due(None, 1_000));
    assert!(!due(Some(1_000), 1_000));
    assert!(!due(Some(1_000), 1_000 + 30 * 60 - 1));
    assert!(due(Some(1_000), 1_000 + 30 * 60));
    assert!(due(Some(5_000), 1_000));
}
