use xxhash_rust::xxh3::xxh3_64;

/// Por debajo de esto se mira el contenido entero; por encima, un muestreo.
const WHOLE_UP_TO: usize = 256 * 1024;
const BLOCKS: usize = 16;
const BLOCK: usize = 4 * 1024;

/// Por debajo del umbral se mira todo, así que el muestreo no puede abarcar
/// más bytes de los que el umbral deja pasar enteros.
const _: () = assert!(WHOLE_UP_TO >= BLOCKS * BLOCK);
/// Ambos se alinean a página.
const _: () = assert!(BLOCK.is_power_of_two() && WHOLE_UP_TO.is_power_of_two());

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
    fn the_sampling_reaches_the_end_of_the_buffer() {
        let big = vec![0x11; 4 * 1024 * 1024];
        let mut tail_changed = big.clone();
        *tail_changed.last_mut().unwrap() = 0x22;
        assert_ne!(
            content_hash(&big),
            content_hash(&tail_changed),
            "si los bloques cayeran todos al principio, esto no se vería"
        );
    }

    /// El límite conocido, escrito a propósito: dieciséis bloques de 4 KB
    /// cubren 64 KB, así que en un buffer de 4 MB se mira el 1,5 %. Un byte
    /// que cambie en un hueco entre bloques no se ve. Es aceptable para lo
    /// que este hash hace —decir «esto es lo mismo que se acaba de copiar»—
    /// y es exactamente la razón por la que los blobs se direccionan con
    /// blake3 sobre el contenido completo y no con esto.
    #[test]
    fn a_change_between_blocks_is_invisible_and_that_is_the_deal() {
        let big = vec![0x11; 4 * 1024 * 1024];
        let mut hole = big.clone();
        hole[2 * 1024 * 1024] = 0x22;
        assert_eq!(
            content_hash(&big),
            content_hash(&hole),
            "si esto cambiara, el muestreo dejó de ser un muestreo"
        );
    }

    #[test]
    fn the_two_paths_meet_at_the_threshold() {
        let under = vec![0x33; WHOLE_UP_TO];
        let over = vec![0x33; WHOLE_UP_TO + 1];
        assert_eq!(
            content_hash(&under),
            xxh3_64(&under),
            "hasta el umbral se mira el contenido entero"
        );
        assert_ne!(
            content_hash(&over),
            xxh3_64(&over),
            "por encima se muestrea, así que no coincide con el hash directo"
        );
    }

    #[test]
    fn each_of_the_sixteen_blocks_is_looked_at() {
        let size = 4 * 1024 * 1024;
        let base = vec![0x44; size];
        let reference = content_hash(&base);
        for block in 0..BLOCKS {
            let mut poked = base.clone();
            let at = (size - BLOCK) * block / (BLOCKS - 1);
            poked[at] = 0x55;
            assert_ne!(
                content_hash(&poked),
                reference,
                "el bloque {block}, que empieza en {at}, no entra en la mezcla"
            );
        }
    }

    #[test]
    fn length_alone_separates_two_otherwise_identical_samples() {
        let a = vec![0x5A; 2 * 1024 * 1024];
        let mut b = a.clone();
        b.push(0x5A);
        assert_ne!(content_hash(&a), content_hash(&b));
    }
}
