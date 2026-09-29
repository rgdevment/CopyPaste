use super::*;

#[test]
fn a_file_that_is_not_there_has_no_metadata() {
    assert_eq!(
        info_for(std::path::Path::new(r"C:\no-existe-nada.mp3")),
        None
    );
}

#[test]
fn a_file_without_media_metadata_says_nothing() {
    let dir = std::env::temp_dir().join("cp-media");
    std::fs::create_dir_all(&dir).expect("carpeta");
    let path = dir.join("texto.txt");
    std::fs::write(&path, b"no soy un video").expect("archivo");
    assert_eq!(info_for(&path), None);
}

#[test]
fn nothing_known_is_nothing_to_search() {
    assert!(MediaInfo::default().is_empty());
    assert_eq!(MediaInfo::default().searchable(), "");
}

#[test]
fn what_is_known_becomes_searchable() {
    let info = MediaInfo {
        title: Some("Canción".into()),
        artist: Some("Alguien".into()),
        album: None,
        ..Default::default()
    };
    assert!(!info.is_empty());
    assert_eq!(info.searchable(), "Canción Alguien");
}

#[test]
fn a_duration_alone_is_metadata_too() {
    let info = MediaInfo {
        duration: Some(12.5),
        ..Default::default()
    };
    assert!(!info.is_empty());
    assert_eq!(
        info.searchable(),
        "",
        "a duration is not searched for by text"
    );
}
