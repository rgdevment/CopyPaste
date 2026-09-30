#[test]
fn the_shortcut_sheet_is_written_in_both_tongues() {
    let spanish = keys_sheet_in(false);
    let english = keys_sheet_in(true);
    assert_eq!(spanish.len(), english.len());
    assert!(!spanish.is_empty());
    for (es, en) in spanish.iter().zip(english.iter()) {
        assert!(!es.label.is_empty() && !en.label.is_empty());
        assert!(!es.preview.is_empty() && !en.preview.is_empty());
    }
    let shared = spanish
        .iter()
        .zip(english.iter())
        .filter(|(es, en)| es.label == en.label)
        .count();
    assert_eq!(shared, 0, "not one row was left untranslated");
}

#[test]
fn the_filter_example_names_kinds_the_search_box_understands() {
    for sheet in [keys_sheet_in(false), keys_sheet_in(true)] {
        let row = sheet
            .iter()
            .find(|row| row.preview.starts_with('#'))
            .expect("the filter row is there");
        for word in row.preview.split('·') {
            let tag = word.trim().trim_start_matches('#');
            assert!(
                crate::view::kind_from_word(tag).is_some(),
                "«{tag}» is not a kind the search box knows"
            );
        }
    }
}

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
fn every_shortcut_the_sheet_promises_is_one_the_panel_handles() {
    const PANEL: &str = include_str!("../ui/panel.slint");
    for row in keys_sheet_in(true) {
        for combination in row.preview.split('·') {
            let parts: Vec<&str> = combination
                .split('+')
                .map(str::trim)
                .filter(|one| !one.is_empty())
                .collect();
            let Some(last) = parts.last() else {
                continue;
            };
            let wanted = match last.to_ascii_lowercase().as_str() {
                one if one.len() == 1 => format!("event.text == \"{one}\""),
                "enter" => "event.text == Key.Return".to_owned(),
                "esc" => "event.text == Key.Escape".to_owned(),
                "backspace" => "event.text == Key.Backspace".to_owned(),
                "delete" => "event.text == Key.Delete".to_owned(),
                "tab" => "event.text == Key.Tab".to_owned(),
                "right arrow" => "event.text == Key.RightArrow".to_owned(),
                _ => continue,
            };
            assert!(
                PANEL.contains(&wanted),
                "the sheet offers «{}» and the panel never looks for {wanted}",
                row.preview
            );
        }
    }
}
