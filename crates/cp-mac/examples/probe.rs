//! Arnés de las invariantes que solo se comprueban contra el sistema.
//!
//! No son pruebas de `cargo test` a propósito: `NSPasteboard` exige el hilo
//! principal y el runner de cargo no lo garantiza. Se ejecuta con
//! `cargo run -p cp-mac --example probe`.

use cp_core::formats::Kind;
use cp_core::item::Payload;
use cp_mac::capture::capture;
use cp_mac_sys::keyboard::{self, QWERTY_V};
use cp_mac_sys::pasteboard::Pasteboard;
use cp_mac_sys::permissions::Readiness;
use objc2_foundation::MainThreadMarker;

fn main() -> std::process::ExitCode {
    let Some(mtm) = MainThreadMarker::new() else {
        eprintln!("esto tiene que correr en el hilo principal");
        return std::process::ExitCode::FAILURE;
    };
    let pb = Pasteboard::general(mtm);
    let mut failed = 0;

    failed += check("el texto plano va y vuelve", || {
        pb.write_text("cp-capture-probe");
        let item = capture(&pb).ok_or("se tomó por contenido secreto")?;
        if item.kind != Some(Kind::Text) {
            return Err(format!("se clasificó como {:?}", item.kind));
        }
        let text = item
            .format("public.utf8-plain-text")
            .ok_or("falta el texto plano")?;
        if text.payload != Payload::Inline(b"cp-capture-probe".to_vec()) {
            return Err(format!("llegó {:?}", text.payload));
        }
        Ok(())
    });

    failed += check("los gemelos legados no se guardan dos veces", || {
        pb.write_text("cp-twin-probe");
        let item = capture(&pb).ok_or("se tomó por contenido secreto")?;
        let ids: Vec<&str> = item.formats.iter().map(|f| f.id.as_str()).collect();
        let mut unique = ids.clone();
        unique.sort_unstable();
        unique.dedup();
        if ids.len() != unique.len() {
            return Err(format!("hay repetidos: {ids:?}"));
        }
        if ids.contains(&"NSStringPboardType") {
            return Err("el gemelo legado no colapsó al moderno".into());
        }
        Ok(())
    });

    failed += check("el contador sube de uno en uno", || {
        let before = pb.change_count();
        pb.write_text("cp-count-probe");
        let after = pb.change_count();
        if after - before != 1 {
            return Err(format!("saltó de {before} a {after}"));
        }
        Ok(())
    });

    failed += check("el vigilante no pierde nada desde su propio hilo", || {
        use cp_core::watch::{Seen, Watcher};
        use std::sync::mpsc;

        let (tell, hear) = mpsc::channel();
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let watching = {
            let stop = stop.clone();
            std::thread::spawn(move || {
                let mut watcher = Watcher::default();
                let mut fresh = 0;
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    if let Seen::Fresh { .. } =
                        watcher.tick(cp_mac_sys::pasteboard::change_count_from_any_thread())
                    {
                        fresh += 1;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
                let _ = tell.send((fresh, watcher.missed()));
            })
        };

        std::thread::sleep(std::time::Duration::from_millis(40));
        let writes = 25;
        for round in 0..writes {
            pb.write_text(&format!("cp-race-{round}"));
            std::thread::sleep(std::time::Duration::from_millis(12));
        }
        std::thread::sleep(std::time::Duration::from_millis(80));
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        watching.join().map_err(|_| "el hilo vigilante murió")?;
        let (fresh, missed) = hear.recv().map_err(|why| why.to_string())?;

        if fresh + missed as i32 != writes {
            return Err(format!(
                "{writes} escrituras, {fresh} vistas y {missed} contadas como perdidas"
            ));
        }
        if missed > 0 {
            println!("        ({missed} de {writes} se perdieron, y se supo)");
        }
        Ok(())
    });

    failed += check("cien copias seguidas, ninguna perdida en silencio", || {
        use cp_core::watch::{Seen, Watcher};

        let mut watcher = Watcher::default();
        watcher.tick(pb.change_count());
        let mut seen = 0;
        for round in 0..100 {
            pb.write_text(&format!("cp-burst-{round}"));
            if let Seen::Fresh { .. } = watcher.tick(pb.change_count()) {
                seen += 1;
            }
        }
        if seen + watcher.missed() as i32 != 100 {
            return Err(format!(
                "100 escrituras, {seen} vistas y {} contadas",
                watcher.missed()
            ));
        }
        if watcher.missed() > 0 {
            println!("        ({} perdidas, y contadas)", watcher.missed());
        }
        Ok(())
    });

    failed += check("un texto de diez megabytes va y vuelve entero", || {
        let big = "a".repeat(10 * 1024 * 1024);
        pb.write_text(&big);
        let item = capture(&pb).ok_or("no se capturó")?;
        let text = item
            .format("public.utf8-plain-text")
            .ok_or("falta el texto")?;
        match &text.payload {
            Payload::Blob(bytes) if bytes.len() == big.len() => Ok(()),
            other => Err(format!(
                "llegó {other:?} en vez de un blob de {}",
                big.len()
            )),
        }
    });

    failed += check("un texto vacío sigue siendo un ítem", || {
        pb.write_text("");
        match capture(&pb) {
            Some(item) => {
                if item.formats.is_empty() {
                    return Err("no se registró ningún formato".into());
                }
                Ok(())
            }
            None => Err("se tomó por contenido secreto".into()),
        }
    });

    failed += check("una copia marcada como secreta no se registra", || {
        // Lo que hace un gestor de contraseñas: pone el secreto y lo marca.
        pb.write_types(&[
            ("public.utf8-plain-text", "contraseña-que-no-debe-guardarse"),
            ("org.nspasteboard.ConcealedType", "1"),
        ]);
        match capture(&pb) {
            None => Ok(()),
            Some(item) => Err(format!(
                "se capturaron {} formatos de algo marcado como oculto",
                item.formats.len()
            )),
        }
    });

    failed += check("el marcador transitorio también se respeta", || {
        pb.write_types(&[
            ("public.utf8-plain-text", "algo efímero"),
            ("org.nspasteboard.TransientType", "1"),
        ]);
        if capture(&pb).is_some() {
            return Err("lo transitorio no debería registrarse".into());
        }
        Ok(())
    });

    failed += check("un gestor legado también queda cubierto", || {
        for marker in [
            "com.agilebits.onepassword",
            "net.antelle.keeweb",
            "PasswordPboardType",
        ] {
            pb.write_types(&[("public.utf8-plain-text", "secreto"), (marker, "1")]);
            if capture(&pb).is_some() {
                return Err(format!("«{marker}» no excluyó el ítem"));
            }
        }
        Ok(())
    });

    failed += check("quitar el marcador vuelve a permitir la captura", || {
        pb.write_text("esto sí se guarda");
        capture(&pb).ok_or("un texto normal debería capturarse")?;
        Ok(())
    });

    failed += check("el destino sobrevive a que el panel tome el frente", || {
        use cp_core::destination::Tracker;

        let mut tracker = Tracker::new(cp_mac_sys::frontmost::our_pid());
        let (pid, bundle) = cp_mac_sys::frontmost::frontmost().ok_or("nadie al frente")?;
        tracker.saw(pid, bundle.as_deref());
        let target = tracker
            .destination()
            .ok_or("no se registró destino")?
            .clone();

        // Lo que ocurre al mostrar el panel: pasamos a ser nosotros.
        tracker.saw(
            cp_mac_sys::frontmost::our_pid(),
            Some("dev.rgdevment.copypaste"),
        );
        match tracker.destination() {
            Some(still) if *still == target => Ok(()),
            other => Err(format!("el destino cambió a {other:?}")),
        }
    });

    let ready = Readiness::probe();
    println!();
    println!("  permisos:");
    println!(
        "    postear eventos (kTCCServicePostEvent) : {}",
        ready.can_post
    );
    println!(
        "    accesibilidad (respaldo por menú)      : {}",
        ready.accessibility
    );
    println!(
        "    input seguro activo ahora              : {}",
        ready.secure_input
    );
    println!(
        "    -> se puede pegar                      : {}",
        ready.can_paste()
    );

    println!();
    println!("  teclado:");
    match keyboard::keycode_with_command('v') {
        Some(code) => {
            println!("    keycode que da «v» con ⌘ : 0x{code:02X}");
            if code == QWERTY_V {
                println!("    coincide con QWERTY, así que este layout no era el problema");
            } else {
                println!(
                    "    DISTINTO de QWERTY (0x{QWERTY_V:02X}): la 2.x pegaría otra tecla aquí"
                );
            }
        }
        None => println!("    el layout activo no produce «v» con ⌘; se usaría 0x{QWERTY_V:02X}"),
    }

    println!();
    println!(
        "  layouts instalados: {}",
        keyboard::installed_layouts().len()
    );
    println!("  el mismo cálculo en otros layouts, sin activarlos:");
    for (name, id) in [
        ("ABC", keyboard::ABC),
        ("Dvorak", keyboard::DVORAK),
        ("Dvorak-QWERTY ⌘", keyboard::DVORAK_COMMAND_QWERTY),
        ("AZERTY (French)", keyboard::AZERTY),
        ("QWERTZ (German)", keyboard::QWERTZ),
        ("Español ISO", keyboard::SPANISH_ISO),
        ("Colemak", keyboard::COLEMAK),
    ] {
        match keyboard::keycode_with_command_in(id, 'v') {
            Some(code) if code == QWERTY_V => {
                println!("    {name:18} 0x{code:02X}  (igual que QWERTY)")
            }
            Some(code) => println!("    {name:18} 0x{code:02X}  <- AQUÍ la 2.x pega otra tecla"),
            None => println!("    {name:18} no instalado"),
        }
    }

    failed += check("el input seguro no impide pegar", || {
        if !ready.can_post && ready.secure_input {
            return Err("se está confundiendo input seguro con falta de permiso".into());
        }
        Ok(())
    });

    println!();
    if failed == 0 {
        println!("todo en verde");
        std::process::ExitCode::SUCCESS
    } else {
        println!("{failed} comprobaciones fallaron");
        std::process::ExitCode::FAILURE
    }
}

fn check(what: &str, run: impl Fn() -> Result<(), String>) -> u32 {
    match run() {
        Ok(()) => {
            println!("  ok    {what}");
            0
        }
        Err(why) => {
            println!("  FALLA {what}: {why}");
            1
        }
    }
}
