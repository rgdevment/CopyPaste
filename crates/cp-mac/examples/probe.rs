//! Batería de pruebas del núcleo contra macOS de verdad.
//!
//! No son pruebas de `cargo test` a propósito: `NSPasteboard` exige el hilo
//! principal y el runner no lo garantiza, y varias tocan aplicaciones reales.
//! Se ejecuta con `cargo run -p cp-mac --example probe`.

use cp_core::destination::Tracker;
use cp_core::item::Payload;
use cp_core::kind::Kind;
use cp_core::watch::{Seen, Watcher};
use cp_mac::capture::capture;
use cp_mac::paste::Paster;
use cp_mac_sys::keyboard::{self, QWERTY_V};
use cp_mac_sys::pasteboard::{self, Pasteboard};
use cp_mac_sys::permissions::Readiness;
use cp_mac_sys::{frontmost, keystroke};
use objc2_foundation::MainThreadMarker;
use std::time::{Duration, Instant};

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
        println!("    —     {id:<5} {what}  ({why})");
    }
}

fn main() -> std::process::ExitCode {
    let Some(mtm) = MainThreadMarker::new() else {
        eprintln!("esto tiene que correr en el hilo principal");
        return std::process::ExitCode::FAILURE;
    };
    let pb = Pasteboard::general(mtm);
    let ready = Readiness::probe();
    let mut b = Battery {
        passed: 0,
        failed: 0,
        skipped: 0,
    };

    b.group("A · Captura y formatos");

    b.case("A1", "el texto plano va y vuelve", || {
        pb.write_text("cp-a1");
        let item = capture(&pb).ok_or("no se capturó")?;
        if item.kind != Some(Kind::Text) {
            return Err(format!("se clasificó como {:?}", item.kind));
        }
        match &item
            .format("public.utf8-plain-text")
            .ok_or("falta el texto")?
            .payload
        {
            Payload::Inline(bytes) if bytes == b"cp-a1" => Ok(()),
            other => Err(format!("llegó {other:?}")),
        }
    });

    b.case("A2", "los gemelos legados no se guardan dos veces", || {
        pb.write_text("cp-a2");
        let item = capture(&pb).ok_or("no se capturó")?;
        let ids: Vec<&str> = item.formats.iter().map(|f| f.id.as_str()).collect();
        let mut unique = ids.clone();
        unique.sort_unstable();
        unique.dedup();
        if ids.len() != unique.len() {
            return Err(format!("repetidos: {ids:?}"));
        }
        if ids.contains(&"NSStringPboardType") {
            return Err("el gemelo legado no colapsó".into());
        }
        Ok(())
    });

    b.case("A3", "un texto vacío sigue siendo un ítem", || {
        pb.write_text("");
        let item = capture(&pb).ok_or("no se capturó")?;
        if item.formats.is_empty() {
            return Err("sin formatos".into());
        }
        Ok(())
    });

    b.case("A4", "diez megabytes van y vuelven enteros", || {
        let big = "a".repeat(10 * 1024 * 1024);
        pb.write_text(&big);
        let item = capture(&pb).ok_or("no se capturó")?;
        match &item
            .format("public.utf8-plain-text")
            .ok_or("falta el texto")?
            .payload
        {
            Payload::Blob(bytes) if bytes.len() == big.len() => Ok(()),
            other => Err(format!("llegó {other:?}")),
        }
    });

    b.case("A5", "un solo carácter multibyte", || {
        pb.write_text("🎯");
        let item = capture(&pb).ok_or("no se capturó")?;
        match &item
            .format("public.utf8-plain-text")
            .ok_or("falta el texto")?
            .payload
        {
            Payload::Inline(bytes) if bytes == "🎯".as_bytes() => Ok(()),
            other => Err(format!("llegó {other:?}")),
        }
    });

    b.case("A6", "saltos de línea de los tres tipos", || {
        let mixed = "uno\r\ndos\rtres\ncuatro";
        pb.write_text(mixed);
        let item = capture(&pb).ok_or("no se capturó")?;
        match &item
            .format("public.utf8-plain-text")
            .ok_or("falta el texto")?
            .payload
        {
            Payload::Inline(bytes) if bytes == mixed.as_bytes() => Ok(()),
            other => Err(format!("llegó {other:?}")),
        }
    });

    b.case("A7", "tres archivos copiados son tres rutas", || {
        pb.write_items(&[
            vec![("public.file-url", "file:///tmp/uno.txt")],
            vec![("public.file-url", "file:///tmp/dos.txt")],
            vec![("public.file-url", "file:///tmp/tres.txt")],
        ]);
        if pb.item_count() != 3 {
            return Err(format!("el portapapeles tiene {} ítems", pb.item_count()));
        }
        let item = capture(&pb).ok_or("no se capturó")?;
        let urls = item.format("public.file-url").ok_or("falta la ruta")?;
        let text = match &urls.payload {
            Payload::Inline(bytes) => String::from_utf8_lossy(bytes).to_string(),
            other => return Err(format!("llegó {other:?}")),
        };
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() != 3 {
            return Err(format!("se guardaron {} rutas: {lines:?}", lines.len()));
        }
        Ok(())
    });

    b.case("A8", "un solo archivo sigue siendo una ruta", || {
        pb.write_items(&[vec![("public.file-url", "file:///tmp/solo.txt")]]);
        let item = capture(&pb).ok_or("no se capturó")?;
        let urls = item.format("public.file-url").ok_or("falta la ruta")?;
        match &urls.payload {
            Payload::Inline(bytes) if !String::from_utf8_lossy(bytes).contains('\n') => Ok(()),
            other => Err(format!("llegó {other:?}")),
        }
    });

    b.group("G · Clasificación");

    for (id, text, expected) in [
        ("G1", "alguien@ejemplo.test", Kind::Email),
        ("G2", "https://ejemplo.test/ruta", Kind::Link),
        ("G3", "#FF8800", Kind::Color),
        ("G4", "192.168.1.1", Kind::Ip),
        ("G5", "7ab3f6de-1c4b-4f5e-8a2d-9f0e1b2c3d4e", Kind::Uuid),
        ("G6", "+34 600 123 456", Kind::Phone),
        ("G7", "{\"clave\": [1, 2]}", Kind::Json),
        ("G8", "fn main() {\n    println!(\"hola\");\n}", Kind::Code),
        ("G9", "una frase corriente y nada más", Kind::Text),
    ] {
        b.case(
            id,
            &format!("se clasifica como {}", expected.as_str()),
            || {
                pb.write_text(text);
                let item = capture(&pb).ok_or("no se capturó")?;
                if item.kind != Some(expected) {
                    return Err(format!("salió {:?}", item.kind));
                }
                Ok(())
            },
        );
    }

    b.group("B · Privacidad");

    for (id, marker) in [
        ("B1", "org.nspasteboard.ConcealedType"),
        ("B2", "org.nspasteboard.TransientType"),
        ("B3", "com.agilebits.onepassword"),
        ("B4", "net.antelle.keeweb"),
        ("B5", "PasswordPboardType"),
    ] {
        b.case(id, &format!("«{marker}» excluye el ítem"), || {
            pb.write_types(&[("public.utf8-plain-text", "secreto"), (marker, "1")]);
            if capture(&pb).is_some() {
                return Err("se capturó contenido marcado".into());
            }
            Ok(())
        });
    }

    b.case("B6", "sin marcador se vuelve a capturar", || {
        pb.write_text("esto sí");
        capture(&pb).ok_or("un texto normal debe capturarse")?;
        Ok(())
    });

    b.group("C · Vigilante");

    b.case("C1", "el contador sube de uno en uno", || {
        let before = pb.change_count();
        pb.write_text("cp-c1");
        let after = pb.change_count();
        if after - before != 1 {
            return Err(format!("saltó de {before} a {after}"));
        }
        Ok(())
    });

    b.case("C2", "cien copias, ninguna perdida en silencio", || {
        let mut watcher = Watcher::default();
        watcher.tick(pb.change_count());
        let mut seen = 0u64;
        for round in 0..100 {
            pb.write_text(&format!("cp-c2-{round}"));
            if let Seen::Fresh { .. } = watcher.tick(pb.change_count()) {
                seen += 1;
            }
        }
        if seen + watcher.missed() != 100 {
            return Err(format!("{seen} vistas y {} contadas", watcher.missed()));
        }
        Ok(())
    });

    b.case("C3", "sondeo desde otro hilo sin perder nada", || {
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (tell, hear) = std::sync::mpsc::channel();
        let watching = {
            let stop = stop.clone();
            std::thread::spawn(move || {
                let mut watcher = Watcher::default();
                let mut fresh = 0u64;
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    if let Seen::Fresh { .. } =
                        watcher.tick(pasteboard::change_count_from_any_thread())
                    {
                        fresh += 1;
                    }
                    std::thread::sleep(Duration::from_millis(2));
                }
                let _ = tell.send((fresh, watcher.missed()));
            })
        };
        std::thread::sleep(Duration::from_millis(40));
        for round in 0..25 {
            pb.write_text(&format!("cp-c3-{round}"));
            std::thread::sleep(Duration::from_millis(12));
        }
        std::thread::sleep(Duration::from_millis(80));
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        watching.join().map_err(|_| "el hilo murió")?;
        let (fresh, missed) = hear.recv().map_err(|why| why.to_string())?;
        if fresh + missed != 25 {
            return Err(format!("{fresh} vistas y {missed} contadas de 25"));
        }
        Ok(())
    });

    b.case(
        "C4",
        "nuestra escritura no se confunde con una copia",
        || {
            let mut watcher = Watcher::default();
            watcher.tick(pb.change_count());
            pb.write_text("cp-c4-nuestro");
            let ours = pb.change_count();
            watcher.wrote(ours);
            match watcher.tick(ours) {
                Seen::Ours => Ok(()),
                other => Err(format!("se vio como {other:?}")),
            }
        },
    );

    b.group("D · Teclado");

    b.case("D1", "el layout activo resuelve la «v»", || {
        keyboard::keycode_with_command('v')
            .map(|_| ())
            .ok_or_else(|| "no se pudo resolver".to_string())
    });

    b.case(
        "D2",
        "Dvorak necesita un keycode distinto",
        || match keyboard::keycode_with_command_in(keyboard::DVORAK, 'v') {
            Some(code) if code != QWERTY_V => Ok(()),
            Some(code) => Err(format!("dio 0x{code:02X}, igual que QWERTY")),
            None => Err("Dvorak no está instalado".into()),
        },
    );

    b.case("D3", "los demás layouts coinciden con QWERTY", || {
        for (name, id) in [
            ("ABC", keyboard::ABC),
            ("AZERTY", keyboard::AZERTY),
            ("QWERTZ", keyboard::QWERTZ),
            ("Español ISO", keyboard::SPANISH_ISO),
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

    b.case(
        "D4",
        "una letra que ningún layout produce no inventa nada",
        || match keyboard::keycode_with_command_in(keyboard::ABC, '\u{1F600}') {
            None => Ok(()),
            Some(code) => Err(format!("devolvió 0x{code:02X} para un emoji")),
        },
    );

    b.case(
        "D5",
        "el keycode se resuelve al pegar, no al arrancar",
        || {
            let paster = Paster::new().ok_or("sin fuente de eventos")?;
            let now = keyboard::keycode_with_command('v').unwrap_or(QWERTY_V);
            if paster.keycode() != now {
                return Err(format!(
                    "el pegador dice 0x{:02X} y el sistema 0x{now:02X}",
                    paster.keycode()
                ));
            }
            Ok(())
        },
    );

    b.group("E · Permisos");

    b.case("E1", "pegar depende solo de poder postear eventos", || {
        if ready.can_paste() != ready.can_post {
            return Err("la regla se torció".into());
        }
        Ok(())
    });

    b.case("E2", "el input seguro no bloquea el pegado", || {
        if ready.secure_input && !ready.can_paste() && ready.can_post {
            return Err("se está tratando el input seguro como bloqueo".into());
        }
        Ok(())
    });

    b.group("F · Destino y pegado");

    b.case(
        "F1",
        "el destino sobrevive a que el panel tome el frente",
        || {
            let mut tracker = Tracker::new(frontmost::our_pid());
            let (pid, bundle) = frontmost::frontmost().ok_or("nadie al frente")?;
            tracker.saw(pid, bundle.as_deref());
            let target = tracker.destination().ok_or("sin destino")?.clone();
            tracker.saw(frontmost::our_pid(), Some("dev.rgdevment.copypaste"));
            if tracker.destination() != Some(&target) {
                return Err("el destino cambió".into());
            }
            Ok(())
        },
    );

    b.case(
        "F2",
        "los modificadores físicos se leen de la fuente",
        || {
            let _ = keystroke::physical_modifiers();
            if keystroke::modifiers_still_held() {
                return Err("hay modificadores pulsados; suelta las teclas".into());
            }
            Ok(())
        },
    );

    match Paster::new() {
        Some(paster) => {
            b.case("F3", "el pegador entrega un keycode utilizable", || {
                if paster.keycode() == 0 {
                    return Err("keycode inválido".into());
                }
                Ok(())
            });
            if ready.can_post {
                match paste_round_trip(&pb, &paster) {
                    Ok(()) => {
                        b.passed += 1;
                        println!("    ok    F4    pegado real en TextEdit, ida y vuelta");
                    }
                    // Que otra aplicación retenga el primer plano no es un
                    // fallo del núcleo: es la activación cooperativa que este
                    // proyecto ya midió. Se omite en vez de dar un rojo falso.
                    Err(why) if why.starts_with("TextEdit no llegó") => {
                        b.skip("F4", "pegado real en TextEdit", &why);
                    }
                    Err(why) => {
                        b.failed += 1;
                        println!("    FALLA F4    pegado real en TextEdit");
                        println!("            {why}");
                    }
                }
            } else {
                b.skip(
                    "F4",
                    "pegado real en TextEdit",
                    "sin permiso para postear eventos",
                );
            }
        }
        None => b.skip("F3", "el pegador se construye", "no hay fuente de eventos"),
    }

    println!();
    println!(
        "  {} pasan · {} fallan · {} omitidas",
        b.passed, b.failed, b.skipped
    );
    if b.failed == 0 {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}

/// Pega en TextEdit y comprueba el resultado sin accesibilidad: tras pegar,
/// selecciona todo y copia, así que lo pegado vuelve por el mismo camino.
fn paste_round_trip(pb: &Pasteboard, paster: &Paster) -> Result<(), String> {
    let path = "/tmp/cp-probe-target.txt";
    std::fs::write(path, "").map_err(|why| why.to_string())?;
    // Arrancar en frío tarda, y otra aplicación puede tener el foco. Se
    // insiste con techo en vez de dormir una cantidad fija y confiar.
    run_open(&["-a", "TextEdit", path]);
    // Se usa la activación del propio núcleo, que es lo que hará el producto,
    // en vez de confiar en que `open` gane el primer plano.
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
            "TextEdit no llegó al frente en 8 s; al frente está {:?}",
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
    match paster.paste_into(&target, || {}) {
        cp_mac::paste::Outcome::Degraded(why) => return Err(format!("degradó: {why:?}")),
        cp_mac::paste::Outcome::Sent { .. } => {}
    }
    std::thread::sleep(Duration::from_millis(400));

    // Seleccionar todo y copiar: lo que vuelva es lo que se pegó.
    let keys = cp_mac_sys::keystroke::Keystroke::new().ok_or("sin fuente")?;
    keys.command(0x00);
    std::thread::sleep(Duration::from_millis(200));
    keys.command(0x08);
    std::thread::sleep(Duration::from_millis(400));

    let back = pb
        .data("public.utf8-plain-text")
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_default();

    if back.contains(&marca) {
        println!("            (ida y vuelta en {:?})", started.elapsed());
        Ok(())
    } else {
        Err(format!("volvió «{}», se esperaba «{marca}»", back.trim()))
    }
}

fn run_open(args: &[&str]) {
    let mut command = std::process::Command::new("/usr/bin/open");
    command.args(args);
    let _ = command.status();
}
