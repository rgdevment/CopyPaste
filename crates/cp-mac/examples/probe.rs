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
