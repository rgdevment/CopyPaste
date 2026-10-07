use cp_core::destination::Tracker;
use cp_core::item::Payload;
use cp_core::kind::Kind;
use cp_core::paste::Route;
use cp_core::watch::{Cadence, Seen, Watcher};
use cp_mac::capture::{Captured, PATIENCE, capture, capture_within};
use cp_mac::paste::Paster;
use cp_mac_sys::keyboard::{self, QWERTY_V};
use cp_mac_sys::pasteboard::{self, Pasteboard};
use cp_mac_sys::permissions::Readiness;
use cp_mac_sys::{frontmost, keystroke};
use objc2_foundation::{MainThreadMarker, NSString, NSURL};
use std::time::{Duration, Instant};

const SKIPPED: &str = "skipped: ";

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
                println!("    FAILS {id:<5} {what}\n            {why}");
            }
        }
    }

    fn case_or_skip(&mut self, id: &str, what: &str, run: impl FnOnce() -> Result<(), String>) {
        match run() {
            Err(why) if why.starts_with(SKIPPED) => {
                self.skip(id, what, why.trim_start_matches(SKIPPED));
            }
            other => self.case(id, what, || other),
        }
    }

    fn skip(&mut self, id: &str, what: &str, why: &str) {
        self.skipped += 1;
        println!("    —     {id:<5} {what}  ({why})");
    }
}

fn what_goes_back_to_the_pasteboard(b: &mut Battery, pb: &Pasteboard) {
    b.group("I · Back to the clipboard");

    b.case(
        "I0",
        "three files come back as three files, not as one broken url",
        || {
            pb.write_items(&[
                vec![("public.file-url", "file:///tmp/cp-i0-uno.txt")],
                vec![("public.file-url", "file:///tmp/cp-i0-dos.txt")],
                vec![("public.file-url", "file:///tmp/cp-i0-tres.txt")],
            ]);
            let captured = capture(pb).kept().ok_or("nothing was captured")?;

            pb.write_text("something else");

            match cp_mac::restore::to_pasteboard(pb, &captured) {
                cp_mac::restore::Restored::Written { .. } => {}
                other => return Err(format!("it restored {other:?}")),
            }
            if pb.item_count() != 3 {
                return Err(format!(
                    "the clipboard holds {} items: Finder pastes what it is given, one url or none",
                    pb.item_count()
                ));
            }
            let back = capture(pb)
                .kept()
                .ok_or("what was restored was not captured")?;
            let urls = back
                .format("public.file-url")
                .ok_or("the paths are missing")?;
            match &urls.payload {
                Payload::Inline(bytes) => {
                    let text = String::from_utf8_lossy(bytes).to_string();
                    let lines: Vec<&str> = text.lines().collect();
                    if lines.len() == 3 {
                        Ok(())
                    } else {
                        Err(format!("{} paths came back: {lines:?}", lines.len()))
                    }
                }
                other => Err(format!("{other:?} arrived")),
            }
        },
    );

    b.case("I1", "an item comes back with every format it had", || {
        pb.write_types(&[
            ("public.utf8-plain-text", "texto plano"),
            ("public.html", "<b>texto plano</b>"),
        ]);
        let captured = capture(pb).kept().ok_or("nothing was captured")?;
        let had = captured.formats.len();

        pb.write_text("something else");

        match cp_mac::restore::to_pasteboard(pb, &captured) {
            cp_mac::restore::Restored::Written { formats, .. } if formats >= 2 => {}
            other => return Err(format!("it restored {other:?} of {had} formats")),
        }

        let back = capture(pb)
            .kept()
            .ok_or("what was restored was not captured")?;
        let text = back
            .format("public.utf8-plain-text")
            .ok_or("the text is missing")?;
        if text.payload != Payload::Inline(b"texto plano".to_vec()) {
            return Err(format!("the text came back as {:?}", text.payload));
        }
        if back.format("public.html").is_none() {
            return Err("the HTML did not come back: pasting it would lose the styling".into());
        }
        Ok(())
    });

    b.case("I2", "an image comes back whole", || {
        let png = std::fs::read("fixtures/texto-en-imagen.png")
            .map_err(|why| format!("the fixture is missing: {why}"))?;
        let item = cp_core::item::Item {
            kind: Some(Kind::Image),
            formats: vec![cp_core::item::Format {
                id: "public.png".into(),
                payload: cp_core::item::Payload::Blob(png.clone()),
            }],
        };
        pb.write_text("something else entirely");
        match cp_mac::restore::to_pasteboard(pb, &item) {
            cp_mac::restore::Restored::Written { .. } => {}
            other => return Err(format!("it was not restored: {other:?}")),
        }
        let back = pb.data("public.png").ok_or("the png did not come back")?;
        if back.len() != png.len() {
            return Err(format!("{} bytes of {} came back", back.len(), png.len()));
        }
        Ok(())
    });

    b.case(
        "I3",
        "an item with no bytes says so instead of emptying the clipboard",
        || {
            let hollow = cp_core::item::Item {
                kind: None,
                formats: vec![cp_core::item::Format {
                    id: "com.apple.icns".into(),
                    payload: cp_core::item::Payload::Announced { size: Some(10) },
                }],
            };
            pb.write_text("what was there before");
            match cp_mac::restore::to_pasteboard(pb, &hollow) {
                cp_mac::restore::Restored::NothingToWrite => {}
                other => return Err(format!("it gave back {other:?}")),
            }
            let kept = pb
                .data("public.utf8-plain-text")
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or_default();
            if kept != "what was there before" {
                return Err("what the person had copied was lost".into());
            }
            Ok(())
        },
    );

    b.case(
        "I4",
        "a format announced without bytes is not counted as one that fell short",
        || {
            let partial = cp_core::item::Item {
                kind: Some(Kind::Text),
                formats: vec![
                    cp_core::item::Format {
                        id: "public.utf8-plain-text".into(),
                        payload: cp_core::item::Payload::Inline(b"algo".to_vec()),
                    },
                    cp_core::item::Format {
                        id: "public.tiff".into(),
                        payload: cp_core::item::Payload::Announced { size: Some(999) },
                    },
                ],
            };
            match cp_mac::restore::to_pasteboard(pb, &partial) {
                cp_mac::restore::Restored::Written {
                    formats: 1,
                    wanted: 1,
                } => Ok(()),
                other => Err(format!("it gave back {other:?}")),
            }
        },
    );
}

