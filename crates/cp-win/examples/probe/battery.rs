use cp_core::formats::{Family, Take};
use cp_core::paste_as::{Form, forms_for, render};
use cp_core::watch::{Cadence, Seen, Watcher};
use cp_win::capture::{self, Captured, capture};
use cp_win::content::content_of;
use cp_win::formats::CATALOG;
use cp_win::paste::{Outcome, paste_into};
use cp_win::restore::{Restored, to_clipboard};
use cp_win::transfer::{self, Transfer};
use cp_win::virtual_files::{self, DESCRIPTOR};
use cp_win::watching::Watching;
use cp_win_sys::clipboard::{self, Clipboard};
use cp_win_sys::formats::{CF_DIB, CF_DIBV5, CF_HDROP, CF_UNICODETEXT, id_of, name_of};
use cp_win_sys::frontmost::{self, Target};
use cp_win_sys::permissions::Readiness;
use cp_win_sys::reading::{self, PATIENCE, Reading};
use cp_win_sys::window::EditWindow;
use cp_win_sys::writing::{Written, text_of, utf16_of};
use cp_win_sys::{files, media, ocr, source, thumbnail};

const CASES: u32 = 50;
const MAY_SKIP: &[&str] = &["B2", "B4", "E1", "E2", "L1", "P1", "P2", "Q1"];
const SKIPPED: &str = "skipped: ";

const VIRTUAL_FILE_NAME: &str = "virtual attachment.txt";
const VIRTUAL_FILE_BYTES: &[u8] = b"cp-e2 content";
const VIRTUAL_FILE_SCRIPT: &str = "Add-Type -AssemblyName System.Windows.Forms; \
$bytes = [System.Text.Encoding]::UTF8.GetBytes('cp-e2 content'); \
$fgd = New-Object byte[] 596; \
[BitConverter]::GetBytes([uint32]1).CopyTo($fgd, 0); \
[BitConverter]::GetBytes([uint32]0x40).CopyTo($fgd, 4); \
[BitConverter]::GetBytes([uint32]$bytes.Length).CopyTo($fgd, 72); \
[System.Text.Encoding]::Unicode.GetBytes('virtual attachment.txt').CopyTo($fgd, 76); \
$d = New-Object System.Windows.Forms.DataObject; \
$d.SetData('FileGroupDescriptorW', (New-Object System.IO.MemoryStream(,$fgd))); \
$d.SetData('FileContents', (New-Object System.IO.MemoryStream(,$bytes))); \
[System.Windows.Forms.Clipboard]::SetDataObject($d, $true); \
'placed'";

