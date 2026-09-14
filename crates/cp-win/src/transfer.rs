//! Si lo que hay en el portapapeles se copió o se cortó.

/// Los bits de `DROPEFFECT`, tal como Windows los define.
const COPY: u32 = 1;
const MOVE: u32 = 2;
const LINK: u32 = 4;

/// Qué pidió la fuente que se hiciera con lo que dejó.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transfer {
    Copy,
    /// Un corte. Las rutas dejan de existir en cuanto alguien pegue, así que
    /// guardarlas como si fueran una copia deja el historial lleno de destinos
    /// muertos.
    Move,
    /// La fuente no dijo nada. Es lo normal fuera del explorador.
    Unsaid,
}

/// Lee `Preferred DropEffect`, que son cuatro bytes en little-endian.
///
/// **Se mira el bit, no el valor.** Medido el 14/09/2026 en Windows 11 26200:
/// el Explorador ofrece **5** al copiar, que es `COPY | LINK`, así que
/// compararlo con `COPY` daría que no es una copia y compararlo con `MOVE`
/// tampoco diría que lo es.
pub fn transfer(value: &[u8]) -> Transfer {
    let [a, b, c, d, ..] = value else {
        return Transfer::Unsaid;
    };
    let effect = u32::from_le_bytes([*a, *b, *c, *d]);
    // El corte manda sobre la copia cuando vienen los dos bits: quien mueve
    // acepta copiar, pero quien copia nunca borra el origen.
    if effect & MOVE != 0 {
        return Transfer::Move;
    }
    // Los dos por separado y no `COPY | LINK`: con bits que no se solapan, el
    // «o» y el «o exclusivo» dan lo mismo y ninguna prueba podría distinguirlos.
    if effect & COPY != 0 || effect & LINK != 0 {
        return Transfer::Copy;
    }
    Transfer::Unsaid
}

#[cfg(test)]
mod tests {
    use super::*;

    /// El valor que el Explorador ofrece de verdad, medido el 14/09/2026 en
    /// trece copias seguidas: siempre cinco, nunca uno.
    #[test]
    fn what_the_explorer_really_offers_when_copying() {
        assert_eq!(transfer(&5u32.to_le_bytes()), Transfer::Copy);
        assert_ne!(
            5u32, COPY,
            "el valor medido no es COPY a secas, y compararlo por igualdad falla"
        );
    }

    #[test]
    fn a_cut_is_recognised_by_its_bit() {
        assert_eq!(transfer(&MOVE.to_le_bytes()), Transfer::Move);
        assert_eq!(transfer(&(MOVE | LINK).to_le_bytes()), Transfer::Move);
    }

    /// Con los dos bits puestos manda el corte: tratar como copia algo que el
    /// origen va a borrar deja rutas muertas en el historial.
    #[test]
    fn a_cut_wins_over_a_copy_when_both_bits_are_set() {
        assert_eq!(transfer(&(COPY | MOVE).to_le_bytes()), Transfer::Move);
        assert_eq!(transfer(&7u32.to_le_bytes()), Transfer::Move);
    }

    #[test]
    fn a_plain_copy_is_a_copy() {
        assert_eq!(transfer(&COPY.to_le_bytes()), Transfer::Copy);
        assert_eq!(transfer(&LINK.to_le_bytes()), Transfer::Copy);
    }

    /// Lo que no se puede leer no se inventa: fuera del explorador este formato
    /// no está, y eso no convierte la copia en un corte.
    #[test]
    fn what_cannot_be_read_says_nothing() {
        assert_eq!(transfer(&[]), Transfer::Unsaid);
        assert_eq!(transfer(&[2]), Transfer::Unsaid);
        assert_eq!(transfer(&[2, 0, 0]), Transfer::Unsaid);
        assert_eq!(transfer(&0u32.to_le_bytes()), Transfer::Unsaid);
    }

    #[test]
    fn the_four_bytes_are_one_little_endian_number() {
        assert_eq!(transfer(&[0, 0, 0, 2]), Transfer::Unsaid);
        assert_eq!(transfer(&[2, 0, 0, 0]), Transfer::Move);
    }

    #[test]
    fn a_longer_value_is_read_by_its_first_four_bytes() {
        assert_eq!(transfer(&[5, 0, 0, 0, 9, 9, 9]), Transfer::Copy);
        assert_eq!(transfer(&[2, 0, 0, 0, 9, 9, 9]), Transfer::Move);
    }

    /// Los bits altos son de otras banderas de `DROPEFFECT` y no dicen nada
    /// sobre copiar o mover.
    #[test]
    fn the_high_bits_decide_nothing() {
        assert_eq!(transfer(&0x8000_0000u32.to_le_bytes()), Transfer::Unsaid);
        assert_eq!(transfer(&0x8000_0002u32.to_le_bytes()), Transfer::Move);
    }
}
