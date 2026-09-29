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
#[path = "drop_test.rs"]
mod tests;
