use cp_core::item::{Item, Payload};
use cp_mac::capture::capture;
use cp_mac_sys::pasteboard::{self, Pasteboard};
use std::time::Duration;

fn main() {
    let pb = Pasteboard::general_from_any_thread();
    println!("copia algo… (esperando la primera copia)");
    let first = next_copy(&pb);
    println!("primera: huella {:016x}", first.fingerprint());
    println!("vuelve a copiar exactamente lo mismo…");
    let second = next_copy(&pb);
    println!("segunda: huella {:016x}", second.fingerprint());
    for format in &first.formats {
        let Some(a) = bytes_of(format) else { continue };
        match second
            .formats
            .iter()
            .find(|f| f.id == format.id)
            .and_then(bytes_of)
        {
            None => println!("  {:40} solo en la primera", format.id),
            Some(b) if a == b => println!("  {:40} {:>8} B  iguales", format.id, a.len()),
            Some(b) => {
                let at = a
                    .iter()
                    .zip(b)
                    .position(|(x, y)| x != y)
                    .unwrap_or(a.len().min(b.len()));
                println!(
                    "  {:40} {:>8} B / {:>8} B  DIFIEREN desde el byte {at}",
                    format.id,
                    a.len(),
                    b.len()
                );
                println!("    1: {}", window(a, at));
                println!("    2: {}", window(b, at));
            }
        }
    }
    for format in &second.formats {
        if bytes_of(format).is_some() && !first.formats.iter().any(|f| f.id == format.id) {
            println!("  {:40} solo en la segunda", format.id);
        }
    }
    if first.fingerprint() == second.fingerprint() {
        println!("misma identidad: se reactivaría, no se duplicaría");
    } else {
        println!("identidades distintas: hoy serían dos ítems");
    }
}

fn next_copy(pb: &Pasteboard) -> Item {
    let started = pasteboard::change_count_from_any_thread();
    loop {
        std::thread::sleep(Duration::from_millis(100));
        if pasteboard::change_count_from_any_thread() != started {
            std::thread::sleep(Duration::from_millis(300));
            if let Some(item) = capture(pb).kept() {
                return item;
            }
        }
    }
}

fn bytes_of(format: &cp_core::item::Format) -> Option<&[u8]> {
    match &format.payload {
        Payload::Inline(bytes) | Payload::Blob(bytes) => Some(bytes),
        _ => None,
    }
}

fn window(bytes: &[u8], at: usize) -> String {
    let from = at.saturating_sub(40);
    let to = (at + 60).min(bytes.len());
    String::from_utf8_lossy(&bytes[from..to])
        .chars()
        .map(|c| if c.is_control() { '·' } else { c })
        .collect()
}
