use std::path::Path;

#[cfg(target_os = "windows")]
use crate::note::note;
#[cfg(target_os = "windows")]
use cp_core::capture::Captured;
#[cfg(target_os = "windows")]
use cp_core::item::Item;
#[cfg(target_os = "windows")]
use cp_core::kind::Kind;
#[cfg(target_os = "windows")]
use cp_store::Store;

pub struct Engine {
    #[cfg(target_os = "windows")]
    watching: cp_win::watching::Watching,
}

#[cfg(target_os = "windows")]
impl Engine {
    pub fn start(db: &Path, fresh: impl Fn(i64) + Send + 'static) -> Result<Self, cp_store::Error> {
        let store = Store::open(db)?;
        let watching = cp_win::watching::Watching::start(move || {
            if let Some(id) = kept(&store) {
                fresh(id);
            }
        });
        Ok(Self { watching })
    }

    pub fn ours(&self) {
        self.watching.ours();
    }
}

#[cfg(not(target_os = "windows"))]
impl Engine {
    pub fn start(
        _db: &Path,
        _fresh: impl Fn(i64) + Send + 'static,
    ) -> Result<Self, cp_store::Error> {
        Ok(Self {})
    }

    pub fn ours(&self) {}
}

#[cfg(target_os = "windows")]
fn kept(store: &Store) -> Option<i64> {
    let item = match cp_win::capture::capture_insisting(
        cp_win::capture::PATIENCE,
        cp_core::watch::RETRY,
    ) {
        Captured::Kept(item) => item,
        Captured::Refused(why) => {
            note(&format!("copia descartada: {why:?}"));
            return None;
        }
        Captured::TooSlow => {
            note("el portapapeles siguió ocupado tras insistir");
            return None;
        }
        Captured::Nothing | Captured::Superseded => return None,
    };
    keep(store, &item, crate::app::now_ms())
}

#[cfg(target_os = "windows")]
fn keep(store: &Store, item: &Item, at: i64) -> Option<i64> {
    match store.find_by_hash(item) {
        Ok(Some(id)) => {
            if let Err(why) = store.reactivate(id, at) {
                note(&format!("la repetida {id} no subió: {why}"));
            }
            return Some(id);
        }
        Ok(None) => {}
        Err(why) => note(&format!("no se pudo buscar si ya estaba: {why}")),
    }
    let id = match store.insert_item(&name_for(at, item), item, &preview_of(item), at) {
        Ok(id) => id,
        Err(why) => {
            note(&format!("lo copiado no se pudo guardar: {why}"));
            return None;
        }
    };
    for job in jobs_for(item) {
        if let Err(why) = store.enqueue(id, job) {
            note(&format!("{id} se quedó sin encolar {job}: {why}"));
        }
    }
    Some(id)
}

#[cfg(target_os = "windows")]
fn preview_of(item: &Item) -> String {
    let content = cp_win::content::content_of(item, None);
    if let Some(text) = content.text {
        return text.into_owned();
    }
    if !content.paths.is_empty() {
        return content.paths.join("\n");
    }
    String::new()
}

#[cfg(target_os = "windows")]
fn name_for(at: i64, item: &Item) -> String {
    format!("{at:x}-{:016x}", item.fingerprint())
}

