use super::*;
use cp_core::kind::Kind;

fn listed(kind: Kind, preview: &str, label: Option<&str>) -> Listed {
    Listed {
        id: 1,
        modified_at: 0,
        created_at: 0,
        kind: Some(kind),
        preview: preview.into(),
        app: None,
        label: label.map(str::to_owned),
        color: 0,
        thumb_path: None,
        paste_count: 0,
        last_used_at: None,
        broken_since: None,
        pinned: false,
        group: String::new(),
        snippet: None,
    }
}

#[test]
fn the_card_is_headed_by_the_name_it_was_given() {
    let row = listed(Kind::Json, "{\"a\":1}", Some("  config  "));
    assert_eq!(subject_of(&row), "config");
}

#[test]
fn without_a_name_the_card_is_headed_by_what_was_copied_on_one_line() {
    let row = listed(Kind::Text, "hola\n   mundo  ", Some("   "));
    assert_eq!(subject_of(&row), "hola mundo");
}

#[test]
fn a_file_is_headed_by_its_name_as_the_panel_heads_it() {
    let row = listed(
        Kind::File,
        "C:\\Users\\ana\\Documents\\informe-final.docx",
        None,
    );
    let body = crate::view::body_of(&row);
    let folder = crate::folder::parts_of(&row.preview);
    assert_eq!(
        subject_of(&row),
        crate::view::headline_of(&row, &body, folder.as_ref())
    );
    assert!(
        !subject_of(&row).contains("Documents"),
        "{}",
        subject_of(&row)
    );
}
