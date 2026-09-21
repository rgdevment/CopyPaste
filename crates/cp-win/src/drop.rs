const HEADER: usize = 20;

pub fn paths_in(drop: &[u8]) -> Vec<String> {
    let Some(offset) = drop
        .get(..4)
        .map(|four| u32::from_le_bytes([four[0], four[1], four[2], four[3]]) as usize)
    else {
        return Vec::new();
    };
    let wide = drop.get(16..20).is_some_and(|flag| flag[0] != 0);
    let Some(names) = drop.get(offset..) else {
        return Vec::new();
    };
    if !wide {
        return names
            .split(|byte| *byte == 0)
            .take_while(|part| !part.is_empty())
            .map(|part| String::from_utf8_lossy(part).into_owned())
            .collect();
    }
    let units: Vec<u16> = names
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes(*pair))
        .collect();
    units
        .split(|unit| *unit == 0)
        .take_while(|part| !part.is_empty())
        .map(String::from_utf16_lossy)
        .collect()
}

pub fn drop_of<S: AsRef<str>>(paths: &[S]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&(HEADER as u32).to_le_bytes());
    out.extend_from_slice(&[0u8; 12]);
    out.extend_from_slice(&1u32.to_le_bytes());
    for path in paths {
        for unit in path.as_ref().encode_utf16() {
            out.extend_from_slice(&unit.to_le_bytes());
        }
        out.extend_from_slice(&0u16.to_le_bytes());
    }
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

#[cfg(test)]
pub(crate) fn ansi_drop_of(paths: &[&str]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&(HEADER as u32).to_le_bytes());
    out.extend_from_slice(&[0u8; 12]);
    out.extend_from_slice(&0u32.to_le_bytes());
    for path in paths {
        out.extend_from_slice(path.as_bytes());
        out.push(0);
    }
    out.push(0);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_copied_path_is_read_not_just_the_first() {
        let paths = [r"C:\uno.txt", r"C:\dos.txt", r"C:\una carpeta"];
        assert_eq!(paths_in(&drop_of(&paths)), paths);
    }

    #[test]
    fn a_legacy_ansi_drop_is_read_too() {
        let paths = [r"C:\uno.txt", r"C:\dos.txt"];
        assert_eq!(paths_in(&ansi_drop_of(&paths)), paths);
    }

    #[test]
    fn a_single_path_comes_back_alone() {
        assert_eq!(paths_in(&drop_of(&[r"C:\solo.png"])), [r"C:\solo.png"]);
    }

    #[test]
    fn nonsense_is_not_a_drop() {
        assert!(paths_in(&[]).is_empty());
        assert!(paths_in(&[0, 0, 0]).is_empty());
        assert!(paths_in(&u32::MAX.to_le_bytes()).is_empty());
    }

    #[test]
    fn a_path_with_accents_and_spaces_survives() {
        let paths = [r"C:\Mis Documentos\informe ñ.pdf"];
        assert_eq!(paths_in(&drop_of(&paths)), paths);
    }

    #[test]
    fn the_drop_we_build_is_the_shape_the_shell_reads() {
        let drop = drop_of(&[r"C:\a.txt"]);
        assert_eq!(
            &drop[..4],
            &20u32.to_le_bytes(),
            "los nombres empiezan tras la cabecera"
        );
        assert_eq!(drop[16], 1, "y van en UTF-16");
        assert_eq!(&drop[drop.len() - 4..], &[0, 0, 0, 0], "doble terminador");
        assert!(paths_in(&drop_of::<&str>(&[])).is_empty());
    }
}
