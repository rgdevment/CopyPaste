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

pub fn base64url(segment: &str) -> Option<Vec<u8>> {
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
mod tests {
    use super::*;

    fn jwt(payload: &str) -> String {
        let encode = |raw: &str| {
            const TABLE: &[u8] =
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
            let bytes = raw.as_bytes();
            let mut out = String::new();
            for chunk in bytes.chunks(3) {
                let mut buffer = 0u32;
                for (i, b) in chunk.iter().enumerate() {
                    buffer |= u32::from(*b) << (16 - 8 * i);
                }
                for i in 0..=chunk.len() {
                    let index = (buffer >> (18 - 6 * i)) & 63;
                    out.push(TABLE[index as usize] as char);
                }
            }
            out
        };
        format!(
            "{}.{}.firma",
            encode(r#"{"alg":"HS256","typ":"JWT"}"#),
            encode(payload)
        )
    }

    #[test]
    fn a_jwt_is_a_token_and_says_what_it_carries() {
        let token = jwt(r#"{"sub":"cp-3","env":"staging","exp":1700000000,"iss":"cp"}"#);
        assert!(looks_like(&token));
        let claims = claims_of(&token).expect("se lee");
        assert_eq!(claims.subject(), Some("cp-3"));
        assert_eq!(claims.issuer(), Some("cp"));
        assert_eq!(claims.expires_at(), Some(1_700_000_000));
        assert_eq!(claims.expired_by(1_700_000_001), Some(true));
        assert_eq!(claims.expired_by(1_600_000_000), Some(false));
        assert_eq!(
            claims.payload.get("env").and_then(|v| v.as_str()),
            Some("staging"),
            "el problema real no es que se vea: es saber cuál de los doce es el de staging"
        );
    }

    #[test]
    fn a_jwt_without_expiry_does_not_pretend_to_know() {
        let claims = claims_of(&jwt(r#"{"sub":"x"}"#)).expect("se lee");
        assert_eq!(claims.expired_by(0), None);
    }

    #[test]
    fn three_dotted_words_are_not_a_jwt() {
        assert!(
            claims_of("uno.dos.tres").is_none(),
            "no es base64 de un JSON"
        );
        assert!(claims_of("a.b").is_none(), "faltan partes");
        assert!(
            claims_of(&format!("{}.extra", jwt("{}"))).is_none(),
            "sobran"
        );
        assert!(!looks_like("uno.dos.tres"));
        assert!(!looks_like("archivo.tar.gz"));
    }

    #[test]
    fn a_header_without_alg_is_just_json_in_base64() {
        let no_alg = format!("{}.firma", {
            let t = jwt("{}");
            t.replace("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9", "e30")
        });
        assert!(claims_of(&no_alg).is_none());
    }

    #[test]
    fn the_known_prefixes_are_tokens_when_long_enough() {
        for token in [
            "sk-not-a-real-key-for-tests-0123456789",
            "sk-ant-not-a-real-key-for-tests-0000000000000000000",
            "ghp_not_a_real_token_for_tests_0000000000",
            "github_pat_not_a_real_token_for_tests_0000000000",
            "xoxb-not-a-real-slack-token-for-tests",
            "AKIAIOSFODNN7EXAMPLE",
            "AIzaNotARealGoogleKeyForTests00000000000000",
            "glpat-not-a-real-token-for-tests",
        ] {
            assert!(looks_like(token), "{token}");
        }
    }

    #[test]
    fn a_prefix_alone_or_with_spaces_is_not_a_token() {
        assert!(!looks_like("sk-"), "vacío detrás");
        assert!(!looks_like("sk-corto"), "demasiado corto");
        assert!(!looks_like(
            "ghp_ tiene espacios dentro 0123456789012345678901"
        ));
        assert!(!looks_like("AKIA con espacios y todo"));
        assert!(!looks_like(
            "sk-not-a-real-key-for-tests-0123456789 y luego"
        ));
        assert!(!looks_like(""));
        assert!(
            !looks_like(&format!("{} con espacio", jwt("{}"))),
            "un JWT seguido de palabras no es un token"
        );
        assert!(
            !looks_like("skeleton-key-of-the-castle-0123456789"),
            "sk- exacto"
        );
    }

    #[test]
    fn what_a_person_copies_every_day_is_not_a_token() {
        for text in [
            "https://ejemplo.test/ruta",
            "alguien@ejemplo.test",
            "7ab3f6de-1c4b-4f5e-8a2d-9f0e1b2c3d4e",
            "d41d8cd98f00b204e9800998ecf8427e",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcdefghijklmnop",
            "AKIA",
            "skater",
        ] {
            assert!(!looks_like(text), "{text}");
        }
    }

    #[test]
    fn base64url_decodes_with_or_without_padding() {
        assert_eq!(base64url("aGVsbG8").as_deref(), Some(&b"hello"[..]));
        assert_eq!(base64url("aGVsbG8=").as_deref(), Some(&b"hello"[..]));
        assert_eq!(base64url("aGk").as_deref(), Some(&b"hi"[..]));
        assert_eq!(
            base64url("_w").as_deref(),
            Some(&[0xff][..]),
            "alfabeto url"
        );
        assert_eq!(
            base64url("/w").as_deref(),
            Some(&[0xff][..]),
            "y el clásico"
        );
        assert_eq!(base64url("-w").as_deref(), Some(&[0xfb][..]));
        assert_eq!(base64url("+w").as_deref(), Some(&[0xfb][..]));
        assert_eq!(base64url(""), Some(Vec::new()));
        assert_eq!(base64url("a b"), None);
        assert_eq!(base64url("ñ"), None);
    }
}
