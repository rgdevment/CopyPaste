const PREFIXES: &[(&str, usize)] = &[
    ("sk-ant-", 40),
    ("sk_live_", 24),
    ("sk_test_", 24),
    ("rk_live_", 24),
    ("sk-", 32),
    ("ghp_", 36),
    ("gho_", 36),
    ("ghu_", 36),
    ("ghs_", 36),
    ("ghr_", 36),
    ("github_pat_", 40),
    ("glpat-", 20),
    ("xoxb-", 20),
    ("xoxp-", 20),
    ("xoxa-", 20),
    ("xapp-", 20),
    ("AKIA", 20),
    ("AIza", 39),
    ("ya29.", 40),
    ("npm_", 36),
    ("pypi-", 40),
    ("shpat_", 38),
    ("SG.", 60),
];

pub fn looks_like(text: &str) -> bool {
    let text = text.trim();
    if text.is_empty() || text.contains(char::is_whitespace) {
        return false;
    }
    if claims_of(text).is_some() {
        return true;
    }
    PREFIXES.iter().any(|(prefix, at_least)| {
        text.starts_with(prefix)
            && text.len() >= *at_least
            && text[prefix.len()..]
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claims {
    pub header: serde_json::Map<String, serde_json::Value>,
    pub payload: serde_json::Map<String, serde_json::Value>,
}

impl Claims {
    pub fn expires_at(&self) -> Option<i64> {
        self.payload.get("exp").and_then(serde_json::Value::as_i64)
    }

    pub fn expired_by(&self, now_seconds: i64) -> Option<bool> {
        self.expires_at().map(|exp| exp <= now_seconds)
    }

    pub fn subject(&self) -> Option<&str> {
        self.payload.get("sub").and_then(serde_json::Value::as_str)
    }

    pub fn issuer(&self) -> Option<&str> {
        self.payload.get("iss").and_then(serde_json::Value::as_str)
    }
}

pub fn claims_of(text: &str) -> Option<Claims> {
    let mut parts = text.trim().split('.');
    let (header, payload, signature) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() || signature.is_empty() {
        return None;
    }
    let header = object_of(header)?;
    if !header.contains_key("alg") {
        return None;
    }
    Some(Claims {
        header,
        payload: object_of(payload)?,
    })
}

fn object_of(segment: &str) -> Option<serde_json::Map<String, serde_json::Value>> {
    let bytes = base64url(segment)?;
    match serde_json::from_slice(&bytes).ok()? {
        serde_json::Value::Object(map) => Some(map),
        _ => None,
    }
}

fn base64url(segment: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(segment.len() * 3 / 4);
    let mut buffer: u32 = 0;
    let mut bits = 0;
    for c in segment.bytes() {
        let value = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'-' | b'+' => 62,
            b'_' | b'/' => 63,
            b'=' => break,
            _ => return None,
        };
        buffer = (buffer << 6) | u32::from(value);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    Some(out)
}

#[cfg(test)]
#[path = "token_test.rs"]
mod tests;
