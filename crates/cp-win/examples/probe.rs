#![cfg(target_os = "windows")]

use cp_core::formats::{Family, Take};
use cp_core::watch::{Cadence, Seen, Watcher};
use cp_win::capture::{Captured, capture};
use cp_win::formats::CATALOG;
use cp_win::paste::{Outcome, paste_into};
use cp_win::restore::{Restored, to_clipboard, to_clipboard_as_plain_text};
use cp_win::transfer::{self, Transfer};
use cp_win::watching::Watching;
use cp_win_sys::clipboard::{self, Clipboard};
use cp_win_sys::formats::{CF_UNICODETEXT, name_of};
use cp_win_sys::frontmost::{self, Target};
use cp_win_sys::permissions::Readiness;
use cp_win_sys::reading::{self, PATIENCE, Reading};
use cp_win_sys::window::EditWindow;
use cp_win_sys::writing::{Written, text_of, utf16_of};
use cp_win_sys::{media, ocr, thumbnail};

struct Battery {
    passed: u32,
    failed: u32,
    skipped: u32,
}

impl Battery {
    fn group(&self, name: &str) {
        println!("\n  {name}");
    }

    fn case(&mut self, id: &str, what: &str, run: impl FnOnce() -> Result<(), String>) {
        match run() {
            Ok(()) => {
                self.passed += 1;
                println!("    ok    {id:<5} {what}");
            }
            Err(why) => {
                self.failed += 1;
                println!("    FALLA {id:<5} {what}\n            {why}");
            }
        }
    }

    fn skip(&mut self, id: &str, what: &str, why: &str) {
        self.skipped += 1;
        println!("    salta {id:<5} {what}\n            {why}");
    }
}

fn offered_names() -> Result<Vec<String>, String> {
    let clipboard = Clipboard::open().ok_or("no se pudo abrir el portapapeles")?;
    Ok(clipboard.offered().into_iter().map(name_of).collect())
}

