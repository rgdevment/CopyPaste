#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

pub fn rgba_of(text: &str) -> Option<Rgba> {
    let said = text.trim();
    if let Some(hex) = said.strip_prefix('#') {
        return from_hex(hex);
    }
    let lower = said.to_ascii_lowercase();
    for prefix in ["rgba(", "rgb(", "hsla(", "hsl("] {
        if let Some(inner) = lower
            .strip_prefix(prefix)
            .and_then(|rest| rest.strip_suffix(')'))
        {
            let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
            return if prefix.starts_with("hsl") {
                from_hsl(&parts)
            } else {
                from_rgb(&parts)
            };
        }
    }
    None
}

fn from_hex(hex: &str) -> Option<Rgba> {
    let digits: Vec<u8> = hex
        .chars()
        .map(|one| one.to_digit(16).map(|got| got as u8))
        .collect::<Option<_>>()?;
    match digits.len() {
        3 => Some(Rgba {
            red: digits[0] * 17,
            green: digits[1] * 17,
            blue: digits[2] * 17,
            alpha: 0xFF,
        }),
        6 => Some(Rgba {
            red: digits[0] * 16 + digits[1],
            green: digits[2] * 16 + digits[3],
            blue: digits[4] * 16 + digits[5],
            alpha: 0xFF,
        }),
        8 => Some(Rgba {
            red: digits[0] * 16 + digits[1],
            green: digits[2] * 16 + digits[3],
            blue: digits[4] * 16 + digits[5],
            alpha: digits[6] * 16 + digits[7],
        }),
        _ => None,
    }
}

fn from_rgb(parts: &[&str]) -> Option<Rgba> {
    if parts.len() != 3 && parts.len() != 4 {
        return None;
    }
    let channel = |said: &str| -> Option<u8> {
        let (said, of_full) = match said.strip_suffix('%') {
            Some(rest) => (rest, 255.0 / 100.0),
            None => (said, 1.0),
        };
        let got: f64 = said.trim().parse().ok()?;
        Some(rounded(got * of_full))
    };
    Some(Rgba {
        red: channel(parts[0])?,
        green: channel(parts[1])?,
        blue: channel(parts[2])?,
        alpha: match parts.get(3) {
            Some(said) => opacity(said)?,
            None => 0xFF,
        },
    })
}

fn from_hsl(parts: &[&str]) -> Option<Rgba> {
    if parts.len() != 3 && parts.len() != 4 {
        return None;
    }
    let turn: f64 = parts[0].trim_end_matches("deg").trim().parse().ok()?;
    let turn = turn.rem_euclid(360.0) / 60.0;
    let reach = ratio(parts[1])?;
    let light = ratio(parts[2])?;
    let chroma = (1.0 - (2.0 * light - 1.0).abs()) * reach;
    let second = chroma * (1.0 - ((turn % 2.0) - 1.0).abs());
    let (red, green, blue) = match turn as u32 {
        0 => (chroma, second, 0.0),
        1 => (second, chroma, 0.0),
        2 => (0.0, chroma, second),
        3 => (0.0, second, chroma),
        4 => (second, 0.0, chroma),
        _ => (chroma, 0.0, second),
    };
    let lift = light - chroma / 2.0;
    Some(Rgba {
        red: rounded((red + lift) * 255.0),
        green: rounded((green + lift) * 255.0),
        blue: rounded((blue + lift) * 255.0),
        alpha: match parts.get(3) {
            Some(said) => opacity(said)?,
            None => 0xFF,
        },
    })
}

fn ratio(said: &str) -> Option<f64> {
    let said = said.trim();
    let got: f64 = said.trim_end_matches('%').trim().parse().ok()?;
    let whole = if said.ends_with('%') {
        got / 100.0
    } else {
        got
    };
    Some(whole.clamp(0.0, 1.0))
}

fn opacity(said: &str) -> Option<u8> {
    Some(rounded(ratio(said)? * 255.0))
}

fn rounded(what: f64) -> u8 {
    if what.is_nan() {
        return 0;
    }
    what.round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
#[path = "paint_test.rs"]
mod tests;
