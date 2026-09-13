//! Arnés de las invariantes que solo se comprueban contra el sistema.
//!
//! No son pruebas de `cargo test` a propósito: `NSPasteboard` exige el hilo
//! principal y el runner de cargo no lo garantiza. Se ejecuta con
//! `cargo run -p cp-mac --example probe`.

use cp_core::formats::Kind;
use cp_core::item::Payload;
use cp_mac::capture::capture;
use cp_mac_sys::pasteboard::Pasteboard;
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
