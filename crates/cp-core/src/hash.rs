use xxhash_rust::xxh3::xxh3_64;

/// Por debajo de esto se mira el contenido entero; por encima, un muestreo.
const WHOLE_UP_TO: usize = 256 * 1024;
const BLOCKS: usize = 16;
const BLOCK: usize = 4 * 1024;

/// Identidad del contenido. El tamaño entra siempre en la mezcla, así que dos
/// payloads de distinta longitud nunca colisionan aunque se muestree lo mismo.
pub fn content_hash(bytes: &[u8]) -> u64 {
    if bytes.len() <= WHOLE_UP_TO {
        return xxh3_64(bytes);
    }
    let mut mixed = Vec::with_capacity(BLOCKS * BLOCK + 8);
    mixed.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    // Repartidos por todo el buffer, no al principio: dos capturas de pantalla
    // del mismo tamaño comparten cabecera y barra de menús, y un muestreo de
    // los primeros bytes las da por idénticas.
    for block in 0..BLOCKS {
        let start = bytes.len().saturating_sub(BLOCK) * block / (BLOCKS - 1);
        let end = (start + BLOCK).min(bytes.len());
        mixed.extend_from_slice(&bytes[start..end]);
    }
    xxh3_64(&mixed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screenshot(shared_header: usize, tail: u8) -> Vec<u8> {
        let mut one = vec![0xAB; shared_header];
        one.resize(1024 * 1024, tail);
        one
    }

    #[test]
    fn two_captures_sharing_a_menu_bar_are_not_the_same_capture() {
        let a = screenshot(512 * 1024, 0x01);
        let b = screenshot(512 * 1024, 0x02);
        assert_eq!(a.len(), b.len(), "mismo tamaño, media imagen idéntica");
        assert_ne!(
            content_hash(&a),
            content_hash(&b),
            "muestrear solo la cabecera las daría por iguales"
        );
    }

    #[test]
    fn two_long_texts_that_begin_alike_are_different_items() {
        let shared = "x".repeat(100);
        let a = format!("{shared}primero");
        let b = format!("{shared}segundo");
        assert_ne!(content_hash(a.as_bytes()), content_hash(b.as_bytes()));
    }

    #[test]
    fn the_same_content_always_hashes_the_same() {
        let big = screenshot(512 * 1024, 0x07);
        assert_eq!(content_hash(&big), content_hash(&big.clone()));
    }

    #[test]
    fn length_alone_separates_two_otherwise_identical_samples() {
        let a = vec![0x5A; 2 * 1024 * 1024];
        let mut b = a.clone();
        b.push(0x5A);
        assert_ne!(content_hash(&a), content_hash(&b));
    }
}