#[cfg(target_os = "windows")]
fn jobs_for(item: &Item) -> &'static [&'static str] {
    match item.kind {
        Some(Kind::Image) => &["thumb", "ocr"],
        Some(Kind::File) | Some(Kind::Folder) => &["thumb"],
        _ => &[],
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;
    use cp_core::item::{Format, Payload, SYNTHETIC_IMAGE, SYNTHETIC_TEXT};

    fn text(what: &str) -> Item {
        Item {
            kind: Some(Kind::Text),
            formats: vec![Format {
                id: SYNTHETIC_TEXT.into(),
                payload: Payload::Inline(what.as_bytes().to_vec()),
            }],
        }
    }

    fn image() -> Item {
        Item {
            kind: Some(Kind::Image),
            formats: vec![Format {
                id: SYNTHETIC_IMAGE.into(),
                payload: Payload::Inline(vec![0x89, b'P', b'N', b'G']),
            }],
        }
    }

    fn files(paths: &[&str]) -> Item {
        Item {
            kind: Some(Kind::File),
            formats: vec![Format {
                id: "CF_HDROP".into(),
                payload: Payload::Inline(cp_win::drop::drop_of(paths)),
            }],
        }
    }

    #[test]
    fn the_preview_of_text_is_the_text_itself() {
        assert_eq!(preview_of(&text("hola mundo")), "hola mundo");
    }

    #[test]
    fn an_image_has_no_preview_until_the_reading_gives_it_one() {
        assert_eq!(preview_of(&image()), "");
    }

    #[test]
    fn files_preview_as_their_paths_one_per_line() {
        let said = preview_of(&files(&["C:\\uno.txt", "C:\\dos.txt"]));
        assert_eq!(said, "C:\\uno.txt\nC:\\dos.txt");
    }

    #[test]
    fn the_same_content_copied_twice_gets_the_same_tail_and_a_different_head() {
        let one = name_for(1_000, &text("igual"));
        let other = name_for(2_000, &text("igual"));
        assert_ne!(one, other);
        assert_eq!(
            one.split_once('-').map(|it| it.1),
            other.split_once('-').map(|it| it.1)
        );
    }

    #[test]
    fn only_what_can_be_enriched_is_queued() {
        assert_eq!(jobs_for(&image()), ["thumb", "ocr"]);
        assert_eq!(jobs_for(&files(&["C:\\uno.txt"])), ["thumb"]);
        assert!(jobs_for(&text("nada que hacer")).is_empty());
    }

    fn somewhere() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().expect("carpeta");
        let store = Store::open(&dir.path().join("history.db")).expect("abrir");
        (dir, store)
    }

    #[test]
    fn what_is_copied_lands_with_its_preview_and_its_kind() {
        let (_dir, store) = somewhere();
        let id = keep(&store, &text("lo primero"), 1_000).expect("guardado");
        let kept = store.item(id).expect("leer").expect("sigue ahí");
        assert_eq!(kept.kind, Some(Kind::Text));
        assert_eq!(store.count().expect("contar"), 1);
    }

    #[test]
    fn the_same_thing_copied_twice_is_one_row_that_rises() {
        let (_dir, store) = somewhere();
        let first = keep(&store, &text("igual"), 1_000).expect("guardado");
        let again = keep(&store, &text("igual"), 5_000).expect("reconocido");
        assert_eq!(first, again);
        assert_eq!(store.count().expect("contar"), 1);
    }

    #[test]
    fn two_different_copies_are_two_rows() {
        let (_dir, store) = somewhere();
        keep(&store, &text("una"), 1_000).expect("guardado");
        keep(&store, &text("otra"), 2_000).expect("guardado");
        assert_eq!(store.count().expect("contar"), 2);
    }

    #[test]
    fn an_image_leaves_its_reading_and_its_thumbnail_pending() {
        let (_dir, store) = somewhere();
        let id = keep(&store, &image(), 1_000).expect("guardada");
        let mut waiting = store.take_pending("ocr", 2_000, 10).expect("cola");
        waiting.extend(store.take_pending("thumb", 2_000, 10).expect("cola"));
        assert_eq!(waiting, [id, id]);
    }

    #[test]
    fn plain_text_asks_nobody_for_anything() {
        let (_dir, store) = somewhere();
        keep(&store, &text("sin adornos"), 1_000).expect("guardado");
        assert!(
            store
                .take_pending("thumb", 2_000, 10)
                .expect("cola")
                .is_empty()
        );
        assert!(
            store
                .take_pending("ocr", 2_000, 10)
                .expect("cola")
                .is_empty()
        );
    }
}
