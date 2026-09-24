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
    #[cfg(target_os = "windows")]
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    #[cfg(target_os = "windows")]
    errands: Option<std::thread::JoinHandle<()>>,
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
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let errands = errands(db, stop.clone());
        Ok(Self {
            watching,
            stop,
            errands,
        })
    }

    pub fn ours(&self) -> bool {
        self.watching.ours()
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

    pub fn ours(&self) -> bool {
        true
    }
}

#[cfg(target_os = "windows")]
impl Drop for Engine {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(thread) = self.errands.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(target_os = "windows")]
const SIDE: i32 = cp_core::thumbnail::MAX_SIDE as i32;
#[cfg(target_os = "windows")]
const NAP: std::time::Duration = std::time::Duration::from_millis(400);
#[cfg(target_os = "windows")]
const LATER: i64 = 60_000;

#[cfg(target_os = "windows")]
fn errands(
    db: &Path,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Option<std::thread::JoinHandle<()>> {
    let store = match Store::open(db) {
        Ok(store) => store,
        Err(why) => {
            note(&format!("nadie enriquece lo copiado: {why}"));
            return None;
        }
    };
    let thumbs = cp_win_sys::paths::thumbs_dir()?;
    Some(std::thread::spawn(move || {
        while !stop.load(std::sync::atomic::Ordering::Relaxed) {
            if !errand(&store, &thumbs) {
                std::thread::sleep(NAP);
            }
        }
    }))
}

#[cfg(target_os = "windows")]
fn errand(store: &Store, thumbs: &Path) -> bool {
    let at = crate::app::now_ms();
    if let Some(id) = first_waiting(store, "thumb", at) {
        thumbed(store, id, at, thumbs);
        return true;
    }
    if let Some(id) = first_waiting(store, "ocr", at) {
        read_out(store, id, at);
        return true;
    }
    false
}

#[cfg(target_os = "windows")]
fn first_waiting(store: &Store, job: &str, at: i64) -> Option<i64> {
    match store.take_pending(job, at, 1) {
        Ok(waiting) => waiting.into_iter().next(),
        Err(why) => {
            note(&format!("no se pudo mirar la cola de {job}: {why}"));
            None
        }
    }
}

#[cfg(target_os = "windows")]
fn thumbed(store: &Store, id: i64, at: i64, thumbs: &Path) {
    let Some(png) = store.item(id).ok().flatten().as_ref().and_then(thumb_of) else {
        give_up(store, id, "thumb", "no se pudo hacer la miniatura", at);
        return;
    };
    let Some(landed) = written(thumbs, id, &png) else {
        give_up(store, id, "thumb", "la miniatura no se pudo guardar", at);
        return;
    };
    if let Err(why) = store.set_thumb(id, Some(&landed), at) {
        note(&format!("{id} con miniatura sin anotar: {why}"));
    }
    done(store, id, "thumb");
}

#[cfg(target_os = "windows")]
fn read_out(store: &Store, id: i64, at: i64) {
    if !cp_win_sys::ocr::is_available() {
        done(store, id, "ocr");
        return;
    }
    let found = store
        .item(id)
        .ok()
        .flatten()
        .as_ref()
        .and_then(|item| cp_win::content::content_of(item, None).image.map(Vec::from))
        .and_then(|image| cp_win_sys::ocr::text_in(&image));
    if let Some(text) = found
        && let Err(why) = store.set_ocr_text(id, &text, at)
    {
        note(&format!("{id} leída sin anotar: {why}"));
    }
    done(store, id, "ocr");
}

#[cfg(target_os = "windows")]
fn thumb_of(item: &Item) -> Option<Vec<u8>> {
    let content = cp_win::content::content_of(item, None);
    if let Some(image) = content.image {
        return cp_core::thumbnail::of_image(image, cp_core::thumbnail::MAX_SIDE);
    }
    let first = content.paths.first()?;
    let dib = cp_win_sys::thumbnail::dib_of_file(std::path::Path::new(first), SIDE)?;
    cp_core::dib::to_png(&dib)
}

#[cfg(target_os = "windows")]
fn written(dir: &Path, id: i64, png: &[u8]) -> Option<String> {
    std::fs::create_dir_all(dir).ok()?;
    let landed = dir.join(format!("{id}.png"));
    std::fs::write(&landed, png).ok()?;
    Some(landed.to_string_lossy().into_owned())
}

#[cfg(target_os = "windows")]
fn done(store: &Store, id: i64, job: &str) {
    if let Err(why) = store.work_done(id, job) {
        note(&format!("{id} sigue en la cola de {job}: {why}"));
    }
}

#[cfg(target_os = "windows")]
fn give_up(store: &Store, id: i64, job: &str, why: &str, at: i64) {
    if let Err(trouble) = store.work_failed(id, job, why, at + LATER) {
        note(&format!("{id} sin anotar el fallo de {job}: {trouble}"));
    }
}

#[cfg(target_os = "windows")]
fn kept(store: &Store) -> Option<i64> {
    let item = match cp_win::capture::capture_insisting(
        cp_win::capture::PATIENCE,
        cp_core::watch::RETRY,
    ) {
        Captured::Kept(item) => item,
        Captured::Refused(_) => {
            note("una copia se descartó por lo que la aplicación de origen pidió");
            return None;
        }
        Captured::TooSlow => {
            note("el portapapeles siguió ocupado tras insistir");
            return None;
        }
        Captured::Nothing | Captured::Superseded => return None,
    };
    let from = cp_win_sys::source::in_front();
    keep(store, &item, crate::app::now_ms(), from.as_deref())
}

#[cfg(target_os = "windows")]
fn keep(store: &Store, item: &Item, at: i64, from: Option<&str>) -> Option<i64> {
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
    if let Some(from) = from
        && let Err(why) = store.set_source(id, from, at)
    {
        note(&format!("{id} se quedó sin saber de dónde vino: {why}"));
    }
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

    fn drawn(side: u32) -> Item {
        let square = image::RgbaImage::from_pixel(side, side, image::Rgba([12, 200, 140, 255]));
        let mut out = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(square)
            .write_to(&mut out, image::ImageFormat::Png)
            .expect("dibujar");
        Item {
            kind: Some(Kind::Image),
            formats: vec![Format {
                id: SYNTHETIC_IMAGE.into(),
                payload: Payload::Inline(out.into_inner()),
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
    fn without_a_window_in_front_the_card_simply_has_no_app() {
        let (_dir, store) = somewhere();
        let id = keep(&store, &text("de ninguna parte"), 1_000, None).expect("guardado");
        let page = store
            .list(&cp_store::Filter::default(), 10, None)
            .expect("listar");
        let mine = page.rows.iter().find(|one| one.id == id).expect("está");
        assert_eq!(mine.app, None);
    }

    #[test]
    fn what_is_copied_lands_with_its_preview_and_its_kind() {
        let (_dir, store) = somewhere();
        let id = keep(&store, &text("lo primero"), 1_000, None).expect("guardado");
        let kept = store.item(id).expect("leer").expect("sigue ahí");
        assert_eq!(kept.kind, Some(Kind::Text));
        assert_eq!(store.count().expect("contar"), 1);
    }

    #[test]
    fn the_same_thing_copied_twice_is_one_row_that_rises() {
        let (_dir, store) = somewhere();
        let first = keep(&store, &text("igual"), 1_000, None).expect("guardado");
        let again = keep(&store, &text("igual"), 5_000, None).expect("reconocido");
        assert_eq!(first, again);
        assert_eq!(store.count().expect("contar"), 1);
    }

    #[test]
    fn two_different_copies_are_two_rows() {
        let (_dir, store) = somewhere();
        keep(&store, &text("una"), 1_000, None).expect("guardado");
        keep(&store, &text("otra"), 2_000, None).expect("guardado");
        assert_eq!(store.count().expect("contar"), 2);
    }

    #[test]
    fn an_image_leaves_its_reading_and_its_thumbnail_pending() {
        let (_dir, store) = somewhere();
        let id = keep(&store, &image(), 1_000, None).expect("guardada");
        let mut waiting = store.take_pending("ocr", 2_000, 10).expect("cola");
        waiting.extend(store.take_pending("thumb", 2_000, 10).expect("cola"));
        assert_eq!(waiting, [id, id]);
    }

    #[test]
    fn what_was_copied_remembers_the_app_it_came_from() {
        let (_dir, store) = somewhere();
        let id =
            keep(&store, &text("desde el navegador"), 1_000, Some("chrome")).expect("guardado");
        let page = store
            .list(&cp_store::Filter::default(), 10, None)
            .expect("listar");
        let mine = page.rows.iter().find(|one| one.id == id).expect("está");
        assert_eq!(mine.app.as_deref(), Some("chrome"));
    }

    #[test]
    fn a_picture_ends_up_with_a_thumbnail_it_can_show() {
        let (dir, store) = somewhere();
        let thumbs = dir.path().join("thumbs");
        let id = keep(&store, &drawn(600), 1_000, None).expect("guardada");
        assert!(errand(&store, &thumbs), "había trabajo que hacer");
        let page = store
            .list(&cp_store::Filter::default(), 10, None)
            .expect("listar");
        let mine = page.rows.iter().find(|one| one.id == id).expect("está");
        let made = mine.thumb_path.as_deref().expect("tiene miniatura");
        assert!(std::path::Path::new(made).exists(), "{made} no se escribió");
        let side =
            cp_core::thumbnail::size_of(&std::fs::read(made).expect("leer")).expect("tamaño");
        assert!(side.width <= cp_core::thumbnail::MAX_SIDE);
        assert!(
            store
                .take_pending("thumb", 2_000, 10)
                .expect("cola")
                .is_empty()
        );
    }

    #[test]
    fn nothing_waiting_means_nothing_to_do() {
        let (dir, store) = somewhere();
        assert!(!errand(&store, &dir.path().join("thumbs")));
    }

    #[test]
    fn something_that_cannot_be_drawn_waits_instead_of_spinning() {
        let (dir, store) = somewhere();
        let thumbs = dir.path().join("thumbs");
        let id = keep(&store, &image(), 1_000, None).expect("guardada");
        assert!(errand(&store, &thumbs), "lo intentó");
        let now = crate::app::now_ms();
        assert!(
            store
                .take_pending("thumb", now, 10)
                .expect("cola")
                .is_empty(),
            "no se reintenta de inmediato"
        );
        let later = store
            .take_pending("thumb", now + LATER + 1_000, 10)
            .expect("cola");
        assert_eq!(later, [id], "vuelve a tocarle el turno más tarde");
    }

    #[test]
    fn plain_text_asks_nobody_for_anything() {
        let (_dir, store) = somewhere();
        keep(&store, &text("sin adornos"), 1_000, None).expect("guardado");
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