struct Battery {
    passed: u32,
    failed: u32,
    skipped: Vec<&'static str>,
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
                println!("    FAIL  {id:<5} {what}\n            {why}");
            }
        }
    }

    fn case_or_skip(
        &mut self,
        id: &'static str,
        what: &str,
        run: impl FnOnce() -> Result<(), String>,
    ) {
        match run() {
            Err(why) if why.starts_with(SKIPPED) => {
                self.skip(id, what, why.trim_start_matches(SKIPPED));
            }
            other => self.case(id, what, || other),
        }
    }

    fn skip(&mut self, id: &'static str, what: &str, why: &str) {
        if MAY_SKIP.contains(&id) {
            self.skipped.push(id);
            println!("    skip  {id:<5} {what}\n            {why}");
        } else {
            self.failed += 1;
            println!("    FAIL  {id:<5} {what}: this case cannot be skipped");
        }
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

fn shell_is_running() -> Result<(), String> {
    match powershell_within("(Get-Process -Name explorer -ErrorAction SilentlyContinue).Count") {
        Some(count) if !count.is_empty() && count != "0" => Ok(()),
        Some(_) => Err(format!(
            "{SKIPPED}there is no Explorer in this session: there is no desktop to open or reveal onto"
        )),
        None => Err(format!("{SKIPPED}PowerShell does not answer")),
    }
}

fn powershell_within(script: &str) -> Option<String> {
    let mut child = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .stdin(std::process::Stdio::null())
        .spawn()
        .ok()?;
    let started = std::time::Instant::now();
    while started.elapsed() < std::time::Duration::from_secs(20) {
        if child.try_wait().ok()?.is_some() {
            let out = child.wait_with_output().ok()?;
            if !out.status.success() {
                return None;
            }
            return Some(String::from_utf8_lossy(&out.stdout).trim().to_owned());
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    child.kill().ok();
    child.wait().ok();
    None
}

fn wait_until(what: &str, mut observed: impl FnMut() -> Option<bool>) -> Result<(), String> {
    for _ in 0..8 {
        std::thread::sleep(std::time::Duration::from_millis(300));
        match observed() {
            Some(true) => return Ok(()),
            Some(false) => {}
            None => return Err("PowerShell does not answer or failed to ask".into()),
        }
    }
    Err(format!("{what}: did not happen in time"))
}

fn offered_names() -> Result<Vec<String>, String> {
    let clipboard = Clipboard::open().ok_or("the clipboard could not be opened")?;
    Ok(clipboard.offered().into_iter().map(name_of).collect())
}

fn what_is_abandoned(b: &mut Battery) {
    b.group("G · What does not deliver in time is abandoned");

    b.case(
        "G1",
        "a format with data responds well under the ceiling",
        || {
            {
                let clipboard = Clipboard::to_write().ok_or("did not open")?;
                clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-g1"))]);
            }
            let started = std::time::Instant::now();
            let seen = reading::within(PATIENCE, || {
                let clipboard = Clipboard::open()?;
                clipboard.bytes(CF_UNICODETEXT)
            });
            let took = started.elapsed();
            if seen.is_too_slow() {
                return Err(format!("did not arrive within {PATIENCE:?}"));
            }
            println!("            {took:?} against a ceiling of {PATIENCE:?}");
            Ok(())
        },
    );

    b.case(
        "G2",
        "what does not answer is abandoned at the ceiling",
        || {
            let started = std::time::Instant::now();
            let seen = reading::within(PATIENCE, || {
                std::thread::sleep(std::time::Duration::from_secs(30));
                Some(Vec::new())
            });
            let took = started.elapsed();
            if seen != Reading::TooSlow {
                return Err(format!("it waited too long and gave {seen:?}"));
            }
            if took > PATIENCE * 3 {
                return Err(format!("took {took:?} to give up"));
            }
            println!("            abandoned at {took:?}, not at the measured 30 s");
            Ok(())
        },
    );

    b.case("G3", "the whole capture has its own ceiling", || {
        let started = std::time::Instant::now();
        let seen = capture::capture_within(std::time::Duration::from_nanos(1));
        let took = started.elapsed();
        if seen != Captured::TooSlow {
            return Err(format!("with an impossible ceiling it gave {seen:?}"));
        }
        if took > std::time::Duration::from_millis(500) {
            return Err(format!("took {took:?} to give up"));
        }
        let after = capture::capture_within(capture::PATIENCE);
        if after == Captured::TooSlow {
            return Err("and the normal ceiling is no longer enough for anything".into());
        }
        println!(
            "            abandoned at {took:?}; with {:?} it does capture",
            capture::PATIENCE
        );
        Ok(())
    });

    b.case(
        "G4",
        "insisting on a source that responds costs no second attempt",
        || {
            {
                let clipboard = Clipboard::to_write().ok_or("did not open")?;
                clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-g4"))]);
            }
            let direct = {
                let clipboard = Clipboard::open().ok_or("did not open")?;
                capture(&clipboard).kept().ok_or("nothing was captured")?
            };
            let started = std::time::Instant::now();
            let got = capture::capture_insisting(capture::PATIENCE, cp_core::watch::RETRY);
            let took = started.elapsed();
            match got {
                Captured::Kept(item) if item == direct => {
                    if took < capture::PATIENCE {
                        Ok(())
                    } else {
                        Err(format!("took {took:?}: there was a pause for no reason"))
                    }
                }
                other => Err(format!("got {other:?}")),
            }
        },
    );

    b.case(
        "G5",
        "a write waits for a read that was given up instead of freeing under it",
        || {
            let big = cp_win_sys::writing::utf16_of(&"abcdefghij".repeat(20_000));
            let mut refused = 0;
            let mut placed = 0;
            let mut quickest = std::time::Duration::MAX;
            for _ in 0..40 {
                let counted = cp_win_sys::clipboard::reading();
                let pending = cp_core::reading::begin(move || {
                    let _held = counted;
                    std::thread::sleep(std::time::Duration::from_millis(30));
                });
                let asked = std::time::Instant::now();
                match Clipboard::to_write() {
                    None => refused += 1,
                    Some(clipboard) => {
                        let took = asked.elapsed();
                        if took < std::time::Duration::from_millis(25) {
                            return Err(format!(
                                "the write took the clipboard after {took:?}, before the read let go"
                            ));
                        }
                        quickest = quickest.min(took);
                        if clipboard.replace(&[(CF_UNICODETEXT, &big)])
                            != (cp_win_sys::writing::Written::Placed { formats: 1 })
                        {
                            return Err("the write was let through and then failed".into());
                        }
                        placed += 1;
                        let back = clipboard
                            .bytes(CF_UNICODETEXT)
                            .ok_or("what was just written does not read back")?;
                        if !back.starts_with(&big) {
                            return Err(format!(
                                "the bytes came back changed: {} of {} match",
                                back.iter().zip(&big).take_while(|(a, b)| a == b).count(),
                                big.len()
                            ));
                        }
                    }
                }
                let _ = pending.wait(std::time::Duration::from_secs(2));
            }
            if placed == 0 {
                return Err("every write was refused, so nothing was proven".into());
            }
            println!(
                "            {placed} writes waited, the quickest {quickest:?}, {refused} gave up"
            );
            Ok(())
        },
    );
}

fn what_the_clipboard_answers(b: &mut Battery) {
    b.group("A · The clipboard responds");

    b.case("A1", "it opens and closes without staying locked", || {
        {
            let _first = Clipboard::open().ok_or("did not open")?;
        }
        let _second = Clipboard::open().ok_or("did not open the second time")?;
        Ok(())
    });

    b.case("A2", "the sequence counter can be read", || {
        clipboard::sequence()
            .map(|_| ())
            .ok_or_else(|| "returned zero: the window station cannot be reached".into())
    });

    b.case("A3", "enumerating does not ask for a single byte", || {
        let names = offered_names()?;
        println!("            {} formats: {}", names.len(), names.join(", "));
        Ok(())
    });
}

fn what_is_read_inside_an_image(b: &mut Battery) {
    b.group("M · Text inside an image");

    b.case("M1", "the system offers a reading engine", || {
        if ocr::is_available() {
            Ok(())
        } else {
            Err("there is no engine for the profile's languages".into())
        }
    });

    b.case("M2", "the text of a real image is read", || {
        let png = std::fs::read("fixtures/texto-en-imagen.png")
            .map_err(|why| format!("could not read the fixture: {why}"))?;
        let started = std::time::Instant::now();
        let text = (0..3)
            .find_map(|_| ocr::text_in(&png))
            .ok_or("nothing was recognised in three tries")?;
        println!(
            "            {:?} to read «{}»",
            started.elapsed(),
            text.lines().next().unwrap_or("").trim()
        );
        Ok(())
    });

    b.case("M3", "a blank image does not invent text", || {
        let blank = image::RgbaImage::from_pixel(120, 60, image::Rgba([255, 255, 255, 255]));
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(blank)
            .write_to(&mut png, image::ImageFormat::Png)
            .map_err(|why| why.to_string())?;
        match ocr::text_in(&png.into_inner()) {
            None => Ok(()),
            Some(invented) => Err(format!("invented «{invented}»")),
        }
    });
}

fn what_the_shell_knows_about_a_document(b: &mut Battery) {
    b.group("Q · Lo que el shell sabe de un documento");

    b.case_or_skip(
        "Q1",
        "un archivo de oficina publica sus propias medidas",
        || {
            let Some(said) = std::env::var_os("CP_PROBE_DOC") else {
                return Err(format!(
                    "{SKIPPED}pon CP_PROBE_DOC con la ruta de un .xlsx, .docx o .pptx real"
                ));
            };
            let path = std::path::PathBuf::from(said);
            if !path.exists() {
                return Err(format!("{SKIPPED}{} no está ahí", path.display()));
            }
            let Some(found) = cp_win_sys::media::everything_about(&path) else {
                return Err("no se pudo abrir la tienda de propiedades de ese archivo".into());
            };
            if found.how_many == 0 {
                return Err("el shell abre el archivo y no publica ni una propiedad".into());
            }
            println!(
                "            {} publicadas, {} legibles como texto:",
                found.how_many,
                found.said.len()
            );
            for (name, value) in &found.said {
                let cut: String = value.chars().take(70).collect();
                println!("              {name} = {cut}");
            }
            let counted: Vec<&str> = found
                .said
                .iter()
                .map(|(name, _)| name.as_str())
                .filter(|name| {
                    name.contains("PageCount")
                        || name.contains("SlideCount")
                        || name.contains("WordCount")
                        || name.contains("LineCount")
                        || name.contains("ParagraphCount")
                })
                .collect();
            if counted.is_empty() {
                return Err(
                    "ninguna propiedad cuenta páginas, palabras ni diapositivas: la vista de \
                     oficina tendrá que leer el zip"
                        .into(),
                );
            }
            println!("            cuentan: {}", counted.join(", "));
            Ok(())
        },
    );
}

fn what_hangs_is_never_asked_for(b: &mut Battery) {
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
}

fn main() -> std::process::ExitCode {
    println!("\nCore battery against the Windows clipboard");

    let mut b = Battery {
        passed: 0,
        failed: 0,
        skipped: Vec::new(),
    };

    what_the_clipboard_answers(&mut b);

    b.group("B · The catalog against what is really there");

    b.case("B1", "everything offered gets a decision", || {
        let names = offered_names()?;
        if names.is_empty() {
            return Err("the clipboard is empty: copy something and try again".into());
        }
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        let family = CATALOG.classify(&refs);
        for name in &names {
            let take = CATALOG.decide_in(family, name);
            let mark = match take {
                Take::Payload => "copy",
                Take::Presence => "note",
                Take::Never => "never",
            };
            println!("            {mark}  {name}");
        }
        Ok(())
    });

    b.case_or_skip("B2", "what is copied can really be read", || {
        let clipboard = Clipboard::open().ok_or("did not open")?;
        let ids = clipboard.offered();
        let names: Vec<String> = ids.iter().map(|id| name_of(*id)).collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        if let Some(refusal) = CATALOG.refusal(&refs) {
            return Err(format!(
                "{SKIPPED}the source asked not to record this ({refusal:?}): there is nothing to read"
            ));
        }
        let family = CATALOG.classify(&refs);
        let mut read = 0usize;
        let mut bytes = 0usize;
        for (id, name) in ids.iter().zip(&names) {
            if CATALOG.decide_in(family, name) != Take::Payload
                || CATALOG.costlier_twin(name, &refs)
            {
                continue;
            }
            match clipboard.size_of(*id) {
                Some(size) => {
                    read += 1;
                    bytes += size;
                    println!("            {size:>9} B  {name}");
                }
                None => println!("            {:>9}  {name}", "no data"),
            }
        }
        if read == 0 {
            return Err("nothing the catalog wants delivered bytes".into());
        }
        println!("            {read} formats, {bytes} bytes");
        Ok(())
    });

    b.case("B3", "the class that comes out is one of the three", || {
        let names = offered_names()?;
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        match CATALOG.classify(&refs) {
            Some(family) => {
                println!("            {family:?}");
                Ok(())
            }
            None => Err(format!("no class for {names:?}")),
        }
    });

    let names = offered_names().unwrap_or_default();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let courtesy = refs.iter().any(|id| CATALOG.embeddable.contains(id))
        && CATALOG.preferred_image(&refs).is_some();
    if courtesy {
        b.case(
            "B4",
            "a spreadsheet is not stored as a photo",
            || match CATALOG.classify(&refs) {
                Some(Family::Text) => Ok(()),
                other => Err(format!("classified as {other:?}")),
            },
        );
    } else {
        b.skip(
            "B4",
            "a spreadsheet is not stored as a photo",
            "what was copied is not a document with an image: copy a range from Excel and try again",
        );
    }

    b.group("F · Round trip, staged by us");

    b.case("F1", "what we write reads back the same", || {
        let written = "cp-f1-round-trip ñ 🦀";
        {
            let clipboard = Clipboard::to_write().ok_or("did not open for writing")?;
            match clipboard.replace(&[(CF_UNICODETEXT, &utf16_of(written))]) {
                Written::Placed { formats: 1 } => {}
                other => return Err(format!("the write gave {other:?}")),
            }
        }
        let clipboard = Clipboard::open().ok_or("did not open for reading")?;
        let bytes = clipboard.bytes(CF_UNICODETEXT).ok_or("returned no bytes")?;
        match text_of(&bytes).as_deref() {
            Some(back) if back == written => Ok(()),
            other => Err(format!("came back «{other:?}»")),
        }
    });

    b.case(
        "F2",
        "writing moves the counter and reading does not",
        || {
            let before = clipboard::sequence().ok_or("no counter")?;
            {
                let clipboard = Clipboard::to_write().ok_or("did not open")?;
                clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-f2"))]);
            }
            let after = clipboard::sequence().ok_or("no counter")?;
            if after == before {
                return Err("the counter did not move when writing".into());
            }
            let quiet = {
                let clipboard = Clipboard::open().ok_or("did not open")?;
                let _ = clipboard.bytes(CF_UNICODETEXT);
                clipboard::sequence().ok_or("no counter")?
            };
            if quiet != after {
                return Err(format!("reading moved the counter from {after} to {quiet}"));
            }
            println!("            {before} → {after} from one write");
            Ok(())
        },
    );

    b.case("F3", "the watcher sees our write as ours", || {
        let mut watcher = Watcher::new(Cadence::Opaque);
        watcher.tick(clipboard::sequence().ok_or("no counter")?);
        {
            let clipboard = Clipboard::to_write().ok_or("did not open")?;
            clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-f3"))]);
        }
        let ours = clipboard::sequence().ok_or("no counter")?;
        watcher.wrote(ours);
        match watcher.tick(ours) {
            Seen::Ours => Ok(()),
            other => Err(format!("seen as {other:?}")),
        }
    });

    b.case("F4", "a foreign copy after ours is not swallowed", || {
        let mut watcher = Watcher::new(Cadence::Opaque);
        watcher.tick(clipboard::sequence().ok_or("no counter")?);
        {
            let clipboard = Clipboard::to_write().ok_or("did not open")?;
            clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-f4-ours"))]);
        }
        let ours = clipboard::sequence().ok_or("no counter")?;
        watcher.wrote(ours);
        if watcher.tick(ours) != Seen::Ours {
            return Err("ours was not recognised".into());
        }
        {
            let clipboard = Clipboard::to_write().ok_or("did not open")?;
            clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-f4-foreign"))]);
        }
        let theirs = clipboard::sequence().ok_or("no counter")?;
        match watcher.tick(theirs) {
            Seen::Fresh { .. } => Ok(()),
            other => Err(format!("the next copy was seen as {other:?}")),
        }
    });

    what_is_abandoned(&mut b);

    b.group("H · The capture, end to end");

    b.case("H1", "a copied text becomes an item", || {
        {
            let clipboard = Clipboard::to_write().ok_or("did not open")?;
            clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("someone@example.test"))]);
        }
        let clipboard = Clipboard::open().ok_or("did not open")?;
        match capture(&clipboard) {
            Captured::Kept(item) => {
                if item.kind != Some(cp_core::kind::Kind::Email) {
                    return Err(format!("the class came out {:?}", item.kind));
                }
                println!(
                    "            {} formats, {} bytes stored, class {:?}",
                    item.formats.len(),
                    item.stored_bytes(),
                    item.kind
                );
                Ok(())
            }
            other => Err(format!("nothing was captured: {other:?}")),
        }
    });

    b.case(
        "H2",
        "the whole set is noted, not just what is copied",
        || {
            let clipboard = Clipboard::open().ok_or("did not open")?;
            let offered = clipboard.offered().len();
            match capture(&clipboard) {
                Captured::Kept(item) => {
                    if item.formats.len() < offered {
                        return Err(format!(
                            "{offered} were offered and only {} were noted",
                            item.formats.len()
                        ));
                    }
                    Ok(())
                }
                other => Err(format!("nothing was captured: {other:?}")),
            }
        },
    );

    b.case(
        "H3",
        "two identical copies have the same fingerprint",
        || {
            let write = |text: &str| {
                let clipboard = Clipboard::to_write()?;
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
            write("cp-h3-same").ok_or("did not write")?;
            let first = taken(()).ok_or("did not capture")?;
            write("cp-h3-same").ok_or("did not write")?;
            let again = taken(()).ok_or("did not capture")?;
            write("cp-h3-different").ok_or("did not write")?;
            let other = taken(()).ok_or("did not capture")?;
            if first != again {
                return Err("the same thing gave two fingerprints".into());
            }
            if first == other {
                return Err("two different contents gave the same fingerprint".into());
            }
            Ok(())
        },
    );

    b.case("H4", "a secrecy marker stops the capture", || {
        let marker = cp_win_sys::formats::name_of(
            cp_win_sys::clipboard::register("Clipboard Viewer Ignore").ok_or("did not register")?,
        );
        if marker != "Clipboard Viewer Ignore" {
            return Err(format!("the format registered as «{marker}»"));
        }
        {
            let clipboard = Clipboard::to_write().ok_or("did not open")?;
            let id = cp_win_sys::clipboard::register("Clipboard Viewer Ignore")
                .ok_or("did not register")?;
            clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("a-password")), (id, &[1u8])]);
        }
        let seen = {
            let clipboard = Clipboard::open().ok_or("did not open")?;
            capture(&clipboard)
        };
        {
            let clipboard = Clipboard::to_write().ok_or("did not open to clean up")?;
            clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-h4-clean"))]);
        }
        match seen {
            Captured::Refused(why) => {
                println!("            refused for {why:?}, and the marker was removed");
                Ok(())
            }
            other => Err(format!("it was captured anyway: {other:?}")),
        }
    });

    b.case("H5", "the battery leaves no markers behind", || {
        let clipboard = Clipboard::open().ok_or("did not open")?;
        let names: Vec<String> = clipboard.offered().into_iter().map(name_of).collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        match CATALOG.refusal(&refs) {
            None => Ok(()),
            Some(left) => Err(format!("{left:?} was left over from the previous case")),
        }
    });

    what_goes_back_to_the_clipboard(&mut b);

    b.case(
        "I6",
        "the clipboard counter moves while a write is still open, and again when it closes",
        || {
            {
                let clipboard = Clipboard::to_write().ok_or("did not open")?;
                clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-i6-before"))]);
            }
            let settled = clipboard::sequence().ok_or("no counter")?;
            let (inside, outside) = {
                let clipboard = Clipboard::to_write().ok_or("did not open")?;
                clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-i6-after"))]);
                let inside = clipboard::sequence().ok_or("no counter")?;
                drop(clipboard);
                (inside, clipboard::sequence().ok_or("no counter")?)
            };
            println!("            before {settled}, still open {inside}, closed {outside}");
            if settled == inside {
                return Err(format!(
                    "the counter stood still at {settled} during the write: marking at close would then be enough, and this case exists because it is not"
                ));
            }
            if inside == outside {
                return Err(format!(
                    "the counter stood still at {inside} on closing, so the write ended where the watcher could already see it"
                ));
            }
            Ok(())
        },
    );

    b.group("J · The watcher in action");

    b.case("J1", "a copy wakes the watcher", || {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        let seen = Arc::new(AtomicUsize::new(0));
        let counter = seen.clone();
        let watching = Watching::every(std::time::Duration::from_millis(10), move || {
            counter.fetch_add(1, Ordering::Relaxed);
        });
        std::thread::sleep(std::time::Duration::from_millis(60));
        {
            let clipboard = Clipboard::to_write().ok_or("did not open")?;
            clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-j1"))]);
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
        drop(watching);
        match seen.load(Ordering::Relaxed) {
            0 => Err("the copy was not seen".into()),
            n => {
                println!("            {n} notice(s) for one copy");
                Ok(())
            }
        }
    });

    b.case("J2", "a quiet clipboard wakes nobody", || {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        {
            let clipboard = Clipboard::to_write().ok_or("did not open")?;
            clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-j2-quiet"))]);
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
            n => Err(format!("{n} notices without anyone copying")),
        }
    });

    b.case("J4", "what we restore is not captured back", || {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        let seen = Arc::new(AtomicUsize::new(0));
        let counter = seen.clone();
        let watching = Watching::every(std::time::Duration::from_millis(10), move || {
            counter.fetch_add(1, Ordering::Relaxed);
        });
        std::thread::sleep(std::time::Duration::from_millis(60));

        {
            let clipboard = Clipboard::to_write().ok_or("did not open")?;
            clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-j4-ours"))]);
        }
        watching.ours();
        std::thread::sleep(std::time::Duration::from_millis(200));
        let after_ours = seen.load(Ordering::Relaxed);

        {
            let clipboard = Clipboard::to_write().ok_or("did not open")?;
            clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-j4-foreign"))]);
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
        let after_theirs = seen.load(Ordering::Relaxed);
        drop(watching);

        if after_ours != 0 {
            return Err(format!("ours woke the watcher {after_ours} time(s)"));
        }
        if after_theirs == 0 {
            return Err(
                "and then it would not see foreign copies either: the test would prove nothing"
                    .into(),
            );
        }
        println!("            ours 0 notices, foreign {after_theirs}");
        Ok(())
    });

    b.case("J3", "polling the counter is almost free", || {
        let rounds = 10_000;
        let started = std::time::Instant::now();
        for _ in 0..rounds {
            let _ = clipboard::sequence();
        }
        let each = started.elapsed() / rounds;
        println!("            {each:?} per poll");
        if each > std::time::Duration::from_micros(50) {
            return Err(format!("{each:?} is too much to poll continuously"));
        }
        Ok(())
    });

    b.group("K · Permissions");

    b.case("K1", "it is known what can be done and what cannot", || {
        let ready = Readiness::probe();
        println!(
            "            station: {}  level: {:?}  elevated: {}",
            ready.can_watch(),
            ready.integrity,
            ready.is_elevated()
        );
        if !ready.can_watch() {
            return Err("the window station cannot be reached".into());
        }
        let ours = ready.integrity.ok_or("no level of our own")?;
        if !ready.can_paste_into(ours) {
            return Err("cannot paste at our own level".into());
        }
        Ok(())
    });

    b.group("L · Pasting for real");

    let stage = EditWindow::open("battery target").and_then(|target| {
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
        Some(target) => b.case("L1", "the text reaches a target window", || {
            let written = "cp-l1-real-paste";
            {
                let clipboard = Clipboard::to_write().ok_or("did not open")?;
                clipboard.replace(&[(CF_UNICODETEXT, &utf16_of(written))]);
            }
            let seen = frontmost::target_for(target.window());
            match paste_into(&seen, || {}) {
                Outcome::Sent { took } => println!("            sent in {took:?}"),
                Outcome::Degraded(why) => return Err(format!("degraded to {why:?}")),
            }
            target.pump(std::time::Duration::from_millis(400));
            let arrived = target.text();
            if arrived.contains(written) {
                Ok(())
            } else {
                Err(format!("got «{arrived}» instead of «{written}»"))
            }
        }),
        None => b.skip(
            "L1",
            "the text reaches a target window",
            "Windows only lets whoever already has it change the foreground: run the battery from a console with focus",
        ),
    }

    b.case(
        "L2",
        "the worst outcome is still pasting it by hand",
        || {
            let written = "cp-l2-degraded";
            {
                let clipboard = Clipboard::to_write().ok_or("did not open")?;
                clipboard.replace(&[(CF_UNICODETEXT, &utf16_of(written))]);
            }
            let gone = Target {
                window: windows::Win32::Foundation::HWND(std::ptr::dangling_mut()),
                focus: None,
                thread: 0,
            };
            match paste_into(&gone, || {}) {
                Outcome::Degraded(cp_core::paste::Failure::TargetGone) => {}
                other => return Err(format!("with a dead target it gave {other:?}")),
            }
            let clipboard = Clipboard::open().ok_or("did not open")?;
            let bytes = clipboard.bytes(CF_UNICODETEXT).ok_or("no text")?;
            match text_of(&bytes).as_deref() {
                Some(back) if back == written => Ok(()),
                other => Err(format!("the clipboard was left with «{other:?}»")),
            }
        },
    );

    what_is_read_inside_an_image(&mut b);

    b.group("N · Thumbnails and media via the shell");

    b.case(
        "N1",
        "the shell gives a thumbnail for an image on disk",
        || {
            let png = std::path::Path::new("fixtures/texto-en-imagen.png");
            let started = std::time::Instant::now();
            let dib = thumbnail::dib_of_file(png, thumbnail::SIDE)
                .ok_or("the shell returned no thumbnail")?;
            let took = started.elapsed();
            let small = cp_core::dib::to_png(&dib).ok_or("the DIB could not be converted")?;
            let original = std::fs::metadata(png).map_err(|why| why.to_string())?.len();
            println!(
                "            {took:?}, {} B of thumbnail against {original} of the original",
                small.len()
            );
            if small.len() as u64 >= original {
                return Err("the thumbnail is not smaller than the original".into());
            }
            Ok(())
        },
    );

    b.case("N2", "what has no thumbnail does not invent one", || {
        let dir = std::env::temp_dir().join("cp-no-thumbnail");
        std::fs::create_dir_all(&dir).map_err(|why| why.to_string())?;
        let path = dir.join("empty.bin");
        std::fs::write(&path, b"   ").map_err(|why| why.to_string())?;
        match thumbnail::dib_of_file(&path, thumbnail::SIDE) {
            None => Ok(()),
            Some(_) => Err("returned something for a file with no preview".into()),
        }
    });

    b.case(
        "N3",
        "a file with no media metadata does not invent any",
        || {
            let dir = std::env::temp_dir().join("cp-no-media");
            std::fs::create_dir_all(&dir).map_err(|why| why.to_string())?;
            let path = dir.join("note.txt");
            std::fs::write(&path, b"just text").map_err(|why| why.to_string())?;
            match media::info_for(&path) {
                None => Ok(()),
                Some(info) => Err(format!("invented {info:?}")),
            }
        },
    );

    b.case(
        "N4",
        "the thumbnail goes to the store and comes back",
        || {
            let dir = std::env::temp_dir().join(format!("cp-probe-thumbs-{}", std::process::id()));
            let blobs = cp_store::Blobs::at(&dir).map_err(|why| why.to_string())?;
            let png = std::fs::read("fixtures/texto-en-imagen.png")
                .map_err(|why| format!("missing fixture: {why}"))?;
            let thumb = cp_core::thumbnail::of_image(&png, cp_core::thumbnail::MAX_SIDE)
                .ok_or("it was not generated")?;
            let digest = blobs.put(&thumb).map_err(|why| why.to_string())?;
            let back = blobs
                .get(&digest)
                .map_err(|why| why.to_string())?
                .ok_or("did not come back")?;
            std::fs::remove_dir_all(&dir).ok();
            if back != thumb {
                return Err("the thumbnail came back different".into());
            }
            Ok(())
        },
    );

    b.group("O · Origin");

    b.case(
        "O1",
        "the source application is stored with its visible name",
        || {
            let pid = frontmost::foreground()
                .and_then(source::process_of)
                .unwrap_or_else(std::process::id);
            let name = source::name_of(pid).ok_or("no visible name")?;
            if name.is_empty() {
                return Err("the name came back empty".into());
            }
            if name.contains('\\') || name.to_ascii_lowercase().ends_with(".exe") {
                return Err(format!("«{name}» is the path, not the name"));
            }

            let store = cp_store::Store::in_memory().map_err(|why| why.to_string())?;
            let id = store
                .insert_text("uuid-origin", "something copied", 1)
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
                    "searching for «{name}» returned {} items",
                    found.len()
                ));
            }
            println!("            origin «{name}», stored and found");
            Ok(())
        },
    );

    b.group("P · Open and reveal");

    b.case_or_skip(
        "P1",
        "revealing a file leaves it selected in Explorer",
        || {
            shell_is_running()?;
            let scratch = Scratch::new("cp-p1")?;
            let file = scratch.file("revealed.txt", b"cp-p1")?;
            if !files::reveal(&file) {
                return Err("reveal said no".into());
            }
            let mut window_seen = false;
            let mut selection = String::new();
            let shown = wait_until("Explorer leaves the file selected", || {
                let listing = powershell_within(
                    "(New-Object -ComObject Shell.Application).Windows() | ForEach-Object { $_.LocationURL + ' | ' + (($_.Document.SelectedItems() | ForEach-Object { $_.Path }) -join ',') }",
                )?;
                let ours = listing.lines().find(|line| line.contains("cp-p1"));
                window_seen |= ours.is_some();
                selection = ours
                    .and_then(|line| line.split_once(" | "))
                    .map(|(_, selected)| selected.trim().to_owned())
                    .unwrap_or_default();
                Some(selection.contains("revealed.txt"))
            });
            powershell_within(
                "(New-Object -ComObject Shell.Application).Windows() | Where-Object { $_.LocationURL -like '*cp-p1*' } | ForEach-Object { $_.Quit() }",
            );
            if shown.is_err() && !window_seen {
                return Err(format!(
                    "{SKIPPED}Explorer did not open any window in this session: no interactive desktop"
                ));
            }
            if shown.is_err() && selection.is_empty() {
                return Err(format!(
                    "{SKIPPED}the folder opened, but the view does not report a selection in this session"
                ));
            }
            shown.map_err(|why| format!("{why}; selected: «{selection}»"))
        },
    );

    b.case_or_skip("P2", "opening a file hands it to its application", || {
        shell_is_running()?;
        let scratch = Scratch::new("cp-p2")?;
        let file = scratch.file("opened.txt", b"cp-p2")?;
        if !files::open(&file) {
            return Err("open said no".into());
        }
        let opened = wait_until("the application has the document open", || {
            powershell_within(
                "Get-Process | Where-Object { $_.MainWindowTitle -like '*opened.txt*' } | ForEach-Object { $_.ProcessName }",
            )
            .map(|names| !names.is_empty())
        });
        powershell_within(
            "Get-Process | Where-Object { $_.MainWindowTitle -like '*opened.txt*' } | ForEach-Object { $_.CloseMainWindow() | Out-Null }",
        );
        opened
    });

    what_the_shell_knows_about_a_document(&mut b);

    what_hangs_is_never_asked_for(&mut b);

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

    b.case_or_skip(
        "E2",
        "un archivo virtual se captura con sus bytes y se pega como archivo",
        || {
            match powershell_within(VIRTUAL_FILE_SCRIPT).as_deref() {
                Some("placed") => {}
                other => {
                    return Err(format!(
                        "{SKIPPED}PowerShell no montó el DataObject: {other:?}"
                    ));
                }
            }
            let item = capture::capture_now().kept().ok_or("no se capturó")?;
            if item.kind != Some(cp_core::kind::Kind::File) {
                return Err(format!("la clase salió {:?}", item.kind));
            }
            let described = item
                .format(DESCRIPTOR)
                .and_then(|one| match &one.payload {
                    cp_core::item::Payload::Inline(bytes) => Some(bytes.as_slice()),
                    _ => None,
                })
                .map(virtual_files::described_in)
                .ok_or("el descriptor no se guardó")?;
            if described.len() != 1 || described[0].name != VIRTUAL_FILE_NAME {
                return Err(format!("el descriptor dice {described:?}"));
            }
            match item
                .format(&virtual_files::contents_id(0))
                .map(|one| &one.payload)
            {
                Some(cp_core::item::Payload::Inline(bytes)) if bytes == VIRTUAL_FILE_BYTES => {}
                other => return Err(format!("el contenido llegó como {other:?}")),
            }
            println!(
                "            {} formatos, {} bytes, clase {:?}",
                item.formats.len(),
                item.stored_bytes(),
                item.kind
            );

            let written = {
                let clipboard = Clipboard::to_write().ok_or("no abrió")?;
                to_clipboard(&clipboard, &item)
            };
            match written {
                Restored::Written { formats: 2, .. } => {}
                other => {
                    return Err(format!(
                        "al pegar dio {other:?}, y no CF_HDROP más el efecto"
                    ));
                }
            }
            let paths = {
                let clipboard = Clipboard::open().ok_or("no abrió")?;
                cp_win::drop::paths_in(&clipboard.bytes(CF_HDROP).ok_or("sin CF_HDROP")?)
            };
            let pasted = match paths.as_slice() {
                [one] if one.ends_with(VIRTUAL_FILE_NAME) => std::path::PathBuf::from(one),
                other => return Err(format!("el drop trae {other:?}")),
            };
            let on_disk = std::fs::read(&pasted).map_err(|why| why.to_string())?;
            if let Some(folder) = pasted.parent() {
                std::fs::remove_dir_all(folder).ok();
            }
            if on_disk != VIRTUAL_FILE_BYTES {
                return Err("lo que llegó al disco no es lo que se copió".into());
            }
            println!("            pegado en {}", pasted.display());
            Ok(())
        },
    );

    let ran = b.passed + b.failed + u32::try_from(b.skipped.len()).unwrap_or(u32::MAX);
    if ran != CASES {
        b.failed += 1;
        println!("    FALLA       corrieron {ran} casos de {CASES}: la bateria se desactivo sola");
    }
    println!(
        "\n  {} ok, {} fallan, {} saltadas\n",
        b.passed,
        b.failed,
        format_args!("{} ({})", b.skipped.len(), b.skipped.join(", "))
    );
    if b.failed > 0 {
        std::process::ExitCode::FAILURE
    } else {
        std::process::ExitCode::SUCCESS
    }
}