fn main() -> std::process::ExitCode {
    let Some(mtm) = MainThreadMarker::new() else {
        eprintln!("this has to run on the main thread");
        return std::process::ExitCode::FAILURE;
    };
    let pb = Pasteboard::general(mtm);
    let ready = Readiness::probe();
    let mut b = Battery {
        passed: 0,
        failed: 0,
        skipped: 0,
    };

    b.group("A · Capture and formats");

    b.case("A1", "plain text goes and comes back", || {
        pb.write_text("cp-a1");
        let item = capture(&pb).kept().ok_or("nothing was captured")?;
        if item.kind != Some(Kind::Text) {
            return Err(format!("it was classified as {:?}", item.kind));
        }
        match &item
            .format("public.utf8-plain-text")
            .ok_or("the text is missing")?
            .payload
        {
            Payload::Inline(bytes) if bytes == b"cp-a1" => Ok(()),
            other => Err(format!("{other:?} arrived")),
        }
    });

    b.case("A2", "the legacy twins are not stored twice", || {
        pb.write_text("cp-a2");
        let item = capture(&pb).kept().ok_or("nothing was captured")?;
        let ids: Vec<&str> = item.formats.iter().map(|f| f.id.as_str()).collect();
        let mut unique = ids.clone();
        unique.sort_unstable();
        unique.dedup();
        if ids.len() != unique.len() {
            return Err(format!("repetidos: {ids:?}"));
        }
        if ids.contains(&"NSStringPboardType") {
            return Err("the legacy twin did not collapse".into());
        }
        Ok(())
    });

    b.case("A3", "an empty text is still an item", || {
        pb.write_text("");
        let item = capture(&pb).kept().ok_or("nothing was captured")?;
        if item.formats.is_empty() {
            return Err("no formats".into());
        }
        Ok(())
    });

    b.case("A4", "ten megabytes go and come back whole", || {
        let big = "a".repeat(10 * 1024 * 1024);
        pb.write_text(&big);
        let item = capture(&pb).kept().ok_or("nothing was captured")?;
        match &item
            .format("public.utf8-plain-text")
            .ok_or("the text is missing")?
            .payload
        {
            Payload::Blob(bytes) if bytes.len() == big.len() => Ok(()),
            other => Err(format!("{other:?} arrived")),
        }
    });

    b.case("A5", "a single multibyte character", || {
        pb.write_text("🎯");
        let item = capture(&pb).kept().ok_or("nothing was captured")?;
        match &item
            .format("public.utf8-plain-text")
            .ok_or("the text is missing")?
            .payload
        {
            Payload::Inline(bytes) if bytes == "🎯".as_bytes() => Ok(()),
            other => Err(format!("{other:?} arrived")),
        }
    });

    b.case("A6", "line breaks of all three kinds", || {
        let mixed = "uno\r\ndos\rtres\ncuatro";
        pb.write_text(mixed);
        let item = capture(&pb).kept().ok_or("nothing was captured")?;
        match &item
            .format("public.utf8-plain-text")
            .ok_or("the text is missing")?
            .payload
        {
            Payload::Inline(bytes) if bytes == mixed.as_bytes() => Ok(()),
            other => Err(format!("{other:?} arrived")),
        }
    });

    b.case("A7", "three copied files are three paths", || {
        pb.write_items(&[
            vec![("public.file-url", "file:///tmp/uno.txt")],
            vec![("public.file-url", "file:///tmp/dos.txt")],
            vec![("public.file-url", "file:///tmp/tres.txt")],
        ]);
        if pb.item_count() != 3 {
            return Err(format!("the clipboard holds {} items", pb.item_count()));
        }
        let item = capture(&pb).kept().ok_or("nothing was captured")?;
        let urls = item
            .format("public.file-url")
            .ok_or("the path is missing")?;
        let text = match &urls.payload {
            Payload::Inline(bytes) => String::from_utf8_lossy(bytes).to_string(),
            other => return Err(format!("{other:?} arrived")),
        };
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() != 3 {
            return Err(format!("{} paths were stored: {lines:?}", lines.len()));
        }
        Ok(())
    });

    b.case("A8", "a single file is still one path", || {
        pb.write_items(&[vec![("public.file-url", "file:///tmp/solo.txt")]]);
        let item = capture(&pb).kept().ok_or("nothing was captured")?;
        let urls = item
            .format("public.file-url")
            .ok_or("the path is missing")?;
        match &urls.payload {
            Payload::Inline(bytes) if !String::from_utf8_lossy(bytes).contains('\n') => Ok(()),
            other => Err(format!("{other:?} arrived")),
        }
    });

    b.case("A9", "a Finder reference is stored as a path", || {
        let dir = std::env::temp_dir().join(format!("cp-a9-{}", pb.change_count()));
        std::fs::create_dir_all(&dir).map_err(|why| why.to_string())?;
        let file = dir.join("cp a9.png");
        std::fs::write(&file, b"png").map_err(|why| why.to_string())?;
        let reference = NSURL::fileURLWithPath(&NSString::from_str(&file.display().to_string()))
            .fileReferenceURL()
            .and_then(|url| url.absoluteString())
            .map(|url| url.to_string())
            .ok_or("the file has no reference")?;
        if !reference.starts_with("file:///.file/id=") {
            return Err(format!(
                "the reference is not shaped the way Finder shapes one: {reference}"
            ));
        }
        pb.write_items(&[vec![("public.file-url", reference.as_str())]]);
        let item = capture(&pb).kept().ok_or("nothing was captured")?;
        let stored = match &item
            .format("public.file-url")
            .ok_or("the path is missing")?
            .payload
        {
            Payload::Inline(bytes) => String::from_utf8_lossy(bytes).to_string(),
            other => return Err(format!("{other:?} arrived")),
        };
        let missing = frontmost::missing_paths(&stored);
        std::fs::remove_dir_all(&dir).ok();
        if stored.contains(".file/id=") || !stored.ends_with("/cp%20a9.png") {
            return Err(format!("«{stored}» was stored"));
        }
        if item.kind != Some(Kind::Image) {
            return Err(format!("it was classified as {:?}", item.kind));
        }
        if !missing.is_empty() {
            return Err(format!("it was taken for deleted: {missing:?}"));
        }
        Ok(())
    });

    b.case("A10", "a copied file does not cost megabytes", || {
        let icon = vec![0u8; 4 * 1024 * 1024];
        pb.write_all(&[
            ("public.file-url", b"file:///tmp/cp-a10.txt"),
            ("public.tiff", &icon),
        ]);
        let item = capture(&pb).kept().ok_or("nothing was captured")?;
        match &item
            .format("public.tiff")
            .ok_or("the icon was not even written down")?
            .payload
        {
            Payload::Announced { size: None } => {}
            other => return Err(format!("the icon was stored: {other:?}")),
        }
        if item.oversized_format().is_some() || item.stored_bytes() > 1024 {
            return Err(format!("{} bytes were stored", item.stored_bytes()));
        }
        Ok(())
    });

    b.case("A11", "a spreadsheet is not stored as a picture", || {
        let png = std::fs::read("fixtures/texto-en-imagen.png")
            .map_err(|why| format!("the fixture is missing: {why}"))?;
        pb.write_all(&[
            ("com.microsoft.Embed-Source", b"\x01"),
            ("public.utf8-plain-text", b"A\tB\nC\tD"),
            ("public.png", &png),
        ]);
        let item = capture(&pb).kept().ok_or("nothing was captured")?;
        if item.kind == Some(Kind::Image) {
            return Err("the range was classified as an image".into());
        }
        match &item
            .format("public.utf8-plain-text")
            .ok_or("the text is missing")?
            .payload
        {
            Payload::Inline(bytes) if bytes == b"A\tB\nC\tD" => Ok(()),
            other => Err(format!("{other:?} arrived")),
        }
    });

    b.case(
        "A12",
        "an image announced without bytes does not make the item an image",
        || {
            pb.write_all(&[("public.utf8-plain-text", b"celda"), ("public.png", b"")]);
            let item = capture(&pb).kept().ok_or("nothing was captured")?;
            match &item
                .format("public.png")
                .ok_or("the image was not even written down")?
                .payload
            {
                Payload::Absent => {}
                other => return Err(format!("an empty image was stored: {other:?}")),
            }
            if item.kind != Some(Kind::Text) {
                return Err(format!("it was classified as {:?}", item.kind));
            }
            Ok(())
        },
    );

    what_goes_back_to_the_pasteboard(&mut b, &pb);

    b.group("J · Files that are gone");

    b.case("J1", "a path that exists is not marked broken", || {
        let path = std::env::temp_dir().join("cp-probe-existe.txt");
        std::fs::write(&path, b"aqui estoy").map_err(|why| why.to_string())?;
        let url = format!("file://{}", path.display());
        let missing = frontmost::missing_paths(&url);
        std::fs::remove_file(&path).ok();
        if !missing.is_empty() {
            return Err(format!("it said this was missing: {missing:?}"));
        }
        Ok(())
    });

    b.case("J2", "a deleted path is noticed", || {
        let path = std::env::temp_dir().join("cp-probe-borrado.txt");
        std::fs::write(&path, "efimero".as_bytes()).map_err(|why| why.to_string())?;
        let url = format!("file://{}", path.display());
        std::fs::remove_file(&path).map_err(|why| why.to_string())?;
        let missing = frontmost::missing_paths(&url);
        if missing.len() != 1 {
            return Err("it never noticed the file is gone".into());
        }
        Ok(())
    });

    b.case("J3", "of three files it says which one is missing", || {
        let dir = std::env::temp_dir();
        let uno = dir.join("cp-probe-uno.txt");
        let dos = dir.join("cp-probe-dos.txt");
        std::fs::write(&uno, b"a").map_err(|why| why.to_string())?;
        std::fs::write(&dos, b"b").map_err(|why| why.to_string())?;
        let urls = format!(
            "file://{}\nfile://{}\nfile://{}",
            uno.display(),
            dos.display(),
            dir.join("cp-probe-fantasma.txt").display()
        );
        let missing = frontmost::missing_paths(&urls);
        std::fs::remove_file(&uno).ok();
        std::fs::remove_file(&dos).ok();
        if missing.len() != 1 || !missing[0].contains("fantasma") {
            return Err(format!("it said these were missing: {missing:?}"));
        }
        Ok(())
    });

    b.case("J4", "a path with spaces and accents is understood", || {
        let path = std::env::temp_dir().join("cp probe ñandú.txt");
        std::fs::write(&path, b"with accents").map_err(|why| why.to_string())?;
        let encoded = format!(
            "file://{}",
            path.display()
                .to_string()
                .replace(' ', "%20")
                .replace('ñ', "%C3%B1")
                .replace('ú', "%C3%BA")
        );
        let missing = frontmost::missing_paths(&encoded);
        std::fs::remove_file(&path).ok();
        if !missing.is_empty() {
            return Err("an escaped path was taken for one that is not there".into());
        }
        Ok(())
    });

    b.group("K · Where it came from");

    b.case(
        "K1",
        "the app it came from is stored under the name people see",
        || {
            let (pid, bundle) = frontmost::frontmost().ok_or("nobody in front")?;
            let name = frontmost::app_name(pid).ok_or("no name people would see")?;
            if name.is_empty() {
                return Err("the name arrived empty".into());
            }
            if Some(name.as_str()) == bundle.as_deref() {
                return Err(format!("«{name}» is the identifier, not the name"));
            }

            let store = cp_store::Store::in_memory().map_err(|why| why.to_string())?;
            let id = store
                .insert_text("uuid-origen", "something copied", 1)
                .map_err(|why| why.to_string())?;
            store
                .set_source(id, &name, 2)
                .map_err(|why| why.to_string())?;
            let found = store
                .list(
                    &cp_store::Filter {
                        query: Some(name.clone()),
                        ..Default::default()
                    },
                    10,
                    None,
                )
                .map_err(|why| why.to_string())?
                .rows;
            if found.len() != 1 {
                return Err(format!(
                    "searching «{name}» turned up {} items",
                    found.len()
                ));
            }
            Ok(())
        },
    );

    b.group("L · Thumbnails and media");

    b.case(
        "L1",
        "a screenshot gives its dimensions without being decoded",
        || {
            let png = std::fs::read("fixtures/texto-en-imagen.png")
                .map_err(|why| format!("the fixture is missing: {why}"))?;
            let size = cp_core::thumbnail::size_of(&png).ok_or("the size was not read")?;
            if size.width != 1440 || size.height != 320 {
                return Err(format!("dijo {}×{}", size.width, size.height));
            }
            Ok(())
        },
    );

    b.case(
        "L2",
        "the thumbnail weighs far less and keeps its proportions",
        || {
            let png = std::fs::read("fixtures/texto-en-imagen.png")
                .map_err(|why| format!("the fixture is missing: {why}"))?;
            let thumb = cp_core::thumbnail::of_image(&png, cp_core::thumbnail::MAX_SIDE)
                .ok_or("it was not generated")?;
            let size = cp_core::thumbnail::size_of(&thumb).ok_or("no size")?;
            if size.width != cp_core::thumbnail::MAX_SIDE {
                return Err(format!("the longer side came out at {}", size.width));
            }
            if thumb.len() >= png.len() {
                return Err(format!("pesa {} frente a {}", thumb.len(), png.len()));
            }
            Ok(())
        },
    );

    b.case(
        "L3",
        "the thumbnail goes to the store and comes back",
        || {
            let dir = std::env::temp_dir().join(format!("cp-probe-thumbs-{}", pb.change_count()));
            let blobs = cp_store::Blobs::at(&dir).map_err(|why| why.to_string())?;
            let png = std::fs::read("fixtures/texto-en-imagen.png")
                .map_err(|why| format!("the fixture is missing: {why}"))?;
            let thumb = cp_core::thumbnail::of_image(&png, cp_core::thumbnail::MAX_SIDE)
                .ok_or("it was not generated")?;
            let digest = blobs.put(&thumb).map_err(|why| why.to_string())?;
            let back = blobs
                .get(&digest)
                .map_err(|why| why.to_string())?
                .ok_or("it did not come back")?;
            std::fs::remove_dir_all(&dir).ok();
            if back != thumb {
                return Err("the thumbnail came back different".into());
            }
            Ok(())
        },
    );

    b.case("L4", "a file that is not media invents no metadata", || {
        let path = std::env::temp_dir().join("cp-probe-no-media.txt");
        std::fs::write(&path, b"text and nothing else").map_err(|why| why.to_string())?;
        let info = cp_mac_sys::media::info_for(&path);
        std::fs::remove_file(&path).ok();
        match info {
            None => Ok(()),
            Some(found) if found.duration.is_none() => Ok(()),
            Some(found) => Err(format!("it made up {found:?}")),
        }
    });

    b.case("L5", "a path that is not there is no media", || {
        let ghost = std::env::temp_dir().join("cp-probe-not-there.mp4");
        if cp_mac_sys::media::info_for(&ghost).is_some() {
            return Err("it gave back data for something that is not there".into());
        }
        Ok(())
    });

    b.case(
        "I5",
        "pasting as plain text does not maim what was stored",
        || {
            pb.write_types(&[
                ("public.utf8-plain-text", "with styling"),
                ("public.html", "<b>with styling</b>"),
                ("public.rtf", "{\\rtf1 with styling}"),
            ]);
            let item = capture(&pb).kept().ok_or("nothing was captured")?;
            let had = item.formats.len();

            let plain = cp_core::paste_as::render(
                cp_core::paste_as::Form::PlainText,
                &cp_mac::content::content_of(&item, None),
            )
            .ok_or("the plain form was never offered")?
            .into_item();
            match cp_mac::restore::to_pasteboard(&pb, &plain) {
                cp_mac::restore::Restored::Written { formats: 1, .. } => {}
                other => return Err(format!("it gave back {other:?}")),
            }
            if pb.data("public.html").is_some() {
                return Err("the HTML stayed: it was not pasted as plain text".into());
            }

            if item.formats.len() != had {
                return Err("the item lost formats".into());
            }
            match cp_mac::restore::to_pasteboard(&pb, &item) {
                cp_mac::restore::Restored::Written { .. } => {}
                other => return Err(format!("it could not be restored with styling: {other:?}")),
            }
            if pb.data("public.html").is_none() {
                return Err("the HTML did not come back: the item had been maimed".into());
            }
            Ok(())
        },
    );

    b.case(
        "I6",
        "an item with no plain text cannot be pasted as plain text",
        || {
            let only_image = cp_core::item::Item {
                kind: Some(Kind::Image),
                formats: vec![cp_core::item::Format {
                    id: "public.png".into(),
                    payload: cp_core::item::Payload::Inline(vec![1, 2, 3]),
                }],
            };
            pb.write_text("what was there");
            let content = cp_mac::content::content_of(&only_image, None);
            let forms = cp_core::paste_as::forms_for(&content);
            if forms.contains(&cp_core::paste_as::Form::PlainText) {
                return Err("pasting as plain text was offered with no text".into());
            }
            match cp_core::paste_as::render(cp_core::paste_as::Form::PlainText, &content) {
                None => Ok(()),
                Some(other) => Err(format!("it gave back {other:?}")),
            }
        },
    );

    b.group("G · Classification");

    for (id, text, expected) in [
        ("G1", "alguien@ejemplo.test", Kind::Email),
        ("G2", "https://ejemplo.test/ruta", Kind::Link),
        ("G3", "#FF8800", Kind::Color),
        ("G4", "192.168.1.1", Kind::Ip),
        ("G5", "7ab3f6de-1c4b-4f5e-8a2d-9f0e1b2c3d4e", Kind::Uuid),
        ("G6", "+34 600 123 456", Kind::Phone),
        ("G7", "{\"clave\": [1, 2]}", Kind::Json),
        ("G8", "fn main() {\n    println!(\"hola\");\n}", Kind::Code),
        ("G9", "an ordinary sentence and nothing more", Kind::Text),
    ] {
        b.case(
            id,
            &format!("it is classified as {}", expected.as_str()),
            || {
                pb.write_text(text);
                let item = capture(&pb).kept().ok_or("nothing was captured")?;
                if item.kind != Some(expected) {
                    return Err(format!("it came out {:?}", item.kind));
                }
                Ok(())
            },
        );
    }

    b.group("H · Clever search");

    b.case("H1", "the text inside an image is recognised", || {
        let png = std::fs::read("fixtures/texto-en-imagen.png")
            .map_err(|why| format!("the fixture is missing: {why}"))?;
        let text =
            cp_mac_sys::ocr::searchable_text(&png).ok_or("Vision could not manage the image")?;
        if !text.contains("AB-4417") {
            return Err(format!("it read «{text}»"));
        }
        Ok(())
    });

    b.case(
        "H2",
        "a copied screenshot is found by what it says inside",
        || {
            let png = std::fs::read("fixtures/texto-en-imagen.png")
                .map_err(|why| format!("the fixture is missing: {why}"))?;
            pb.write_data("public.png", &png);

            let item = capture(&pb).kept().ok_or("nothing was captured")?;
            if item.kind != Some(Kind::Image) {
                return Err(format!("it was classified as {:?}", item.kind));
            }
            let bytes = match &item
                .format("public.png")
                .ok_or("the png is missing")?
                .payload
            {
                Payload::Inline(bytes) | Payload::Blob(bytes) => bytes.clone(),
                other => return Err(format!("{other:?} arrived")),
            };

            let store = cp_store::Store::in_memory().map_err(|why| why.to_string())?;
            let light = cp_core::item::Item {
                kind: item.kind,
                formats: vec![cp_core::item::Format {
                    id: "public.png".into(),
                    payload: cp_core::item::Payload::Announced {
                        size: Some(bytes.len()),
                    },
                }],
            };
            let id = store
                .insert_item("uuid-captura", &light, "", 1)
                .map_err(|why| why.to_string())?;
            let recognised = cp_mac_sys::ocr::searchable_text(&bytes)
                .ok_or("Vision could not manage what was captured")?;
            store
                .set_ocr_text(id, &recognised, 2)
                .map_err(|why| why.to_string())?;

            let hits = store
                .list(
                    &cp_store::Filter {
                        query: Some("pedido".into()),
                        ..Default::default()
                    },
                    10,
                    None,
                )
                .map_err(|why| why.to_string())?
                .rows;
            if hits.len() != 1 {
                return Err(format!("searching «order» turned up {} items", hits.len()));
            }
            Ok(())
        },
    );

    b.group("B · Privacy");

    for (id, marker) in [
        ("B1", "org.nspasteboard.ConcealedType"),
        ("B2", "org.nspasteboard.TransientType"),
        ("B3", "com.agilebits.onepassword"),
        ("B4", "net.antelle.keeweb"),
        ("B5", "PasswordPboardType"),
    ] {
        b.case(id, &format!("«{marker}» leaves the item out"), || {
            pb.write_types(&[("public.utf8-plain-text", "secreto"), (marker, "1")]);
            if capture(&pb).kept().is_some() {
                return Err("content that was marked got captured".into());
            }
            Ok(())
        });
    }

    b.case("B6", "with no marker it is captured again", || {
        pb.write_text("this one does");
        capture(&pb)
            .kept()
            .ok_or("an ordinary text has to be captured")?;
        Ok(())
    });

    b.group("C · The watcher");

    b.case("C1", "the counter rises one at a time", || {
        let before = pb.change_count();
        pb.write_text("cp-c1");
        let after = pb.change_count();
        if after - before != 1 {
            return Err(format!("it jumped from {before} to {after}"));
        }
        Ok(())
    });

    b.case("C2", "a hundred copies, not one lost in silence", || {
        let mut watcher = Watcher::new(Cadence::OnePerCopy);
        watcher.tick(pb.change_count());
        let mut seen = 0u64;
        for round in 0..100 {
            pb.write_text(&format!("cp-c2-{round}"));
            if let Seen::Fresh { .. } = watcher.tick(pb.change_count()) {
                seen += 1;
            }
        }
        let missed = watcher.missed().ok_or("the macOS cadence counts")?;
        if seen + missed != 100 {
            return Err(format!("{seen} seen and {missed} counted"));
        }
        Ok(())
    });

    b.case(
        "C3",
        "polling from another thread without losing a thing",
        || {
            let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let (tell, hear) = std::sync::mpsc::channel();
            let watching = {
                let stop = stop.clone();
                std::thread::spawn(move || {
                    let mut watcher = Watcher::new(Cadence::OnePerCopy);
                    let mut fresh = 0u64;
                    while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                        if let Seen::Fresh { .. } =
                            watcher.tick(pasteboard::change_count_from_any_thread())
                        {
                            fresh += 1;
                        }
                        std::thread::sleep(Duration::from_millis(2));
                    }
                    let _ = tell.send((fresh, watcher.missed().unwrap_or(0)));
                })
            };
            std::thread::sleep(Duration::from_millis(40));
            for round in 0..25 {
                pb.write_text(&format!("cp-c3-{round}"));
                std::thread::sleep(Duration::from_millis(12));
            }
            std::thread::sleep(Duration::from_millis(80));
            stop.store(true, std::sync::atomic::Ordering::Relaxed);
            watching.join().map_err(|_| "the thread died")?;
            let (fresh, missed) = hear.recv().map_err(|why| why.to_string())?;
            if fresh + missed != 25 {
                return Err(format!("{fresh} seen and {missed} counted out of 25"));
            }
            Ok(())
        },
    );

    b.case("C4", "a write of ours is never mistaken for a copy", || {
        let mut watcher = Watcher::new(Cadence::OnePerCopy);
        watcher.tick(pb.change_count());
        pb.write_text("cp-c4-nuestro");
        let ours = pb.change_count();
        watcher.wrote(ours);
        match watcher.tick(ours) {
            Seen::Ours => Ok(()),
            other => Err(format!("it was seen as {other:?}")),
        }
    });

    b.case(
        "C5",
        "the bounded capture gives back what the direct one gives",
        || {
            pb.write_text("cp-c5");
            let direct = capture(&pb).kept().ok_or("nothing was captured")?;
            match capture_within(PATIENCE) {
                Captured::Kept(item) if item == direct => Ok(()),
                other => Err(format!("{other:?} arrived")),
            }
        },
    );

    b.case(
        "C6",
        "insisting on a source that answers costs no second attempt",
        || {
            pb.write_text("cp-c6");
            let direct = capture(&pb).kept().ok_or("nothing was captured")?;
            let started = std::time::Instant::now();
            let got = cp_mac::capture::capture_insisting(PATIENCE, cp_core::watch::RETRY);
            let took = started.elapsed();
            match got {
                Captured::Kept(item) if item == direct => {
                    if took < PATIENCE {
                        Ok(())
                    } else {
                        Err(format!("it took {took:?}: there was a pause for no reason"))
                    }
                }
                other => Err(format!("{other:?} arrived")),
            }
        },
    );

    b.group("D · Keyboard");

    b.case("D1", "the active layout resolves the «v»", || {
        keyboard::keycode_with_command('v')
            .map(|_| ())
            .ok_or_else(|| "it could not be resolved".to_string())
    });

    b.case(
        "D2",
        "Dvorak needs a different keycode",
        || match keyboard::keycode_with_command_in(keyboard::DVORAK, 'v') {
            Some(code) if code != QWERTY_V => Ok(()),
            Some(code) => Err(format!("it gave 0x{code:02X}, the same as QWERTY")),
            None => Err("Dvorak is not installed".into()),
        },
    );

    b.case("D3", "every other layout agrees with QWERTY", || {
        for (name, id) in [
            ("ABC", keyboard::ABC),
            ("AZERTY", keyboard::AZERTY),
            ("QWERTZ", keyboard::QWERTZ),
            ("Spanish ISO", keyboard::SPANISH_ISO),
            ("Colemak", keyboard::COLEMAK),
            ("Dvorak-QWERTY ⌘", keyboard::DVORAK_COMMAND_QWERTY),
        ] {
            if let Some(code) = keyboard::keycode_with_command_in(id, 'v')
                && code != QWERTY_V
            {
                return Err(format!("{name} dio 0x{code:02X}"));
            }
        }
        Ok(())
    });

    b.case("D4", "a letter no layout produces invents nothing", || {
        match keyboard::keycode_with_command_in(keyboard::ABC, '\u{1F600}') {
            None => Ok(()),
            Some(code) => Err(format!("it gave back 0x{code:02X} for an emoji")),
        }
    });

    b.case(
        "D5",
        "the keycode is resolved when pasting, not when starting",
        || {
            let paster = Paster::new().ok_or("no event source")?;
            let now = keyboard::keycode_with_command('v').unwrap_or(QWERTY_V);
            if paster.keycode() != now {
                return Err(format!(
                    "the paster says 0x{:02X} and the system says 0x{now:02X}",
                    paster.keycode()
                ));
            }
            Ok(())
        },
    );

    b.group("E · Permissions");

    b.case(
        "E1",
        "pasting hangs only on being able to post events",
        || {
            if ready.can_paste() != ready.can_post {
                return Err("the rule bent".into());
            }
            Ok(())
        },
    );

    b.case("E2", "secure input does not block pasting", || {
        if ready.secure_input && !ready.can_paste() && ready.can_post {
            return Err("secure input is being treated as a block".into());
        }
        Ok(())
    });

    b.group("F · Target and pasting");

    b.case(
        "F1",
        "the target survives the panel taking the front",
        || {
            let mut tracker = Tracker::new(frontmost::our_pid());
            let (pid, bundle) = frontmost::frontmost().ok_or("nobody in front")?;
            tracker.saw(pid, bundle.as_deref());
            let target = tracker.destination().ok_or("no target")?.clone();
            tracker.saw(frontmost::our_pid(), Some("dev.rgdevment.copypaste"));
            if tracker.destination() != Some(&target) {
                return Err("the target changed".into());
            }
            Ok(())
        },
    );

    b.case(
        "F2",
        "the physical modifiers are read from the source",
        || {
            let _ = keystroke::physical_modifiers();
            if keystroke::modifiers_still_held() {
                return Err("modifiers are held down; let the keys go".into());
            }
            Ok(())
        },
    );

    match Paster::new() {
        Some(paster) => {
            b.case(
                "F3",
                "the paster hands over a keycode that can be used",
                || {
                    if paster.keycode() == 0 {
                        return Err("an invalid keycode".into());
                    }
                    Ok(())
                },
            );
            for (id, route, what, allowed, permission) in [
                (
                    "F4",
                    Route::Keystroke,
                    "pegado real en TextEdit",
                    ready.can_post,
                    "no permission to post events",
                ),
                (
                    "F5",
                    Route::Menu,
                    "pasted through the Edit menu when ⌘V will not go in",
                    ready.accessibility,
                    "no Accessibility permission",
                ),
            ] {
                if !allowed {
                    b.skip(id, what, permission);
                    continue;
                }
                match paste_round_trip(&pb, &paster, route) {
                    Ok(()) => {
                        b.passed += 1;
                        println!("    ok    {id:<5} {what}, there and back");
                    }
                    Err(why) if why.starts_with("TextEdit never arrived") => {
                        b.skip(id, what, &why);
                    }
                    Err(why) => {
                        b.failed += 1;
                        println!("    FAILS {id:<5} {what}\n            {why}");
                    }
                }
            }
        }
        None => b.skip("F3", "the paster is built", "there is no event source"),
    }

    b.group("M · Open and reveal");

    b.case_or_skip(
        "M1",
        "revealing a file leaves it selected in the Finder",
        || {
            let scratch = Scratch::new("cp-m1")?;
            let file = scratch.file("revelado.txt", b"cp-m1")?;
            if !cp_mac_sys::files::reveal(&file) {
                return Err("reveal said no".into());
            }
            wait_until("the Finder leaves the file selected", || {
                osascript_within("tell application \"Finder\" to get selection as text")
                    .map(|selected| selected.contains("revelado.txt"))
            })
        },
    );

    b.case_or_skip("M2", "opening a file hands it to its application", || {
        let scratch = Scratch::new("cp-m2")?;
        let file = scratch.file("abierto.txt", b"cp-m2")?;
        if !cp_mac_sys::files::open(&file) {
            return Err("open said no".into());
        }
        let opened = wait_until("TextEdit has the document open", || {
            osascript_within("tell application \"TextEdit\" to get name of every document")
                .map(|names| names.contains("abierto.txt"))
        });
        osascript_within(
            "tell application \"TextEdit\" to close (every document whose name is \"abierto.txt\") saving no",
        );
        opened
    });

    b.group("N · LinkUnbound");

    b.case_or_skip(
        "N1",
        "what answers the scheme is a bundle that is really on disk",
        || {
            let app = cp_mac_sys::files::app_for_link("linkunbound:")
                .ok_or(format!("{SKIPPED}LinkUnbound is not installed here"))?;
            if !std::path::Path::new(&app).is_dir() {
                return Err(format!("{app} is not a directory on disk"));
            }
            if !app.ends_with(".app") {
                return Err(format!("{app} is not an application bundle"));
            }
            Ok(())
        },
    );

    b.case_or_skip(
        "N2",
        "the link handed over is escaped whole and lands there",
        || {
            let holder = cp_mac_sys::files::app_for_link("linkunbound:")
                .ok_or(format!("{SKIPPED}LinkUnbound is not installed here"))?;
            let asked =
                cp_core::linkunbound::asked_for("https://example.com/a path?x=1&y=ñ#top", true)
                    .ok_or("no link was built for it")?;
            let payload = asked
                .strip_prefix("linkunbound://open?url=")
                .ok_or_else(|| format!("{asked} is not the form LinkUnbound reads"))?;
            if let Some(loose) = payload
                .chars()
                .find(|one| !one.is_ascii_alphanumeric() && *one != '%')
            {
                return Err(format!("{loose:?} went through unescaped in {payload}"));
            }
            match cp_mac_sys::files::app_for_link(&asked) {
                Some(app) if app == holder => Ok(()),
                other => Err(format!("{other:?} would open it instead of {holder}")),
            }
        },
    );

    b.group("R · Dragging out");

    b.case(
        "R1",
        "the only Objective-C class this panel defines answers what a drag asks of it",
        || {
            if !cp_mac_sys::dragging::source_answers() {
                return Err(
                    "CopyPasteDragSource does not answer                      draggingSession:sourceOperationMaskForDraggingContext:, so AppKit would                      refuse every drag out of the panel"
                        .into(),
                );
            }
            Ok(())
        },
    );

    b.case(
        "R2",
        "a drag really starts, from a window of our own, carrying a name with spaces in it",
        || {
            let at = std::env::temp_dir().join("cp-r2 dragged by its name.txt");
            std::fs::write(&at, b"cp-r2").map_err(|why| why.to_string())?;
            let said = cp_mac_sys::dragging::a_drag_would_start(&[at.as_path()])
                .ok_or("this needs the main thread and a window of its own")?;
            let _ = std::fs::remove_file(&at);
            if said != cp_mac_sys::dragging::Dragged::Started {
                return Err(format!(
                    "no drag would start from a window of ours: {said:?}"
                ));
            }
            Ok(())
        },
    );

    b.case(
        "R3",
        "a drag of nothing, and of a path macOS cannot spell, never leaves the window",
        || {
            let empty = cp_mac_sys::dragging::a_drag_would_start(&[])
                .ok_or("this needs the main thread and a window of its own")?;
            if empty != cp_mac_sys::dragging::Dragged::Nothing {
                return Err(format!("a drag of nothing answered {empty:?}"));
            }
            use std::os::unix::ffi::OsStrExt;
            let bad = std::ffi::OsStr::from_bytes(&[0xff, 0xfe]);
            let said = cp_mac_sys::dragging::a_drag_would_start(&[std::path::Path::new(bad)])
                .ok_or("this needs the main thread and a window of its own")?;
            if said != cp_mac_sys::dragging::Dragged::Nothing {
                return Err(format!("a path that is not text answered {said:?}"));
            }
            Ok(())
        },
    );

    the_panel_and_its_windows(&mut b);

    println!();
    println!(
        "  {} pass · {} fail · {} skipped",
        b.passed, b.failed, b.skipped
    );
    if b.failed == 0 {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}

