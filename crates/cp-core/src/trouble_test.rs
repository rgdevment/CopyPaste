use super::*;

#[test]
fn every_trouble_comes_back_from_its_own_key() {
    for one in ALL {
        assert_eq!(Trouble::from_key(one.key()), Some(one), "{one:?}");
    }
}

#[test]
fn no_two_troubles_share_a_key() {
    for (at, one) in ALL.iter().enumerate() {
        for other in &ALL[at + 1..] {
            assert_ne!(one.key(), other.key());
        }
    }
}

#[test]
fn a_key_nobody_sends_is_no_trouble_at_all() {
    assert_eq!(Trouble::from_key(""), None);
    assert_eq!(Trouble::from_key("the panel is not answering"), None);
    assert_eq!(Trouble::from_key("Stopped"), None);
}

#[test]
fn a_key_survives_the_line_it_travelled_on() {
    assert_eq!(Trouble::from_key("stopped\r"), Some(Trouble::Stopped));
}

#[test]
fn keys_travel_as_a_single_word() {
    for one in ALL {
        assert!(
            one.key()
                .chars()
                .all(|said| said.is_ascii_lowercase() || said == '-'),
            "«{}» would not survive the line it travels on",
            one.key()
        );
    }
}

#[test]
fn each_trouble_is_worded_in_both_tongues_and_they_differ() {
    for one in ALL {
        let es = one.worded(true);
        let en = one.worded(false);
        assert!(!es.is_empty() && !en.is_empty(), "{one:?}");
        assert_ne!(es, en, "{one:?} was left untranslated");
    }
}

#[test]
fn no_two_troubles_read_the_same() {
    for spanish in [true, false] {
        for (at, one) in ALL.iter().enumerate() {
            for other in &ALL[at + 1..] {
                assert_ne!(one.worded(spanish), other.worded(spanish));
            }
        }
    }
}

#[test]
fn the_english_words_are_the_ones_the_log_already_used() {
    assert_eq!(
        Trouble::Stopped.worded(false),
        "the panel stopped watching the clipboard"
    );
    assert_eq!(
        Trouble::ClipboardDenied.worded(false),
        crate::capture::Unreadable::Denied.said()
    );
    assert_eq!(
        Trouble::Unsteady.worded(true),
        "el panel no se mantiene abierto; reinicia CopyPaste"
    );
}

#[test]
fn the_settings_window_has_words_for_every_key() {
    let locales = include_str!("../../../app/src/locales.ts");
    for one in ALL {
        let key = one.key();
        assert!(
            locales.contains(&format!("  {key}: \"panel"))
                || locales.contains(&format!("  \"{key}\": \"panel")),
            "Settings would say «{key}» as something went wrong"
        );
    }
}

#[test]
fn only_the_notes_that_leave_copying_on_say_it_is_still_keeping() {
    let keeping: Vec<Trouble> = ALL.into_iter().filter(|one| one.still_keeping()).collect();
    assert_eq!(keeping, [Trouble::HistoryReplaced, Trouble::Unemptied]);
}
