#[cfg(target_os = "windows")]
mod on_windows {
    use cp_core::item::{Format, Item, Payload};
    use cp_win::capture::capture_now;
    use cp_win_sys::clipboard;
    use std::time::Duration;

    pub fn main() {
        println!("copia algo… (esperando la primera copia)");
        let first = next_copy();
        println!("primera: huella {:016x}", first.fingerprint());
        println!("vuelve a copiar exactamente lo mismo…");
        let second = next_copy();
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
                    println!("    1: {}", window(&format.id, a, at));
                    println!("    2: {}", window(&format.id, b, at));
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

    fn next_copy() -> Item {
        let started = clipboard::sequence();
        loop {
            std::thread::sleep(Duration::from_millis(100));
            if clipboard::sequence() != started {
                std::thread::sleep(Duration::from_millis(300));
                if let Some(item) = capture_now().kept() {
                    return item;
                }
            }
        }
    }

    fn bytes_of(format: &Format) -> Option<&[u8]> {
        match &format.payload {
            Payload::Inline(bytes) | Payload::Blob(bytes) => Some(bytes),
            _ => None,
        }
    }

    fn window(id: &str, bytes: &[u8], at: usize) -> String {
        let text = if id == "CF_UNICODETEXT" {
            let units: Vec<u16> = bytes
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| u16::from_le_bytes(*pair))
                .collect();
            let at = at / 2;
            let from = at.saturating_sub(40);
            let to = (at + 60).min(units.len());
            String::from_utf16_lossy(&units[from..to])
        } else {
            let from = at.saturating_sub(40);
            let to = (at + 60).min(bytes.len());
            String::from_utf8_lossy(&bytes[from..to]).into_owned()
        };
        text.chars()
            .map(|c| if c.is_control() { '·' } else { c })
            .collect()
    }
}

#[cfg(target_os = "windows")]
fn main() {
    on_windows::main();
}

#[cfg(not(target_os = "windows"))]
fn main() {}