struct Scratch {
    dir: std::path::PathBuf,
}

impl Scratch {
    fn new(prefix: &str) -> Result<Self, String> {
        let dir = std::env::temp_dir().join(format!("{prefix}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).map_err(|why| why.to_string())?;
        Ok(Self { dir })
    }

    fn file(&self, name: &str, bytes: &[u8]) -> Result<std::path::PathBuf, String> {
        let path = self.dir.join(name);
        std::fs::write(&path, bytes).map_err(|why| why.to_string())?;
        Ok(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.dir).ok();
    }
}

fn osascript_within(script: &str) -> Option<String> {
    let mut child = std::process::Command::new("/usr/bin/osascript")
        .args(["-e", script])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(3) {
        if child.try_wait().ok()?.is_some() {
            let out = child.wait_with_output().ok()?;
            if !out.status.success() {
                return None;
            }
            return String::from_utf8(out.stdout)
                .ok()
                .map(|text| text.trim().to_owned());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    child.kill().ok();
    child.wait().ok();
    None
}

fn wait_until(what: &str, mut observed: impl FnMut() -> Option<bool>) -> Result<(), String> {
    for _ in 0..25 {
        std::thread::sleep(Duration::from_millis(300));
        match observed() {
            Some(true) => return Ok(()),
            Some(false) => {}
            None => {
                return Err(format!(
                    "{SKIPPED}the app does not answer AppleScript: no Automation permission, or a dialog is waiting"
                ));
            }
        }
    }
    Err(format!("{what}: it did not happen within 7.5 s"))
}

fn paste_round_trip(pb: &Pasteboard, paster: &Paster, route: Route) -> Result<(), String> {
    let path = "/tmp/cp-probe-target.txt";
    std::fs::write(path, "").map_err(|why| why.to_string())?;
    run_open(&["-a", "TextEdit", path]);
    let mut front = None;
    for _ in 0..25 {
        std::thread::sleep(Duration::from_millis(300));
        let textedit = std::process::Command::new("/usr/bin/pgrep")
            .args(["-x", "TextEdit"])
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .and_then(|pids| pids.split_whitespace().next()?.parse::<i32>().ok());
        if let Some(pid) = textedit {
            frontmost::bring_to_front(pid);
            std::thread::sleep(Duration::from_millis(250));
            cp_mac_sys::runloop::pump(0.0);
            if let Some((front_pid, bundle)) = frontmost::frontmost()
                && front_pid == pid
            {
                front = Some((front_pid, bundle));
                break;
            }
        }
    }
    let (pid, bundle) = front.ok_or_else(|| {
        format!(
            "TextEdit did not come to the front within 8 s; in front is {:?}",
            frontmost::frontmost().and_then(|(_, b)| b)
        )
    })?;
    let target = cp_core::destination::Destination {
        pid,
        bundle_id: bundle,
    };

    let marca = format!("CP-PASTE-{}", pb.change_count());
    pb.write_text(&marca);
    std::thread::sleep(Duration::from_millis(120));

    let started = Instant::now();
    match pasted_pumping(paster, route, &target)? {
        cp_mac::paste::Outcome::Degraded(why) => return Err(format!("it degraded: {why:?}")),
        cp_mac::paste::Outcome::Sent { via, .. } if via != route => {
            return Err(format!("it went by {via:?}"));
        }
        cp_mac::paste::Outcome::Sent { .. } => {}
    }
    std::thread::sleep(Duration::from_millis(400));

    let keys = cp_mac_sys::keystroke::Keystroke::new().ok_or("no source")?;
    keys.command(0x00);
    std::thread::sleep(Duration::from_millis(200));
    keys.command(0x08);
    std::thread::sleep(Duration::from_millis(400));

    let back = pb
        .data("public.utf8-plain-text")
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_default();

    if back.contains(&marca) {
        println!("            (there and back in {:?})", started.elapsed());
        Ok(())
    } else {
        Err(format!(
            "«{}» came back, «{marca}» was expected",
            back.trim()
        ))
    }
}

fn pasted_pumping(
    paster: &Paster,
    route: Route,
    target: &cp_core::destination::Destination,
) -> Result<cp_mac::paste::Outcome, String> {
    let mut pasting = paster
        .start_via(Some(route), target.clone(), || {})
        .map_err(|outcome| format!("it never started: {outcome:?}"))?;
    loop {
        match paster.advance(&mut pasting) {
            cp_mac::paste::Advance::Done(outcome) => return Ok(outcome),
            cp_mac::paste::Advance::After(pause) => {
                std::thread::sleep(pause);
                cp_mac_sys::runloop::pump(0.0);
            }
        }
    }
}

fn run_open(args: &[&str]) {
    let mut command = std::process::Command::new("/usr/bin/open");
    command.args(args);
    let _ = command.status();
}