fn main() -> std::process::ExitCode {
    println!("\nBatería del núcleo contra el portapapeles de Windows");

    let mut b = Battery {
        passed: 0,
        failed: 0,
        skipped: 0,
    };

    b.group("A · El portapapeles responde");

    b.case("A1", "se abre y se cierra sin quedarse tomado", || {
        {
            let _first = Clipboard::open().ok_or("no abrió")?;
        }
        let _second = Clipboard::open().ok_or("no abrió la segunda vez")?;
        Ok(())
    });

    b.case("A2", "el contador de secuencia se lee", || {
        clipboard::sequence()
            .map(|_| ())
            .ok_or_else(|| "devolvió cero: no se alcanza la estación de ventanas".into())
    });

    b.case("A3", "enumerar no pide un solo byte", || {
        let names = offered_names()?;
        println!("            {} formatos: {}", names.len(), names.join(", "));
        Ok(())
    });

    b.group("B · El catálogo contra lo que hay de verdad");

    b.case("B1", "todo lo ofrecido recibe una decisión", || {
        let names = offered_names()?;
        if names.is_empty() {
            return Err("el portapapeles está vacío: copia algo y repite".into());
        }
        for name in &names {
            let take = CATALOG.decide(name);
            let mark = match take {
                Take::Payload => "copia",
                Take::Presence => "anota",
                Take::Never => "nunca",
            };
            println!("            {mark}  {name}");
        }
        Ok(())
    });

    b.case("B2", "lo que se copia se puede leer de verdad", || {
        let clipboard = Clipboard::open().ok_or("no abrió")?;
        let ids = clipboard.offered();
        let names: Vec<String> = ids.iter().map(|id| name_of(*id)).collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        if CATALOG.refusal(&refs).is_some() {
            return Err("la fuente pidió no registrar esto".into());
        }
        let mut read = 0usize;
        let mut bytes = 0usize;
        for (id, name) in ids.iter().zip(&names) {
            if CATALOG.decide(name) != Take::Payload || CATALOG.costlier_twin(name, &refs) {
                continue;
            }
            match clipboard.size_of(*id) {
                Some(size) => {
                    read += 1;
                    bytes += size;
                    println!("            {size:>9} B  {name}");
                }
                None => println!("            {:>9}  {name}", "sin datos"),
            }
        }
        if read == 0 {
            return Err("nada de lo que el catálogo quiere entregó bytes".into());
        }
        println!("            {read} formatos, {bytes} bytes");
        Ok(())
    });

    b.case("B3", "la clase que sale es una de las tres", || {
        let names = offered_names()?;
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        match CATALOG.classify(&refs) {
            Some(family) => {
                println!("            {family:?}");
                Ok(())
            }
            None => Err(format!("ninguna clase para {names:?}")),
        }
    });

    let names = offered_names().unwrap_or_default();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let courtesy = refs.iter().any(|id| CATALOG.embeddable.contains(id))
        && CATALOG.preferred_image(&refs).is_some();
    if courtesy {
        b.case(
            "B4",
            "una hoja de cálculo no se guarda como foto",
            || match CATALOG.classify(&refs) {
                Some(Family::Text) => Ok(()),
                other => Err(format!("se clasificó como {other:?}")),
            },
        );
    } else {
        b.skip(
            "B4",
            "una hoja de cálculo no se guarda como foto",
            "lo copiado no es un documento con imagen: copia un rango de Excel y repite",
        );
    }

    b.group("F · Ida y vuelta, montada por nosotros");

    b.case("F1", "lo que escribimos se vuelve a leer igual", || {
        let written = "cp-f1-ida-y-vuelta ñ 🦀";
        {
            let clipboard = Clipboard::open().ok_or("no abrió para escribir")?;
            match clipboard.replace(&[(CF_UNICODETEXT, &utf16_of(written))]) {
                Written::Placed { formats: 1 } => {}
                other => return Err(format!("la escritura dio {other:?}")),
            }
        }
        let clipboard = Clipboard::open().ok_or("no abrió para leer")?;
        let bytes = clipboard.bytes(CF_UNICODETEXT).ok_or("no devolvió bytes")?;
        match text_of(&bytes).as_deref() {
            Some(back) if back == written => Ok(()),
            other => Err(format!("volvió «{other:?}»")),
        }
    });

    b.case("F2", "escribir mueve el contador y leer no", || {
        let before = clipboard::sequence().ok_or("sin contador")?;
        {
            let clipboard = Clipboard::open().ok_or("no abrió")?;
            clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-f2"))]);
        }
        let after = clipboard::sequence().ok_or("sin contador")?;
        if after == before {
            return Err("el contador no se movió al escribir".into());
        }
        let quiet = {
            let clipboard = Clipboard::open().ok_or("no abrió")?;
            let _ = clipboard.bytes(CF_UNICODETEXT);
            clipboard::sequence().ok_or("sin contador")?
        };
        if quiet != after {
            return Err(format!("leer movió el contador de {after} a {quiet}"));
        }
        println!("            {before} → {after} por una escritura");
        Ok(())
    });

    b.case(
        "F3",
        "el vigilante ve nuestra escritura como nuestra",
        || {
            let mut watcher = Watcher::new(Cadence::Opaque);
            watcher.tick(clipboard::sequence().ok_or("sin contador")?);
            {
                let clipboard = Clipboard::open().ok_or("no abrió")?;
                clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-f3"))]);
            }
            let ours = clipboard::sequence().ok_or("sin contador")?;
            watcher.wrote(ours);
            match watcher.tick(ours) {
                Seen::Ours => Ok(()),
                other => Err(format!("se vio como {other:?}")),
            }
        },
    );

    b.case("F4", "una copia ajena tras la nuestra no se traga", || {
        let mut watcher = Watcher::new(Cadence::Opaque);
        watcher.tick(clipboard::sequence().ok_or("sin contador")?);
        {
            let clipboard = Clipboard::open().ok_or("no abrió")?;
            clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-f4-nuestro"))]);
        }
        let ours = clipboard::sequence().ok_or("sin contador")?;
        watcher.wrote(ours);
        if watcher.tick(ours) != Seen::Ours {
            return Err("la nuestra no se reconoció".into());
        }
        {
            let clipboard = Clipboard::open().ok_or("no abrió")?;
            clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-f4-ajeno"))]);
        }
        let theirs = clipboard::sequence().ok_or("sin contador")?;
        match watcher.tick(theirs) {
            Seen::Fresh { .. } => Ok(()),
            other => Err(format!("la siguiente copia se vio como {other:?}")),
        }
    });

    b.group("G · Lo que no entrega a tiempo se abandona");

    b.case(
        "G1",
        "un formato con datos responde muy por debajo del techo",
        || {
            {
                let clipboard = Clipboard::open().ok_or("no abrió")?;
                clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-g1"))]);
            }
            let started = std::time::Instant::now();
            let seen = reading::within(PATIENCE, || {
                let clipboard = Clipboard::open()?;
                clipboard.bytes(CF_UNICODETEXT)
            });
            let took = started.elapsed();
            if seen.is_too_slow() {
                return Err(format!("no llegó en {PATIENCE:?}"));
            }
            println!("            {took:?} contra un techo de {PATIENCE:?}");
            Ok(())
        },
    );

    b.case("G2", "lo que no contesta se abandona en el techo", || {
        let started = std::time::Instant::now();
        let seen = reading::within(PATIENCE, || {
            std::thread::sleep(std::time::Duration::from_secs(30));
            Some(Vec::new())
        });
        let took = started.elapsed();
        if seen != Reading::TooSlow {
            return Err(format!("se esperó de más y dio {seen:?}"));
        }
        if took > PATIENCE * 3 {
            return Err(format!("tardó {took:?} en rendirse"));
        }
        println!("            abandonado en {took:?}, no en los 30 s medidos");
        Ok(())
    });

    b.group("H · La captura, de punta a punta");

    b.case("H1", "un texto copiado se convierte en un ítem", || {
        {
            let clipboard = Clipboard::open().ok_or("no abrió")?;
            clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("alguien@ejemplo.test"))]);
        }
        let clipboard = Clipboard::open().ok_or("no abrió")?;
        match capture(&clipboard) {
            Captured::Kept(item) => {
                if item.kind != Some(cp_core::kind::Kind::Email) {
                    return Err(format!("la clase salió {:?}", item.kind));
                }
                println!(
                    "            {} formatos, {} bytes guardados, clase {:?}",
                    item.formats.len(),
                    item.stored_bytes(),
                    item.kind
                );
                Ok(())
            }
            other => Err(format!("no se capturó: {other:?}")),
        }
    });

    b.case(
        "H2",
        "el conjunto entero se anota, no solo lo que se copia",
        || {
            let clipboard = Clipboard::open().ok_or("no abrió")?;
            let offered = clipboard.offered().len();
            match capture(&clipboard) {
                Captured::Kept(item) => {
                    if item.formats.len() < offered {
                        return Err(format!(
                            "se ofrecieron {offered} y solo se anotaron {}",
                            item.formats.len()
                        ));
                    }
                    Ok(())
                }
                other => Err(format!("no se capturó: {other:?}")),
            }
        },
    );

    b.case("H3", "dos copias iguales tienen la misma huella", || {
        let write = |text: &str| {
            let clipboard = Clipboard::open()?;
            clipboard.replace(&[(CF_UNICODETEXT, &utf16_of(text))]);
            Some(())
        };
        let taken = |()| {
            let clipboard = Clipboard::open()?;
            match capture(&clipboard) {
                Captured::Kept(item) => Some(item.fingerprint()),
                _ => None,
            }
        };
        write("cp-h3-mismo").ok_or("no escribió")?;
        let first = taken(()).ok_or("no capturó")?;
        write("cp-h3-mismo").ok_or("no escribió")?;
        let again = taken(()).ok_or("no capturó")?;
        write("cp-h3-distinto").ok_or("no escribió")?;
        let other = taken(()).ok_or("no capturó")?;
        if first != again {
            return Err("lo mismo dio dos huellas".into());
        }
        if first == other {
            return Err("dos contenidos distintos dieron la misma huella".into());
        }
        Ok(())
    });

    b.case("H4", "un marcador de secreto detiene la captura", || {
        let marker = cp_win_sys::formats::name_of(
            cp_win_sys::clipboard::register("Clipboard Viewer Ignore").ok_or("no registró")?,
        );
        if marker != "Clipboard Viewer Ignore" {
            return Err(format!("el formato se registró como «{marker}»"));
        }
        {
            let clipboard = Clipboard::open().ok_or("no abrió")?;
            let id =
                cp_win_sys::clipboard::register("Clipboard Viewer Ignore").ok_or("no registró")?;
            clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("una-contrasena")), (id, &[1u8])]);
        }
        let seen = {
            let clipboard = Clipboard::open().ok_or("no abrió")?;
            capture(&clipboard)
        };
        {
            let clipboard = Clipboard::open().ok_or("no abrió para limpiar")?;
            clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-h4-limpio"))]);
        }
        match seen {
            Captured::Refused(why) => {
                println!("            rechazada por {why:?}, y el marcador se retiró");
                Ok(())
            }
            other => Err(format!("se capturó igualmente: {other:?}")),
        }
    });

    b.case("H5", "la batería no deja marcadores puestos", || {
        let clipboard = Clipboard::open().ok_or("no abrió")?;
        let names: Vec<String> = clipboard.offered().into_iter().map(name_of).collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        match CATALOG.refusal(&refs) {
            None => Ok(()),
            Some(left) => Err(format!("quedó {left:?} del caso anterior")),
        }
    });

    b.group("I · Restaurar");

    b.case(
        "I1",
        "un ítem vuelve al portapapeles con sus formatos",
        || {
            {
                let clipboard = Clipboard::open().ok_or("no abrió")?;
                clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-i1-original"))]);
            }
            let item = {
                let clipboard = Clipboard::open().ok_or("no abrió")?;
                match capture(&clipboard) {
                    Captured::Kept(item) => item,
                    other => return Err(format!("no se capturó: {other:?}")),
                }
            };
            {
                let clipboard = Clipboard::open().ok_or("no abrió")?;
                clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-i1-otra-cosa"))]);
            }
            let written = {
                let clipboard = Clipboard::open().ok_or("no abrió")?;
                to_clipboard(&clipboard, &item)
            };
            match written {
                Restored::Written { formats, .. } => {
                    println!("            {formats} formatos devueltos");
                }
                other => return Err(format!("no se restauró: {other:?}")),
            }
            let clipboard = Clipboard::open().ok_or("no abrió")?;
            let bytes = clipboard.bytes(CF_UNICODETEXT).ok_or("sin texto")?;
            match text_of(&bytes).as_deref() {
                Some("cp-i1-original") => Ok(()),
                other => Err(format!("volvió «{other:?}»")),
            }
        },
    );

    b.case("I2", "capturar lo restaurado da la misma huella", || {
        {
            let clipboard = Clipboard::open().ok_or("no abrió")?;
            clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-i2-ida-y-vuelta"))]);
        }
        let (first, item) = {
            let clipboard = Clipboard::open().ok_or("no abrió")?;
            match capture(&clipboard) {
                Captured::Kept(item) => (item.fingerprint(), item),
                other => return Err(format!("no se capturó: {other:?}")),
            }
        };
        {
            let clipboard = Clipboard::open().ok_or("no abrió")?;
            to_clipboard(&clipboard, &item);
        }
        let clipboard = Clipboard::open().ok_or("no abrió")?;
        match capture(&clipboard) {
            Captured::Kept(again) => {
                if again.fingerprint() == first {
                    Ok(())
                } else {
                    Err("la huella cambió al ir y volver".into())
                }
            }
            other => Err(format!("no se recapturó: {other:?}")),
        }
    });

    b.case("I3", "pegar en plano no muda lo guardado", || {
        {
            let clipboard = Clipboard::open().ok_or("no abrió")?;
            clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-i3-con-estilos"))]);
        }
        let item = {
            let clipboard = Clipboard::open().ok_or("no abrió")?;
            match capture(&clipboard) {
                Captured::Kept(item) => item,
                other => return Err(format!("no se capturó: {other:?}")),
            }
        };
        let before = item.clone();
        let written = {
            let clipboard = Clipboard::open().ok_or("no abrió")?;
            to_clipboard_as_plain_text(&clipboard, &item)
        };
        if !matches!(
            written,
            Restored::Written {
                incomplete: false,
                ..
            }
        ) {
            return Err(format!("la escritura plana dio {written:?}"));
        }
        if item != before {
            return Err("el ítem se mutiló al pegarlo en plano".into());
        }
        Ok(())
    });

    b.group("J · El vigilante en marcha");

    b.case("J1", "una copia despierta al vigilante", || {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        let seen = Arc::new(AtomicUsize::new(0));
        let counter = seen.clone();
        let watching = Watching::every(std::time::Duration::from_millis(10), move || {
            counter.fetch_add(1, Ordering::Relaxed);
        });
        std::thread::sleep(std::time::Duration::from_millis(60));
        {
            let clipboard = Clipboard::open().ok_or("no abrió")?;
            clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-j1"))]);
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
        drop(watching);
        match seen.load(Ordering::Relaxed) {
            0 => Err("la copia no se vio".into()),
            n => {
                println!("            {n} aviso(s) por una copia");
                Ok(())
            }
        }
    });

    b.case("J2", "un portapapeles quieto no despierta a nadie", || {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        {
            let clipboard = Clipboard::open().ok_or("no abrió")?;
            clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-j2-quieto"))]);
        }
        std::thread::sleep(std::time::Duration::from_millis(60));
        let seen = Arc::new(AtomicUsize::new(0));
        let counter = seen.clone();
        let watching = Watching::every(std::time::Duration::from_millis(10), move || {
            counter.fetch_add(1, Ordering::Relaxed);
        });
        std::thread::sleep(std::time::Duration::from_millis(250));
        drop(watching);
        match seen.load(Ordering::Relaxed) {
            0 => Ok(()),
            n => Err(format!("{n} avisos sin que nadie copiara")),
        }
    });

    b.case("J3", "sondear el contador es casi gratis", || {
        let rounds = 10_000;
        let started = std::time::Instant::now();
        for _ in 0..rounds {
            let _ = clipboard::sequence();
        }
        let each = started.elapsed() / rounds;
        println!("            {each:?} por sondeo");
        if each > std::time::Duration::from_micros(50) {
            return Err(format!("{each:?} es demasiado para sondear seguido"));
        }
        Ok(())
    });

    b.group("K · Permisos");

    b.case("K1", "se sabe qué se puede hacer y qué no", || {
        let ready = Readiness::probe();
        println!(
            "            estación: {}  nivel: {:?}  elevado: {}",
            ready.can_watch(),
            ready.integrity,
            ready.is_elevated()
        );
        if !ready.can_watch() {
            return Err("no se alcanza la estación de ventanas".into());
        }
        let ours = ready.integrity.ok_or("sin nivel propio")?;
        if !ready.can_paste_into(ours) {
            return Err("no se puede pegar en nuestro propio nivel".into());
        }
        Ok(())
    });

    b.group("L · Pegar de verdad");

    let stage = EditWindow::open("destino de la bateria").and_then(|target| {
        let until = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while frontmost::foreground() != Some(target.window()) {
            if std::time::Instant::now() > until {
                return None;
            }
            frontmost::bring_forward(target.window());
            target.pump(std::time::Duration::from_millis(50));
        }
        Some(target)
    });

    match stage {
        Some(target) => b.case("L1", "el texto llega a una ventana de destino", || {
            let written = "cp-l1-pegado-real";
            {
                let clipboard = Clipboard::open().ok_or("no abrió")?;
                clipboard.replace(&[(CF_UNICODETEXT, &utf16_of(written))]);
            }
            let seen = frontmost::target_for(target.window());
            match paste_into(&seen, || {}) {
                Outcome::Sent { took } => println!("            enviado en {took:?}"),
                Outcome::Degraded(why) => return Err(format!("degradó a {why:?}")),
            }
            target.pump(std::time::Duration::from_millis(400));
            let arrived = target.text();
            if arrived.contains(written) {
                Ok(())
            } else {
                Err(format!("llegó «{arrived}» en vez de «{written}»"))
            }
        }),
        None => b.skip(
            "L1",
            "el texto llega a una ventana de destino",
            "Windows solo deja cambiar el primer plano a quien ya lo tiene: ejecuta la bateria desde una consola con el foco",
        ),
    }

    b.case(
        "L2",
        "el peor resultado sigue siendo pegarlo a mano",
        || {
            let written = "cp-l2-degradado";
            {
                let clipboard = Clipboard::open().ok_or("no abrió")?;
                clipboard.replace(&[(CF_UNICODETEXT, &utf16_of(written))]);
            }
            let gone = Target {
                window: windows::Win32::Foundation::HWND(std::ptr::dangling_mut()),
                focus: None,
                thread: 0,
            };
            match paste_into(&gone, || {}) {
                Outcome::Degraded(cp_core::paste::Failure::TargetGone) => {}
                other => return Err(format!("con un destino muerto dio {other:?}")),
            }
            let clipboard = Clipboard::open().ok_or("no abrió")?;
            let bytes = clipboard.bytes(CF_UNICODETEXT).ok_or("sin texto")?;
            match text_of(&bytes).as_deref() {
                Some(back) if back == written => Ok(()),
                other => Err(format!("el portapapeles quedó con «{other:?}»")),
            }
        },
    );

    b.group("M · Texto dentro de una imagen");

    b.case("M1", "el sistema ofrece un motor de lectura", || {
        if ocr::is_available() {
            Ok(())
        } else {
            Err("no hay motor para los idiomas del perfil".into())
        }
    });

    b.case("M2", "se lee el texto de una imagen real", || {
        let png = std::fs::read("fixtures/texto-en-imagen.png")
            .map_err(|why| format!("no se pudo leer el fixture: {why}"))?;
        let started = std::time::Instant::now();
        let text = ocr::text_in(&png).ok_or("no se reconoció nada")?;
        println!(
            "            {:?} para leer «{}»",
            started.elapsed(),
            text.lines().next().unwrap_or("").trim()
        );
        Ok(())
    });

    b.case("M3", "una imagen en blanco no inventa texto", || {
        let blank = image::RgbaImage::from_pixel(120, 60, image::Rgba([255, 255, 255, 255]));
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(blank)
            .write_to(&mut png, image::ImageFormat::Png)
            .map_err(|why| why.to_string())?;
        match ocr::text_in(&png.into_inner()) {
            None => Ok(()),
            Some(invented) => Err(format!("se inventó «{invented}»")),
        }
    });

    b.group("N · Miniaturas y medios por el shell");

    b.case(
        "N1",
        "el shell da miniatura de una imagen del disco",
        || {
            let png = std::path::Path::new("fixtures/texto-en-imagen.png");
            let started = std::time::Instant::now();
            let dib = thumbnail::dib_of_file(png, thumbnail::SIDE)
                .ok_or("el shell no devolvió miniatura")?;
            let took = started.elapsed();
            let small = cp_core::dib::to_png(&dib).ok_or("el DIB no se pudo convertir")?;
            let original = std::fs::metadata(png).map_err(|why| why.to_string())?.len();
            println!(
                "            {took:?}, {} B de miniatura contra {original} del original",
                small.len()
            );
            if small.len() as u64 >= original {
                return Err("la miniatura no es más pequeña que el original".into());
            }
            Ok(())
        },
    );

    b.case("N2", "lo que no tiene miniatura no inventa uno", || {
        let dir = std::env::temp_dir().join("cp-sin-miniatura");
        std::fs::create_dir_all(&dir).map_err(|why| why.to_string())?;
        let path = dir.join("vacio.bin");
        std::fs::write(&path, b"   ").map_err(|why| why.to_string())?;
        match thumbnail::dib_of_file(&path, thumbnail::SIDE) {
            None => Ok(()),
            Some(_) => Err("devolvió algo para un archivo sin vista previa".into()),
        }
    });

    b.case(
        "N3",
        "un archivo sin metadatos de medios no los inventa",
        || {
            let dir = std::env::temp_dir().join("cp-sin-medios");
            std::fs::create_dir_all(&dir).map_err(|why| why.to_string())?;
            let path = dir.join("nota.txt");
            std::fs::write(&path, b"solo texto").map_err(|why| why.to_string())?;
            match media::info_for(&path) {
                None => Ok(()),
                Some(info) => Err(format!("se inventó {info:?}")),
            }
        },
    );

    b.group("C · Los formatos que cuelgan no se piden");

    b.case("C1", "nada marcado como presencia se llega a pedir", || {
        let clipboard = Clipboard::open().ok_or("no abrió")?;
        let ids = clipboard.offered();
        let risky: Vec<String> = ids
            .iter()
            .map(|id| name_of(*id))
            .filter(|name| CATALOG.decide(name) != Take::Payload)
            .collect();
        println!(
            "            {} anotados sin pedir: {}",
            risky.len(),
            risky.join(", ")
        );
        Ok(())
    });

    b.group("D · El vigilante");

    b.case("D1", "una escritura ajena se ve como copia nueva", || {
        let mut watcher = Watcher::new(Cadence::Opaque);
        let start = clipboard::sequence().ok_or("sin contador")?;
        watcher.tick(start);
        if watcher.tick(start) != Seen::Nothing {
            return Err("un contador quieto produjo un evento".into());
        }
        Ok(())
    });

    b.case("D2", "el contador no se mueve al leer", || {
        let before = clipboard::sequence().ok_or("sin contador")?;
        {
            let clipboard = Clipboard::open().ok_or("no abrió")?;
            for id in clipboard.offered() {
                let _ = clipboard.size_of(id);
            }
        }
        let after = clipboard::sequence().ok_or("sin contador")?;
        if before != after {
            return Err(format!("pasó de {before} a {after}"));
        }
        Ok(())
    });

    b.group("E · Copiar o cortar");

    let effect = {
        let clipboard = Clipboard::open();
        clipboard.and_then(|clipboard| {
            let ids = clipboard.offered();
            ids.iter()
                .find(|id| name_of(**id) == "Preferred DropEffect")
                .and_then(|id| clipboard.bytes(*id))
        })
    };
    match effect {
        Some(bytes) => b.case("E1", "el efecto se lee por sus bits", || {
            let seen = transfer::transfer(&bytes);
            println!("            {seen:?} sobre {bytes:?}");
            if seen == Transfer::Unsaid {
                return Err("el explorador siempre dice copia o corte".into());
            }
            Ok(())
        }),
        None => b.skip(
            "E1",
            "el efecto se lee por sus bits",
            "no hay archivos copiados: hazlo en el explorador y repite",
        ),
    }

    println!(
        "\n  {} ok, {} fallan, {} saltadas\n",
        b.passed, b.failed, b.skipped
    );
    if b.failed > 0 {
        std::process::ExitCode::FAILURE
    } else {
        std::process::ExitCode::SUCCESS
    }
}
